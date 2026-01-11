use super::NullSafetyChecker;
use crate::error::SemanticError;
use crate::type_infer::TypeInferer;
use beryl_syntax::ast::{Expr, Stmt, Type};

pub fn check_stmt(checker: &mut NullSafetyChecker, stmt: &mut Stmt) {
    eprintln!("Checking stmt: {:?}", stmt);
    match stmt {
        Stmt::VarDecl {
            name,
            ty,
            value,
            span,
        } => {
            check_var_decl(checker, name, ty.as_ref(), value, span);
        }
        Stmt::Assignment {
            target,
            value,
            span,
        } => {
            check_assignment(checker, target, value, span);
        }
        Stmt::Expression(expr) => {
            checker.check_expr(expr);
        }
        Stmt::Block(stmts) => {
            checker.with_child_scope(|checker: &mut NullSafetyChecker| {
                for stmt in stmts {
                    checker.check_stmt(stmt);
                }
            });
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            check_if(checker, condition, then_block, else_block.as_deref_mut());
        }
        Stmt::While {
            condition, body, ..
        } => {
            checker.check_expr(condition);
            // While loop has scope
            checker.with_child_scope(|checker: &mut NullSafetyChecker| {
                for stmt in body {
                    checker.check_stmt(stmt);
                }
            });
        }
        Stmt::For {
            init,
            condition,
            update,
            body,
            ..
        } => {
            // For loop has scope
            checker.with_child_scope(|checker: &mut NullSafetyChecker| {
                // 检查初始化语句
                if let Some(init_stmt) = init {
                    checker.check_stmt(init_stmt);
                }

                // 检查条件表达式
                if let Some(cond) = condition {
                    checker.check_expr(cond);
                }

                // 检查更新语句
                if let Some(upd) = update {
                    checker.check_stmt(upd);
                }

                for stmt in body {
                    checker.check_stmt(stmt);
                }
            });
        }
        Stmt::ForIn { iterable, body, .. } => {
            checker.check_expr(iterable);
            // ForIn has scope
            checker.with_child_scope(|checker: &mut NullSafetyChecker| {
                for stmt in body {
                    checker.check_stmt(stmt);
                }
            });
        }
        Stmt::Return { value, .. } => {
            if let Some(expr) = value {
                checker.check_expr(expr);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {
            // 控制流语句无空安全问题
        }
    }
}

/// 检查变量声明的 null 安全性
fn check_var_decl(
    checker: &mut NullSafetyChecker,
    _name: &str,
    declared_ty: Option<&Type>,
    value: &mut Expr,
    span: &std::ops::Range<usize>,
) {
    // 检查是否将 null 赋给非空类型
    if checker.is_null_literal(value) {
        if let Some(ty) = declared_ty {
            if !checker.is_nullable(ty) {
                checker
                    .errors
                    .push(SemanticError::NullAssignmentToNonNullable {
                        ty: ty.to_string(),
                        span: span.clone(),
                    });
            }
        }
    }

    checker.check_expr(value);
}

/// 检查赋值的 null 安全性
fn check_assignment(
    checker: &mut NullSafetyChecker,
    target: &mut Expr,
    value: &mut Expr,
    span: &std::ops::Range<usize>,
) {
    // 获取目标类型
    let mut inferer = TypeInferer::with_scope(checker.scopes, checker.current_scope);
    if let Ok(target_ty) = inferer.infer(target) {
        // 检查是否将 null 赋给非空类型
        if checker.is_null_literal(value) && !checker.is_nullable(&target_ty) {
            checker
                .errors
                .push(SemanticError::NullAssignmentToNonNullable {
                    ty: target_ty.to_string(),
                    span: span.clone(),
                });
        }
    }

    checker.check_expr(value);
}

/// 检查 if 语句（处理智能转换）
fn check_if(
    checker: &mut NullSafetyChecker,
    condition: &mut Expr,
    then_block: &mut [Stmt],
    else_block: Option<&mut [Stmt]>,
) {
    // 检查是否是 `x != null` 形式
    let narrowed_var = checker.extract_null_check(condition);

    // check condition
    checker.check_expr(condition);

    // 保存当前状态
    let prev_known = checker.known_non_null.clone();

    // 在 then 分支中，被检查的变量已知非空
    if let Some(var_name) = &narrowed_var {
        checker.known_non_null.insert(var_name.clone());
    }

    // Check then block using with_child_scope
    checker.with_child_scope(|checker| {
        for stmt in then_block {
            checker.check_stmt(stmt);
        }
    });

    // 恢复状态
    checker.known_non_null = prev_known.clone();

    // 在 else 分支中，变量可能为空（或者是 null）
    if let Some(else_stmts) = else_block {
        checker.with_child_scope(|checker| {
            for stmt in else_stmts {
                checker.check_stmt(stmt);
            }
        });
    }

    // 退出 if 后恢复状态
    checker.known_non_null = prev_known;
}
