//! Generic Instantiation Collector
//!
//! 遍历 AST，收集所有泛型类型实例化（如 `Box<int>`）。
//! 用于驱动单态化过程：发现需要生成的具体类型。

use beryl_syntax::ast::*;
use std::collections::HashSet;

pub struct Collector {
    /// 收集到的泛型类型实例化
    /// 例如: Type::Generic("Box", [int])
    pub instantiations: HashSet<Type>,
    /// 收集到的泛型函数实例化
    /// 例如: ("identity", [int])
    pub function_instantiations: HashSet<(String, Vec<Type>)>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            instantiations: HashSet::new(),
            function_instantiations: HashSet::new(),
        }
    }

    pub fn collect_program(&mut self, program: &Program) {
        for decl in &program.decls {
            self.collect_decl(decl);
        }
    }

    pub fn collect_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Struct { fields, .. } => {
                for field in fields {
                    self.collect_type(&field.ty);
                }
            }
            Decl::Function {
                params,
                return_type,
                body,
                ..
            } => {
                for param in params {
                    self.collect_type(&param.ty);
                }
                self.collect_type(return_type);
                for stmt in body {
                    self.collect_stmt(stmt);
                }
            }
            Decl::ExternFunction {
                params,
                return_type,
                ..
            } => {
                for param in params {
                    self.collect_type(&param.ty);
                }
                self.collect_type(return_type);
            }
            Decl::Impl { methods, .. } => {
                for method in methods {
                    self.collect_decl(method);
                }
            }
            // Trait 定义：目前不需要收集泛型实例化
            Decl::Trait { .. } => {}
        }
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { ty, value, .. } => {
                if let Some(t) = ty {
                    self.collect_type(t);
                }
                self.collect_expr(value);
            }
            Stmt::Assignment { target, value, .. } => {
                self.collect_expr(target);
                self.collect_expr(value);
            }
            Stmt::Expression(expr) => self.collect_expr(expr),
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.collect_stmt(s);
                }
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.collect_expr(condition);
                for s in then_block {
                    self.collect_stmt(s);
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.collect_stmt(s);
                    }
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.collect_expr(condition);
                for s in body {
                    self.collect_stmt(s);
                }
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                if let Some(i) = init {
                    self.collect_stmt(i);
                }
                if let Some(c) = condition {
                    self.collect_expr(c);
                }
                if let Some(u) = update {
                    self.collect_stmt(u);
                }
                for s in body {
                    self.collect_stmt(s);
                }
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.collect_expr(iterable);
                for s in body {
                    self.collect_stmt(s);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.collect_expr(v);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn collect_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(_) => {}
            ExprKind::Variable(_) => {}
            ExprKind::Binary(l, _, r) => {
                self.collect_expr(l);
                self.collect_expr(r);
            }
            ExprKind::Unary(_, e) => self.collect_expr(e),
            ExprKind::Call { callee, args } => {
                self.collect_expr(callee);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprKind::Get { object, .. } | ExprKind::SafeGet { object, .. } => {
                self.collect_expr(object);
            }
            ExprKind::Array(elems) => {
                for e in elems {
                    self.collect_expr(e);
                }
            }
            ExprKind::Index { array, index } => {
                self.collect_expr(array);
                self.collect_expr(index);
            }
            ExprKind::Print(e) => self.collect_expr(e),
            ExprKind::StructLiteral { type_, fields } => {
                self.collect_type(type_);
                for (_, e) in fields {
                    self.collect_expr(e);
                }
            }
            ExprKind::VecLiteral(elems) => {
                for e in elems {
                    self.collect_expr(e);
                }
            }
            ExprKind::Match {
                value,
                cases,
                default,
            } => {
                self.collect_expr(value);
                for c in cases {
                    self.collect_expr(&c.body);
                }
                if let Some(d) = default {
                    self.collect_expr(d);
                }
            }
            ExprKind::GenericInstantiation { base, args } => {
                // 如果 base 是 simple variable，则收集为函数实例化
                if let ExprKind::Variable(name) = &base.kind {
                    self.function_instantiations
                        .insert((name.clone(), args.clone()));
                }
                self.collect_expr(base);
                for arg in args {
                    self.collect_type(arg);
                }
            }
        }
    }

    fn collect_type(&mut self, ty: &Type) {
        match ty {
            Type::Generic(_, args) => {
                // Collection found generic instantiation!
                self.instantiations.insert(ty.clone());
                // Recurse
                for arg in args {
                    self.collect_type(arg);
                }
            }
            Type::Vec(inner) => self.collect_type(inner),
            Type::Array { element_type, .. } => self.collect_type(element_type),
            Type::Nullable(inner) => self.collect_type(inner),
            _ => {}
        }
    }
}
