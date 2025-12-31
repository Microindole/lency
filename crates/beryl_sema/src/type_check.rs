//! Type Checking
//!
//! 类型检查模块，验证程序的类型正确性。
//! 遵循 Beryl "Safety by Default" 哲学：严格的类型检查，拒绝隐式错误。

use crate::error::SemanticError;
use crate::scope::ScopeStack;
use crate::symbol::Symbol;
use crate::type_infer::{is_compatible, TypeInferer};
use beryl_syntax::ast::{Decl, Expr, ExprKind, Program, Stmt, Type};

/// 类型检查器
pub struct TypeChecker<'a> {
    scopes: &'a mut ScopeStack,
    errors: Vec<SemanticError>,
    /// 当前函数的返回类型（用于检查 return 语句）
    current_return_type: Option<Type>,
    /// 下一个要处理的子作用域索引（用于同步作用域遍历）
    next_child_index: usize,
}

impl<'a> TypeChecker<'a> {
    pub fn new(scopes: &'a mut ScopeStack) -> Self {
        Self {
            scopes,
            errors: Vec::new(),
            current_return_type: None,
            next_child_index: 0,
        }
    }

    /// 检查整个程序
    pub fn check(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        for decl in &program.decls {
            self.check_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 检查声明
    fn check_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function {
                name,
                params,
                return_type,
                body,
                span,
            } => {
                self.check_function(name, params, return_type, body, span);
            }
            Decl::Class {
                name,
                fields,
                methods,
                ..
            } => {
                self.check_class(name, fields, methods);
            }
        }
    }

    /// 检查函数
    fn check_function(
        &mut self,
        name: &str,
        _params: &[beryl_syntax::ast::Param],
        return_type: &Type,
        body: &[Stmt],
        span: &std::ops::Range<usize>,
    ) {
        // 保存当前作用域
        let parent_scope = self.scopes.current_scope();

        // 获取所有子作用域（按创建顺序）
        let children = self.scopes.get_child_scopes(parent_scope);

        // 进入下一个函数作用域
        if let Some(&func_scope) = children.get(self.next_child_index) {
            self.scopes.set_current(func_scope);
            self.next_child_index += 1;
        }

        // 设置当前函数返回类型
        let prev_return = self.current_return_type.replace(return_type.clone());

        // 检查函数体中的每个语句
        for stmt in body {
            self.check_stmt(stmt);
        }

        // 检查非 void 函数是否有返回值
        if *return_type != Type::Void && !self.has_return(body) {
            self.errors.push(SemanticError::MissingReturn {
                name: name.to_string(),
                ty: return_type.to_string(),
                span: span.clone(),
            });
        }

        // 恢复返回类型
        self.current_return_type = prev_return;

        // 恢复作用域
        self.scopes.set_current(parent_scope);
    }

    /// 检查类
    fn check_class(&mut self, _name: &str, _fields: &[beryl_syntax::ast::Field], methods: &[Decl]) {
        // 保存当前作用域和索引
        let parent_scope = self.scopes.current_scope();
        let children = self.scopes.get_child_scopes(parent_scope);

        // 进入类作用域（如果有）
        if let Some(&class_scope) = children.get(self.next_child_index) {
            self.scopes.set_current(class_scope);
            self.next_child_index += 1;

            // 重置子索引以处理方法
            let prev_index = self.next_child_index;
            self.next_child_index = 0;

            // 检查每个方法
            for method in methods {
                self.check_decl(method);
            }

            // 恢复索引
            self.next_child_index = prev_index;

            // 恢复作用域
            self.scopes.set_current(parent_scope);
        }
    }

