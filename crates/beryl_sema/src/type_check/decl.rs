use super::TypeChecker;
use crate::error::SemanticError;

use beryl_syntax::ast::{Decl, Stmt, Type};

pub fn check_decl(checker: &mut TypeChecker, decl: &Decl) {
    match decl {
        Decl::Function {
            name,
            params,
            return_type,
            body,
            span,
        } => {
            check_function(checker, name, params, return_type, body, span);
        }
        Decl::Class {
            name,
            fields,
            methods,
            ..
        } => {
            check_class(checker, name, fields, methods);
        }
        Decl::ExternFunction { .. } => {
            // Nothing to check for extern declarations (types checked at parser/resolver level implicitly)
        }
    }
}

pub fn check_function(
    checker: &mut TypeChecker,
    name: &str,
    _params: &[beryl_syntax::ast::Param],
    return_type: &Type,
    body: &[Stmt],
    span: &std::ops::Range<usize>,
) {
    // 保存当前作用域
    let parent_scope = checker.scopes.current_scope();

    // 获取所有子作用域（按创建顺序）
    let children = checker.scopes.get_child_scopes(parent_scope);

    // 进入下一个函数作用域
    if let Some(&func_scope) = children.get(checker.next_child_index) {
        checker.scopes.set_current(func_scope);
        checker.next_child_index += 1;
    }

    // 保存并重置子索引（为函数体内的子作用域做准备）
    let prev_child_index = checker.next_child_index;
    checker.next_child_index = 0;

    // 设置当前函数返回类型
    let prev_return = checker.current_return_type.replace(return_type.clone());

    // 检查函数体中的每个语句
    for stmt in body {
        checker.check_stmt(stmt);
    }

    // 检查非 void 函数是否有返回值
    if *return_type != Type::Void && !checker.has_return(body) {
        checker.errors.push(SemanticError::MissingReturn {
            name: name.to_string(),
            ty: return_type.to_string(),
            span: span.clone(),
        });
    }

    // 恢复返回类型
    checker.current_return_type = prev_return;

    // 恢复子索引
    checker.next_child_index = prev_child_index;

    // 恢复作用域
    checker.scopes.set_current(parent_scope);
}

pub fn check_class(
    checker: &mut TypeChecker,
    _name: &str,
    _fields: &[beryl_syntax::ast::Field],
    methods: &[Decl],
) {
    // 保存当前作用域和索引
    let parent_scope = checker.scopes.current_scope();
    let children = checker.scopes.get_child_scopes(parent_scope);

    // 进入类作用域（如果有）
    if let Some(&class_scope) = children.get(checker.next_child_index) {
        checker.scopes.set_current(class_scope);
        checker.next_child_index += 1;

        // 重置子索引以处理方法
        let prev_index = checker.next_child_index;
        checker.next_child_index = 0;

        // 检查每个方法
        for method in methods {
            check_decl(checker, method);
        }

        // 恢复索引
        checker.next_child_index = prev_index;

        // 恢复作用域
        checker.scopes.set_current(parent_scope);
    }
}
