#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub extern_functions: Vec<ExternFunction>,
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub global_variables: Vec<GlobalVariable>,
}

#[derive(Debug, Clone)]
pub struct GlobalVariable {
    pub name: String,
    pub var_type: Option<Type>,
    pub mutable: bool,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    Char,
    String,
    Array(Box<Type>),
    DynamicArray(Box<Type>),
    Pointer(Box<Type>),
    Struct(String),
    Enum(String),  // Enum type by name
    Void,
    // Generic type with type parameters (e.g., Option<int>, Result<int, string>)
    Generic {
        name: String,
        type_params: Vec<Type>,
    },
    // Type parameter placeholder (e.g., T in fn foo<T>(x: T))
    TypeParam(String),
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,  // Explicit value if specified
}

#[derive(Debug, Clone)]
pub struct ExternFunction {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub variadic: bool,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub module: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Export {
    pub item: ExportItem,
}

#[derive(Debug, Clone)]
pub enum ExportItem {
    Function(String), // function name
    Struct(String),   // struct name
    Enum(String),     // enum name
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        var_type: Option<Type>,
        mutable: bool,
        initializer: Option<Expression>,
    },
    Const {
        name: String,
        var_type: Option<Type>,
        initializer: Expression,
    },
    Assignment {
        target: Expression,
        value: Expression,
    },
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Break,
    Continue,
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    DynamicArrayLiteral {
        element_type: Box<Type>,
        elements: Vec<Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    StructAccess {
        object: Box<Expression>,
        field: String,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },
    New(Box<Expression>),
    Delete(Box<Expression>),
    Cast {
        expression: Box<Expression>,
        target_type: Type,
    },
    Ternary {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
    EnumAccess {
        enum_name: String,
        variant: String,
    },
    Match {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    TryOperator {
        expression: Box<Expression>,
    },
    InterpolatedString {
        parts: Vec<StringPart>,  // Alternating text and expressions
    },
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Text(String),           // Plain text between interpolations
    Interpolation(Expression),  // Expression to interpolate like :name:
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,                              // _
    Literal(Literal),                      // 42, 'a', "str"
    EnumVariant { 
        enum_name: String, 
        variant: String,
        binding: Option<String>,           // Option::Some(x) - the 'x' part
    }, // TokenKind::EOF or Option::Some(value)
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
    Dereference,
    AddressOf,
}