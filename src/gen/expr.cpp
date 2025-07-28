#include "expr.h"
#include <stdexcept>

void ExprGenerator::generate(ExprNode* node) {
    if (auto* intLit = dynamic_cast<IntLiteralNode*>(node)) {
        generateIntLiteral(intLit);
    } else if (auto* longLit = dynamic_cast<LongLiteralNode*>(node)) {
        generateLongLiteral(longLit);
    } else if (auto* floatLit = dynamic_cast<FloatLiteralNode*>(node)) {
        generateFloatLiteral(floatLit);
    } else if (auto* doubleLit = dynamic_cast<DoubleLiteralNode*>(node)) {
        generateDoubleLiteral(doubleLit);
    } else if (auto* stringLit = dynamic_cast<StringLiteralNode*>(node)) {
        generateStringLiteral(stringLit);
    } else if (auto* boolLit = dynamic_cast<BoolLiteralNode*>(node)) {
        generateBoolLiteral(boolLit);
    } else if (auto* ident = dynamic_cast<IdentifierNode*>(node)) {
        generateIdentifier(ident);
    } else if (auto* arrayLit = dynamic_cast<ArrayLiteralNode*>(node)) {
        generateArrayLiteral(arrayLit);
    } else if (auto* indexNode = dynamic_cast<IndexNode*>(node)) {
        generateIndex(indexNode);
    } else if (auto* binOp = dynamic_cast<BinaryOpNode*>(node)) {
        generateBinaryOp(binOp);
    } else if (auto* unaryOp = dynamic_cast<UnaryOpNode*>(node)) {
        generateUnaryOp(unaryOp);
    } else if (auto* call = dynamic_cast<CallNode*>(node)) {
        generateCall(call);
    } else if (auto* addrOf = dynamic_cast<AddressOfNode*>(node)) {
        generateAddressOf(addrOf);
    } else if (auto* deref = dynamic_cast<DereferenceNode*>(node)) {
        generateDereference(deref);
    } else if (auto* fieldAccess = dynamic_cast<FieldAccessNode*>(node)) {
        generateFieldAccess(fieldAccess);
    } else if (auto* structInit = dynamic_cast<StructInitNode*>(node)) {
        generateStructInit(structInit);
    } else if (auto* unionInit = dynamic_cast<UnionInitNode*>(node)) {
        generateUnionInit(unionInit);
    } else if (auto* methodCall = dynamic_cast<MethodCallNode*>(node)) {
        generateMethodCall(methodCall);
    }
}

void ExprGenerator::generateIntLiteral(IntLiteralNode* node) {
    emit(std::to_string(node->value));
}

void ExprGenerator::generateLongLiteral(LongLiteralNode* node) {
    emit(std::to_string(node->value) + "L");
}

void ExprGenerator::generateFloatLiteral(FloatLiteralNode* node) {
    emit(std::to_string(node->value) + "f");
}

void ExprGenerator::generateDoubleLiteral(DoubleLiteralNode* node) {
    emit(std::to_string(node->value));
}

void ExprGenerator::generateStringLiteral(StringLiteralNode* node) {
    emit("\"");
    for (char c : node->value) {
        switch (c) {
            case '\n': emit("\\n"); break;
            case '\t': emit("\\t"); break;
            case '\r': emit("\\r"); break;
            case '\\': emit("\\\\"); break;
            case '"': emit("\\\""); break;
            default: emit(std::string(1, c)); break;
        }
    }
    emit("\"");
}

void ExprGenerator::generateBoolLiteral(BoolLiteralNode* node) {
    emit(node->value ? "1" : "0");
}

void ExprGenerator::generateIdentifier(IdentifierNode* node) {
    emit(node->name);
}

void ExprGenerator::generateArrayLiteral(ArrayLiteralNode* node) {
    emit("{");
    for (size_t i = 0; i < node->elements.size(); i++) {
        if (i > 0) emit(", ");
        generate(node->elements[i].get());
    }
    emit("}");
}

void ExprGenerator::generateIndex(IndexNode* node) {
    generate(node->array.get());
    emit("[");
    generate(node->index.get());
    emit("]");
}

void ExprGenerator::generateBinaryOp(BinaryOpNode* node) {
    emit("(");
    generate(node->left.get());
    emit(" ");
    emit(node->op);
    emit(" ");
    generate(node->right.get());
    emit(")");
}

void ExprGenerator::generateUnaryOp(UnaryOpNode* node) {
    emit(node->op);
    emit("(");
    generate(node->operand.get());
    emit(")");
}

