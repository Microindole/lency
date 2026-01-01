use super::TypeChecker;
use crate::error::SemanticError;
use crate::type_infer::is_compatible;
use beryl_syntax::ast::{Expr, Stmt, Type};

/// 辅助函数：进入新的作用域并检查代码块
fn check_block_with_scope(checker: &mut TypeChecker, stmts: &[Stmt]) {
    // 保存当前作用域
    let parent_scope = checker.scopes.current_scope();
    let children = checker.scopes.get_child_scopes(parent_scope);

    // 进入子作用域
    if let Some(&child_scope) = children.get(checker.next_child_index) {
        checker.scopes.set_current(child_scope);
        checker.next_child_index += 1;

        // 保存并重置子索引
        let prev_child_index = checker.next_child_index;
        checker.next_child_index = 0;

        for stmt in stmts {
            check_stmt(checker, stmt);
        }

        // 恢复子索引
        checker.next_child_index = prev_child_index;
        // 恢复作用域
        checker.scopes.set_current(parent_scope);
    } else {
        // 如果找不到作用域，仍然检查语句（Fallback）
        for stmt in stmts {
            check_stmt(checker, stmt);
        }
    }
}

pub fn check_stmt(checker: &mut TypeChecker, stmt: &Stmt) {
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
            // 表达式语句只需检查表达式是否有效
            let _ = checker.infer_type(expr);
        }
        Stmt::Block(stmts) => {
            check_block_with_scope(checker, stmts);
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
            span,
        } => {
            check_if(checker, condition, then_block, else_block.as_deref(), span);
        }
        Stmt::While {
            condition,
            body,
            span,
        } => {
            check_while(checker, condition, body, span);
        }
        Stmt::For {
            init,
            condition,
            update,
            body,
            span,
        } => {
            check_for(
                checker,
                init.as_deref(),
                condition.as_ref(),
                update.as_deref(),
                body,
                span,
            );
        }
        Stmt::Return { value, span } => {
            check_return(checker, value.as_ref(), span);
        }
        Stmt::Break { span } => {
            if checker.loop_depth == 0 {
                checker
                    .errors
                    .push(SemanticError::BreakOutsideLoop { span: span.clone() });
            }
        }
        Stmt::Continue { span } => {
            if checker.loop_depth == 0 {
                checker
                    .errors
                    .push(SemanticError::ContinueOutsideLoop { span: span.clone() });
            }
        }
    }
}

fn check_var_decl(
    checker: &mut TypeChecker,
    _name: &str,
    declared_ty: Option<&Type>,
    value: &Expr,
    span: &std::ops::Range<usize>,
) {
    // 推导初始化表达式的类型
    let value_ty = match checker.infer_type(value) {
        Ok(ty) => ty,
        Err(e) => {
            checker.errors.push(e);
            return;
        }
    };

    // 如果有显式类型声明，检查兼容性
    if let Some(expected) = declared_ty {
        if !is_compatible(expected, &value_ty) {
            checker.errors.push(SemanticError::TypeMismatch {
                expected: expected.to_string(),
                found: value_ty.to_string(),
                span: span.clone(),
            });
        }
    }
}

fn check_assignment(
    checker: &mut TypeChecker,
    target: &Expr,
    value: &Expr,
    span: &std::ops::Range<usize>,
) {
    let target_ty = match checker.infer_type(target) {
        Ok(ty) => ty,
        Err(e) => {
            checker.errors.push(e);
            return;
        }
    };

    let value_ty = match checker.infer_type(value) {
        Ok(ty) => ty,
        Err(e) => {
            checker.errors.push(e);
            return;
        }
    };

    if !is_compatible(&target_ty, &value_ty) {
        checker.errors.push(SemanticError::TypeMismatch {
            expected: target_ty.to_string(),
            found: value_ty.to_string(),
            span: span.clone(),
        });
    }
}

