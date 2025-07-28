#include "compiler.h"
#include <fstream>
#include <sstream>
#include <iostream>
#include <cstdlib>

std::string PeachCompiler::generateCSource(const std::string& filename) {
    // Read source file
    std::ifstream file(filename);
    if (!file.is_open()) {
        throw std::runtime_error("Cannot open file: " + filename);
    }
    
    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string source = buffer.str();
    file.close();
    
    if (verbose) {
        std::cout << "  Lexical analysis...\n";
    }
    
    // Lexical analysis
    Lexer lexer(source);
    auto tokens = lexer.tokenize();
    
    if (verbose) {
        std::cout << "  Parsing...\n";
    }
    
    // Parsing
    Parser parser(tokens);
    auto ast = parser.parse();
    
    if (verbose) {
        std::cout << "  Code generation...\n";
    }
    
    // Code generation
    CodeGenerator codegen;
    std::string cCode = codegen.generate(ast);
    
    // Write C code to file
    std::string cFilename = filename.substr(0, filename.find_last_of('.')) + ".c";
    std::ofstream cFile(cFilename);
    cFile << cCode;
    cFile.close();
    
    return cFilename;
}

std::string PeachCompiler::compileToObject(const std::string& filename) {
    // First generate C source
    std::string cFilename = generateCSource(filename);
    
    // Compile C to object file
    std::string objFilename = filename.substr(0, filename.find_last_of('.')) + ".o";
    std::string command = "gcc -std=c11 -c -o " + objFilename + " " + cFilename;
    
    if (verbose) {
        std::cout << "  Running: " << command << "\n";
    }
    
    int result = std::system(command.c_str());
    if (result != 0) {
        // Clean up C file
        std::remove(cFilename.c_str());
        throw std::runtime_error("GCC compilation failed");
    }
    
    // Clean up C file
    std::remove(cFilename.c_str());
    
    return objFilename;
}

void PeachCompiler::compile(const std::string& filename) {
    // Generate C source
    std::string cFilename = generateCSource(filename);
    
    // Keep track of generated C files for linking
    generatedCFiles.push_back(cFilename);
}

void PeachCompiler::generateExecutable(const std::string& outputName) {
    if (generatedCFiles.empty()) {
        throw std::runtime_error("No source files compiled");
    }
    
    // Build gcc command with C11 standard
    std::string command = "gcc -std=c11 -o " + outputName;
    for (const auto& cFile : generatedCFiles) {
        command += " " + cFile;
    }
    
    if (verbose) {
        std::cout << "Linking: " << command << "\n";
    }
    
    // Execute gcc
    int result = std::system(command.c_str());
    if (result != 0) {
        throw std::runtime_error("GCC compilation failed");
    }
    
    // Clean up generated C files
    for (const auto& cFile : generatedCFiles) {
        std::remove(cFile.c_str());
    }
}