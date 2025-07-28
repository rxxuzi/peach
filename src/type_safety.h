#pragma once
#include <string>
#include <unordered_set>
#include <memory>
#include "ast.h"

// Type safety checker for compile-time verification
class TypeSafetyChecker {
private:
    std::unordered_set<std::string> declaredTypes;
    std::unordered_set<std::string> declaredFunctions;
    std::unordered_set<std::string> declaredVariables;
    
public:
    struct TypeSafetyResult {
        bool isValid;
        std::string errorMessage;
        int line;
        int column;
        
        TypeSafetyResult(bool valid = true, const std::string& msg = "", int l = 0, int c = 0)
            : isValid(valid), errorMessage(msg), line(l), column(c) {}
    };
    
    TypeSafetyResult checkProgram(ProgramNode* program);
    TypeSafetyResult checkFunction(FunctionNode* function);
    TypeSafetyResult checkStatement(StmtNode* statement);
    TypeSafetyResult checkExpression(ExprNode* expression);
    
    // Type compatibility checking
    bool areTypesCompatible(const std::string& type1, const std::string& type2);
    bool isTypeDeclarated(const std::string& typeName);
    
    // Register types for checking
    void registerType(const std::string& typeName);
    void registerFunction(const std::string& functionName);
    void registerVariable(const std::string& variableName);
    
    // Clear state for new compilation unit
    void reset();
    
private:
    // Helper methods
    bool isBuiltinType(const std::string& typeName);
    bool isBuiltinFunction(const std::string& functionName);
};