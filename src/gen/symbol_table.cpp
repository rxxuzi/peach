#include "symbol_table.h"

void SymbolTable::addSymbol(const std::string& name, const std::string& type) {
    symbols[name] = type;
}

std::string SymbolTable::getSymbolType(const std::string& name) const {
    auto it = symbols.find(name);
    if (it != symbols.end()) {
        return it->second;
    }
    return ""; // Unknown symbol
}

bool SymbolTable::hasSymbol(const std::string& name) const {
    return symbols.find(name) != symbols.end();
}

void SymbolTable::clear() {
    symbols.clear();
}

SymbolTable::SymbolTable(const SymbolTable& parent) {
    symbols = parent.symbols;
}