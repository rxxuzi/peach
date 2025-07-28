#pragma once
#include <unordered_map>
#include <unordered_set>
#include <string>
#include <memory>
#include <vector>

// Memory safety analyzer for detecting potential memory issues
class MemorySafetyAnalyzer {
private:
    // Track variable lifetimes and ownership
    std::unordered_map<std::string, bool> variableInitialized;
    std::unordered_set<std::string> danglingPointers;
    std::unordered_map<std::string, std::string> pointerTargets;
    
public:
    struct MemoryIssue {
        enum Type {
            UNINITIALIZED_USE,
            DANGLING_POINTER,
            DOUBLE_FREE,
            MEMORY_LEAK,
            BUFFER_OVERFLOW
        };
        
        Type type;
        std::string message;
        std::string variableName;
        int line;
        int column;
        
        MemoryIssue(Type t, const std::string& msg, const std::string& var = "", int l = 0, int c = 0)
            : type(t), message(msg), variableName(var), line(l), column(c) {}
    };
    
    std::vector<MemoryIssue> analyzeProgram(class ProgramNode* program);
    std::vector<MemoryIssue> analyzeFunction(class FunctionNode* function);
    std::vector<MemoryIssue> analyzeStatement(class StmtNode* statement);
    std::vector<MemoryIssue> analyzeExpression(class ExprNode* expression);
    
    // Variable tracking
    void markVariableInitialized(const std::string& varName);
    void markVariableUninitialized(const std::string& varName);
    bool isVariableInitialized(const std::string& varName) const;
    
    // Pointer tracking
    void trackPointer(const std::string& pointerName, const std::string& target);
    void markPointerDangling(const std::string& pointerName);
    bool isPointerDangling(const std::string& pointerName) const;
    
    // Reset for new analysis
    void reset();
};