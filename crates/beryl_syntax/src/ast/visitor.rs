use super::*;

// 泛型 R: 返回值 (Result)
pub trait Visitor<R> {
    // 访问程序
    fn visit_program(&mut self, program: &Program) -> R;

    // 访问声明
    fn visit_decl(&mut self, decl: &Decl) -> R;

    // 访问语句
    fn visit_stmt(&mut self, stmt: &Stmt) -> R;

    // 访问表达式
    fn visit_expr(&mut self, expr: &Expr) -> R;
}
