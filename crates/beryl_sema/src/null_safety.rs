//! Null Safety Checker
//!
//! Beryl 核心安全特性：Null Safety
//!
//! 规则：
//! 1. `T` 类型永远不能是 null
//! 2. `T?` 可以是 null，但使用前必须检查
//! 3. `if x != null { ... }` 后 x 自动转为 T (智能转换)
//!
//! 遵循 "Safety by Default" 哲学：编译时捕获所有潜在的 null 错误。

use crate::error::SemanticError;
use crate::scope::ScopeStack;
use crate::type_infer::TypeInferer;
use beryl_syntax::ast::{Decl, Expr, ExprKind, Literal, Program, Stmt, Type};
use std::collections::HashSet;

/// Null Safety 检查器
pub struct NullSafetyChecker<'a> {
    scopes: &'a ScopeStack,
    errors: Vec<SemanticError>,
    /// 当前已知非空的变量（通过 if != null 检查后）
    known_non_null: HashSet<String>,
}

impl<'a> NullSafetyChecker<'a> {
    pub fn new(scopes: &'a ScopeStack) -> Self {
        Self {
            scopes,
            errors: Vec::new(),
            known_non_null: HashSet::new(),
        }
    }

    /// 检查整个程序
    pub fn check(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        for decl in &program.decls {
            self.check_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 检查声明
    fn check_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function { body, .. } => {
                // 每个函数开始时清空已知非空集合
                self.known_non_null.clear();
                for stmt in body {
                    self.check_stmt(stmt);
                }
            }
            Decl::Class { methods, .. } => {
                for method in methods {
                    self.check_decl(method);
                }
            }
            Decl::ExternFunction { .. } => {}
        }
    }

