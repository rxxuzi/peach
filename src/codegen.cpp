#include "codegen.h"
#include "gen/builtin.h"
#include "gen/func.h"
#include "gen/stmt.h"
#include <stdexcept>

CodeGenerator::CodeGenerator() : indentLevel(0) {}

std::string CodeGenerator::generate(std::unique_ptr<ProgramNode>& ast) {
    output.str("");
    output.clear();
    
    // First pass: build type registry
    buildTypeRegistry(ast.get());
    
    // Second pass: analyze usage
    analyzeUsage(ast.get());
    
    // Generate built-in functions and includes
    BuiltinGenerator builtinGen(output, indentLevel, usageTracker);
    builtinGen.generateAll();
    
    // Generate the program
    generateProgram(ast.get());
    
    return output.str();
}

void CodeGenerator::generateProgram(ProgramNode* node) {
    // Create generators with type registry support
    SymbolTable globalSymbols;
    TypeGenerator typeGen(output, indentLevel, &globalSymbols, &typeRegistry);
    StmtGenerator stmtGen(output, indentLevel, &typeRegistry);
    ExprGenerator exprGen(output, indentLevel, &globalSymbols, &typeRegistry);
    
    // Generate struct definitions first
    for (auto& structDef : node->structs) {
        generateStruct(structDef.get());
        output << "\n";
    }
    
    // Generate union definitions
    for (auto& unionDef : node->unions) {
        generateUnion(unionDef.get());
        output << "\n";
    }
    
    // Generate enum definitions
    for (auto& enumDef : node->enums) {
        generateEnum(enumDef.get());
        output << "\n";
    }
    
    // Generate global declarations
    for (auto& decl : node->globalDeclarations) {
        stmtGen.generate(decl.get());
        output << ";\n";
    }
    
    if (!node->globalDeclarations.empty()) {
        output << "\n";
    }
    
    // Generate methods from impl blocks first (before functions that might use them)
    FuncGenerator funcGen(output, indentLevel, &typeRegistry);
    
    for (auto& implBlock : node->implBlocks) {
        generateImplBlock(implBlock.get(), funcGen);
        output << "\n";
    }
    
    // Generate regular functions
    for (auto& func : node->functions) {
        funcGen.generate(func.get());
        output << "\n";
    }
}

void CodeGenerator::analyzeUsage(ProgramNode* node) {
    // Analyze global declarations
    for (auto& decl : node->globalDeclarations) {
        analyzeStatement(decl.get());
    }
    
    // Analyze functions
    for (auto& func : node->functions) {
        analyzeFunction(func.get());
    }
}

void CodeGenerator::analyzeFunction(FunctionNode* node) {
    // Analyze function body
    analyzeStatement(node->body.get());
}

void CodeGenerator::analyzeStatement(StmtNode* node) {
    if (auto* block = dynamic_cast<BlockNode*>(node)) {
        for (auto& stmt : block->statements) {
            analyzeStatement(stmt.get());
        }
    } else if (auto* exprStmt = dynamic_cast<ExprStmtNode*>(node)) {
        analyzeExpression(exprStmt->expr.get());
    } else if (auto* varDecl = dynamic_cast<VarDeclNode*>(node)) {
        if (varDecl->initializer) {
            analyzeExpression(varDecl->initializer.get());
            
            // Register variable type in type registry
            if (varDecl->type) {
                typeRegistry.registerVariable(varDecl->name, varDecl->type->toCType());
            } else {
                // Infer type from initializer
                TypeGenerator typeGen(output, indentLevel, nullptr, &typeRegistry);
                std::string inferredType = typeGen.inferType(varDecl->initializer.get());
                typeRegistry.registerVariable(varDecl->name, inferredType);
            }
        }
        if (varDecl->type) {
            // Track type usage
            if (auto* basicType = dynamic_cast<BasicTypeNode*>(varDecl->type.get())) {
                usageTracker.trackType(basicType->typeName);
            }
        }
    } else if (auto* ifNode = dynamic_cast<IfNode*>(node)) {
        analyzeExpression(ifNode->condition.get());
        analyzeStatement(ifNode->thenBranch.get());
        if (ifNode->elseBranch) {
            analyzeStatement(ifNode->elseBranch.get());
        }
    } else if (auto* whileNode = dynamic_cast<WhileNode*>(node)) {
        analyzeExpression(whileNode->condition.get());
        analyzeStatement(whileNode->body.get());
    } else if (auto* forNode = dynamic_cast<ForNode*>(node)) {
        analyzeExpression(forNode->collection.get());
        analyzeStatement(forNode->body.get());
    } else if (auto* returnNode = dynamic_cast<ReturnNode*>(node)) {
        if (returnNode->value) {
            analyzeExpression(returnNode->value.get());
        }
    }
}

