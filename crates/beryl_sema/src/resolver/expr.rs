use super::Resolver;
use crate::error::SemanticError;
use beryl_syntax::ast::{Expr, ExprKind};

pub fn resolve_expr(resolver: &mut Resolver, expr: &Expr) {
    match &expr.kind {
        ExprKind::Variable(name) => {
            // 检查变量是否已定义
            if resolver.scopes.lookup(name).is_none() {
                resolver.errors.push(SemanticError::UndefinedVariable {
                    name: name.clone(),
                    span: expr.span.clone(),
                });
            }
        }
        ExprKind::Binary(left, _, right) => {
            resolver.resolve_expr(left);
            resolver.resolve_expr(right);
        }
        ExprKind::Unary(_, operand) => {
            resolver.resolve_expr(operand);
        }
        ExprKind::Call { callee, args } => {
            resolver.resolve_expr(callee);
            for arg in args {
                resolver.resolve_expr(arg);
            }
        }
        ExprKind::Get { object, .. } => {
            resolver.resolve_expr(object);
            // 字段名的解析在类型检查阶段完成
        }
        ExprKind::New {
            class_name, args, ..
        } => {
            // 检查类是否存在
            if resolver.scopes.lookup(class_name).is_none() {
                resolver.errors.push(SemanticError::UndefinedType {
                    name: class_name.clone(),
                    span: expr.span.clone(),
                });
            }
            for arg in args {
                resolver.resolve_expr(arg);
            }
        }
        ExprKind::Array(elements) => {
            for elem in elements {
                resolver.resolve_expr(elem);
            }
        }
        ExprKind::Literal(_) => {
            // 字面量不需要解析
        }
        ExprKind::Match {
            value,
            cases,
            default,
        } => {
            resolver.resolve_expr(value);
            for case in cases {
                resolver.resolve_expr(&case.body);
            }
            if let Some(default_expr) = default {
                resolver.resolve_expr(default_expr);
            }
        }
        ExprKind::Print(expr) => {
            resolver.resolve_expr(expr);
        }
        ExprKind::Index { array, index } => {
            resolver.resolve_expr(array);
            resolver.resolve_expr(index);
        }
        ExprKind::StructLiteral { type_name, fields } => {
            // TODO: Check struct type exists (Phase 2)
            let _ = type_name;
            for (_, value) in fields {
                resolver.resolve_expr(value);
            }
        }
    }
}
