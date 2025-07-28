#pragma once
#include <set>
#include <string>

class UsageTracker {
private:
    std::set<std::string> usedFunctions;
    std::set<std::string> usedTypes;
    bool usesRange;
    bool usesPrint;
    bool usesLen;
    bool usesSizeof;
    
public:
    UsageTracker() : usesRange(false), usesPrint(false), usesLen(false), usesSizeof(false) {}
    
    void trackFunction(const std::string& name);
    void trackType(const std::string& type);
    
    bool isRangeUsed() const { return usesRange; }
    bool isPrintUsed() const { return usesPrint; }
    bool isLenUsed() const { return usesLen; }
    bool isSizeofUsed() const { return usesSizeof; }
    
    const std::set<std::string>& getUsedTypes() const { return usedTypes; }
};