fn check_if(
    checker: &mut TypeChecker,
    condition: &Expr,
    then_block: &[Stmt],
    else_block: Option<&[Stmt]>,
    span: &std::ops::Range<usize>,
) {
    // 条件必须是 bool
    match checker.infer_type(condition) {
        Ok(ty) if ty != Type::Bool => {
            checker.errors.push(SemanticError::TypeMismatch {
                expected: "bool".to_string(),
                found: ty.to_string(),
                span: span.clone(),
            });
        }
        Err(e) => checker.errors.push(e),
        _ => {}
    }

    // 检查 then 分支 (带作用域)
    check_block_with_scope(checker, then_block);

    // 检查 else 分支 (带作用域)
    if let Some(else_stmts) = else_block {
        check_block_with_scope(checker, else_stmts);
    }
}

fn check_while(
    checker: &mut TypeChecker,
    condition: &Expr,
    body: &[Stmt],
    span: &std::ops::Range<usize>,
) {
    // 条件必须是 bool
    match checker.infer_type(condition) {
        Ok(ty) if ty != Type::Bool => {
            checker.errors.push(SemanticError::TypeMismatch {
                expected: "bool".to_string(),
                found: ty.to_string(),
                span: span.clone(),
            });
        }
        Err(e) => checker.errors.push(e),
        _ => {}
    }

    checker.loop_depth += 1;
    // 检查循环体 (带作用域)
    check_block_with_scope(checker, body);
    checker.loop_depth -= 1;
}

fn check_for(
    checker: &mut TypeChecker,
    init: Option<&Stmt>,
    condition: Option<&Expr>,
    update: Option<&Stmt>,
    body: &[Stmt],
    span: &std::ops::Range<usize>,
) {
    // 保存当前作用域
    let parent_scope = checker.scopes.current_scope();
    let children = checker.scopes.get_child_scopes(parent_scope);

    // 进入 For 循环作用域
    if let Some(&for_scope) = children.get(checker.next_child_index) {
        checker.scopes.set_current(for_scope);
        checker.next_child_index += 1;

        // 保存并重置子索引
        let prev_child_index = checker.next_child_index;
        checker.next_child_index = 0;

        // 1. 检查初始化语句
        if let Some(init_stmt) = init {
            check_stmt(checker, init_stmt);
        }

        // 2. 检查条件表达式
        if let Some(cond) = condition {
            match checker.infer_type(cond) {
                Ok(ty) if ty != Type::Bool => {
                    checker.errors.push(SemanticError::TypeMismatch {
                        expected: "bool".to_string(),
                        found: ty.to_string(),
                        span: span.clone(),
                    });
                }
                Err(e) => checker.errors.push(e),
                _ => {}
            }
        }

        // 3. 检查更新语句
        if let Some(upd) = update {
            check_stmt(checker, upd);
        }

        // 4. 检查循环体
        checker.loop_depth += 1;
        for stmt in body {
            check_stmt(checker, stmt);
        }
        checker.loop_depth -= 1;

        // 恢复子索引
        checker.next_child_index = prev_child_index;
        // 恢复作用域
        checker.scopes.set_current(parent_scope);
    } else {
        // Fallback just in case
        if let Some(init_stmt) = init {
            check_stmt(checker, init_stmt);
        }
        if let Some(cond) = condition {
            let _ = checker.infer_type(cond);
        }
        if let Some(upd) = update {
            check_stmt(checker, upd);
        }
        checker.loop_depth += 1;
        for stmt in body {
            check_stmt(checker, stmt);
        }
        checker.loop_depth -= 1;
    }
}

fn check_return(checker: &mut TypeChecker, value: Option<&Expr>, span: &std::ops::Range<usize>) {
    let expected = match &checker.current_return_type {
        Some(ty) => ty.clone(),
        None => {
            // 不在函数内
            return;
        }
    };

    match (value, &expected) {
        (Some(expr), _) => match checker.infer_type(expr) {
            Ok(actual) => {
                if !is_compatible(&expected, &actual) {
                    checker.errors.push(SemanticError::ReturnTypeMismatch {
                        expected: expected.to_string(),
                        found: actual.to_string(),
                        span: span.clone(),
                    });
                }
            }
            Err(e) => checker.errors.push(e),
        },
        (None, ty) if *ty != Type::Void => {
            checker.errors.push(SemanticError::ReturnTypeMismatch {
                expected: expected.to_string(),
                found: "void".to_string(),
                span: span.clone(),
            });
        }
        (None, Type::Void) => {}
        _ => {}
    }
}
