
#pragma once
#include <memory>
#include <vector>
#include <string>

// Forward declarations
class ASTNode;
class ExprNode;
class StmtNode;
class TypeNode;

using ASTNodePtr = std::unique_ptr<ASTNode>;
using ExprNodePtr = std::unique_ptr<ExprNode>;
using StmtNodePtr = std::unique_ptr<StmtNode>;
using TypeNodePtr = std::unique_ptr<TypeNode>;

// Base AST Node
class ASTNode {
public:
    virtual ~ASTNode() = default;
};

// Type nodes
class TypeNode : public ASTNode {
public:
    virtual std::string toCType() const = 0;
};

class BasicTypeNode : public TypeNode {
public:
    std::string typeName;
    
    explicit BasicTypeNode(const std::string& name) : typeName(name) {}
    std::string toCType() const override;
};

class PointerTypeNode : public TypeNode {
public:
    TypeNodePtr baseType;
    
    explicit PointerTypeNode(TypeNodePtr base) : baseType(std::move(base)) {}
    std::string toCType() const override;
};

class ArrayTypeNode : public TypeNode {
public:
    TypeNodePtr elementType;
    ExprNodePtr size; // nullptr if size is inferred
    
    ArrayTypeNode(TypeNodePtr elem, ExprNodePtr s = nullptr) 
        : elementType(std::move(elem)), size(std::move(s)) {}
    std::string toCType() const override;
};

class StructTypeNode : public TypeNode {
public:
    std::string structName;
    
    explicit StructTypeNode(const std::string& name) : structName(name) {}
    std::string toCType() const override;
};
// Expression nodes
class ExprNode : public ASTNode {
public:
    virtual ~ExprNode() = default;
};

class IntLiteralNode : public ExprNode {
public:
    int value;
    explicit IntLiteralNode(int val) : value(val) {}
};

class LongLiteralNode : public ExprNode {
public:
    long value;
    explicit LongLiteralNode(long val) : value(val) {}
};

class FloatLiteralNode : public ExprNode {
public:
    float value;
    explicit FloatLiteralNode(float val) : value(val) {}
};

class DoubleLiteralNode : public ExprNode {
public:
    double value;
    explicit DoubleLiteralNode(double val) : value(val) {}
};

class StringLiteralNode : public ExprNode {
public:
    std::string value;
    explicit StringLiteralNode(const std::string& val) : value(val) {}
};

class BoolLiteralNode : public ExprNode {
public:
    bool value;
    explicit BoolLiteralNode(bool val) : value(val) {}
};

class IdentifierNode : public ExprNode {
public:
    std::string name;
    explicit IdentifierNode(const std::string& n) : name(n) {}
};

class ArrayLiteralNode : public ExprNode {
public:
    std::vector<ExprNodePtr> elements;
    explicit ArrayLiteralNode(std::vector<ExprNodePtr> elems) 
        : elements(std::move(elems)) {}
};

class IndexNode : public ExprNode {
public:
    ExprNodePtr array;
    ExprNodePtr index;
    
    IndexNode(ExprNodePtr arr, ExprNodePtr idx)
        : array(std::move(arr)), index(std::move(idx)) {}
};

class BinaryOpNode : public ExprNode {
public:
    ExprNodePtr left;
    ExprNodePtr right;
    std::string op;
    
    BinaryOpNode(ExprNodePtr l, ExprNodePtr r, const std::string& o)
        : left(std::move(l)), right(std::move(r)), op(o) {}
};

class UnaryOpNode : public ExprNode {
public:
    ExprNodePtr operand;
    std::string op;
    
    UnaryOpNode(ExprNodePtr o, const std::string& operation)
        : operand(std::move(o)), op(operation) {}
};

class CallNode : public ExprNode {
public:
    std::string functionName;
    std::vector<ExprNodePtr> arguments;
    
    CallNode(const std::string& name, std::vector<ExprNodePtr> args)
        : functionName(name), arguments(std::move(args)) {}
};

class AddressOfNode : public ExprNode {
public:
    ExprNodePtr operand;
    explicit AddressOfNode(ExprNodePtr op) : operand(std::move(op)) {}
};

class DereferenceNode : public ExprNode {
public:
    ExprNodePtr operand;
    explicit DereferenceNode(ExprNodePtr op) : operand(std::move(op)) {}
};

class FieldAccessNode : public ExprNode {
public:
    ExprNodePtr object;
    std::string fieldName;
    
    FieldAccessNode(ExprNodePtr obj, const std::string& field)
        : object(std::move(obj)), fieldName(field) {}
};

class StructInitNode : public ExprNode {
public:
    std::string structName;
    std::vector<std::pair<std::string, ExprNodePtr>> fields; // field name -> value
    
    StructInitNode(const std::string& name, std::vector<std::pair<std::string, ExprNodePtr>> f)
        : structName(name), fields(std::move(f)) {}
};

