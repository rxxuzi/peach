#pragma once
#include <string>
#include <unordered_map>
#include <vector>
#include <memory>

struct MethodInfo {
    std::string name;
    std::string returnType;
    std::vector<std::string> parameterTypes;
    bool isPointerReceiver;
    
    MethodInfo(const std::string& n, const std::string& ret, 
               const std::vector<std::string>& params, bool ptrReceiver)
        : name(n), returnType(ret), parameterTypes(params), isPointerReceiver(ptrReceiver) {}
};

struct StructInfo {
    std::string name;
    std::unordered_map<std::string, std::string> fields; // field name -> type
    std::vector<MethodInfo> methods;
};

class TypeRegistry {
private:
    std::unordered_map<std::string, StructInfo> structs;
    std::unordered_map<std::string, std::string> variables; // variable name -> type
    
public:
    // Struct management
    void registerStruct(const std::string& name);
    void addStructField(const std::string& structName, const std::string& fieldName, const std::string& fieldType);
    void addStructMethod(const std::string& structName, const MethodInfo& method);
    
    // Variable type tracking
    void registerVariable(const std::string& varName, const std::string& varType);
    std::string getVariableType(const std::string& varName) const;
    
    // Type queries
    bool isStruct(const std::string& typeName) const;
    std::string getFieldType(const std::string& structName, const std::string& fieldName) const;
    std::string getMethodReturnType(const std::string& structName, const std::string& methodName) const;
    
    // Clear registry (for new compilation units)
    void clear();
};