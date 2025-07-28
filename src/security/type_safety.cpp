#include "type_safety.h"
#include <algorithm>

TypeSafetyChecker::TypeSafetyResult TypeSafetyChecker::checkProgram(ProgramNode* program) {
    if (!program) {
        return TypeSafetyResult(false, "Null program node", 0, 0);
    }
    
    // First pass: register all types and functions
    for (const auto& structDef : program->structs) {
        registerType("struct " + structDef->name);
    }
    
    for (const auto& unionDef : program->unions) {
        registerType("union " + unionDef->name);
    }
    
    for (const auto& enumDef : program->enums) {
        registerType("enum " + enumDef->name);
    }
    
    for (const auto& function : program->functions) {
        registerFunction(function->name);
    }
    
    // Second pass: check function implementations
    for (const auto& function : program->functions) {
        auto result = checkFunction(function.get());
        if (!result.isValid) {
            return result;
        }
    }
    
    return TypeSafetyResult(true);
}

TypeSafetyChecker::TypeSafetyResult TypeSafetyChecker::checkFunction(FunctionNode* function) {
    if (!function) {
        return TypeSafetyResult(false, "Null function node", 0, 0);
    }
    
    // Check parameter types
    for (const auto& param : function->parameters) {
        std::string paramType = param.second->toCType();
        if (!isTypeDeclarated(paramType) && !isBuiltinType(paramType)) {
            return TypeSafetyResult(false, 
                "Unknown parameter type: " + paramType + " in function " + function->name, 0, 0);
        }
        registerVariable(param.first);
    }
    
    // Check return type
    if (function->returnType) {
        std::string returnType = function->returnType->toCType();
        if (!isTypeDeclarated(returnType) && !isBuiltinType(returnType)) {
            return TypeSafetyResult(false, 
                "Unknown return type: " + returnType + " in function " + function->name, 0, 0);
        }
    }
    
    // Check function body
    return checkStatement(function->body.get());
}

TypeSafetyChecker::TypeSafetyResult TypeSafetyChecker::checkStatement(StmtNode* statement) {
    if (!statement) {
        return TypeSafetyResult(true); // Empty statement is valid
    }
    
    // Check variable declarations
    if (auto* varDecl = dynamic_cast<VarDeclNode*>(statement)) {
        if (varDecl->type) {
            std::string declType = varDecl->type->toCType();
            if (!isTypeDeclarated(declType) && !isBuiltinType(declType)) {
                return TypeSafetyResult(false, 
                    "Unknown type in variable declaration: " + declType, 0, 0);
            }
        }
        
        if (varDecl->initializer) {
            auto exprResult = checkExpression(varDecl->initializer.get());
            if (!exprResult.isValid) {
                return exprResult;
            }
        }
        
        registerVariable(varDecl->name);
    }
    
    // Check block statements
    if (auto* block = dynamic_cast<BlockNode*>(statement)) {
        for (const auto& stmt : block->statements) {
            auto result = checkStatement(stmt.get());
            if (!result.isValid) {
                return result;
            }
        }
    }
    
    // Check expression statements
    if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(statement)) {
        return checkExpression(exprStmt->expr.get());
    }
    
    return TypeSafetyResult(true);
}

TypeSafetyChecker::TypeSafetyResult TypeSafetyChecker::checkExpression(ExprNode* expression) {
    if (!expression) {
        return TypeSafetyResult(false, "Null expression", 0, 0);
    }
    
    // Check function calls
    if (auto* call = dynamic_cast<CallNode*>(expression)) {
        if (declaredFunctions.find(call->functionName) == declaredFunctions.end() && 
            !isBuiltinFunction(call->functionName)) {
            return TypeSafetyResult(false, 
                "Undefined function: " + call->functionName, 0, 0);
        }
        
        // Check arguments
        for (const auto& arg : call->arguments) {
            auto result = checkExpression(arg.get());
            if (!result.isValid) {
                return result;
            }
        }
    }
    
    // Check identifiers
    if (auto* ident = dynamic_cast<IdentifierNode*>(expression)) {
        if (declaredVariables.find(ident->name) == declaredVariables.end()) {
            return TypeSafetyResult(false, 
                "Undefined variable: " + ident->name, 0, 0);
        }
    }
    
    return TypeSafetyResult(true);
}

bool TypeSafetyChecker::areTypesCompatible(const std::string& type1, const std::string& type2) {
    if (type1 == type2) return true;
    
    // Check numeric type compatibility
    std::unordered_set<std::string> numericTypes = {"int", "long", "float", "double"};
    if (numericTypes.count(type1) && numericTypes.count(type2)) {
        return true; // Allow numeric conversions
    }
    
    return false;
}

bool TypeSafetyChecker::isTypeDeclarated(const std::string& typeName) {
    return declaredTypes.find(typeName) != declaredTypes.end();
}

void TypeSafetyChecker::registerType(const std::string& typeName) {
    declaredTypes.insert(typeName);
}

void TypeSafetyChecker::registerFunction(const std::string& functionName) {
    declaredFunctions.insert(functionName);
}

void TypeSafetyChecker::registerVariable(const std::string& variableName) {
    declaredVariables.insert(variableName);
}

void TypeSafetyChecker::reset() {
    declaredTypes.clear();
    declaredFunctions.clear();
    declaredVariables.clear();
    
    // Register built-in types
    declaredTypes.insert("int");
    declaredTypes.insert("long");
    declaredTypes.insert("float");
    declaredTypes.insert("double");
    declaredTypes.insert("bool");
    declaredTypes.insert("string");
    declaredTypes.insert("void");
}

bool TypeSafetyChecker::isBuiltinType(const std::string& typeName) {
    static std::unordered_set<std::string> builtins = {
        "int", "long", "float", "double", "bool", "string", "void", 
        "const char*", "int*", "long*", "float*", "double*"
    };
    return builtins.count(typeName) > 0;
}

bool TypeSafetyChecker::isBuiltinFunction(const std::string& functionName) {
    static std::unordered_set<std::string> builtins = {
        "print", "printf", "range"
    };
    return builtins.count(functionName) > 0;
}