    /// 检查语句
    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                ty,
                value,
                span,
            } => {
                self.check_var_decl(name, ty.as_ref(), value, span);
            }
            Stmt::Assignment {
                target,
                value,
                span,
            } => {
                self.check_assignment(target, value, span);
            }
            Stmt::Expression(expr) => {
                self.check_expr(expr);
            }
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.check_if(condition, then_block, else_block.as_deref());
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.check_expr(condition);
                for stmt in body {
                    self.check_stmt(stmt);
                }
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                // 检查初始化语句
                if let Some(init_stmt) = init {
                    self.check_stmt(init_stmt);
                }

                // 检查条件表达式
                if let Some(cond) = condition {
                    self.check_expr(cond);
                }

                // 检查更新语句
                if let Some(upd) = update {
                    self.check_stmt(upd);
                }

                // 检查循环体
                for stmt in body {
                    self.check_stmt(stmt);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.check_expr(expr);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {
                // 控制流语句无空安全问题
            }
        }
    }

    /// 检查变量声明的 null 安全性
    fn check_var_decl(
        &mut self,
        _name: &str,
        declared_ty: Option<&Type>,
        value: &Expr,
        span: &std::ops::Range<usize>,
    ) {
        // 检查是否将 null 赋给非空类型
        if self.is_null_literal(value) {
            if let Some(ty) = declared_ty {
                if !self.is_nullable(ty) {
                    self.errors
                        .push(SemanticError::NullAssignmentToNonNullable {
                            ty: ty.to_string(),
                            span: span.clone(),
                        });
                }
            }
        }

        self.check_expr(value);
    }

    /// 检查赋值的 null 安全性
    fn check_assignment(&mut self, target: &Expr, value: &Expr, span: &std::ops::Range<usize>) {
        // 获取目标类型
        let inferer = TypeInferer::new(self.scopes);
        if let Ok(target_ty) = inferer.infer(target) {
            // 检查是否将 null 赋给非空类型
            if self.is_null_literal(value) && !self.is_nullable(&target_ty) {
                self.errors
                    .push(SemanticError::NullAssignmentToNonNullable {
                        ty: target_ty.to_string(),
                        span: span.clone(),
                    });
            }
        }

        self.check_expr(value);
    }

    /// 检查 if 语句（处理智能转换）
    fn check_if(&mut self, condition: &Expr, then_block: &[Stmt], else_block: Option<&[Stmt]>) {
        // 检查是否是 `x != null` 形式
        let narrowed_var = self.extract_null_check(condition);

        // 保存当前状态
        let prev_known = self.known_non_null.clone();

        // 在 then 分支中，被检查的变量已知非空
        if let Some(var_name) = &narrowed_var {
            self.known_non_null.insert(var_name.clone());
        }

        for stmt in then_block {
            self.check_stmt(stmt);
        }

        // 恢复状态
        self.known_non_null = prev_known.clone();

        // 在 else 分支中，变量可能为空（或者是 null）
        if let Some(else_stmts) = else_block {
            for stmt in else_stmts {
                self.check_stmt(stmt);
            }
        }

        // 退出 if 后恢复状态
        self.known_non_null = prev_known;
    }

    /// 检查表达式的 null 安全性
    fn check_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Get { object, .. } => {
                // 检查是否在可空类型上访问成员
                let inferer = TypeInferer::new(self.scopes);
                if let Ok(Type::Nullable(inner)) = inferer.infer(object) {
                    // 检查变量是否已知非空
                    if let ExprKind::Variable(name) = &object.kind {
                        if !self.known_non_null.contains(name) {
                            self.errors.push(SemanticError::PossibleNullAccess {
                                ty: format!("{}?", inner),
                                span: expr.span.clone(),
                            });
                        }
                    } else {
                        self.errors.push(SemanticError::PossibleNullAccess {
                            ty: format!("{}?", inner),
                            span: expr.span.clone(),
                        });
                    }
                }

                // 递归检查子表达式
                self.check_expr(object);
            }
            ExprKind::Call { callee, args } => {
                self.check_expr(callee);
                for arg in args {
                    self.check_expr(arg);
                }
            }
            ExprKind::Binary(left, _, right) => {
                self.check_expr(left);
                self.check_expr(right);
            }
            ExprKind::Unary(_, operand) => {
                self.check_expr(operand);
            }
            ExprKind::Array(elements) => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }
            ExprKind::Match {
                value,
                cases,
                default,
            } => {
                self.check_expr(value);
                for case in cases {
                    self.check_expr(&case.body);
                }
                if let Some(def) = default {
                    self.check_expr(def);
                }
            }
            ExprKind::Print(expr) => {
                self.check_expr(expr);
            }
            ExprKind::New { args, .. } => {
                for arg in args {
                    self.check_expr(arg);
                }
            }
            // Variable 和 Literal 不需要递归检查
            _ => {}
        }
    }

    /// 检查表达式是否是 null 字面量
    fn is_null_literal(&self, expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Literal(Literal::Null))
    }

    /// 检查类型是否可空
    fn is_nullable(&self, ty: &Type) -> bool {
        matches!(ty, Type::Nullable(_))
    }

    /// 从 `x != null` 条件中提取变量名
    fn extract_null_check(&self, condition: &Expr) -> Option<String> {
        if let ExprKind::Binary(left, op, right) = &condition.kind {
            use beryl_syntax::ast::BinaryOp;
            if *op == BinaryOp::Neq {
                // x != null
                if let ExprKind::Variable(name) = &left.kind {
                    if matches!(&right.kind, ExprKind::Literal(Literal::Null)) {
                        return Some(name.clone());
                    }
                }
                // null != x
                if let ExprKind::Variable(name) = &right.kind {
                    if matches!(&left.kind, ExprKind::Literal(Literal::Null)) {
                        return Some(name.clone());
                    }
                }
            }
        }
        None
    }

    /// 获取收集到的错误
    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nullable() {
        let scopes = ScopeStack::new();
        let checker = NullSafetyChecker::new(&scopes);

        assert!(!checker.is_nullable(&Type::Int));
        assert!(!checker.is_nullable(&Type::String));
        assert!(checker.is_nullable(&Type::Nullable(Box::new(Type::String))));
    }
}
