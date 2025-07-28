#include "builtin.h"
#include <vector>

void BuiltinGenerator::generateAll() {
    generateIncludes();
    
    if (usage.isRangeUsed()) {
        generateRangeStructs();
    }
    
    if (usage.isPrintUsed()) {
        generatePrintFunctions();
    }
    
    if (usage.isLenUsed() || usage.isSizeofUsed()) {
        generateUtilityMacros();
    }
    
}

void BuiltinGenerator::generateIncludes() {
    emitLine("#include <stdio.h>");
    emitLine("#include <stdlib.h>");
    emitLine("#include <string.h>");
    emitLine("#include <stdbool.h>");
    emitLine("");
}

void BuiltinGenerator::generateRangeStructs() {
    emitLine("// Range iterator structure");
    emitLine("typedef struct {");
    emitLine("    int current;");
    emitLine("    int stop;");
    emitLine("    int step;");
    emitLine("} Range;");
    emitLine("");
    
    emitLine("// Range constructor functions");
    emitLine("static Range range1(int stop) {");
    emitLine("    Range r = {0, stop, 1};");
    emitLine("    return r;");
    emitLine("}");
    emitLine("");
    
    emitLine("static Range range2(int start, int stop) {");
    emitLine("    Range r = {start, stop, 1};");
    emitLine("    return r;");
    emitLine("}");
    emitLine("");
    
    emitLine("static Range range3(int start, int stop, int step) {");
    emitLine("    Range r = {start, stop, step};");
    emitLine("    return r;");
    emitLine("}");
    emitLine("");
}

void BuiltinGenerator::generatePrintFunctions() {
    emitLine("// Print functions for different types");
    
    // Generate specific print functions for each type
    const auto& types = usage.getUsedTypes();
    
    if (types.find("int") != types.end() || types.empty()) {
        emitLine("static void print_int(int x) { printf(\"%d\\n\", x); }");
    }
    if (types.find("long") != types.end()) {
        emitLine("static void print_long(long x) { printf(\"%ld\\n\", x); }");
    }
    if (types.find("float") != types.end()) {
        emitLine("static void print_float(float x) { printf(\"%.6f\\n\", x); }");
    }
    if (types.find("double") != types.end()) {
        emitLine("static void print_double(double x) { printf(\"%.6f\\n\", x); }");
    }
    if (types.find("string") != types.end() || types.empty()) {
        emitLine("static void print_string(const char* x) { printf(\"%s\\n\", x); }");
    }
    if (types.find("bool") != types.end()) {
        emitLine("static void print_bool(_Bool x) { printf(\"%s\\n\", x ? \"true\" : \"false\"); }");
    }
    emitLine("");
    
    // Generic print macro using C11 _Generic
    emitLine("// Generic print macro using _Generic (C11)");
    emitLine("#define print(x) _Generic((x), \\");
    
    const auto& allTypes = usage.getUsedTypes();
    std::vector<std::string> entries;
    
    if (allTypes.find("int") != allTypes.end() || allTypes.empty()) {
        entries.push_back("    int: print_int");
    }
    if (allTypes.find("long") != allTypes.end()) {
        entries.push_back("    long: print_long");
    }
    if (allTypes.find("float") != allTypes.end()) {
        entries.push_back("    float: print_float");
    }
    if (allTypes.find("double") != allTypes.end()) {
        entries.push_back("    double: print_double");
    }
    if (allTypes.find("string") != allTypes.end() || allTypes.empty()) {
        entries.push_back("    char*: print_string");
        entries.push_back("    const char*: print_string");
    }
    if (allTypes.find("bool") != allTypes.end()) {
        entries.push_back("    _Bool: print_bool");
    }
    
    // Generate macro entries
    for (size_t i = 0; i < entries.size(); i++) {
        if (i < entries.size() - 1) {
            emitLine(entries[i] + ", \\");
        } else {
            emitLine(entries[i] + ", \\");
        }
    }
    emitLine("    default: print_int \\");
    emitLine(")(x)");
    emitLine("");
}

void BuiltinGenerator::generateUtilityMacros() {
    if (usage.isLenUsed()) {
        emitLine("// Array length macro");
        emitLine("#define len(arr) (sizeof(arr) / sizeof((arr)[0]))");
        emitLine("");
    }
}

