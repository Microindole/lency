use crate::ast::expr::{Expr, Span};
use crate::ast::types::Type;

// 顶层定义：只能出现在文件最外层
#[derive(Debug, Clone)]
pub enum Decl {
    // 函数定义: int add(int a, int b) { ... }
    Function {
        span: Span,
        name: String,
        params: Vec<Param>,
        return_type: Type,
        body: Vec<Stmt>,
    },

    // 外部函数声明: extern int print(int n);
    ExternFunction {
        span: Span,
        name: String,
        params: Vec<Param>,
        return_type: Type,
    },

    // 结构体定义: struct Point { int x int y }
    Struct {
        span: Span,
        name: String,
        fields: Vec<Field>,
    },

    // impl 块: impl Point { ... }
    Impl {
        span: Span,
        type_name: String,  // 实现的类型名称
        methods: Vec<Decl>, // 方法列表（都是 Function）
    },
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

// 语句：出现在函数体内部
#[derive(Debug, Clone)]
pub enum Stmt {
    // 变量声明: var x: int = 1; 或 var x = 1;
    VarDecl {
        span: Span,
        name: String,
        ty: Option<Type>, // None 表示需要类型推导
        value: Expr,
    },

    // 赋值: x = x + 1; (注意赋值在 Beryl 里是语句，不是表达式)
    Assignment {
        span: Span,
        target: Expr, // target 可以是 x，也可以是 user.age
        value: Expr,
    },

    // 表达式语句: print("hi");
    Expression(Expr),

    // 块: { ... }
    Block(Vec<Stmt>),

    // 控制流: if (expr) { ... } else { ... }
    If {
        span: Span,
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },

    // 循环: while (expr) { ... }
    While {
        span: Span,
        condition: Expr,
        body: Vec<Stmt>,
    },

    // For 循环: for init; condition; update { ... }
    For {
        span: Span,
        init: Option<Box<Stmt>>,   // 初始化语句: var i = 0
        condition: Option<Expr>,   // 条件表达式: i < 10
        update: Option<Box<Stmt>>, // 更新语句: i = i + 1
        body: Vec<Stmt>,           // 循环体
    },

    // For-In 循环: for x in arr { ... }
    ForIn {
        span: Span,
        iterator: String, // 迭代变量名
        iterable: Expr,   // 数组
        body: Vec<Stmt>,
    },

    // 返回: return 1;
    Return {
        span: Span,
        value: Option<Expr>,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
}
