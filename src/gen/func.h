#pragma once
#include "base.h"
#include "stmt.h"
#include "type.h"
#include "../ast.h"

class FuncGenerator : public CodeGenBase {
private:
    StmtGenerator stmtGen;
    
public:
    FuncGenerator(std::stringstream& out, int& indent) 
        : CodeGenBase(out, indent), stmtGen(out, indent) {}
    
    void generate(FunctionNode* node);
    
private:
    void generateSignature(FunctionNode* node);
    void generateParameters(const std::vector<std::pair<std::string, TypeNodePtr>>& params);
    void generateBody(FunctionNode* node);
    std::string inferReturnType(StmtNode* body);
    std::string inferReturnTypeWithContext(StmtNode* body, const std::vector<std::pair<std::string, TypeNodePtr>>& parameters);
};