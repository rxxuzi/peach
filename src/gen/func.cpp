#include "func.h"
#include "expr.h"

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
        // Infer return type - for now default to void
        emit("void");
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
            emit(params[i].second->toCType());
            emit(" ");
            emit(params[i].first);
        }
    }
}

void FuncGenerator::generateBody(FunctionNode* node) {
    if (auto* block = dynamic_cast<BlockNode*>(node->body.get())) {
        stmtGen.generate(node->body.get());
    } else if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(node->body.get())) {
        // Single expression body - wrap in block with return
        emitLine("{");
        indentLevel++;
        if (node->returnType && node->returnType->toCType() != "void") {
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