    /// 检查语句
    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                ty,
                value,
                span,
            } => {
                self.check_var_decl(name, ty.as_ref(), value, span);
            }
            Stmt::Assignment {
                target,
                value,
                span,
            } => {
                self.check_assignment(target, value, span);
            }
            Stmt::Expression(expr) => {
                // 表达式语句只需检查表达式是否有效
                let _ = self.infer_type(expr);
            }
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                span,
            } => {
                self.check_if(condition, then_block, else_block.as_deref(), span);
            }
            Stmt::While {
                condition,
                body,
                span,
            } => {
                self.check_while(condition, body, span);
            }
            Stmt::Return { value, span } => {
                self.check_return(value.as_ref(), span);
            }
        }
    }

    /// 检查变量声明
    fn check_var_decl(
        &mut self,
        _name: &str,
        declared_ty: Option<&Type>,
        value: &Expr,
        span: &std::ops::Range<usize>,
    ) {
        // 推导初始化表达式的类型
        let value_ty = match self.infer_type(value) {
            Ok(ty) => ty,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };

        // 如果有显式类型声明，检查兼容性
        if let Some(expected) = declared_ty {
            if !is_compatible(expected, &value_ty) {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: expected.to_string(),
                    found: value_ty.to_string(),
                    span: span.clone(),
                });
            }
        }
    }

    /// 检查赋值语句
    fn check_assignment(&mut self, target: &Expr, value: &Expr, span: &std::ops::Range<usize>) {
        let target_ty = match self.infer_type(target) {
            Ok(ty) => ty,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };

        let value_ty = match self.infer_type(value) {
            Ok(ty) => ty,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };

        if !is_compatible(&target_ty, &value_ty) {
            self.errors.push(SemanticError::TypeMismatch {
                expected: target_ty.to_string(),
                found: value_ty.to_string(),
                span: span.clone(),
            });
        }
    }

    /// 检查 if 语句
    fn check_if(
        &mut self,
        condition: &Expr,
        then_block: &[Stmt],
        else_block: Option<&[Stmt]>,
        span: &std::ops::Range<usize>,
    ) {
        // 条件必须是 bool
        match self.infer_type(condition) {
            Ok(ty) if ty != Type::Bool => {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "bool".to_string(),
                    found: ty.to_string(),
                    span: span.clone(),
                });
            }
            Err(e) => self.errors.push(e),
            _ => {}
        }

        // 检查 then 分支
        for stmt in then_block {
            self.check_stmt(stmt);
        }

        // 检查 else 分支
        if let Some(else_stmts) = else_block {
            for stmt in else_stmts {
                self.check_stmt(stmt);
            }
        }
    }

    /// 检查 while 语句
    fn check_while(&mut self, condition: &Expr, body: &[Stmt], span: &std::ops::Range<usize>) {
        // 条件必须是 bool
        match self.infer_type(condition) {
            Ok(ty) if ty != Type::Bool => {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "bool".to_string(),
                    found: ty.to_string(),
                    span: span.clone(),
                });
            }
            Err(e) => self.errors.push(e),
            _ => {}
        }

        // 检查循环体
        for stmt in body {
            self.check_stmt(stmt);
        }
    }

    /// 检查 return 语句
    fn check_return(&mut self, value: Option<&Expr>, span: &std::ops::Range<usize>) {
        let expected = match &self.current_return_type {
            Some(ty) => ty.clone(),
            None => {
                // 不在函数内
                return;
            }
        };

        match (value, &expected) {
            // 有返回值
            (Some(expr), _) => match self.infer_type(expr) {
                Ok(actual) => {
                    if !is_compatible(&expected, &actual) {
                        self.errors.push(SemanticError::ReturnTypeMismatch {
                            expected: expected.to_string(),
                            found: actual.to_string(),
                            span: span.clone(),
                        });
                    }
                }
                Err(e) => self.errors.push(e),
            },
            // 没有返回值，但期望有
            (None, ty) if *ty != Type::Void => {
                self.errors.push(SemanticError::ReturnTypeMismatch {
                    expected: expected.to_string(),
                    found: "void".to_string(),
                    span: span.clone(),
                });
            }
            // void 函数返回空
            (None, Type::Void) => {}
            _ => {}
        }
    }

    /// 检查函数调用
    pub fn check_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // 获取函数符号
        let func = match &callee.kind {
            ExprKind::Variable(name) => match self.scopes.lookup(name) {
                Some(Symbol::Function(f)) => f.clone(),
                Some(_) => {
                    return Err(SemanticError::NotCallable {
                        ty: name.clone(),
                        span: span.clone(),
                    });
                }
                None => {
                    return Err(SemanticError::UndefinedFunction {
                        name: name.clone(),
                        span: span.clone(),
                    });
                }
            },
            _ => {
                // 复杂调用表达式暂不支持
                return Ok(Type::Error);
            }
        };

        // 检查参数数量
        if args.len() != func.params.len() {
            return Err(SemanticError::ArgumentCountMismatch {
                name: func.name.clone(),
                expected: func.params.len(),
                found: args.len(),
                span: span.clone(),
            });
        }

        // 检查每个参数类型
        for (_i, (arg, (_, expected_ty))) in args.iter().zip(func.params.iter()).enumerate() {
            let arg_ty = self.infer_type(arg)?;
            if !is_compatible(expected_ty, &arg_ty) {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: expected_ty.to_string(),
                    found: arg_ty.to_string(),
                    span: arg.span.clone(),
                });
            }
        }

        Ok(func.return_type.clone())
    }

    /// 推导表达式类型（封装 TypeInferer）
    fn infer_type(&self, expr: &Expr) -> Result<Type, SemanticError> {
        let inferer = TypeInferer::new(self.scopes);
        inferer.infer(expr)
    }

    /// 检查代码块是否有返回语句
    fn has_return(&self, stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Return { .. } => return true,
                Stmt::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    // 只有两个分支都有 return 才算完整覆盖
                    if self.has_return(then_block) {
                        if let Some(else_stmts) = else_block {
                            if self.has_return(else_stmts) {
                                return true;
                            }
                        }
                    }
                }
                Stmt::Block(inner) => {
                    if self.has_return(inner) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// 获取收集到的错误
    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }
}
