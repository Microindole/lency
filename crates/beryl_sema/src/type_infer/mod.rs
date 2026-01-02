//! Type Inference
//!
//! 类型推导模块，处理 Beryl 中 `var x = 10` 这种省略类型声明的情况。
//! 遵循 "Crystal Clear" 哲学：推导规则透明可预测。
//!
//! 模块结构遵循开闭原则 (OCP)，将不同类型的表达式推导逻辑分拆到子模块中。

mod access;
mod call;
mod control;
mod literal;
mod operators;

#[cfg(test)]
mod tests;

use crate::error::SemanticError;
use crate::operators::{BinaryOpRegistry, UnaryOpRegistry};
use crate::scope::{ScopeId, ScopeStack};
use crate::symbol::Symbol;
use beryl_syntax::ast::{Expr, ExprKind, Type};

/// 类型推导器
///
/// 注意：TypeInferer 需要知道当前所在的作用域，
/// 因为变量查找需要从正确的作用域开始。
pub struct TypeInferer<'a> {
    pub(crate) scopes: &'a ScopeStack,
    /// 当前作用域 ID（由调用者设置，用于正确的符号查找）
    pub(crate) current_scope: ScopeId,
    /// 二元运算符注册表
    pub(crate) binary_ops: BinaryOpRegistry,
    /// 一元运算符注册表
    pub(crate) unary_ops: UnaryOpRegistry,
}

impl<'a> TypeInferer<'a> {
    pub fn new(scopes: &'a ScopeStack) -> Self {
        Self {
            scopes,
            current_scope: scopes.current_scope(),
            binary_ops: BinaryOpRegistry::new(),
            unary_ops: UnaryOpRegistry::new(),
        }
    }

    /// 创建一个指定作用域的推导器
    pub fn with_scope(scopes: &'a ScopeStack, scope_id: ScopeId) -> Self {
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
    pub fn infer(&self, expr: &Expr) -> Result<Type, SemanticError> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(self.infer_literal(lit)),

            ExprKind::Variable(name) => self.infer_variable(name, &expr.span),

            ExprKind::Binary(left, op, right) => self.infer_binary(left, op, right, &expr.span),

            ExprKind::Unary(op, operand) => self.infer_unary(op, operand, &expr.span),

            ExprKind::Call { callee, args: _ } => self.infer_call(callee, &expr.span),

            ExprKind::Get { object, name } => self.infer_get(object, name, &expr.span),

            ExprKind::Array(elements) => self.infer_array(elements, &expr.span),

            ExprKind::Index { array, index } => self.infer_index(array, index, &expr.span),

            ExprKind::Match {
                value,
                cases,
                default,
            } => self.infer_match(value, cases, default.as_deref(), &expr.span),

            ExprKind::Print(expr) => {
                self.infer(expr)?;
                Ok(Type::Void)
            }

            ExprKind::StructLiteral { type_name, fields } => {
                // 查找结构体类型并检查字段
                if let Some(crate::symbol::Symbol::Struct(struct_sym)) = self.lookup(type_name) {
                    // 检查所有字段
                    for (field_name, field_expr) in fields {
                        // 验证字段存在
                        if struct_sym.get_field(field_name).is_none() {
                            return Err(SemanticError::UndefinedField {
                                class: type_name.clone(),
                                field: field_name.clone(),
                                span: field_expr.span.clone(),
                            });
                        }
                        // 推导字段值的类型
                        self.infer(field_expr)?;
                    }
                    Ok(Type::Struct(type_name.clone()))
                } else {
                    Err(SemanticError::UndefinedType {
                        name: type_name.clone(),
                        span: expr.span.clone(),
                    })
                }
            }
        }
    }
}

/// 检查两个类型是否兼容（用于赋值）
pub fn is_compatible(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        // 完全相同
        (a, b) if a == b => true,

        // int 可以隐式转为 float（Beryl 设计决策：这是唯一允许的隐式转换）
        (Type::Float, Type::Int) => true,

        // null 字面量 (Type::Nullable(Type::Error)) 可以赋给任何可空类型
        (Type::Nullable(_), Type::Nullable(inner)) if matches!(**inner, Type::Error) => true,

        // 可空类型可以接受非空类型
        (Type::Nullable(inner), actual) => is_compatible(inner, actual),

        // Error 类型用于错误恢复，总是兼容
        (Type::Error, _) | (_, Type::Error) => true,

        _ => false,
    }
}