class UnionInitNode : public ExprNode {
public:
    std::string unionName;
    std::string activeMember; // which member is being initialized
    ExprNodePtr value;
    
    UnionInitNode(const std::string& name, const std::string& member, ExprNodePtr val)
        : unionName(name), activeMember(member), value(std::move(val)) {}
};

class MethodCallNode : public ExprNode {
public:
    ExprNodePtr receiver;
    std::string methodName;
    std::vector<ExprNodePtr> arguments;
    
    MethodCallNode(ExprNodePtr rec, const std::string& name, std::vector<ExprNodePtr> args)
        : receiver(std::move(rec)), methodName(name), arguments(std::move(args)) {}
};

// Statement nodes
class StmtNode : public ASTNode {
public:
    virtual ~StmtNode() = default;
};

class ExprStmtNode : public StmtNode {
public:
    ExprNodePtr expr;
    explicit ExprStmtNode(ExprNodePtr e) : expr(std::move(e)) {}
};

class VarDeclNode : public StmtNode {
public:
    bool isConst;
    std::string name;
    TypeNodePtr type;
    ExprNodePtr initializer;
    
    VarDeclNode(bool c, const std::string& n, TypeNodePtr t, ExprNodePtr init)
        : isConst(c), name(n), type(std::move(t)), initializer(std::move(init)) {}
};

class AssignmentNode : public StmtNode {
public:
    ExprNodePtr target;
    ExprNodePtr value;
    
    AssignmentNode(ExprNodePtr t, ExprNodePtr v)
        : target(std::move(t)), value(std::move(v)) {}
};

class BlockNode : public StmtNode {
public:
    std::vector<StmtNodePtr> statements;
    
    explicit BlockNode(std::vector<StmtNodePtr> stmts) 
        : statements(std::move(stmts)) {}
};

class ReturnNode : public StmtNode {
public:
    ExprNodePtr value;
    explicit ReturnNode(ExprNodePtr val = nullptr) : value(std::move(val)) {}
};

class IfNode : public StmtNode {
public:
    ExprNodePtr condition;
    StmtNodePtr thenBranch;
    StmtNodePtr elseBranch;
    
    IfNode(ExprNodePtr cond, StmtNodePtr then, StmtNodePtr els = nullptr)
        : condition(std::move(cond)), thenBranch(std::move(then)), 
          elseBranch(std::move(els)) {}
};

class WhileNode : public StmtNode {
public:
    ExprNodePtr condition;
    StmtNodePtr body;
    
    WhileNode(ExprNodePtr cond, StmtNodePtr b)
        : condition(std::move(cond)), body(std::move(b)) {}
};

class ForNode : public StmtNode {
public:
    std::string iteratorName;
    ExprNodePtr collection;
    StmtNodePtr body;
    
    ForNode(const std::string& iter, ExprNodePtr coll, StmtNodePtr b)
        : iteratorName(iter), collection(std::move(coll)), body(std::move(b)) {}
};

struct StructField {
    std::string name;
    TypeNodePtr type;
    
    StructField(const std::string& n, TypeNodePtr t) 
        : name(n), type(std::move(t)) {}
};

class StructDefNode : public StmtNode {
public:
    std::string name;
    std::vector<StructField> fields;
    
    StructDefNode(const std::string& n, std::vector<StructField> f)
        : name(n), fields(std::move(f)) {}
};

class UnionDefNode : public StmtNode {
public:
    std::string name;
    std::vector<StructField> fields; // Reuse StructField for union members
    
    UnionDefNode(const std::string& n, std::vector<StructField> f)
        : name(n), fields(std::move(f)) {}
};

// Function and program nodes
class FunctionNode : public ASTNode {
public:
    std::string name;
    std::vector<std::pair<std::string, TypeNodePtr>> parameters;
    TypeNodePtr returnType;
    StmtNodePtr body;
    
    FunctionNode(const std::string& n, 
                 std::vector<std::pair<std::string, TypeNodePtr>> params,
                 TypeNodePtr ret,
                 StmtNodePtr b)
        : name(n), parameters(std::move(params)), 
          returnType(std::move(ret)), body(std::move(b)) {}
};

enum class ReceiverType {
    Value,    // Point
    Pointer,  // *Point  
    Reference // &Point
};

class ImplBlockNode : public StmtNode {
public:
    ReceiverType receiverType;
    std::string structName;
    std::vector<std::unique_ptr<FunctionNode>> methods;
    
    ImplBlockNode(ReceiverType type, const std::string& name, std::vector<std::unique_ptr<FunctionNode>> m)
        : receiverType(type), structName(name), methods(std::move(m)) {}
};

class ProgramNode : public ASTNode {
public:
    std::vector<std::unique_ptr<FunctionNode>> functions;
    std::vector<StmtNodePtr> globalDeclarations;
    std::vector<std::unique_ptr<StructDefNode>> structs;
    std::vector<std::unique_ptr<UnionDefNode>> unions;
    std::vector<std::unique_ptr<ImplBlockNode>> implBlocks;
};