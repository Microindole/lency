use super::{check_block_with_scope, check_stmt};
use crate::error::SemanticError;
use crate::symbol::Symbol;
use crate::type_check::TypeChecker;
use beryl_syntax::ast::{BinaryOp, Expr, ExprKind, Literal, Stmt, Type};

pub fn check_if(
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

    // --- Smart Casting (Flow Analysis) ---
    // Extract variable name from condition
    let narrowed_var = extract_smart_cast_var(condition);

    if let Some(name) = narrowed_var {
        // Find variable type (Extract data to avoid holding borrow)
        let refinement_data = if let Some(symbol) = checker.scopes.lookup(&name) {
            if let Some(Type::Nullable(inner)) = symbol.ty() {
                Some(*inner.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(inner_ty) = refinement_data {
            // We want to add refinement to the THEN block scope.
            let parent_scope = checker.scopes.current_scope();
            let children = checker.scopes.get_child_scopes(parent_scope);

            if let Some(&child_id) = children.get(checker.next_child_index) {
                if let Some(child_scope) = checker.scopes.get_scope_mut(child_id) {
                    child_scope.add_refinement(name, inner_ty);
                }
            }
        }
    }

    // 检查 then 分支 (带作用域 - refinements already injected)
    check_block_with_scope(checker, then_block);

    // No need to restore anything, scope exit handles it.

    // 检查 else 分支 (带作用域)
    if let Some(else_stmts) = else_block {
        check_block_with_scope(checker, else_stmts);
    }
}

/// Helper: Extract variable name from `x != null` check
fn extract_smart_cast_var(condition: &Expr) -> Option<String> {
    if let ExprKind::Binary(left, op, right) = &condition.kind {
        if *op == BinaryOp::Neq {
            // x != null
            if let ExprKind::Variable(name) = &left.kind {
                if matches!(&right.kind, ExprKind::Literal(Literal::Null)) {
                    return Some(name.clone());
                }
            }
            // null != x
            if let ExprKind::Variable(name) = &right.kind {
                if matches!(&left.kind, ExprKind::Literal(Literal::Null)) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

pub fn check_while(
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

pub fn check_for(
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

pub fn check_for_in(
    checker: &mut TypeChecker,
    iterator: &str,
    iterable: &Expr,
    body: &[Stmt],
    _span: &std::ops::Range<usize>,
) {
    // 1. check iterable (outside of loop scope)
    let elem_ty = match checker.infer_type(iterable) {
        Ok(Type::Array { element_type, .. }) => *element_type,
        Ok(ty) => {
            checker.errors.push(SemanticError::TypeMismatch {
                expected: "Array".to_string(),
                found: ty.to_string(),
                span: iterable.span.clone(),
            });
            Type::Error
        }
        Err(e) => {
            checker.errors.push(e);
            Type::Error
        }
    };

    // 2. Enter scope (aligned with Resolver)
    let parent_scope = checker.scopes.current_scope();
    let children = checker.scopes.get_child_scopes(parent_scope);

    if let Some(&loop_scope) = children.get(checker.next_child_index) {
        checker.scopes.set_current(loop_scope);
        checker.next_child_index += 1;

        let prev_child_index = checker.next_child_index;
        checker.next_child_index = 0;

        // 3. Update iterator type in this scope
        if let Some(symbol_id) = checker.scopes.lookup_id(iterator) {
            if let Some(Symbol::Variable(ref mut var_sym)) =
                checker.scopes.get_symbol_mut(symbol_id)
            {
                var_sym.ty = elem_ty;
            }
        }

        // 4. Check body
        checker.loop_depth += 1;
        for stmt in body {
            check_stmt(checker, stmt);
        }
        checker.loop_depth -= 1;

        // Restore
        checker.next_child_index = prev_child_index;
        checker.scopes.set_current(parent_scope);
    } else {
        // Fallback
        checker.loop_depth += 1;
        for stmt in body {
            check_stmt(checker, stmt);
        }
        checker.loop_depth -= 1;
    }
}