void ExprGenerator::generateCall(CallNode* node) {
    // Special handling for print function
    if (node->functionName == "print") {
        if (node->arguments.size() == 1) {
            emit("print(");
            generate(node->arguments[0].get());
            emit(")");
            return;
        } else if (node->arguments.size() == 0) {
            emit("printf(\"\\n\")");
            return;
        }
        // For multiple arguments, print each separately
        for (size_t i = 0; i < node->arguments.size(); i++) {
            if (i > 0) emit("; ");
            emit("print(");
            generate(node->arguments[i].get());
            emit(")");
        }
        return;
    }
    
    // Handle range function with different arities
    if (node->functionName == "range") {
        if (node->arguments.size() == 1) {
            emit("range1");
        } else if (node->arguments.size() == 2) {
            emit("range2");
        } else if (node->arguments.size() == 3) {
            emit("range3");
        } else {
            emit(node->functionName);
        }
    } else {
        emit(node->functionName);
    }
    
    emit("(");
    for (size_t i = 0; i < node->arguments.size(); i++) {
        if (i > 0) emit(", ");
        generate(node->arguments[i].get());
    }
    emit(")");
}

void ExprGenerator::generateAddressOf(AddressOfNode* node) {
    emit("&(");
    generate(node->operand.get());
    emit(")");
}

void ExprGenerator::generateDereference(DereferenceNode* node) {
    emit("*(");
    generate(node->operand.get());
    emit(")");
}

void ExprGenerator::generateFieldAccess(FieldAccessNode* node) {
    generate(node->object.get());
    emit(".");
    emit(node->fieldName);
}

void ExprGenerator::generateStructInit(StructInitNode* node) {
    emit("(struct " + node->structName + "){");
    
    for (size_t i = 0; i < node->fields.size(); i++) {
        if (i > 0) emit(", ");
        
        const auto& field = node->fields[i];
        if (!field.first.empty()) {
            // Named field initialization: .fieldName = value
            emit("." + field.first + " = ");
        }
        generate(field.second.get());
    }
    
    emit("}");
}

void ExprGenerator::generateMethodCall(MethodCallNode* node) {
    // Try to determine the struct type of the receiver
    std::string structName;
    
    if (auto* ident = dynamic_cast<IdentifierNode*>(node->receiver.get())) {
        // Look up variable type from symbol table or type registry
        if (symbolTable && symbolTable->hasSymbol(ident->name)) {
            std::string varType = symbolTable->getSymbolType(ident->name);
            // Extract struct name from "struct StructName" format
            if (varType.find("struct ") == 0) {
                structName = varType.substr(7); // Remove "struct " prefix
            }
        } else if (typeRegistry) {
            std::string varType = typeRegistry->getVariableType(ident->name);
            if (varType.find("struct ") == 0) {
                structName = varType.substr(7);
            }
        }
    } else if (auto* fieldAccess = dynamic_cast<FieldAccessNode*>(node->receiver.get())) {
        // Handle nested field access like c1.center.magnitude()
        if (auto* baseIdent = dynamic_cast<IdentifierNode*>(fieldAccess->object.get())) {
            std::string baseType;
            if (symbolTable && symbolTable->hasSymbol(baseIdent->name)) {
                baseType = symbolTable->getSymbolType(baseIdent->name);
            } else if (typeRegistry) {
                baseType = typeRegistry->getVariableType(baseIdent->name);
            }
            
            if (baseType.find("struct ") == 0) {
                std::string baseStructName = baseType.substr(7);
                // Look up the field type in the struct
                if (typeRegistry) {
                    std::string fieldType = typeRegistry->getFieldType(baseStructName, fieldAccess->fieldName);
                    if (fieldType.find("struct ") == 0) {
                        structName = fieldType.substr(7);
                    }
                }
            }
        }
    }
    
    // If we couldn't determine the struct type, use a fallback
    if (structName.empty()) {
        // Debug: emit a comment to help diagnose type resolution issues
        if (auto* id = dynamic_cast<IdentifierNode*>(node->receiver.get())) {
            emit("/* ERROR: Could not determine struct type for " + id->name + " */ ");
        } else {
            emit("/* ERROR: Could not determine struct type for receiver */ ");
        }
        structName = "UnknownStruct";
    }
    
    // Generate function call: __StructName_methodName(receiver, args...)
    emit("__" + structName + "_" + node->methodName + "(");
    generate(node->receiver.get());
    
    for (auto& arg : node->arguments) {
        emit(", ");
        generate(arg.get());
    }
    
    emit(")");
}

void ExprGenerator::generateUnionInit(UnionInitNode* node) {
    emit("(union " + node->unionName + "){." + node->activeMember + " = ");
    generate(node->value.get());
    emit("}");
}

