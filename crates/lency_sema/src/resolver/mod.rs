//! Name Resolver
//!
//! 名称解析 Pass，收集所有定义并解析标识符引用。
//! 这是语义分析的第一步，为后续类型检查奠定基础。

use crate::error::SemanticError;
use crate::scope::ScopeStack;
use crate::symbol::Symbol;
use lency_syntax::ast::{Decl, Expr, Program, Span, Stmt, Type};

pub mod decl;
mod decl_impl;
pub mod expr;
pub mod stmt;

/// 名称解析器
pub struct Resolver {
    pub(crate) scopes: ScopeStack,
    pub(crate) errors: Vec<SemanticError>,
    /// 已访问的模块路径（防止循环导入）
    pub(crate) visited_modules: std::collections::HashSet<std::path::PathBuf>,
    /// 已加载的程序模块（保持 AST 所有权）
    pub(crate) loaded_programs: Vec<Program>,
    /// 项目根目录
    pub(crate) root_dir: std::path::PathBuf,
}

impl Resolver {
    pub fn new() -> Self {
        let mut scopes = ScopeStack::new();
        // Register built-in structural types (int, bool, string, float)
        // This allows `impl int` to find a "struct" symbol to attach methods to.
        let dummy_span = 0..0;
        let mut define_builtin = |name: &str| {
            // We use StructSymbol for primitives to allow method attachment
            let sym = Symbol::Struct(crate::symbol::StructSymbol::new(
                name.to_string(),
                dummy_span.clone(),
            ));
            scopes.define(sym).ok(); // ignore duplicate error in new
        };

        define_builtin("int");
        define_builtin("bool");
        define_builtin("string");
        define_builtin("float");

        Self {
            scopes,
            errors: Vec::new(),
            visited_modules: std::collections::HashSet::new(),
            loaded_programs: Vec::new(),
            root_dir: std::env::current_dir().unwrap_or_default(),
        }
    }

    /// 设置项目根目录
    pub fn set_root_dir(&mut self, path: std::path::PathBuf) {
        self.root_dir = path;
    }

    /// Normalize types (e.g., Vec<T> -> Type::Vec(T))
    /// This allows users to write `Vec<int>` and have it treated as the built-in vector type.
    pub fn normalize_type(&mut self, ty: &mut Type) {
        match ty {
            Type::Generic(name, args) if name == "Vec" => {
                if args.len() != 1 {
                    // This error will be caught during validation elsewhere or we can report here?
                    // normalize is called before validation.
                    // Let's just normalize for now, and validation will complain if Vec is not found?
                    // Actually, Vec is not in symbol table. So if we don't normalize, it's undefined type.
                    // If we do normalize, it becomes Type::Vec.
                    // If validation sees Type::Vec, checking its inner.
                    // If args != 1, we can't form valid Type::Vec.
                    return;
                }
                // Rewrite in place
                let inner = args.remove(0);
                *ty = Type::Vec(Box::new(inner));

                // Recurse on inner
                if let Type::Vec(inner) = ty {
                    self.normalize_type(inner);
                }
            }
            Type::Generic(_, args) => {
                for arg in args {
                    self.normalize_type(arg);
                }
            }
            Type::Nullable(inner) => {
                self.normalize_type(inner);
            }
            Type::Vec(inner) => {
                // Already normalized Vec
                self.normalize_type(inner);
            }
            Type::Array { element_type, .. } => {
                self.normalize_type(element_type);
            }
            _ => {}
        }
    }

