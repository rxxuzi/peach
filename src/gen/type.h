#pragma once
#include "base.h"
#include "symbol_table.h"
#include "../ast.h"
#include <vector>

class TypeGenerator : public CodeGenBase {
private:
    SymbolTable* symbolTable;
    
public:
    TypeGenerator(std::stringstream& out, int& indent) 
        : CodeGenBase(out, indent), symbolTable(nullptr) {}
    
    TypeGenerator(std::stringstream& out, int& indent, SymbolTable* symbols) 
        : CodeGenBase(out, indent), symbolTable(symbols) {}
    
    // Generate array declaration with proper C syntax
    std::string generateArrayDeclaration(ArrayTypeNode* arrayType, 
                                       const std::string& varName,
                                       ExprNode* initializer = nullptr);
    
    // Infer type from expression
    std::string inferType(ExprNode* expr);
    
    // Infer type with symbol table context
    std::string inferTypeWithContext(ExprNode* expr, SymbolTable* symbols);
    
    // Calculate array size from literal
    int calculateArraySize(ArrayLiteralNode* literal);
    
private:
    // Helper to collect array dimensions
    void collectArrayDimensions(TypeNode* type, std::vector<std::string>& dimensions);
};