void CodeGenerator::analyzeExpression(ExprNode* node) {
    if (auto* call = dynamic_cast<CallNode*>(node)) {
        usageTracker.trackFunction(call->functionName);
        for (auto& arg : call->arguments) {
            analyzeExpression(arg.get());
        }
    } else if (auto* binOp = dynamic_cast<BinaryOpNode*>(node)) {
        analyzeExpression(binOp->left.get());
        analyzeExpression(binOp->right.get());
    } else if (auto* unaryOp = dynamic_cast<UnaryOpNode*>(node)) {
        analyzeExpression(unaryOp->operand.get());
    } else if (auto* indexNode = dynamic_cast<IndexNode*>(node)) {
        analyzeExpression(indexNode->array.get());
        analyzeExpression(indexNode->index.get());
    } else if (auto* addrOf = dynamic_cast<AddressOfNode*>(node)) {
        analyzeExpression(addrOf->operand.get());
    } else if (auto* deref = dynamic_cast<DereferenceNode*>(node)) {
        analyzeExpression(deref->operand.get());
    } else if (auto* arrayLit = dynamic_cast<ArrayLiteralNode*>(node)) {
        for (auto& elem : arrayLit->elements) {
            analyzeExpression(elem.get());
        }
    } else if (auto* fieldAccess = dynamic_cast<FieldAccessNode*>(node)) {
        analyzeExpression(fieldAccess->object.get());
        // Track that we're accessing fields (might need struct types)
    } else if (auto* structInit = dynamic_cast<StructInitNode*>(node)) {
        for (const auto& field : structInit->fields) {
            analyzeExpression(field.second.get());
        }
    } else if (auto* methodCall = dynamic_cast<MethodCallNode*>(node)) {
        analyzeExpression(methodCall->receiver.get());
        for (auto& arg : methodCall->arguments) {
            analyzeExpression(arg.get());
        }
    } else if (dynamic_cast<DoubleLiteralNode*>(node)) {
        usageTracker.trackType("double");
    } else if (dynamic_cast<FloatLiteralNode*>(node)) {
        usageTracker.trackType("float");
    } else if (dynamic_cast<IntLiteralNode*>(node)) {
        usageTracker.trackType("int");
    } else if (dynamic_cast<LongLiteralNode*>(node)) {
        usageTracker.trackType("long");
    } else if (dynamic_cast<StringLiteralNode*>(node)) {
        usageTracker.trackType("string");
    } else if (dynamic_cast<BoolLiteralNode*>(node)) {
        usageTracker.trackType("bool");
    }
    // Identifiers don't need analysis
}

void CodeGenerator::generateStruct(StructDefNode* node) {
    output << "struct " << node->name << " {\n";
    
    for (const auto& field : node->fields) {
        output << "    " << field.type->toCType() << " " << field.name << ";\n";
    }
    
    output << "};\n";
}

void CodeGenerator::generateUnion(UnionDefNode* node) {
    output << "union " << node->name << " {\n";
    
    for (const auto& field : node->fields) {
        output << "    " << field.type->toCType() << " " << field.name << ";\n";
    }
    
    output << "};\n";
}

void CodeGenerator::generateEnum(EnumDefNode* node) {
    output << "enum " << node->name << " {\n";
    
    for (size_t i = 0; i < node->members.size(); i++) {
        const auto& member = node->members[i];
        output << "    " << member.name;
        
        if (member.value) {
            output << " = ";
            // Generate the expression for the enum value
            ExprGenerator exprGen(output, indentLevel);
            exprGen.generate(member.value.get());
        }
        
        if (i < node->members.size() - 1) {
            output << ",";
        }
        output << "\n";
    }
    
    output << "};\n";
}

void CodeGenerator::generateImplBlock(ImplBlockNode* node, FuncGenerator& funcGen) {
    // Generate methods with special naming convention and receiver parameter
    for (auto& method : node->methods) {
        // Create method name: __StructName_methodName format
        std::string methodName = "__" + node->structName + "_" + method->name;
        
        // Add suffix for pointer receiver
        if (node->receiverType == ReceiverType::Pointer) {
            methodName += "_p";
        }
        
        // Generate function signature
        std::string returnType = method->returnType ? method->returnType->toCType() : "void";
        output << returnType << " " << methodName << "(";
        
        // Add receiver parameter first
        if (node->receiverType == ReceiverType::Value) {
            output << "struct " << node->structName << " self";
        } else if (node->receiverType == ReceiverType::Pointer) {
            output << "struct " << node->structName << "* self";
        } else { // Reference
            output << "struct " << node->structName << "* self";
        }
        
        // Add other parameters
        for (const auto& param : method->parameters) {
            output << ", " << param.second->toCType() << " " << param.first;
        }
        
        output << ") ";
        
        // Generate function body using the existing function generator
        funcGen.generateBody(method.get());
        output << "\n";
    }
}

void CodeGenerator::buildTypeRegistry(ProgramNode* node) {
    // Clear previous type information
    typeRegistry.clear();
    
    // Register structs and their fields
    for (const auto& structDef : node->structs) {
        typeRegistry.registerStruct(structDef->name);
        
        for (const auto& field : structDef->fields) {
            typeRegistry.addStructField(structDef->name, field.name, field.type->toCType());
        }
    }
    
    // Register unions
    for (const auto& unionDef : node->unions) {
        typeRegistry.registerStruct(unionDef->name); // Use same registration for unions
        
        for (const auto& field : unionDef->fields) {
            typeRegistry.addStructField(unionDef->name, field.name, field.type->toCType());
        }
    }
    
    // Register enums (treat them as basic types)
    for (const auto& enumDef : node->enums) {
        typeRegistry.registerStruct(enumDef->name); // Register enum as a type
    }
    
    // Register methods from impl blocks
    for (const auto& implBlock : node->implBlocks) {
        for (const auto& method : implBlock->methods) {
            std::vector<std::string> paramTypes;
            for (const auto& param : method->parameters) {
                paramTypes.push_back(param.second->toCType());
            }
            
            std::string returnType = method->returnType ? method->returnType->toCType() : "void";
            MethodInfo methodInfo(method->name, returnType, paramTypes, 
                                  implBlock->receiverType == ReceiverType::Pointer);
            
            typeRegistry.addStructMethod(implBlock->structName, methodInfo);
        }
    }
}