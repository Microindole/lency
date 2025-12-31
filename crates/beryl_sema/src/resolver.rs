//! Name Resolver
//!
//! 名称解析 Pass，收集所有定义并解析标识符引用。
//! 这是语义分析的第一步，为后续类型检查奠定基础。

use crate::error::SemanticError;
use crate::scope::{ScopeKind, ScopeStack};
use crate::symbol::{ClassSymbol, FunctionSymbol, ParameterSymbol, Symbol, VariableSymbol};
use beryl_syntax::ast::{Decl, Expr, ExprKind, Program, Stmt, Type};

/// 名称解析器
pub struct Resolver {
    scopes: ScopeStack,
    errors: Vec<SemanticError>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            errors: Vec::new(),
        }
    }

    /// 解析整个程序
    ///
    /// 采用两遍扫描：
    /// 1. 第一遍：收集所有顶层声明（函数、类）
    /// 2. 第二遍：解析函数体内的引用
    pub fn resolve(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        // Pass 1: 收集顶层声明
        for decl in &program.decls {
            self.collect_decl(decl);
        }

        // Pass 2: 解析函数体
        for decl in &program.decls {
            self.resolve_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 收集顶层声明（Pass 1）
    fn collect_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function {
                name,
                params,
                return_type,
                span,
                ..
            } => {
                let func_symbol = FunctionSymbol::new(
                    name.clone(),
                    params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_type.clone(),
                    span.clone(),
                );

                if let Err(e) = self.scopes.define(Symbol::Function(func_symbol)) {
                    self.errors.push(e);
                }
            }
            Decl::Class {
                name,
                generics,
                fields,
                span,
                ..
            } => {
                let mut class_symbol =
                    ClassSymbol::new(name.clone(), generics.clone(), span.clone());

                // 收集字段
                for field in fields {
                    class_symbol.add_field(
                        field.name.clone(),
                        field.ty.clone(),
                        span.clone(), // 使用类的 span（理想情况下应该用字段的 span）
                    );
                }

                if let Err(e) = self.scopes.define(Symbol::Class(class_symbol)) {
                    self.errors.push(e);
                }
            }
        }
    }

    /// 解析声明（Pass 2）
    fn resolve_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function {
                name: _,
                params,
                body,
                span,
                ..
            } => {
                // 进入函数作用域
                self.scopes.enter_scope(ScopeKind::Function);

                // 注册参数
                for (i, param) in params.iter().enumerate() {
                    let param_symbol =
                        ParameterSymbol::new(param.name.clone(), param.ty.clone(), span.clone(), i);
                    if let Err(e) = self.scopes.define(Symbol::Parameter(param_symbol)) {
                        self.errors.push(e);
                    }
                }

                // 解析函数体
                for stmt in body {
                    self.resolve_stmt(stmt);
                }

                // 退出函数作用域
                self.scopes.exit_scope();
            }
            Decl::Class { methods, .. } => {
                // 进入类作用域
                self.scopes.enter_scope(ScopeKind::Class);

                // 解析方法
                for method in methods {
                    self.resolve_decl(method);
                }

                // 退出类作用域
                self.scopes.exit_scope();
            }
        }
    }

    /// 解析语句
    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                ty,
                value,
                span,
            } => {
                // 先解析初始化表达式（变量在自己的初始化器中不可见）
                self.resolve_expr(value);

                // 推导类型（如果没有显式声明）
                let var_ty = ty.clone().unwrap_or_else(|| {
                    // 这里暂时用 Error 占位，实际推导在 TypeChecker 中完成
                    Type::Error
                });

                // 添加变量到当前作用域
                let var_symbol = VariableSymbol::new(
                    name.clone(),
                    var_ty,
                    true, // var 是可变的
                    span.clone(),
                );
                if let Err(e) = self.scopes.define(Symbol::Variable(var_symbol)) {
                    self.errors.push(e);
                }
            }
            Stmt::Assignment { target, value, .. } => {
                self.resolve_expr(target);
                self.resolve_expr(value);
            }
            Stmt::Expression(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Block(stmts) => {
                // 块语句创建新作用域
                self.scopes.enter_scope(ScopeKind::Block);
                for stmt in stmts {
                    self.resolve_stmt(stmt);
                }
                self.scopes.exit_scope();
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.resolve_expr(condition);

                // then 分支
                self.scopes.enter_scope(ScopeKind::Block);
                for stmt in then_block {
                    self.resolve_stmt(stmt);
                }
                self.scopes.exit_scope();

                // else 分支
                if let Some(else_stmts) = else_block {
                    self.scopes.enter_scope(ScopeKind::Block);
                    for stmt in else_stmts {
                        self.resolve_stmt(stmt);
                    }
                    self.scopes.exit_scope();
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.resolve_expr(condition);

                self.scopes.enter_scope(ScopeKind::Block);
                for stmt in body {
                    self.resolve_stmt(stmt);
                }
                self.scopes.exit_scope();
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.resolve_expr(expr);
                }
            }
        }
    }

    /// 解析表达式
    fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Variable(name) => {
                // 检查变量是否已定义
                if self.scopes.lookup(name).is_none() {
                    self.errors.push(SemanticError::UndefinedVariable {
                        name: name.clone(),
                        span: expr.span.clone(),
                    });
                }
            }
            ExprKind::Binary(left, _, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ExprKind::Unary(_, operand) => {
                self.resolve_expr(operand);
            }
            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::Get { object, .. } => {
                self.resolve_expr(object);
                // 字段名的解析在类型检查阶段完成
            }
            ExprKind::New {
                class_name, args, ..
            } => {
                // 检查类是否存在
                if self.scopes.lookup(class_name).is_none() {
                    self.errors.push(SemanticError::UndefinedType {
                        name: class_name.clone(),
                        span: expr.span.clone(),
                    });
                }
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::Array(elements) => {
                for elem in elements {
                    self.resolve_expr(elem);
                }
            }
            ExprKind::Literal(_) => {
                // 字面量不需要解析
            }
        }
    }

    /// 获取作用域栈的引用
    pub fn scopes(&self) -> &ScopeStack {
        &self.scopes
    }

    /// 获取作用域栈的所有权
    pub fn into_scopes(self) -> ScopeStack {
        self.scopes
    }

    /// 获取收集到的错误
    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
