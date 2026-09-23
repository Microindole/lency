use super::Specializer;
use lency_syntax::ast::{Expr, ExprKind, MatchCase};

pub fn specialize(spec: &Specializer, expr: &Expr) -> Expr {
    let new_kind = match &expr.kind {
        ExprKind::Literal(lit) => ExprKind::Literal(lit.clone()), // Lit 不变
        ExprKind::Unit => ExprKind::Unit,
        ExprKind::Variable(name) => ExprKind::Variable(name.clone()),
        ExprKind::Binary(lhs, op, rhs) => ExprKind::Binary(
            Box::new(spec.specialize_expr(lhs)),
            op.clone(),
            Box::new(spec.specialize_expr(rhs)),
        ),
        ExprKind::Unary(op, operand) => {
            ExprKind::Unary(op.clone(), Box::new(spec.specialize_expr(operand)))
        }
        ExprKind::Call { callee, args } => ExprKind::Call {
            callee: Box::new(spec.specialize_expr(callee)),
            args: args.iter().map(|a| spec.specialize_expr(a)).collect(),
        },
        ExprKind::Get { object, name } => ExprKind::Get {
            object: Box::new(spec.specialize_expr(object)),
            name: name.clone(),
        },
        ExprKind::SafeGet { object, name } => ExprKind::SafeGet {
            object: Box::new(spec.specialize_expr(object)),
            name: name.clone(),
        },
        ExprKind::Array(elements) => {
            ExprKind::Array(elements.iter().map(|e| spec.specialize_expr(e)).collect())
        }
        ExprKind::Index { array, index } => ExprKind::Index {
            array: Box::new(spec.specialize_expr(array)),
            index: Box::new(spec.specialize_expr(index)),
        },
        ExprKind::StructLiteral { type_, fields } => ExprKind::StructLiteral {
            type_: spec.specialize_type(type_),
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), spec.specialize_expr(e)))
                .collect(),
        },

        ExprKind::VecLiteral(elements) => {
            ExprKind::VecLiteral(elements.iter().map(|e| spec.specialize_expr(e)).collect())
        }

        ExprKind::GenericInstantiation { base, args } => ExprKind::GenericInstantiation {
            base: Box::new(spec.specialize_expr(base)),
            args: args.iter().map(|t| spec.specialize_type(t)).collect(),
        },

        ExprKind::Match {
            value,
            cases,
            default,
        } => ExprKind::Match {
            value: Box::new(spec.specialize_expr(value)),
            cases: cases
                .iter()
                .map(|c| MatchCase {
                    pattern: c.pattern.clone(),
                    body: Box::new(spec.specialize_expr(&c.body)),
                    span: c.span.clone(),
                })
                .collect(),
            default: default.as_ref().map(|e| Box::new(spec.specialize_expr(e))),
        },
    };

    Expr {
        kind: new_kind,
        span: expr.span.clone(),
    }
}
