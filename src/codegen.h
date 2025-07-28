#pragma once
#include <string>
#include <sstream>
#include <memory>
#include "ast.h"
#include "usage_tracker.h"
#include "type_registry.h"

class CodeGenerator {
private:
    std::stringstream output;
    int indentLevel;
    UsageTracker usageTracker;
    TypeRegistry typeRegistry;
    
public:
    CodeGenerator();
    std::string generate(std::unique_ptr<ProgramNode>& ast);
    
private:
    void generateProgram(ProgramNode* node);
    void generateStruct(StructDefNode* node);
    void generateUnion(UnionDefNode* node);
    void generateImplBlock(ImplBlockNode* node, class FuncGenerator& funcGen);
    void analyzeUsage(ProgramNode* node);
    void analyzeFunction(FunctionNode* node);
    void analyzeStatement(StmtNode* node);
    void analyzeExpression(ExprNode* node);
    void buildTypeRegistry(ProgramNode* node);
};