//! Type Inference
//!
//! 类型推导模块，处理 Lency 中 `var x = 10` 这种省略类型声明的情况。
//! 遵循 "Crystal Clear" 哲学：推导规则透明可预测。
//!
//! 模块结构遵循开闭原则 (OCP)，将不同类型的表达式推导逻辑分拆到子模块中。

mod access;
mod adt;
mod call;
mod control;
mod intrinsics;
mod literal;
mod operators;

#[cfg(test)]
mod tests;

// Re-export specific items for internal or external use
pub(crate) use adt::substitute_type; // Re-export for other sema modules

use crate::error::SemanticError;
use crate::operators::{BinaryOpRegistry, UnaryOpRegistry};
use crate::scope::{ScopeId, ScopeStack};
use crate::symbol::Symbol;
use lency_syntax::ast::{Expr, ExprKind, Type};

/// 类型推导器
pub struct TypeInferer<'a> {
    pub(crate) scopes: &'a mut ScopeStack,
    /// 当前作用域 ID（由调用者设置，用于正确的符号查找）
    pub(crate) current_scope: ScopeId,
    /// 二元运算符注册表
    pub(crate) binary_ops: BinaryOpRegistry,
    /// 一元运算符注册表
    pub(crate) unary_ops: UnaryOpRegistry,
}

impl<'a> TypeInferer<'a> {
    pub fn new(scopes: &'a mut ScopeStack) -> Self {
        Self {
            current_scope: scopes.current_scope(),
            scopes,
            binary_ops: BinaryOpRegistry::new(),
            unary_ops: UnaryOpRegistry::new(),
        }
    }

    /// 创建一个指定作用域的推导器
    pub fn with_scope(scopes: &'a mut ScopeStack, scope_id: ScopeId) -> Self {
        Self {
            scopes,
            current_scope: scope_id,
            binary_ops: BinaryOpRegistry::new(),
            unary_ops: UnaryOpRegistry::new(),
        }
    }

    /// 设置当前作用域
    pub fn set_scope(&mut self, scope_id: ScopeId) {
        self.current_scope = scope_id;
    }

    /// 查找符号（从当前作用域向上）
    fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.scopes.lookup_from(name, self.current_scope)
    }

    /// 推导表达式的类型
    pub fn infer(&mut self, expr: &mut Expr) -> Result<Type, SemanticError> {
        match &mut expr.kind {
            ExprKind::Literal(lit) => Ok(self.infer_literal(lit)),

            ExprKind::Unit => Ok(Type::Void),

            ExprKind::Variable(name) => self.infer_variable(name, &expr.span),

            ExprKind::Binary(left, op, right) => self.infer_binary(left, op, right, &expr.span),

            ExprKind::Unary(op, operand) => self.infer_unary(op, operand, &expr.span),

            ExprKind::Call { callee, args } => self.infer_call(callee, args, &expr.span),

            ExprKind::Get { object, name } => self.infer_get(object, name, &expr.span),

            ExprKind::SafeGet { object, name } => self.infer_safe_get(object, name, &expr.span),

            ExprKind::Array(elements) => self.infer_array(elements, &expr.span),

            ExprKind::Index { array, index } => self.infer_index(array, index, &expr.span),

            ExprKind::Match {
                value,
                cases,
                default,
            } => self.infer_match(value, cases, default.as_deref_mut(), &expr.span),

            ExprKind::Print(print_expr) => {
                self.infer(print_expr)?;
                Ok(Type::Void)
            }

            // ADT (Structs, Enums, Closures, Result, Vec) -> adt.rs
            ExprKind::StructLiteral { .. }
            | ExprKind::VecLiteral(_)
            | ExprKind::GenericInstantiation { .. }
            | ExprKind::Try(_)
            | ExprKind::Ok(_)
            | ExprKind::Err(_)
            | ExprKind::Closure { .. } => self.infer_adt(expr),

            // Intrinsics -> intrinsics.rs
            ExprKind::ReadFile(_)
            | ExprKind::WriteFile(_, _)
            | ExprKind::Len(_)
            | ExprKind::Trim(_)
            | ExprKind::Split(_, _)
            | ExprKind::Join(_, _)
            | ExprKind::Substr(_, _, _)
            | ExprKind::CharToString(_)
            | ExprKind::Panic(_) => self.infer_intrinsic(expr),
        }
    }
}

/// 检查两个类型是否兼容（用于赋值）
pub fn is_compatible(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        // 完全相同
        (a, b) if a == b => true,

        // int 可以隐式转为 float
        (Type::Float, Type::Int) => true,

        // null 字面量 (Type::Nullable(Type::Error)) 可以赋给任何可空类型
        (Type::Nullable(_), Type::Nullable(inner)) if matches!(**inner, Type::Error) => true,

        // 可空类型可以接受非空类型
        (Type::Nullable(inner), actual) => is_compatible(inner, actual),

        // Vec 兼容性
        (Type::Vec(t1), Type::Vec(t2)) => {
            if matches!(**t2, Type::Void) {
                true
            } else {
                t1 == t2
            }
        }

        // Result 兼容性 (Built-in)
        (
            Type::Result {
                ok_type: expected_ok,
                err_type: expected_err,
            },
            Type::Result {
                ok_type: actual_ok,
                err_type: actual_err,
            },
        ) => {
            // 如果 actual_ok 是 Void (来自 Err 构造器)，视为兼容
            let ok_compat =
                matches!(**actual_ok, Type::Void) || is_compatible(expected_ok, actual_ok);
            // 如果 actual_err 是 Void (来自 Ok 构造器)，视为兼容
            let err_compat =
                matches!(**actual_err, Type::Void) || is_compatible(expected_err, actual_err);
            ok_compat && err_compat
        }

        // Result 兼容性 (Generic "Result")
        (Type::Generic(name1, args1), Type::Generic(name2, args2))
            if name1 == "Result" && name2 == "Result" =>
        {
            if args1.len() == 2 && args2.len() == 2 {
                let expected_ok = &args1[0];
                let expected_err = &args1[1];
                let actual_ok = &args2[0];
                let actual_err = &args2[1];

                let ok_compat =
                    matches!(actual_ok, Type::Void) || is_compatible(expected_ok, actual_ok);
                let err_compat =
                    matches!(actual_err, Type::Void) || is_compatible(expected_err, actual_err);
                ok_compat && err_compat
            } else {
                false
            }
        }

        // Result 兼容性 (Generic vs Built-in)
        (
            Type::Generic(name, args),
            Type::Result {
                ok_type: actual_ok,
                err_type: actual_err,
            },
        ) if name == "Result" && args.len() == 2 => {
            let expected_ok = &args[0];
            let expected_err = &args[1];

            let ok_compat =
                matches!(**actual_ok, Type::Void) || is_compatible(expected_ok, actual_ok);
            let err_compat =
                matches!(**actual_err, Type::Void) || is_compatible(expected_err, actual_err);
            ok_compat && err_compat
        }
        (
            Type::Result {
                ok_type: expected_ok,
                err_type: expected_err,
            },
            Type::Generic(name, args),
        ) if name == "Result" && args.len() == 2 => {
            let actual_ok = &args[0];
            let actual_err = &args[1];

            let ok_compat =
                matches!(actual_ok, Type::Void) || is_compatible(expected_ok, actual_ok);
            let err_compat =
                matches!(actual_err, Type::Void) || is_compatible(expected_err, actual_err);
            ok_compat && err_compat
        }

        // Error 类型用于错误恢复，总是兼容
        (Type::Error, _) | (_, Type::Error) => true,

        _ => false,
    }
}
