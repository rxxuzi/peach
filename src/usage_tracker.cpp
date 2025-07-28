#include "usage_tracker.h"

void UsageTracker::trackFunction(const std::string& name) {
    usedFunctions.insert(name);
    
    if (name == "range" || name == "range1" || name == "range2" || name == "range3") {
        usesRange = true;
    } else if (name == "print") {
        usesPrint = true;
    } else if (name == "len") {
        usesLen = true;
    } else if (name == "sizeof") {
        usesSizeof = true;
    }
}

void UsageTracker::trackType(const std::string& type) {
    usedTypes.insert(type);
}