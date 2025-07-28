#include "type_registry.h"

void TypeRegistry::registerStruct(const std::string& name) {
    structs[name] = StructInfo{name, {}, {}};
}

void TypeRegistry::addStructField(const std::string& structName, const std::string& fieldName, const std::string& fieldType) {
    auto it = structs.find(structName);
    if (it != structs.end()) {
        it->second.fields[fieldName] = fieldType;
    }
}

void TypeRegistry::addStructMethod(const std::string& structName, const MethodInfo& method) {
    auto it = structs.find(structName);
    if (it != structs.end()) {
        it->second.methods.push_back(method);
    }
}

void TypeRegistry::registerVariable(const std::string& varName, const std::string& varType) {
    variables[varName] = varType;
}

std::string TypeRegistry::getVariableType(const std::string& varName) const {
    auto it = variables.find(varName);
    if (it != variables.end()) {
        return it->second;
    }
    return "";
}

bool TypeRegistry::isStruct(const std::string& typeName) const {
    return structs.find(typeName) != structs.end();
}

std::string TypeRegistry::getFieldType(const std::string& structName, const std::string& fieldName) const {
    auto structIt = structs.find(structName);
    if (structIt != structs.end()) {
        auto fieldIt = structIt->second.fields.find(fieldName);
        if (fieldIt != structIt->second.fields.end()) {
            return fieldIt->second;
        }
    }
    return "";
}

std::string TypeRegistry::getMethodReturnType(const std::string& structName, const std::string& methodName) const {
    auto structIt = structs.find(structName);
    if (structIt != structs.end()) {
        for (const auto& method : structIt->second.methods) {
            if (method.name == methodName) {
                return method.returnType;
            }
        }
    }
    return "";
}

void TypeRegistry::clear() {
    structs.clear();
    variables.clear();
}