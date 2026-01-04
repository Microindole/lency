// 简单的 Span 定义 (也就是源码中的起止位置: 0..5)
pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    // 字面量: 1, "hello", true, null
    Literal(Literal),

    // 变量使用: x, count
    Variable(String),

    // 二元操作: a + b, a == b
    Binary(Box<Expr>, BinaryOp, Box<Expr>),

    // 一元操作: -a, !b
    Unary(UnaryOp, Box<Expr>),

    // 函数调用: print("hi"), add(1, 2)
    Call {
        callee: Box<Expr>, // 通常是 Variable("print")
        args: Vec<Expr>,
    },

    // 成员访问: user.name, list.length
    Get {
        object: Box<Expr>,
        name: String,
    },

    // Safe navigation: user?.name
    SafeGet {
        object: Box<Expr>,
        name: String,
    },

    // 数组/列表字面量: [1, 2, 3]
    Array(Vec<Expr>),

    // Match 表达式
    Match {
        value: Box<Expr>,
        cases: Vec<MatchCase>,
        default: Option<Box<Expr>>, // Derived from `_ => ...`
    },

    // Intrinsic Print
    Print(Box<Expr>),

    // 数组索引: arr[i]
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    // 结构体字面量: Point { x: 10, y: 20 }
    StructLiteral {
        type_name: String,
        fields: Vec<(String, Expr)>, // (field_name, value)
    },

    // Vec 字面量: vec![1, 2, 3]
    VecLiteral(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: MatchPattern,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Literal(Literal), // Only Int for now
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod, // +, -, *, /, %
    Eq,
    Neq, // ==, !=
    Lt,
    Gt,
    Leq,
    Geq, // <, >, <=, >=
    And,
    Or,    // &&, ||
    Elvis, // ??
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -x
    Not, // !x
}
