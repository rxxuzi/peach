#pragma once
#include <unordered_map>
#include <string>

// Simple symbol table for tracking variable and parameter types
class SymbolTable {
private:
    std::unordered_map<std::string, std::string> symbols;
    
public:
    // Add a symbol with its type
    void addSymbol(const std::string& name, const std::string& type);
    
    // Look up a symbol's type
    std::string getSymbolType(const std::string& name) const;
    
    // Check if a symbol exists
    bool hasSymbol(const std::string& name) const;
    
    // Clear all symbols (for new scope)
    void clear();
    
    // Copy constructor for scope management
    SymbolTable(const SymbolTable& parent);
    
    // Default constructor
    SymbolTable() = default;
};