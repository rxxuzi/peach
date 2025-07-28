#include "codegen.h"
#include "gen/builtin.h"
#include "gen/func.h"
#include "gen/stmt.h"
#include <stdexcept>

CodeGenerator::CodeGenerator() : indentLevel(0) {}

std::string CodeGenerator::generate(std::unique_ptr<ProgramNode>& ast) {
    output.str("");
    output.clear();
    
    // First pass: analyze usage
    analyzeUsage(ast.get());
    
    // Generate built-in functions and includes
    BuiltinGenerator builtinGen(output, indentLevel, usageTracker);
    builtinGen.generateAll();
    
    // Generate the program
    generateProgram(ast.get());
    
    return output.str();
}

void CodeGenerator::generateProgram(ProgramNode* node) {
    StmtGenerator stmtGen(output, indentLevel);
    
    // Generate global declarations
    for (auto& decl : node->globalDeclarations) {
        stmtGen.generate(decl.get());
        output << ";\n";
    }
    
    if (!node->globalDeclarations.empty()) {
        output << "\n";
    }
    
    // Generate functions
    FuncGenerator funcGen(output, indentLevel);
    for (auto& func : node->functions) {
        funcGen.generate(func.get());
        output << "\n";
    }
}

void CodeGenerator::analyzeUsage(ProgramNode* node) {
    // Analyze global declarations
    for (auto& decl : node->globalDeclarations) {
        analyzeStatement(decl.get());
    }
    
    // Analyze functions
    for (auto& func : node->functions) {
        analyzeFunction(func.get());
    }
}

void CodeGenerator::analyzeFunction(FunctionNode* node) {
    // Analyze function body
    analyzeStatement(node->body.get());
}

void CodeGenerator::analyzeStatement(StmtNode* node) {
    if (auto* block = dynamic_cast<BlockNode*>(node)) {
        for (auto& stmt : block->statements) {
            analyzeStatement(stmt.get());
        }
    } else if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(node)) {
        analyzeExpression(exprStmt->expr.get());
    } else if (auto* varDecl = dynamic_cast<VarDeclNode*>(node)) {
        if (varDecl->initializer) {
            analyzeExpression(varDecl->initializer.get());
        }
        if (varDecl->type) {
            // Track type usage
            if (auto* basicType = dynamic_cast<BasicTypeNode*>(varDecl->type.get())) {
                usageTracker.trackType(basicType->typeName);
            }
        }
    } else if (auto* ifNode = dynamic_cast<IfNode*>(node)) {
        analyzeExpression(ifNode->condition.get());
        analyzeStatement(ifNode->thenBranch.get());
        if (ifNode->elseBranch) {
            analyzeStatement(ifNode->elseBranch.get());
        }
    } else if (auto* whileNode = dynamic_cast<WhileNode*>(node)) {
        analyzeExpression(whileNode->condition.get());
        analyzeStatement(whileNode->body.get());
    } else if (auto* forNode = dynamic_cast<ForNode*>(node)) {
        analyzeExpression(forNode->collection.get());
        analyzeStatement(forNode->body.get());
    } else if (auto* returnNode = dynamic_cast<ReturnNode*>(node)) {
        if (returnNode->value) {
            analyzeExpression(returnNode->value.get());
        }
    }
}

void CodeGenerator::analyzeExpression(ExprNode* node) {
    if (auto* call = dynamic_cast<CallNode*>(node)) {
        usageTracker.trackFunction(call->functionName);
        for (auto& arg : call->arguments) {
            analyzeExpression(arg.get());
        }
    } else if (auto* binOp = dynamic_cast<BinaryOpNode*>(node)) {
        analyzeExpression(binOp->left.get());
        analyzeExpression(binOp->right.get());
    } else if (auto* unaryOp = dynamic_cast<UnaryOpNode*>(node)) {
        analyzeExpression(unaryOp->operand.get());
    } else if (auto* indexNode = dynamic_cast<IndexNode*>(node)) {
        analyzeExpression(indexNode->array.get());
        analyzeExpression(indexNode->index.get());
    } else if (auto* addrOf = dynamic_cast<AddressOfNode*>(node)) {
        analyzeExpression(addrOf->operand.get());
    } else if (auto* deref = dynamic_cast<DereferenceNode*>(node)) {
        analyzeExpression(deref->operand.get());
    } else if (auto* arrayLit = dynamic_cast<ArrayLiteralNode*>(node)) {
        for (auto& elem : arrayLit->elements) {
            analyzeExpression(elem.get());
        }
    }
    // Literals and identifiers don't need analysis
}