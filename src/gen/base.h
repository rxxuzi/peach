#pragma once
#include <string>
#include <sstream>

// Base class for all code generators
class CodeGenBase {
protected:
    std::stringstream& output;
    int& indentLevel;
    
    void indent();
    void emit(const std::string& code);
    void emitLine(const std::string& line);
    
public:
    CodeGenBase(std::stringstream& out, int& indent) 
        : output(out), indentLevel(indent) {}
    virtual ~CodeGenBase() = default;
};