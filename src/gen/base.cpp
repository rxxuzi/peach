#include "base.h"

void CodeGenBase::indent() {
    for (int i = 0; i < indentLevel; i++) {
        output << "    ";
    }
}

void CodeGenBase::emit(const std::string& code) {
    output << code;
}

void CodeGenBase::emitLine(const std::string& line) {
    indent();
    output << line << "\n";
}