pub mod expr;
pub mod stmt;
pub mod types;
pub mod visitor;

// 重新导出核心类型，方便外部直接使用 beryl_syntax::ast::Expr 等
pub use expr::{BinaryOp, Expr, ExprKind, Literal, MatchCase, MatchPattern, Span, UnaryOp};
pub use stmt::{Decl, Field, Param, Stmt};
pub use types::Type;
pub use visitor::Visitor;

// 整个程序的数据结构
// 这里的 Decl 指的是顶层定义（Top Level Declarations），如 class, function
#[derive(Debug, Clone)]
pub struct Program {
    pub decls: Vec<Decl>,
}
