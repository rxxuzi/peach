#include "memory_safety.h"
#include "ast.h"
#include <algorithm>

std::vector<MemorySafetyAnalyzer::MemoryIssue> MemorySafetyAnalyzer::analyzeProgram(ProgramNode* program) {
    std::vector<MemoryIssue> issues;
    
    if (!program) {
        issues.emplace_back(MemoryIssue::MEMORY_LEAK, "Null program node", "", 0, 0);
        return issues;
    }
    
    // Analyze all functions
    for (const auto& function : program->functions) {
        auto functionIssues = analyzeFunction(function.get());
        issues.insert(issues.end(), functionIssues.begin(), functionIssues.end());
    }
    
    return issues;
}

std::vector<MemorySafetyAnalyzer::MemoryIssue> MemorySafetyAnalyzer::analyzeFunction(FunctionNode* function) {
    std::vector<MemoryIssue> issues;
    
    if (!function) {
        issues.emplace_back(MemoryIssue::MEMORY_LEAK, "Null function node", "", 0, 0);
        return issues;
    }
    
    // Mark parameters as initialized
    for (const auto& param : function->parameters) {
        markVariableInitialized(param.first);
    }
    
    // Analyze function body
    auto bodyIssues = analyzeStatement(function->body.get());
    issues.insert(issues.end(), bodyIssues.begin(), bodyIssues.end());
    
    return issues;
}

std::vector<MemorySafetyAnalyzer::MemoryIssue> MemorySafetyAnalyzer::analyzeStatement(StmtNode* statement) {
    std::vector<MemoryIssue> issues;
    
    if (!statement) {
        return issues; // Empty statement is safe
    }
    
    // Variable declarations
    if (auto* varDecl = dynamic_cast<VarDeclNode*>(statement)) {
        if (varDecl->initializer) {
            // Variable is initialized
            markVariableInitialized(varDecl->name);
            auto exprIssues = analyzeExpression(varDecl->initializer.get());
            issues.insert(issues.end(), exprIssues.begin(), exprIssues.end());
        } else {
            // Variable is declared but not initialized
            markVariableUninitialized(varDecl->name);
        }
        
        // Check for pointer types that might cause issues
        if (varDecl->type) {
            std::string typeStr = varDecl->type->toCType();
            if (typeStr.find("*") != std::string::npos) {
                // This is a pointer type - track it
                trackPointer(varDecl->name, "");
            }
        }
    }
    
    // Block statements
    if (auto* block = dynamic_cast<BlockNode*>(statement)) {
        for (const auto& stmt : block->statements) {
            auto stmtIssues = analyzeStatement(stmt.get());
            issues.insert(issues.end(), stmtIssues.begin(), stmtIssues.end());
        }
    }
    
    // Expression statements
    if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(statement)) {
        auto exprIssues = analyzeExpression(exprStmt->expr.get());
        issues.insert(issues.end(), exprIssues.begin(), exprIssues.end());
    }
    
    // Assignment statements (check for use of uninitialized variables)
    if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(statement)) {
        if (auto* binOp = dynamic_cast<BinaryOpNode*>(exprStmt->expr.get())) {
            if (binOp->op == "=") {
                // This is an assignment
                auto rhsIssues = analyzeExpression(binOp->right.get());
                issues.insert(issues.end(), rhsIssues.begin(), rhsIssues.end());
                
                // Mark LHS as initialized if it's a simple identifier
                if (auto* lhsIdent = dynamic_cast<IdentifierNode*>(binOp->left.get())) {
                    markVariableInitialized(lhsIdent->name);
                }
            }
        }
    }
    
    return issues;
}

std::vector<MemorySafetyAnalyzer::MemoryIssue> MemorySafetyAnalyzer::analyzeExpression(ExprNode* expression) {
    std::vector<MemoryIssue> issues;
    
    if (!expression) {
        return issues;
    }
    
    // Check identifier usage
    if (auto* ident = dynamic_cast<IdentifierNode*>(expression)) {
        if (!isVariableInitialized(ident->name)) {
            issues.emplace_back(MemoryIssue::UNINITIALIZED_USE, 
                "Use of uninitialized variable: " + ident->name, ident->name, 0, 0);
        }
        
        if (isPointerDangling(ident->name)) {
            issues.emplace_back(MemoryIssue::DANGLING_POINTER,
                "Use of dangling pointer: " + ident->name, ident->name, 0, 0);
        }
    }
    
    // Check binary operations
    if (auto* binOp = dynamic_cast<BinaryOpNode*>(expression)) {
        auto leftIssues = analyzeExpression(binOp->left.get());
        auto rightIssues = analyzeExpression(binOp->right.get());
        issues.insert(issues.end(), leftIssues.begin(), leftIssues.end());
        issues.insert(issues.end(), rightIssues.begin(), rightIssues.end());
    }
    
    // Check unary operations
    if (auto* unaryOp = dynamic_cast<UnaryOpNode*>(expression)) {
        auto operandIssues = analyzeExpression(unaryOp->operand.get());
        issues.insert(issues.end(), operandIssues.begin(), operandIssues.end());
    }
    
    // Check function calls
    if (auto* call = dynamic_cast<CallNode*>(expression)) {
        for (const auto& arg : call->arguments) {
            auto argIssues = analyzeExpression(arg.get());
            issues.insert(issues.end(), argIssues.begin(), argIssues.end());
        }
    }
    
    // Check array access
    if (auto* indexNode = dynamic_cast<IndexNode*>(expression)) {
        auto arrayIssues = analyzeExpression(indexNode->array.get());
        auto indexIssues = analyzeExpression(indexNode->index.get());
        issues.insert(issues.end(), arrayIssues.begin(), arrayIssues.end());
        issues.insert(issues.end(), indexIssues.begin(), indexIssues.end());
        
        // TODO: Add bounds checking analysis
    }
    
    // Check dereference operations
    if (auto* deref = dynamic_cast<DereferenceNode*>(expression)) {
        auto operandIssues = analyzeExpression(deref->operand.get());
        issues.insert(issues.end(), operandIssues.begin(), operandIssues.end());
        
        // Check if dereferencing a potentially null pointer
        if (auto* ident = dynamic_cast<IdentifierNode*>(deref->operand.get())) {
            if (isPointerDangling(ident->name)) {
                issues.emplace_back(MemoryIssue::DANGLING_POINTER,
                    "Dereferencing dangling pointer: " + ident->name, ident->name, 0, 0);
            }
        }
    }
    
    return issues;
}

void MemorySafetyAnalyzer::markVariableInitialized(const std::string& varName) {
    variableInitialized[varName] = true;
}

void MemorySafetyAnalyzer::markVariableUninitialized(const std::string& varName) {
    variableInitialized[varName] = false;
}

bool MemorySafetyAnalyzer::isVariableInitialized(const std::string& varName) const {
    auto it = variableInitialized.find(varName);
    return it != variableInitialized.end() && it->second;
}

void MemorySafetyAnalyzer::trackPointer(const std::string& pointerName, const std::string& target) {
    pointerTargets[pointerName] = target;
    danglingPointers.erase(pointerName); // No longer dangling if we set a target
}

void MemorySafetyAnalyzer::markPointerDangling(const std::string& pointerName) {
    danglingPointers.insert(pointerName);
}

bool MemorySafetyAnalyzer::isPointerDangling(const std::string& pointerName) const {
    return danglingPointers.find(pointerName) != danglingPointers.end();
}

void MemorySafetyAnalyzer::reset() {
    variableInitialized.clear();
    danglingPointers.clear();
    pointerTargets.clear();
}