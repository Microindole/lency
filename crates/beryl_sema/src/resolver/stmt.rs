use super::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{Symbol, VariableSymbol};
use beryl_syntax::ast::{Stmt, Type};

pub fn resolve_stmt(resolver: &mut Resolver, stmt: &Stmt) {
    match stmt {
        Stmt::VarDecl {
            name,
            ty,
            value,
            span,
        } => {
            // 先解析初始化表达式（变量在自己的初始化器中不可见）
            resolver.resolve_expr(value);

            // 如果有显式类型声明，验证类型
            if let Some(t) = ty {
                resolver.resolve_type(t, span);
            }

            // 推导类型（如果没有显式声明）
            let var_ty = ty.clone().unwrap_or_else(|| {
                // 使用 TypeInferer 推导变量类型
                let inferer = crate::type_infer::TypeInferer::with_scope(
                    &resolver.scopes,
                    resolver.scopes.current_scope(),
                );
                inferer.infer(value).unwrap_or(Type::Error)
            });

            // 添加变量到当前作用域
            let var_symbol = VariableSymbol::new(
                name.clone(),
                var_ty,
                true, // var 是可变的
                span.clone(),
            );

            if let Err(e) = resolver.scopes.define(Symbol::Variable(var_symbol)) {
                resolver.errors.push(e);
            }
        }
        Stmt::Assignment { target, value, .. } => {
            resolver.resolve_expr(target);
            resolver.resolve_expr(value);
        }
        Stmt::Expression(expr) => {
            resolver.resolve_expr(expr);
        }
        Stmt::Block(stmts) => {
            // 块语句创建新作用域
            resolver.scopes.enter_scope(ScopeKind::Block);
            for stmt in stmts {
                resolver.resolve_stmt(stmt);
            }
            resolver.scopes.exit_scope();
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            resolver.resolve_expr(condition);

            // then 分支
            resolver.scopes.enter_scope(ScopeKind::Block);
            for stmt in then_block {
                resolver.resolve_stmt(stmt);
            }
            resolver.scopes.exit_scope();

            // else 分支
            if let Some(else_stmts) = else_block {
                resolver.scopes.enter_scope(ScopeKind::Block);
                for stmt in else_stmts {
                    resolver.resolve_stmt(stmt);
                }
                resolver.scopes.exit_scope();
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            resolver.resolve_expr(condition);

            resolver.scopes.enter_scope(ScopeKind::Block);
            for stmt in body {
                resolver.resolve_stmt(stmt);
            }
            resolver.scopes.exit_scope();
        }
        Stmt::For {
            init,
            condition,
            update,
            body,
            ..
        } => {
            // For 循环创建自己的作用域（因为 init 中可能声明变量）
            resolver.scopes.enter_scope(ScopeKind::Block);

            // 解析初始化语句
            if let Some(init_stmt) = init {
                resolver.resolve_stmt(init_stmt);
            }

            // 解析条件表达式
            if let Some(cond) = condition {
                resolver.resolve_expr(cond);
            }

            // 解析更新语句
            if let Some(upd) = update {
                resolver.resolve_stmt(upd);
            }

            // 解析循环体
            for stmt in body {
                resolver.resolve_stmt(stmt);
            }

            resolver.scopes.exit_scope();
        }
        Stmt::ForIn {
            span,
            iterator,
            iterable,
            body,
        } => {
            // Resolve iterable (outside loop scope)
            resolver.resolve_expr(iterable);

            // Create loop scope
            resolver.scopes.enter_scope(ScopeKind::Block);

            // Define iterator variable
            // Type is unknown at this stage, use Void as placeholder (TypeChecker will fix it)
            let var_symbol = VariableSymbol::new(
                iterator.clone(),
                Type::Void, // Placeholder
                false,      // Iterator is immutable
                span.clone(),
            );

            if let Err(e) = resolver.scopes.define(Symbol::Variable(var_symbol)) {
                resolver.errors.push(e);
            }

            // Resolve body
            for stmt in body {
                resolver.resolve_stmt(stmt);
            }

            resolver.scopes.exit_scope();
        }
        Stmt::Return { value, .. } => {
            if let Some(expr) = value {
                resolver.resolve_expr(expr);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {
            // 不需要解析
        }
    }
}
