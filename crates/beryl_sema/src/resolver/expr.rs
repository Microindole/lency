use super::Resolver;
use crate::error::SemanticError;
use crate::scope::ScopeKind;
use crate::symbol::{Symbol, VariableSymbol};
use beryl_syntax::ast::{Expr, ExprKind, MatchPattern, Type};

pub fn resolve_expr(resolver: &mut Resolver, expr: &mut Expr) {
    match &mut expr.kind {
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
        ExprKind::SafeGet { object, .. } => {
            resolver.resolve_expr(object);
        }

        ExprKind::Array(elements) => {
            for elem in elements {
                resolver.resolve_expr(elem);
            }
        }
        ExprKind::VecLiteral(elements) => {
            // 解析 Vec 字面量中的每个元素
            for elem in elements {
                resolver.resolve_expr(elem);
            }
        }
        ExprKind::GenericInstantiation { base, args: _ } => {
            // Resolve the base expression (the function being called)
            resolver.resolve_expr(base);
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
                resolver.scopes.enter_scope(ScopeKind::Block);
                declare_pattern_vars(resolver, &case.pattern);
                resolver.resolve_expr(&mut case.body);
                resolver.scopes.exit_scope();
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
        ExprKind::StructLiteral { type_, fields } => {
            // Check Struct type (handles generics)
            resolver.resolve_type(type_, &expr.span);

            // 解析每个字段的值表达式
            for (_, value) in fields {
                resolver.resolve_expr(value);
            }
        }
        // Result 相关表达式
        ExprKind::Try(inner) => resolver.resolve_expr(inner),
        ExprKind::Ok(inner) => resolver.resolve_expr(inner),
        ExprKind::Err(inner) => resolver.resolve_expr(inner),
        // 闭包
        ExprKind::Closure { params, body } => {
            // 进入闭包作用域
            resolver.scopes.enter_scope(ScopeKind::Function);
            // 注册参数
            for param in params {
                resolver.resolve_type(&param.ty, &expr.span);
                let param_sym = crate::symbol::ParameterSymbol::new(
                    param.name.clone(),
                    param.ty.clone(),
                    expr.span.clone(),
                    0,
                );
                if let Err(e) = resolver.scopes.define(Symbol::Parameter(param_sym)) {
                    resolver.errors.push(e);
                }
            }
            // 解析闭包体
            resolver.resolve_expr(body);
            resolver.scopes.exit_scope();
        }
    }
}

fn declare_pattern_vars(resolver: &mut Resolver, pattern: &MatchPattern) {
    match pattern {
        MatchPattern::Variable(name) => {
            let var_sym = VariableSymbol::new(
                name.clone(),
                Type::Void, // TypeChecker will infer later
                false,      // Immutable binding
                0..0,       // Span dummy? Or we should pass span?
            );
            if let Err(e) = resolver.scopes.define(Symbol::Variable(var_sym)) {
                resolver.errors.push(e);
            }
        }
        MatchPattern::Variant { sub_patterns, .. } => {
            for pat in sub_patterns {
                declare_pattern_vars(resolver, pat);
            }
        }
        _ => {}
    }
}
