use super::NullSafetyChecker;
use crate::error::SemanticError;
use crate::type_infer::TypeInferer;
use beryl_syntax::ast::{Expr, ExprKind, Type};

pub fn check_expr(checker: &mut NullSafetyChecker, expr: &Expr) {
    match &expr.kind {
        ExprKind::Get { object, .. } => {
            // 检查是否在可空类型上访问成员
            let inferer = TypeInferer::with_scope(checker.scopes, checker.current_scope);
            if let Ok(Type::Nullable(inner)) = inferer.infer(object) {
                // 检查变量是否已知非空
                let is_safe = if let ExprKind::Variable(name) = &object.kind {
                    checker.known_non_null.contains(name)
                } else {
                    false
                };

                if !is_safe {
                    checker.errors.push(SemanticError::PossibleNullAccess {
                        ty: format!("{}?", inner),
                        span: expr.span.clone(),
                    });
                }
            }

            // 递归检查子表达式
            checker.check_expr(object);
        }
        ExprKind::Call { callee, args } => {
            checker.check_expr(callee);
            for arg in args {
                checker.check_expr(arg);
            }
        }
        ExprKind::Binary(left, _, right) => {
            checker.check_expr(left);
            checker.check_expr(right);
        }
        ExprKind::Unary(_, operand) => {
            checker.check_expr(operand);
        }
        ExprKind::Array(elements) => {
            for elem in elements {
                checker.check_expr(elem);
            }
        }
        ExprKind::Index { array, index } => {
            // 检查不可空索引访问
            let inferer = TypeInferer::with_scope(checker.scopes, checker.current_scope);
            if let Ok(Type::Nullable(inner)) = inferer.infer(array) {
                let is_safe = if let ExprKind::Variable(name) = &array.kind {
                    checker.known_non_null.contains(name)
                } else {
                    false
                };

                if !is_safe {
                    checker.errors.push(SemanticError::PossibleNullAccess {
                        ty: format!("{}?", inner),
                        span: expr.span.clone(),
                    });
                }
            }

            checker.check_expr(array);
            checker.check_expr(index);
        }
        ExprKind::Match {
            value,
            cases,
            default,
        } => {
            checker.check_expr(value);
            for case in cases {
                checker.check_expr(&case.body);
            }
            if let Some(def) = default {
                checker.check_expr(def);
            }
        }
        ExprKind::Print(expr) => {
            checker.check_expr(expr);
        }
        ExprKind::New { args, .. } => {
            for arg in args {
                checker.check_expr(arg);
            }
        }
        ExprKind::StructLiteral { fields, .. } => {
            // Check all field value expressions
            for (_, value) in fields {
                checker.check_expr(value);
            }
        }
        // Variable 和 Literal 不需要递归检查
        _ => {}
    }
}
