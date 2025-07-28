#pragma once
#include "base.h"
#include "stmt.h"
#include "type.h"
#include "symbol_table.h"
#include "../ast.h"
#include "../type_registry.h"

class FuncGenerator : public CodeGenBase {
private:
    TypeRegistry* typeRegistry;
    
public:
    FuncGenerator(std::stringstream& out, int& indent, TypeRegistry* types = nullptr) 
        : CodeGenBase(out, indent), typeRegistry(types) {}
    
    void generate(FunctionNode* node);
    void generateBody(FunctionNode* node);
    
private:
    void generateSignature(FunctionNode* node);
    void generateParameters(const std::vector<std::pair<std::string, TypeNodePtr>>& params);
    std::string inferReturnType(StmtNode* body);
    std::string inferReturnTypeWithContext(StmtNode* body, const std::vector<std::pair<std::string, TypeNodePtr>>& parameters);
};