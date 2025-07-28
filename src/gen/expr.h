#pragma once
#include "base.h"
#include "symbol_table.h"
#include "../ast.h"
#include "../type_registry.h"

class ExprGenerator : public CodeGenBase {
private:
    SymbolTable* symbolTable;
    TypeRegistry* typeRegistry;
    
public:
    ExprGenerator(std::stringstream& out, int& indent) 
        : CodeGenBase(out, indent), symbolTable(nullptr), typeRegistry(nullptr) {}
    
    ExprGenerator(std::stringstream& out, int& indent, SymbolTable* symbols, TypeRegistry* types = nullptr) 
        : CodeGenBase(out, indent), symbolTable(symbols), typeRegistry(types) {}
    
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
    void generateFieldAccess(FieldAccessNode* node);
    void generateStructInit(StructInitNode* node);
    void generateMethodCall(MethodCallNode* node);
};