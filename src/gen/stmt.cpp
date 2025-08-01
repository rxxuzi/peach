#include "stmt.h"
#include "type.h"

void StmtGenerator::generate(StmtNode* node) {
    if (auto* varDecl = dynamic_cast<VarDeclNode*>(node)) {
        generateVarDecl(varDecl);
    } else if (auto* block = dynamic_cast<BlockNode*>(node)) {
        generateBlock(block);
    } else if (auto* ifNode = dynamic_cast<IfNode*>(node)) {
        generateIf(ifNode);
    } else if (auto* whileNode = dynamic_cast<WhileNode*>(node)) {
        generateWhile(whileNode);
    } else if (auto* forNode = dynamic_cast<ForNode*>(node)) {
        generateFor(forNode);
    } else if (auto* returnNode = dynamic_cast<ReturnNode*>(node)) {
        generateReturn(returnNode);
        emit(";\n");
    } else if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(node)) {
        generateExprStmt(exprStmt);
    }
}

void StmtGenerator::generateVarDecl(VarDeclNode* node) {
    indent();
    
    // Handle array types specially (const arrays cause issues with pointer passing)
    if (auto* arrayType = dynamic_cast<ArrayTypeNode*>(node->type.get())) {
        TypeGenerator typeGen(output, indentLevel);
        std::string decl = typeGen.generateArrayDeclaration(arrayType, node->name, node->initializer.get());
        emit(decl);
    } else if (node->type) {
        // Generate const for non-array types
        if (node->isConst) {
            emit("const ");
        }
        emit(node->type->toCType());
        emit(" ");
        emit(node->name);
    } else if (node->initializer) {
        // Infer type from initializer
        TypeGenerator typeGen(output, indentLevel, nullptr, typeRegistry);
        std::string inferredType = typeGen.inferType(node->initializer.get());
        
        if (auto* arrayLit = dynamic_cast<ArrayLiteralNode*>(node->initializer.get())) {
            // For array literals, don't use const to avoid pointer passing issues
            int size = arrayLit->elements.size();
            emit(inferredType + " " + node->name + "[" + std::to_string(size) + "]");
        } else {
            // Generate const for non-array types
            if (node->isConst) {
                emit("const ");
            }
            emit(inferredType + " " + node->name);
        }
    } else {
        emit("int " + node->name); // Default type
    }
    
    if (node->initializer) {
        emit(" = ");
        
        // Create expression generator with current scope
        ExprGenerator exprGen(output, indentLevel, currentScope, typeRegistry);
        exprGen.generate(node->initializer.get());
        
        // Register variable type in current scope and type registry
        std::string varType;
        if (node->type) {
            varType = node->type->toCType();
        } else {
            // Infer type from initializer
            TypeGenerator typeGen(output, indentLevel, currentScope, typeRegistry);
            varType = typeGen.inferType(node->initializer.get());
        }
        
        if (currentScope && !varType.empty()) {
            currentScope->addSymbol(node->name, varType);
        }
        if (typeRegistry && !varType.empty()) {
            typeRegistry->registerVariable(node->name, varType);
        }
    }
    emit(";\n");
}

void StmtGenerator::generateBlock(BlockNode* node) {
    emitLine("{");
    indentLevel++;
    
    for (auto& stmt : node->statements) {
        generate(stmt.get());
        // Each statement type now handles its own semicolons
    }
    
    indentLevel--;
    emitLine("}");
}

void StmtGenerator::generateIf(IfNode* node) {
    indent();
    emit("if (");
    ExprGenerator exprGen(output, indentLevel, currentScope, typeRegistry);
    exprGen.generate(node->condition.get());
    emit(") ");
    
    if (dynamic_cast<BlockNode*>(node->thenBranch.get())) {
        emit("\n");
        generate(node->thenBranch.get());
    } else {
        emit("{\n");
        indentLevel++;
        generate(node->thenBranch.get());
        if (dynamic_cast<ExprStmtNode*>(node->thenBranch.get())) {
            emit(";\n");
        }
        indentLevel--;
        emitLine("}");
    }
    
    if (node->elseBranch) {
        indent();
        emit("else ");
        if (dynamic_cast<BlockNode*>(node->elseBranch.get()) || 
            dynamic_cast<IfNode*>(node->elseBranch.get())) {
            emit("\n");
            generate(node->elseBranch.get());
        } else {
            emit("{\n");
            indentLevel++;
            generate(node->elseBranch.get());
            if (dynamic_cast<ExprStmtNode*>(node->elseBranch.get())) {
                emit(";\n");
            }
            indentLevel--;
            emitLine("}");
        }
    }
}

void StmtGenerator::generateWhile(WhileNode* node) {
    indent();
    emit("while (");
    ExprGenerator exprGen(output, indentLevel, currentScope, typeRegistry);
    exprGen.generate(node->condition.get());
    emit(") ");
    
    if (dynamic_cast<BlockNode*>(node->body.get())) {
        emit("\n");
        generate(node->body.get());
    } else {
        emit("{\n");
        indentLevel++;
        generate(node->body.get());
        if (dynamic_cast<ExprStmtNode*>(node->body.get())) {
            emit(";\n");
        }
        indentLevel--;
        emitLine("}");
    }
}

