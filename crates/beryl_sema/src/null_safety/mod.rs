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
    pub(crate) scopes: &'a mut ScopeStack,
    pub(crate) current_scope: ScopeId,
    pub(crate) next_child_index: usize,
    pub(crate) errors: Vec<SemanticError>,
    /// 当前已知非空的变量（通过 if != null 检查后）
    pub(crate) known_non_null: HashSet<String>,
}

impl<'a> NullSafetyChecker<'a> {
    pub fn new(scopes: &'a mut ScopeStack) -> Self {
        Self {
            current_scope: scopes.current_scope(), // Start at root/global
            scopes,
            next_child_index: 0,
            errors: Vec::new(),
            known_non_null: HashSet::new(),
        }
    }

    /// 检查整个程序
    pub fn check(&mut self, program: &mut Program) -> Result<(), Vec<SemanticError>> {
        // 全局作用域
        self.next_child_index = 0;

        for decl in &mut program.decls {
            self.check_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 进入子作用域，并在闭包执行完后恢复
    pub fn with_child_scope<F>(&mut self, f: F)
    where
        F: FnOnce(&mut NullSafetyChecker),
    {
        let prev_scope = self.current_scope;
        // 使用 ScopeStack 计算子作用域 ID (假设已经构建好作用域树)
        // NullSafetyChecker 复用 Resolver构建的作用域栈
        // 我们假设作用域是按先序遍历顺序创建的，可以通过 next_child_index 找到

        // 注意：这里需要 ScopeStack 提供查找子作用域的功能
        // ScopeStack::get_child_scopes 返回 Vec<ScopeId>
        // 我们通过索引获取对应的子作用域
        let children = self.scopes.get_child_scopes(prev_scope);
        if self.next_child_index < children.len() {
            self.current_scope = children[self.next_child_index];
            self.next_child_index = 0; // 重置子作用域计数器（针对下一层）

            f(self);

            // 恢复
            self.current_scope = prev_scope;
            self.next_child_index += 1; // 移动到下一个兄弟作用域 (在父作用域视角)
        } else {
            // 如果找不到子作用域（理论上不应发生，如果 AST 结构一致），
            // 我们就在当前作用域继续（为了健壮性）
            // 或者这可能意味着 decls 没有对应的 scope (如 extern func)
            f(self);
        }
    }

    /// 检查声明
    fn check_decl(&mut self, decl: &mut Decl) {
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
                // Struct 字段的空安全在 resolver 阶段已验证
            }
            Decl::Impl { methods, .. } => {
                // 递归检查每个方法的空安全
                for method in methods {
                    self.check_decl(method);
                }
            }
            // Trait 定义：目前不需要空安全检查（方法签名无函数体）
            Decl::Trait { .. } => {}

            // Enum 定义：也不需要（变体类型检查在 Type Check）
            Decl::Enum { .. } => {}
        }
    }

    // --- Delegation methods ---

    pub(crate) fn check_stmt(&mut self, stmt: &mut Stmt) {
        stmt::check_stmt(self, stmt);
    }

    pub(crate) fn check_expr(&mut self, expr: &mut Expr) {
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
        let mut scopes = ScopeStack::new();
        let checker = NullSafetyChecker::new(&mut scopes);

        assert!(!checker.is_nullable(&Type::Int));
        assert!(!checker.is_nullable(&Type::String));
        assert!(checker.is_nullable(&Type::Nullable(Box::new(Type::String))));
    }
}