    /// 解析整个程序
    ///
    /// 采用两遍扫描：
    /// 1. 第一遍：收集所有顶层声明（函数、类）
    /// 2. 第二遍：解析函数体内的引用
    pub fn resolve(&mut self, program: &mut Program) -> Result<(), Vec<SemanticError>> {
        eprintln!("Resolver::resolve started");
        // Pass 1: 收集顶层声明 (包括 main program 和递归加载的模块)

        // 收集新生成的合成声明 (Synthetics)
        // 例如：import ... as alias 生成 struct Alias, impl Alias;
        // 这些声明需要被加入到 program.decls 中，以便 Pass 2 和 CodeGen 处理。
        let mut synthetics_to_add = Vec::new();

        for decl in &program.decls {
            let mut synthetics = self.collect_decl(decl);
            synthetics_to_add.append(&mut synthetics);
        }

        // 将合成声明加入主程序
        program.decls.append(&mut synthetics_to_add);

        // 注意：新加入的 decls 已经被递归 collect_decl 处理过 (in resolve_import_as)，
        // 所以我们不需要再次遍历它们进行 Pass 1。

        // Pass 2: 解析函数体 (Main Program + Synthetics)
        for decl in &mut program.decls {
            self.resolve_decl(decl);
        }

        // Pass 2: 解析函数体 (Loaded Modules)
        // 以前的逻辑：加载的模块被 source-included 或者在这里处理。
        // 目前 resolve_import (Direct) 只是 collect definitions into Global Scope.
        // It does NOT merge AST. CodeGen relies on Global Scope lookup?
        // Wait, CodeGen iterates ctx.program.decls (Main Program).
        // If imports are NOT merged into Main Program, CodeGen WON'T generate code for them!
        // Direct Import: `import std.io;` -> `collect_decl` processes `std.io`.
        // Registers symbols. CodeGen sees calls to `std.io` functions.
        // BUT CodeGen does not generate function bodies for `std.io` in THIS module.
        // Lency uses "Source Inclusion" model?
        // If so, we MUST merge all loaded declarations into Main Program AST!

        // PREVIOUS LEGACY CODE (Lines 82-93 in original file):
        // Merged loaded_programs into program.decls!
        // We MUST restore that logic!

        let mut all_loaded_decls: Vec<Decl> = Vec::new();
        for mut prog in std::mem::take(&mut self.loaded_programs) {
            all_loaded_decls.append(&mut prog.decls);
        }

        // Run Pass 2 on loaded decls
        for decl in &mut all_loaded_decls {
            self.resolve_decl(decl);
        }

        // Merge
        program.decls.append(&mut all_loaded_decls);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 解析导入模块
    pub fn resolve_import(
        &mut self,
        path_components: &[String],
        span: &lency_syntax::ast::Span,
    ) -> Vec<Decl> {
        // 1. 构建路径
        let mut path_buf = self.root_dir.clone();
        if !path_components.is_empty() && path_components[0] == "std" {
            path_buf.push("lib");
            path_buf.push("std");
            for comp in &path_components[1..] {
                path_buf.push(comp);
            }
        } else {
            for comp in path_components {
                path_buf.push(comp);
            }
        }
        path_buf.set_extension("lcy");

        // 2. 检查环形依赖
        if self.visited_modules.contains(&path_buf) {
            return Vec::new();
        }

        // 3. 将当前模块加入栈
        self.visited_modules.insert(path_buf.clone());

        // 4. 读取并解析文件
        println!("Importing module: {:?}", path_buf);
        let source = match std::fs::read_to_string(&path_buf) {
            Ok(s) => s,
            Err(e) => {
                let err = SemanticError::ImportError {
                    message: format!("failed to read file '{:?}': {}", path_buf, e),
                    span: span.clone(),
                };
                self.errors.push(err);
                return Vec::new();
            }
        };

        // 5. Parse
        match lency_syntax::parser::parse(&source) {
            Ok(prog) => {
                // 6. 递归收集 (Pass 1)
                let mut all_synthetics = Vec::new();
                for decl in &prog.decls {
                    let mut synthetics = self.collect_decl(decl);
                    all_synthetics.append(&mut synthetics);
                }

                // Store program
                // Let's modify `prog` BEFORE pushing.
                let mut prog = prog;
                prog.decls.append(&mut all_synthetics);
                self.loaded_programs.push(prog);

                Vec::new()
            }
            Err(e) => {
                let err = SemanticError::ImportError {
                    message: format!("parse error in '{:?}': {:?}", path_buf, e),
                    span: span.clone(),
                };
                self.errors.push(err);
                Vec::new()
            }
        }
    }

    /// 解析带别名的导入 `import path as alias`
    fn resolve_import_as(
        &mut self,
        path_components: &[String],
        alias: &str,
        span: &lency_syntax::ast::Span,
    ) -> Vec<Decl> {
        // 1. 构建路径
        let mut path_buf = self.root_dir.clone();
        if !path_components.is_empty() && path_components[0] == "std" {
            path_buf.push("lib");
            path_buf.push("std");
            for comp in &path_components[1..] {
                path_buf.push(comp);
            }
        } else {
            for comp in path_components {
                path_buf.push(comp);
            }
        }
        path_buf.set_extension("lcy");

        let already_loaded = self.visited_modules.contains(&path_buf);

        if !already_loaded {
            self.visited_modules.insert(path_buf.clone());
            println!("Importing module as '{}': {:?}", alias, path_buf);
        }

        let source = match std::fs::read_to_string(&path_buf) {
            Ok(s) => s,
            Err(e) => {
                self.errors.push(SemanticError::ImportError {
                    message: format!("failed to read file '{:?}': {}", path_buf, e),
                    span: span.clone(),
                });
                return Vec::new();
            }
        };

        match lency_syntax::parser::parse(&source) {
            Ok(prog) => {
                // Synthesize Wrapper
                let mut synthetic_decls = Vec::new();
                let struct_name = format!("{}__Module", alias);

                // Struct
                synthetic_decls.push(Decl::Struct {
                    span: span.clone(),
                    name: struct_name.clone(),
                    generic_params: Vec::new(),
                    fields: Vec::new(),
                });

                let mut methods = Vec::new();
                let mut other_decls = Vec::new();

                for decl in prog.decls {
                    match decl {
                        Decl::Function {
                            span,
                            name,
                            generic_params,
                            params,
                            return_type,
                            body,
                        } => {
                            methods.push(Decl::Function {
                                span,
                                name,
                                generic_params,
                                params,
                                return_type,
                                body,
                            });
                        }
                        _ => {
                            if !already_loaded {
                                other_decls.push(decl);
                            }
                        }
                    }
                }

                // Impl
                synthetic_decls.push(Decl::Impl {
                    span: span.clone(),
                    trait_ref: None,
                    type_name: lency_syntax::ast::Type::Struct(struct_name.clone()),
                    generic_params: Vec::new(),
                    methods,
                });

                // Globals (if not loaded)
                synthetic_decls.append(&mut other_decls);

                // Global Var
                synthetic_decls.push(Decl::Var {
                    span: span.clone(),
                    name: alias.to_string(),
                    ty: Some(lency_syntax::ast::Type::Struct(struct_name.clone())),
                    value: lency_syntax::ast::Expr {
                        kind: lency_syntax::ast::ExprKind::StructLiteral {
                            type_: lency_syntax::ast::Type::Struct(struct_name.clone()),
                            fields: Vec::new(),
                        },
                        span: span.clone(),
                    },
                });

                // Recursively collect symbols for synthetics
                let mut deep_synthetics = Vec::new();
                for decl in &synthetic_decls {
                    let mut nested = self.collect_decl(decl);
                    deep_synthetics.append(&mut nested);
                }

                synthetic_decls.append(&mut deep_synthetics);

                synthetic_decls
            }
            Err(e) => {
                self.errors.push(SemanticError::ImportError {
                    message: format!("parse error in '{:?}': {:?}", path_buf, e),
                    span: span.clone(),
                });
                Vec::new()
            }
        }
    }

    // --- Delegation ---

    pub(crate) fn collect_decl(&mut self, decl: &Decl) -> Vec<Decl> {
        decl::collect_decl(self, decl)
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
