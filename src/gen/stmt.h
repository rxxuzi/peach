#pragma once
#include "base.h"
#include "expr.h"
#include "../ast.h"

class StmtGenerator : public CodeGenBase {
private:
    ExprGenerator exprGen;
    
public:
    StmtGenerator(std::stringstream& out, int& indent) 
        : CodeGenBase(out, indent), exprGen(out, indent) {}
    
    void generate(StmtNode* node);
    
private:
    void generateVarDecl(VarDeclNode* node);
    void generateBlock(BlockNode* node);
    void generateIf(IfNode* node);
    void generateWhile(WhileNode* node);
    void generateFor(ForNode* node);
    void generateReturn(ReturnNode* node);
    void generateExprStmt(ExprStmtNode* node);
    
    // Helper methods
    void generateForRange(ForNode* node, CallNode* rangeCall);
    void generateForArray(ForNode* node);
};