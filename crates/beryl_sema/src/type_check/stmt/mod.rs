use super::TypeChecker;
use crate::error::SemanticError;
use crate::type_infer::is_compatible;
use beryl_syntax::ast::{Expr, Stmt, Type};

pub mod control_flow;
use control_flow::{check_for, check_for_in, check_if, check_while};

/// 辅助函数：进入新的作用域并检查代码块
pub(crate) fn check_block_with_scope(checker: &mut TypeChecker, stmts: &[Stmt]) {
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
            // 表达式语句需要检查表达式的类型正确性
            if let Err(e) = checker.infer_type(expr) {
                checker.errors.push(e);
            }
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
        Stmt::ForIn {
            iterator,
            iterable,
            body,
            span,
        } => {
            check_for_in(checker, iterator, iterable, body, span);
        }
    }
}

fn check_var_decl(
    checker: &mut TypeChecker,
    name: &str,
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

    // 更新符号表中的变量类型
    // 因为 Resolver 阶段可能只记录了声明类型（如果没有就是 Error/Unknown），
    // 这里需要根据推导结果更新类型，以便后续使用
    // 注意：如果是显式声明类型，通常 Resolver 已经设好了，但如果 var p = ... (没有类型)，这里必须更新
    if let Some(symbol_id) = checker.scopes.lookup_id(name) {
        if let Some(crate::symbol::Symbol::Variable(var_sym)) =
            checker.scopes.get_symbol_mut(symbol_id)
        {
            // 如果没有显式声明类型，或者推导类型更具体（例如 null check?）
            // 对于 var declaration，我们通常使用推导出的类型（如果有显式声明且兼容，推导出的可能更具体？）
            // 简单起见，如果声明了类型，就用声明的（已经check兼容）。如果没有，用推导的。
            if declared_ty.is_none() {
                var_sym.ty = value_ty;
            }
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
