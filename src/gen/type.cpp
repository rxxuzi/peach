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
        // Since we don't have full type information here, assume int
        return "int";
    } else if (auto* binOp = dynamic_cast<BinaryOpNode*>(expr)) {
        // For binary operations, infer from the left operand
        // Assignment operations should not appear here in variable initialization
        if (binOp->op == "=") {
            // This shouldn't happen in a proper parse
            return "int";
        }
        return inferType(binOp->left.get());
    } else if (auto* addrOf = dynamic_cast<AddressOfNode*>(expr)) {
        // Address-of gives a pointer type
        // We'd need more context to determine the exact pointer type
        return "int*";
    } else {
        return "int"; // Default type
    }
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