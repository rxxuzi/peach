#include <iostream>
#include <fstream>
#include <vector>
#include <string>
#include <cstdlib>
#include <chrono>
#include <iomanip>
#include <getopt.h>
#include "compiler.h"
#include "security/type_safety.h"
#include "security/memory_safety.h"

void printUsage(const std::string& programName) {
    std::cout << "Usage: " << programName << " [options] <source.peach> [source2.peach ...]\n";
    std::cout << "\nOptions:\n";
    std::cout << "  -h, --help          Show this help message\n";
    std::cout << "  -o, --output FILE   Specify output file name\n";
    std::cout << "  -s, --source        Generate C source file only (don't compile to executable)\n";
    std::cout << "  -c, --compile       Compile to object file only (don't link)\n";
    std::cout << "  -E, --preprocess    Run preprocessor only (not implemented yet)\n";
    std::cout << "  -v, --verbose       Enable verbose output\n";
}

int main(int argc, char* argv[]) {
    std::string outputName;
    bool generateSourceOnly = false;
    bool compileToObjectOnly = false;
    bool verbose = false;
    
    // Parse command line options
    static struct option long_options[] = {
        {"help",         no_argument,       0, 'h'},
        {"output",       required_argument, 0, 'o'},
        {"source",       no_argument,       0, 's'},
        {"compile",      no_argument,       0, 'c'},
        {"preprocess",   no_argument,       0, 'E'},
        {"verbose",      no_argument,       0, 'v'},
        {0, 0, 0, 0}
    };
    
    int opt;
    int option_index = 0;
    while ((opt = getopt_long(argc, argv, "ho:scEv", long_options, &option_index)) != -1) {
        switch (opt) {
            case 'h':
                printUsage(argv[0]);
                return 0;
            case 'o':
                outputName = optarg;
                break;
            case 's':
                generateSourceOnly = true;
                break;
            case 'c':
                compileToObjectOnly = true;
                break;
            case 'E':
                std::cerr << "Error: Preprocessing (-E) is not implemented yet\n";
                return 1;
            case 'v':
                verbose = true;
                break;
            default:
                printUsage(argv[0]);
                return 1;
        }
    }
    
    // Check for conflicting options
    if (generateSourceOnly && compileToObjectOnly) {
        std::cerr << "Error: Cannot use -s and -c together\n";
        return 1;
    }
    
    // Get source files
    if (optind >= argc) {
        std::cerr << "Error: No source files specified\n";
        printUsage(argv[0]);
        return 1;
    }
    
    std::vector<std::string> sourceFiles;
    for (int i = optind; i < argc; i++) {
        sourceFiles.push_back(argv[i]);
    }
    
    try {
        auto startTime = std::chrono::high_resolution_clock::now();
        
        PeachCompiler compiler;
        compiler.setVerbose(verbose);
        
        if (generateSourceOnly) {
            // Generate C source files only
            for (const auto& file : sourceFiles) {
                if (verbose) std::cout << "Translating " << file << " to C...\n";
                
                std::string cFileName = compiler.generateCSource(file);
                
                // If output name is specified and there's only one source file,
                // rename the generated C file
                if (!outputName.empty() && sourceFiles.size() == 1) {
                    std::string newName = outputName;
                    // Ensure it ends with .c
                    if (newName.length() < 2 || newName.substr(newName.length() - 2) != ".c") {
                        newName += ".c";
                    }
                    std::rename(cFileName.c_str(), newName.c_str());
                    std::cout << "Generated: " << newName << "\n";
                } else {
                    std::cout << "Generated: " << cFileName << "\n";
                }
            }
        } else if (compileToObjectOnly) {
            // Compile to object files only
            for (const auto& file : sourceFiles) {
                if (verbose) std::cout << "Compiling " << file << " to object file...\n";
                
                std::string objFileName = compiler.compileToObject(file);
                
                // If output name is specified and there's only one source file,
                // rename the object file
                if (!outputName.empty() && sourceFiles.size() == 1) {
                    std::string newName = outputName;
                    // Ensure it ends with .o
                    if (newName.length() < 2 || newName.substr(newName.length() - 2) != ".o") {
                        newName += ".o";
                    }
                    std::rename(objFileName.c_str(), newName.c_str());
                    std::cout << "Generated: " << newName << "\n";
                } else {
                    std::cout << "Generated: " << objFileName << "\n";
                }
            }
        } else {
            // Compile all source files to executable
            for (const auto& file : sourceFiles) {
                if (verbose) std::cout << "Compiling " << file << "...\n";
                compiler.compile(file);
            }
            
            // Generate executable
            if (outputName.empty()) {
                outputName = "a.out";
            }
            compiler.generateExecutable(outputName);
            
            if (verbose) {
                auto endTime = std::chrono::high_resolution_clock::now();
                auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(endTime - startTime);
                std::cout << "Compilation completed in " << duration.count() << "ms\n";
            }
            std::cout << "Compilation successful! Output: " << outputName << "\n";
        }
        
    } catch (const std::exception& e) {
        std::cerr << "Compilation error: " << e.what() << "\n";
        return 1;
    } catch (...) {
        std::cerr << "Unknown error during compilation\n";
        return 1;
    }
    
    return 0;
}