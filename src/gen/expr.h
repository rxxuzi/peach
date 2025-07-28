#pragma once
#include "base.h"
#include "symbol_table.h"
#include "../ast.h"

class ExprGenerator : public CodeGenBase {
private:
    SymbolTable* symbolTable;
    
public:
    ExprGenerator(std::stringstream& out, int& indent) 
        : CodeGenBase(out, indent), symbolTable(nullptr) {}
    
    ExprGenerator(std::stringstream& out, int& indent, SymbolTable* symbols) 
        : CodeGenBase(out, indent), symbolTable(symbols) {}
    
    void generate(ExprNode* node);
    
private:
    void generateIntLiteral(IntLiteralNode* node);
    void generateLongLiteral(LongLiteralNode* node);
    void generateFloatLiteral(FloatLiteralNode* node);
    void generateDoubleLiteral(DoubleLiteralNode* node);
    void generateStringLiteral(StringLiteralNode* node);
    void generateBoolLiteral(BoolLiteralNode* node);
    void generateIdentifier(IdentifierNode* node);
    void generateArrayLiteral(ArrayLiteralNode* node);
    void generateIndex(IndexNode* node);
    void generateBinaryOp(BinaryOpNode* node);
    void generateUnaryOp(UnaryOpNode* node);
    void generateCall(CallNode* node);
    void generateAddressOf(AddressOfNode* node);
    void generateDereference(DereferenceNode* node);
};