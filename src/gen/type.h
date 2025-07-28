#pragma once
#include "base.h"
#include "../ast.h"
#include <vector>

class TypeGenerator : public CodeGenBase {
public:
    TypeGenerator(std::stringstream& out, int& indent) 
        : CodeGenBase(out, indent) {}
    
    // Generate array declaration with proper C syntax
    std::string generateArrayDeclaration(ArrayTypeNode* arrayType, 
                                       const std::string& varName,
                                       ExprNode* initializer = nullptr);
    
    // Infer type from expression
    std::string inferType(ExprNode* expr);
    
    // Calculate array size from literal
    int calculateArraySize(ArrayLiteralNode* literal);
    
private:
    // Helper to collect array dimensions
    void collectArrayDimensions(TypeNode* type, std::vector<std::string>& dimensions);
};