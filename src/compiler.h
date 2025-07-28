#pragma once
#include <string>
#include <vector>
#include <memory>
#include "lexer.h"
#include "parser.h"
#include "codegen.h"

class PeachCompiler {
private:
    std::vector<std::string> generatedCFiles;
    bool verbose;
    
public:
    PeachCompiler() : verbose(false) {}
    
    void setVerbose(bool v) { verbose = v; }
    void compile(const std::string& filename);
    std::string generateCSource(const std::string& filename);
    std::string compileToObject(const std::string& filename);
    void generateExecutable(const std::string& outputName);
};