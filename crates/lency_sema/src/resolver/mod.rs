//! Name Resolver
//!
//! 名称解析 Pass，收集所有定义并解析标识符引用。
//! 这是语义分析的第一步，为后续类型检查奠定基础。

mod builtins;
pub mod decl;
mod decl_impl;
pub mod expr;
mod imports;
pub mod stmt;
mod types;

use crate::error::SemanticError;
use crate::scope::ScopeStack;
use crate::symbol::Symbol;
use lency_syntax::ast::{Decl, Expr, Program, Span, Stmt, Type};

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
        let dummy_span = 0..0;
        let mut define_builtin = |name: &str| {
            let sym = Symbol::Struct(crate::symbol::StructSymbol::new(
                name.to_string(),
                dummy_span.clone(),
            ));
            scopes.define(sym).ok();
        };

        define_builtin("int");
        define_builtin("bool");
        define_builtin("string");
        define_builtin("float");

        // Sprint 15: Register Result<T, E> as built-in enum to allow impl definitions
        let result_symbol = Symbol::Enum(crate::symbol::EnumSymbol {
            name: "Result".to_string(),
            generic_params: vec![
                crate::symbol::GenericParamSymbol::new("T".to_string(), None, dummy_span.clone()),
                crate::symbol::GenericParamSymbol::new("E".to_string(), None, dummy_span.clone()),
            ],
            variants: std::collections::HashMap::new(), // Ok and Err are handled by compiler
            methods: std::collections::HashMap::new(),  // Will be populated by user impl
            span: dummy_span.clone(),
        });
        scopes.define(result_symbol).ok();

        // Register built-in extern functions
        builtins::register_builtins(&mut scopes);

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

    /// Normalize types (delegated to types.rs)
    pub fn normalize_type(&mut self, ty: &mut Type) {
        types::normalize_type(self, ty);
    }

    /// Normalize types with generics (delegated to types.rs)
    pub fn normalize_type_with_generics(
        &mut self,
        ty: &mut Type,
        generics: &[crate::symbol::GenericParamSymbol],
    ) {
        types::normalize_type_with_generics(self, ty, generics);
    }

    /// 解析整个程序
    pub fn resolve(&mut self, program: &mut Program) -> Result<(), Vec<SemanticError>> {
        eprintln!("Resolver::resolve started");

        // Pass 1: 收集顶层声明
        let mut synthetics_to_add = Vec::new();
        for decl in &program.decls {
            let mut synthetics = self.collect_decl(decl);
            synthetics_to_add.append(&mut synthetics);
        }
        program.decls.append(&mut synthetics_to_add);

        // Pass 2: 解析函数体 (Main Program)
        for decl in &mut program.decls {
            self.resolve_decl(decl);
        }

        // Merge loaded modules into program
        let mut all_loaded_decls: Vec<Decl> = Vec::new();
        for mut prog in std::mem::take(&mut self.loaded_programs) {
            all_loaded_decls.append(&mut prog.decls);
        }

        // Run Pass 2 on loaded decls
        for decl in &mut all_loaded_decls {
            self.resolve_decl(decl);
        }

        program.decls.append(&mut all_loaded_decls);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 解析导入模块 (delegated to imports.rs)
    pub fn resolve_import(
        &mut self,
        path_components: &[String],
        span: &lency_syntax::ast::Span,
    ) -> Vec<Decl> {
        imports::resolve_import(self, path_components, span)
    }

    /// 解析带别名的导入 (delegated to imports.rs)
    fn resolve_import_as(
        &mut self,
        path_components: &[String],
        alias: &str,
        span: &lency_syntax::ast::Span,
    ) -> Vec<Decl> {
        imports::resolve_import_as(self, path_components, alias, span)
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

    ///Verify type reference (delegated to types.rs)
    pub fn resolve_type(&mut self, ty: &Type, span: &Span) {
        types::resolve_type(self, ty, span);
    }

    // --- Accessors ---

    pub fn scopes(&self) -> &ScopeStack {
        &self.scopes
    }

    pub fn into_scopes(self) -> ScopeStack {
        self.scopes
    }

    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