void StmtGenerator::generateFor(ForNode* node) {
    // Check if it's a range-based for loop
    if (auto* call = dynamic_cast<CallNode*>(node->collection.get())) {
        if (call->functionName == "range") {
            generateForRange(node, call);
            return;
        }
    }
    
    // Otherwise, it's an array iteration
    generateForArray(node);
}

void StmtGenerator::generateForRange(ForNode* node, CallNode* rangeCall) {
    indent();
    ExprGenerator exprGen(output, indentLevel, currentScope, typeRegistry);
    
    if (rangeCall->arguments.size() == 1) {
        emit("for (int " + node->iteratorName + " = 0; " + 
             node->iteratorName + " < ");
        exprGen.generate(rangeCall->arguments[0].get());
        emit("; " + node->iteratorName + "++)");
    } else if (rangeCall->arguments.size() == 2) {
        emit("for (int " + node->iteratorName + " = ");
        exprGen.generate(rangeCall->arguments[0].get());
        emit("; " + node->iteratorName + " < ");
        exprGen.generate(rangeCall->arguments[1].get());
        emit("; " + node->iteratorName + "++)");
    } else if (rangeCall->arguments.size() == 3) {
        emit("for (int " + node->iteratorName + " = ");
        exprGen.generate(rangeCall->arguments[0].get());
        emit("; " + node->iteratorName + " < ");
        exprGen.generate(rangeCall->arguments[1].get());
        emit("; " + node->iteratorName + " += ");
        exprGen.generate(rangeCall->arguments[2].get());
        emit(")");
    }
    
    if (dynamic_cast<BlockNode*>(node->body.get())) {
        emit(" \n");
        generate(node->body.get());
    } else {
        emit(" {\n");
        indentLevel++;
        generate(node->body.get());
        if (dynamic_cast<ExprStmtNode*>(node->body.get())) {
            emit(";\n");
        }
        indentLevel--;
        emitLine("}");
    }
}

void StmtGenerator::generateForArray(ForNode* node) {
    indent();
    ExprGenerator exprGen(output, indentLevel, currentScope, typeRegistry);
    
    // Check if the collection is an identifier (array variable)
    std::string arrayName;
    std::string arrayType;
    int arraySize = -1;
    bool isPointerParam = false;
    
    if (auto* ident = dynamic_cast<IdentifierNode*>(node->collection.get())) {
        arrayName = ident->name;
        // Look up array type and size from symbol table or type registry
        if (currentScope && currentScope->hasSymbol(arrayName)) {
            arrayType = currentScope->getSymbolType(arrayName);
        } else if (typeRegistry) {
            arrayType = typeRegistry->getVariableType(arrayName);
        }
        
        // Check if it's a pointer parameter (int*) 
        if (arrayType.find("*") != std::string::npos) {
            isPointerParam = true;
        } else if (arrayType.find("[") != std::string::npos) {
            // Extract array size from type like "[5]int"
            size_t start = arrayType.find("[");
            size_t end = arrayType.find("]");
            if (end != std::string::npos) {
                std::string sizeStr = arrayType.substr(start + 1, end - start - 1);
                arraySize = std::stoi(sizeStr);
            }
        }
    }
    
    emit("// For-each loop for array\n");
    indent();
    
    if (isPointerParam) {
        // For pointer parameters, we need a different approach
        // For now, emit an error since we don't know the size
        emit("/* ERROR: Cannot iterate over pointer parameter without size */ \n");
        indent();
        emit("for (int _i = 0; _i < 1 /* UNKNOWN SIZE */; _i++) {\n");
    } else if (arraySize > 0) {
        // Use known array size
        emit("for (int _i = 0; _i < " + std::to_string(arraySize) + "; _i++) {\n");
    } else {
        // Fallback to sizeof approach (works for local arrays)
        emit("for (int _i = 0; _i < sizeof(");
        exprGen.generate(node->collection.get());
        emit(")/sizeof(");
        exprGen.generate(node->collection.get());
        emit("[0]); _i++) {\n");
    }
    
    indentLevel++;
    indent();
    emit("int " + node->iteratorName + " = ");
    exprGen.generate(node->collection.get());
    emit("[_i];\n");
    
    if (auto* block = dynamic_cast<BlockNode*>(node->body.get())) {
        // Extract statements from block without generating extra braces
        for (auto& stmt : block->statements) {
            generate(stmt.get());
            if (dynamic_cast<VarDeclNode*>(stmt.get()) || 
                dynamic_cast<ExprStmtNode*>(stmt.get())) {
                emit(";\n");
            }
        }
    } else {
        generate(node->body.get());
        if (dynamic_cast<ExprStmtNode*>(node->body.get())) {
            emit(";\n");
        }
    }
    
    indentLevel--;
    emitLine("}");
}

void StmtGenerator::generateReturn(ReturnNode* node) {
    indent();
    emit("return");
    if (node->value) {
        emit(" ");
        ExprGenerator exprGen(output, indentLevel, currentScope, typeRegistry);
        exprGen.generate(node->value.get());
    }
}

void StmtGenerator::generateExprStmt(ExprStmtNode* node) {
    indent();
    ExprGenerator exprGen(output, indentLevel, currentScope, typeRegistry);
    exprGen.generate(node->expr.get());
    emit(";\n");
}