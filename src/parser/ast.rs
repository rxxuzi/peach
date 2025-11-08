// Abstract Syntax Tree definitions

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Span { line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Void,  // New!
}

impl Type {
    pub fn to_string(&self) -> String {
        match self {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
        }
    }

    pub fn to_c_type(&self) -> String {
        match self {
            Type::I8 => "int8_t".to_string(),
            Type::I16 => "int16_t".to_string(),
            Type::I32 => "int32_t".to_string(),
            Type::I64 => "int64_t".to_string(),
            Type::U8 => "uint8_t".to_string(),
            Type::U16 => "uint16_t".to_string(),
            Type::U32 => "uint32_t".to_string(),
            Type::U64 => "uint64_t".to_string(),
            Type::F32 => "float".to_string(),
            Type::F64 => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Return(ReturnStatement),
    VarDecl(VarDeclStatement),
    Assignment(AssignmentStatement),
    If(IfStatement),
    While(WhileStatement),
    Loop(LoopStatement),
    For(ForStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Expression(Expression),  // Expression statement (for function calls)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclStatement {
    pub is_mutable: bool,  // true for `var`, false for `val`
    pub name: String,
    pub var_type: Option<Type>,  // None for type inference
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStatement {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    Variable(String),
    Binary(BinaryExpression),
    Unary(UnaryExpression),      // New!
    Call(CallExpression),         // New!
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,        // +
    Subtract,   // -
    Multiply,   // *
    Divide,     // /
    Modulo,     // %
    // Comparison
    Equal,          // ==
    NotEqual,       // !=
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    // Logical
    And,        // &&
    Or,         // ||
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,        // !
    Negate,     // -
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    pub callee: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStatement {
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub variable: String,
    pub iterable: ForIterable,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForIterable {
    Range(Box<Expression>, Box<Expression>),  // start..end
    Array(Vec<Expression>),  // [1, 2, 3, 4]
}

#[derive(Debug, Clone, PartialEq)]
pub struct BreakStatement {}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStatement {}