#include "func.h"
#include "expr.h"
#include "symbol_table.h"

void FuncGenerator::generate(FunctionNode* node) {
    generateSignature(node);
    emit(" ");
    generateBody(node);
}

void FuncGenerator::generateSignature(FunctionNode* node) {
    // Generate return type
    if (node->returnType) {
        emit(node->returnType->toCType());
    } else {
        // Infer return type from function body with parameter context
        std::string inferredType = inferReturnTypeWithContext(node->body.get(), node->parameters);
        emit(inferredType);
    }
    emit(" ");
    
    // Generate function name
    emit(node->name);
    emit("(");
    
    // Generate parameters
    generateParameters(node->parameters);
    
    emit(")");
}

void FuncGenerator::generateParameters(const std::vector<std::pair<std::string, TypeNodePtr>>& params) {
    if (params.empty()) {
        emit("void");
    } else {
        for (size_t i = 0; i < params.size(); i++) {
            if (i > 0) emit(", ");
            std::string paramType = params[i].second->toCType();
            // Convert array parameters to pointer parameters for C compatibility
            if (paramType.find("[") != std::string::npos) {
                // Extract element type from array type like "[5]int" -> "int*"
                size_t bracketPos = paramType.find("[");
                size_t closeBracketPos = paramType.find("]");
                if (closeBracketPos != std::string::npos) {
                    std::string elementType = paramType.substr(closeBracketPos + 1);
                    paramType = elementType + "*";
                }
            }
            emit(paramType);
            emit(" ");
            emit(params[i].first);
        }
    }
}

void FuncGenerator::generateBody(FunctionNode* node) {
    // Create symbol table for function scope
    SymbolTable functionScope;
    
    // Add parameters to symbol table
    for (const auto& param : node->parameters) {
        functionScope.addSymbol(param.first, param.second->toCType());
    }
    
    // Create statement generator with function scope
    StmtGenerator stmtGen(output, indentLevel, typeRegistry);
    stmtGen.setCurrentScope(&functionScope);
    
    if (auto* block = dynamic_cast<BlockNode*>(node->body.get())) {
        stmtGen.generate(node->body.get());
    } else if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(node->body.get())) {
        // Single expression body - wrap in block with return
        emitLine("{");
        indentLevel++;
        
        std::string returnType;
        if (node->returnType) {
            returnType = node->returnType->toCType();
        } else {
            returnType = inferReturnTypeWithContext(node->body.get(), node->parameters);
        }
        
        if (returnType != "void") {
            indent();
            emit("return ");
            ExprGenerator exprGen(output, indentLevel);
            exprGen.generate(exprStmt->expr.get());
            emit(";\n");
        } else {
            stmtGen.generate(node->body.get());
        }
        indentLevel--;
        emitLine("}");
    } else {
        stmtGen.generate(node->body.get());
    }
}

std::string FuncGenerator::inferReturnType(StmtNode* body) {
    std::vector<std::pair<std::string, TypeNodePtr>> emptyParams;
    return inferReturnTypeWithContext(body, emptyParams);
}

std::string FuncGenerator::inferReturnTypeWithContext(StmtNode* body, const std::vector<std::pair<std::string, TypeNodePtr>>& parameters) {
    // Create symbol table with function parameters
    SymbolTable symbolTable;
    for (const auto& param : parameters) {
        symbolTable.addSymbol(param.first, param.second->toCType());
    }
    
    // Handle expression statements (single expression functions)
    if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(body)) {
        TypeGenerator typeGen(output, indentLevel, &symbolTable);
        return typeGen.inferType(exprStmt->expr.get());
    }
    
    // Handle block statements
    if (auto* block = dynamic_cast<BlockNode*>(body)) {
        std::string returnType = "void"; // Default
        
        // Look for return statements in the block
        for (const auto& stmt : block->statements) {
            if (auto* returnStmt = dynamic_cast<ReturnNode*>(stmt.get())) {
                if (returnStmt->value) {
                    TypeGenerator typeGen(output, indentLevel, &symbolTable);
                    return typeGen.inferType(returnStmt->value.get());
                } else {
                    return "void";
                }
            }
            
            // Recursively check nested blocks
            if (auto* nestedBlock = dynamic_cast<BlockNode*>(stmt.get())) {
                std::string nestedType = inferReturnType(nestedBlock);
                if (nestedType != "void") {
                    returnType = nestedType;
                }
            }
            
            // Check if statements
            if (auto* ifStmt = dynamic_cast<IfNode*>(stmt.get())) {
                std::string thenType = inferReturnType(ifStmt->thenBranch.get());
                if (thenType != "void") {
                    returnType = thenType;
                }
                if (ifStmt->elseBranch) {
                    std::string elseType = inferReturnType(ifStmt->elseBranch.get());
                    if (elseType != "void") {
                        returnType = elseType;
                    }
                }
            }
        }
        
        return returnType;
    }
    
    // Default case
    return "void";
}