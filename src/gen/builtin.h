#pragma once
#include "base.h"
#include "../usage_tracker.h"

class BuiltinGenerator : public CodeGenBase {
private:
    const UsageTracker& usage;
    
public:
    BuiltinGenerator(std::stringstream& out, int& indent, const UsageTracker& tracker) 
        : CodeGenBase(out, indent), usage(tracker) {}
    
    void generateAll();
    
private:
    void generateIncludes();
    void generateRangeStructs();
    void generatePrintFunctions();
    void generateUtilityMacros();
};