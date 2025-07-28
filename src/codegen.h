#pragma once
#include <string>
#include <sstream>
#include <memory>
#include "ast.h"
#include "usage_tracker.h"

class CodeGenerator {
private:
    std::stringstream output;
    int indentLevel;
    UsageTracker usageTracker;
    
public:
    CodeGenerator();
    std::string generate(std::unique_ptr<ProgramNode>& ast);
    
private:
    void generateProgram(ProgramNode* node);
    void analyzeUsage(ProgramNode* node);
    void analyzeFunction(FunctionNode* node);
    void analyzeStatement(StmtNode* node);
    void analyzeExpression(ExprNode* node);
};