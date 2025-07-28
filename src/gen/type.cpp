#include "type.h"

std::string TypeGenerator::generateArrayDeclaration(ArrayTypeNode* arrayType, 
                                                  const std::string& varName,
                                                  ExprNode* initializer) {
    std::string baseType = arrayType->elementType->toCType();
    std::string result = baseType + " " + varName;
    
    // Collect array dimensions
    std::vector<std::string> dimensions;
    
    // If size is not specified and we have an initializer, infer size
    if (!arrayType->size && initializer) {
        if (auto* arrayLit = dynamic_cast<ArrayLiteralNode*>(initializer)) {
            int size = calculateArraySize(arrayLit);
            dimensions.push_back("[" + std::to_string(size) + "]");
        } else {
            dimensions.push_back("[]");
        }
    } else {
        collectArrayDimensions(arrayType, dimensions);
    }
    
    // Apply dimensions
    for (const auto& dim : dimensions) {
        result += dim;
    }
    
    return result;
}

std::string TypeGenerator::inferType(ExprNode* expr) {
    if (dynamic_cast<IntLiteralNode*>(expr)) {
        return "int";
    } else if (dynamic_cast<LongLiteralNode*>(expr)) {
        return "long";
    } else if (dynamic_cast<FloatLiteralNode*>(expr)) {
        return "float";
    } else if (dynamic_cast<DoubleLiteralNode*>(expr)) {
        return "double";
    } else if (dynamic_cast<StringLiteralNode*>(expr)) {
        return "const char*";
    } else if (dynamic_cast<BoolLiteralNode*>(expr)) {
        return "int";
    } else if (dynamic_cast<ArrayLiteralNode*>(expr)) {
        // For array literals, we need to look at the first element
        auto* arrayLit = dynamic_cast<ArrayLiteralNode*>(expr);
        if (!arrayLit->elements.empty()) {
            return inferType(arrayLit->elements[0].get());
        }
        return "int"; // Default array element type
    } else if (auto* deref = dynamic_cast<DereferenceNode*>(expr)) {
        // Dereference of a pointer gives the pointed-to type
        std::string ptrType = inferType(deref->operand.get());
        // Remove the trailing '*' if it exists
        if (ptrType.length() > 1 && ptrType.back() == '*') {
            return ptrType.substr(0, ptrType.length() - 1);
        }
        return "int"; // Fallback
    } else if (auto* binOp = dynamic_cast<BinaryOpNode*>(expr)) {
        // For binary operations, need to consider type promotion
        if (binOp->op == "=") {
            // This shouldn't happen in a proper parse
            return "int";
        }
        
        std::string leftType = inferType(binOp->left.get());
        std::string rightType = inferType(binOp->right.get());
        
        // Type promotion rules
        if (leftType == "double" || rightType == "double") {
            return "double";
        } else if (leftType == "float" || rightType == "float") {
            return "float";
        } else if (leftType == "long" || rightType == "long") {
            return "long";
        } else {
            return "int";
        }
    } else if (auto* addrOf = dynamic_cast<AddressOfNode*>(expr)) {
        // Address-of gives a pointer type
        // Try to determine the type of the operand
        std::string operandType = inferType(addrOf->operand.get());
        return operandType + "*";
    } else if (auto* call = dynamic_cast<CallNode*>(expr)) {
        // Function calls - for now assume int, but could be extended
        // to track function return types
        return "int";
    } else if (auto* ident = dynamic_cast<IdentifierNode*>(expr)) {
        // Identifiers - use symbol table for type lookup
        if (symbolTable && symbolTable->hasSymbol(ident->name)) {
            return symbolTable->getSymbolType(ident->name);
        }
        return "int"; // Fallback
    } else {
        return "int"; // Default type
    }
}

std::string TypeGenerator::inferTypeWithContext(ExprNode* expr, SymbolTable* symbols) {
    // Temporarily set the symbol table
    SymbolTable* oldTable = symbolTable;
    symbolTable = symbols;
    
    // Use the existing inferType method
    std::string result = inferType(expr);
    
    // Restore the old symbol table
    symbolTable = oldTable;
    
    return result;
}

int TypeGenerator::calculateArraySize(ArrayLiteralNode* literal) {
    return literal->elements.size();
}

void TypeGenerator::collectArrayDimensions(TypeNode* type, std::vector<std::string>& dimensions) {
    ArrayTypeNode* current = dynamic_cast<ArrayTypeNode*>(type);
    while (current) {
        if (current->size) {
            if (auto* intLit = dynamic_cast<IntLiteralNode*>(current->size.get())) {
                dimensions.push_back("[" + std::to_string(intLit->value) + "]");
            } else {
                dimensions.push_back("[1]"); // Default size
            }
        } else {
            dimensions.push_back("[]");
        }
        current = dynamic_cast<ArrayTypeNode*>(current->elementType.get());
    }
}