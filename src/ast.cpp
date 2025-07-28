#include "ast.h"

std::string BasicTypeNode::toCType() const {
    if (typeName == "int") return "int";
    if (typeName == "long") return "long";
    if (typeName == "float") return "float";
    if (typeName == "double") return "double";
    if (typeName == "bool") return "int"; // C doesn't have bool
    if (typeName == "string") return "char*";
    if (typeName == "void") return "void";
    return typeName; // fallback
}

std::string PointerTypeNode::toCType() const {
    return baseType->toCType() + "*";
}

std::string ArrayTypeNode::toCType() const {
    // Note: This returns just the element type
    // The array size is handled differently in C declarations
    return elementType->toCType();
}