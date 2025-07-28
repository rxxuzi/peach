#pragma once
#include "base.h"
#include "expr.h"
#include "../ast.h"
#include "../type_registry.h"

class StmtGenerator : public CodeGenBase {
private:
    TypeRegistry* typeRegistry;
    SymbolTable* currentScope;
    
public:
    StmtGenerator(std::stringstream& out, int& indent, TypeRegistry* types = nullptr) 
        : CodeGenBase(out, indent), typeRegistry(types), currentScope(nullptr) {}
    
    void setCurrentScope(SymbolTable* scope) { currentScope = scope; }
    
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