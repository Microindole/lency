//! Name Resolver
//!
//! 名称解析 Pass，收集所有定义并解析标识符引用。
//! 这是语义分析的第一步，为后续类型检查奠定基础。

use crate::error::SemanticError;
use crate::scope::ScopeStack;
use crate::symbol::Symbol;
use beryl_syntax::ast::{Decl, Expr, Program, Span, Stmt, Type};

pub mod decl;
pub mod expr;
pub mod stmt;

/// 名称解析器
pub struct Resolver {
    pub(crate) scopes: ScopeStack,
    pub(crate) errors: Vec<SemanticError>,
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
    pub fn resolve(&mut self, program: &mut Program) -> Result<(), Vec<SemanticError>> {
        eprintln!("Resolver::resolve started");
        // Pass 1: 收集顶层声明
        for decl in &mut program.decls {
            self.collect_decl(decl);
        }

        // Pass 2: 解析函数体
        for decl in &mut program.decls {
            self.resolve_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    // --- Delegation ---

    pub(crate) fn collect_decl(&mut self, decl: &Decl) {
        decl::collect_decl(self, decl);
    }

    pub(crate) fn resolve_decl(&mut self, decl: &mut Decl) {
        decl::resolve_decl(self, decl);
    }

    pub(crate) fn resolve_stmt(&mut self, stmt: &mut Stmt) {
        stmt::resolve_stmt(self, stmt);
    }

    pub(crate) fn resolve_expr(&mut self, expr: &mut Expr) {
        expr::resolve_expr(self, expr);
    }

    /// 验证类型引用（包括泛型参数检查）
    pub fn resolve_type(&mut self, ty: &Type, span: &Span) {
        match ty {
            Type::Generic(name, args) => {
                // 1. 验证泛型类型本身
                let sym = self.scopes.lookup(name);
                match sym {
                    Some(Symbol::Struct(s)) => {
                        // 2. 检查参数数量
                        if s.generic_params.len() != args.len() {
                            self.errors.push(SemanticError::GenericArityMismatch {
                                name: name.clone(),
                                expected: s.generic_params.len(),
                                found: args.len(),
                                span: span.clone(),
                            });
                        }
                    }
                    Some(_) => {
                        self.errors.push(SemanticError::NotAGenericType {
                            name: name.clone(),
                            span: span.clone(),
                        });
                    }
                    None => {
                        self.errors.push(SemanticError::UndefinedType {
                            name: name.clone(),
                            span: span.clone(),
                        });
                    }
                }
                // 3. 递归验证参数
                for arg in args {
                    self.resolve_type(arg, span);
                }
            }
            Type::Struct(name) => {
                // 可能是普通结构体，也可能是泛型结构体但未带参数
                match self.scopes.lookup(name) {
                    Some(Symbol::Struct(s)) => {
                        if !s.generic_params.is_empty() {
                            // 引用了泛型结构体但没带参数 -> Arity Mismatch
                            self.errors.push(SemanticError::GenericArityMismatch {
                                name: name.clone(),
                                expected: s.generic_params.len(),
                                found: 0,
                                span: span.clone(),
                            });
                        }
                    }
                    Some(Symbol::GenericParam(_)) => {
                        // 引用泛型参数 (如 T)，合法
                    }
                    Some(Symbol::Enum(e)) => {
                        // 引用枚举类型
                        if !e.generic_params.is_empty() {
                            self.errors.push(SemanticError::GenericArityMismatch {
                                name: name.clone(),
                                expected: e.generic_params.len(),
                                found: 0,
                                span: span.clone(),
                            });
                        }
                    }
                    Some(_) => {
                        self.errors.push(SemanticError::UndefinedType {
                            name: name.clone(),
                            span: span.clone(),
                        });
                    }
                    None => {
                        self.errors.push(SemanticError::UndefinedType {
                            name: name.clone(),
                            span: span.clone(),
                        });
                    }
                }
            }
            Type::Vec(inner)
            | Type::Array {
                element_type: inner,
                ..
            }
            | Type::Nullable(inner) => {
                self.resolve_type(inner, span);
            }
            // 基础类型无需验证
            _ => {}
        }
    }

    // --- Accessors ---

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
