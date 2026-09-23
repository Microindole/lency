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
        ExprKind::Print(e) => ExprKind::Print(Box::new(spec.specialize_expr(e))),

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
        // File I/O intrinsics
        ExprKind::ReadFile(path) => ExprKind::ReadFile(Box::new(spec.specialize_expr(path))),
        ExprKind::WriteFile(path, content) => ExprKind::WriteFile(
            Box::new(spec.specialize_expr(path)),
            Box::new(spec.specialize_expr(content)),
        ),
        // 字符串内置函数 (Sprint 12)
        ExprKind::Len(arg) => ExprKind::Len(Box::new(spec.specialize_expr(arg))),
        ExprKind::Trim(arg) => ExprKind::Trim(Box::new(spec.specialize_expr(arg))),
        ExprKind::Split(str_arg, delim) => ExprKind::Split(
            Box::new(spec.specialize_expr(str_arg)),
            Box::new(spec.specialize_expr(delim)),
        ),
        ExprKind::Join(vec_arg, sep) => ExprKind::Join(
            Box::new(spec.specialize_expr(vec_arg)),
            Box::new(spec.specialize_expr(sep)),
        ),
        ExprKind::Substr(str_arg, start, len) => ExprKind::Substr(
            Box::new(spec.specialize_expr(str_arg)),
            Box::new(spec.specialize_expr(start)),
            Box::new(spec.specialize_expr(len)),
        ),
        ExprKind::CharToString(arg) => ExprKind::CharToString(Box::new(spec.specialize_expr(arg))),
        ExprKind::Panic(arg) => ExprKind::Panic(Box::new(spec.specialize_expr(arg))),
        ExprKind::Format(template, args) => ExprKind::Format(
            Box::new(spec.specialize_expr(template)),
            Box::new(spec.specialize_expr(args)),
        ),
    };

    Expr {
        kind: new_kind,
        span: expr.span.clone(),
    }
}
