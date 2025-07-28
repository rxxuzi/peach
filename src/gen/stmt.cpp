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
        TypeGenerator typeGen(output, indentLevel);
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
        exprGen.generate(node->initializer.get());
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
    emit("// For-each loop for array\n");
    indent();
    emit("for (int _i = 0; _i < sizeof(");
    exprGen.generate(node->collection.get());
    emit(")/sizeof(");
    exprGen.generate(node->collection.get());
    emit("[0]); _i++) {\n");
    
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
        exprGen.generate(node->value.get());
    }
}

void StmtGenerator::generateExprStmt(ExprStmtNode* node) {
    indent();
    exprGen.generate(node->expr.get());
    emit(";\n");
}