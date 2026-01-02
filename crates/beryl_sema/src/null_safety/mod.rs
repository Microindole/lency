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
use crate::scope::{ScopeId, ScopeStack};
use beryl_syntax::ast::{Decl, Expr, ExprKind, Literal, Program, Stmt, Type};
use std::collections::HashSet;

pub mod expr;
pub mod stmt;

/// Null Safety 检查器
pub struct NullSafetyChecker<'a> {
    pub(crate) scopes: &'a ScopeStack,
    pub(crate) current_scope: ScopeId,
    pub(crate) next_child_index: usize,
    pub(crate) errors: Vec<SemanticError>,
    /// 当前已知非空的变量（通过 if != null 检查后）
    pub(crate) known_non_null: HashSet<String>,
}

impl<'a> NullSafetyChecker<'a> {
    pub fn new(scopes: &'a ScopeStack) -> Self {
        Self {
            scopes,
            current_scope: scopes.current_scope(), // Start at root/global
            next_child_index: 0,
            errors: Vec::new(),
            known_non_null: HashSet::new(),
        }
    }

    /// 检查整个程序
    pub fn check(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        // 全局作用域
        self.next_child_index = 0;

        for decl in &program.decls {
            self.check_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 辅助函数：进入新的作用域并运行闭包
    pub(crate) fn with_child_scope<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let parent_scope = self.current_scope;
        let children = self.scopes.get_child_scopes(parent_scope);

        let child_scope = children.get(self.next_child_index).copied();

        if let Some(scope_id) = child_scope {
            // Update pointers
            self.current_scope = scope_id;
            self.next_child_index += 1;

            // Save prev index (for recursion)
            let prev_child_index = self.next_child_index;
            // Reset for new scope
            self.next_child_index = 0;

            // Run
            f(self);

            // Restore
            self.next_child_index = prev_child_index;
            self.current_scope = parent_scope;
        } else {
            // Fallback if scope missing (shouldn't happen if Resolver is consistent)
            f(self);
        }
    }

    /// 检查声明
    fn check_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function { body, .. } => {
                // Enter function scope
                self.with_child_scope(|checker| {
                    // 每个函数开始时清空已知非空集合
                    checker.known_non_null.clear();
                    for stmt in body {
                        checker.check_stmt(stmt);
                    }
                });
            }

            Decl::ExternFunction { .. } => {
                // Extern function does not have a body/scope
            }
            Decl::Struct { .. } => {
                // TODO: Check struct (Phase 2)
            }
            Decl::Impl { methods, .. } => {
                // TODO: Check impl methods (Phase 2)
                for method in methods {
                    self.check_decl(method);
                }
            }
        }
    }

    // --- Delegation methods ---

    pub(crate) fn check_stmt(&mut self, stmt: &Stmt) {
        stmt::check_stmt(self, stmt);
    }

    pub(crate) fn check_expr(&mut self, expr: &Expr) {
        expr::check_expr(self, expr);
    }

    // --- Helper methods ---

    /// 检查表达式是否是 null 字面量
    pub(crate) fn is_null_literal(&self, expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Literal(Literal::Null))
    }

    /// 检查类型是否可空
    pub(crate) fn is_nullable(&self, ty: &Type) -> bool {
        matches!(ty, Type::Nullable(_))
    }

    /// 从 `x != null` 条件中提取变量名
    pub(crate) fn extract_null_check(&self, condition: &Expr) -> Option<String> {
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
