//! 多文件源码采用互不重叠的字节区间，保留导入及单态化后的原始位置。

use crate::ast::{Decl, Expr, ExprKind, GenericParam, Program, Span, Stmt};

#[derive(Debug, Clone)]
struct SourceFile {
    path: String,
    source: String,
    start: usize,
    extent: usize,
    line_starts: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl SourceMap {
    pub fn main(path: &str, source: &str) -> Self {
        Self {
            files: vec![SourceFile {
                path: path.into(),
                source: source.into(),
                start: 0,
                extent: source.len(),
                line_starts: std::iter::once(0)
                    .chain(source.match_indices('\n').map(|(index, _)| index + 1))
                    .collect(),
            }],
        }
    }

    pub fn reserve_main(&mut self, extent: usize) {
        if self.files.is_empty() {
            *self = Self::main("<source>", "");
        }
        self.files[0].extent = self.files[0].extent.max(extent);
    }

    pub fn add(&mut self, path: &str, source: &str) -> usize {
        if let Some(file) = self.files.iter().find(|file| file.path == path) {
            return file.start;
        }
        let start = self
            .files
            .last()
            .map(|file| file.start + file.extent + 1)
            .unwrap_or(0);
        self.files.push(SourceFile {
            path: path.into(),
            source: source.into(),
            start,
            extent: source.len(),
            line_starts: std::iter::once(0)
                .chain(source.match_indices('\n').map(|(index, _)| index + 1))
                .collect(),
        });
        start
    }

    pub fn location(&self, offset: usize) -> SourceLocation {
        let file = self
            .files
            .iter()
            .rev()
            .find(|file| file.start <= offset && offset <= file.start + file.extent);
        let Some(file) = file else {
            return SourceLocation {
                file: "<unknown>".into(),
                line: 0,
                column: 0,
            };
        };
        if file.source.is_empty() {
            return SourceLocation {
                file: file.path.clone(),
                line: 0,
                column: 0,
            };
        }
        let relative = (offset - file.start).min(file.source.len());
        let line = file.line_starts.partition_point(|start| *start <= relative);
        let column = file.source[file.line_starts[line - 1]..relative]
            .chars()
            .count() as u32
            + 1;
        SourceLocation {
            file: file.path.clone(),
            line: line as u32,
            column,
        }
    }

    pub fn resolve_span(&self, span: &Span) -> Option<(&str, &str, Span)> {
        let file = self
            .files
            .iter()
            .rev()
            .find(|file| file.start <= span.start && span.start <= file.start + file.extent)?;
        Some((
            &file.path,
            &file.source,
            (span.start - file.start)..span.end.saturating_sub(file.start).min(file.source.len()),
        ))
    }

    pub fn is_imported(&self, offset: usize) -> bool {
        self.files
            .iter()
            .skip(1)
            .any(|file| file.start <= offset && offset <= file.start + file.extent)
    }
}

fn shift(span: &mut Span, offset: usize) {
    span.start += offset;
    span.end += offset;
}

pub fn decl_span(decl: &Decl) -> &Span {
    match decl {
        Decl::Function { span, .. }
        | Decl::ExternFunction { span, .. }
        | Decl::Struct { span, .. }
        | Decl::Impl { span, .. }
        | Decl::Trait { span, .. }
        | Decl::Enum { span, .. }
        | Decl::Var { span, .. }
        | Decl::Import { span, .. } => span,
    }
}

fn parameters(params: &mut [GenericParam], offset: usize) {
    for parameter in params {
        shift(&mut parameter.span, offset);
    }
}

pub fn shift_program(program: &mut Program, offset: usize) {
    for decl in &mut program.decls {
        shift_decl(decl, offset);
    }
}

fn shift_decl(decl: &mut Decl, offset: usize) {
    match decl {
        Decl::Function {
            span,
            generic_params,
            body,
            ..
        } => {
            shift(span, offset);
            parameters(generic_params, offset);
            statements(body, offset);
        }
        Decl::Impl {
            span,
            generic_params,
            methods,
            ..
        } => {
            shift(span, offset);
            parameters(generic_params, offset);
            for method in methods {
                shift_decl(method, offset);
            }
        }
        Decl::ExternFunction {
            span,
            generic_params,
            ..
        }
        | Decl::Struct {
            span,
            generic_params,
            ..
        }
        | Decl::Trait {
            span,
            generic_params,
            ..
        }
        | Decl::Enum {
            span,
            generic_params,
            ..
        } => {
            shift(span, offset);
            parameters(generic_params, offset);
        }
        Decl::Var { span, value, .. } => {
            shift(span, offset);
            expression(value, offset);
        }
        Decl::Import { span, .. } => shift(span, offset),
    }
}

fn statements(stmts: &mut [Stmt], offset: usize) {
    for stmt in stmts {
        statement(stmt, offset);
    }
}

fn statement(stmt: &mut Stmt, offset: usize) {
    match stmt {
        Stmt::VarDecl { span, value, .. }
        | Stmt::Return {
            span,
            value: Some(value),
        } => {
            shift(span, offset);
            expression(value, offset);
        }
        Stmt::Assignment {
            span,
            target,
            value,
        } => {
            shift(span, offset);
            expression(target, offset);
            expression(value, offset);
        }
        Stmt::Expression(expr) => expression(expr, offset),
        Stmt::Block(body) => statements(body, offset),
        Stmt::If {
            span,
            condition,
            then_block,
            else_block,
        } => {
            shift(span, offset);
            expression(condition, offset);
            statements(then_block, offset);
            if let Some(body) = else_block {
                statements(body, offset);
            }
        }
        Stmt::While {
            span,
            condition,
            body,
        } => {
            shift(span, offset);
            expression(condition, offset);
            statements(body, offset);
        }
        Stmt::For {
            span,
            init,
            condition,
            update,
            body,
        } => {
            shift(span, offset);
            if let Some(init) = init {
                statement(init, offset);
            }
            if let Some(condition) = condition {
                expression(condition, offset);
            }
            if let Some(update) = update {
                statement(update, offset);
            }
            statements(body, offset);
        }
        Stmt::ForIn {
            span,
            iterable,
            body,
            ..
        } => {
            shift(span, offset);
            expression(iterable, offset);
            statements(body, offset);
        }
        Stmt::Return { span, value: None } | Stmt::Break { span } | Stmt::Continue { span } => {
            shift(span, offset)
        }
    }
}

fn expression(expr: &mut Expr, offset: usize) {
    shift(&mut expr.span, offset);
    match &mut expr.kind {
        ExprKind::Binary(left, _, right) => {
            expression(left, offset);
            expression(right, offset);
        }
        ExprKind::Unary(_, value)
        | ExprKind::Get { object: value, .. }
        | ExprKind::SafeGet { object: value, .. }
        | ExprKind::GenericInstantiation { base: value, .. } => expression(value, offset),
        ExprKind::Call { callee, args } => {
            expression(callee, offset);
            for arg in args {
                expression(arg, offset);
            }
        }
        ExprKind::Index { array, index } => {
            expression(array, offset);
            expression(index, offset);
        }
        ExprKind::Array(values) | ExprKind::VecLiteral(values) => {
            for value in values {
                expression(value, offset);
            }
        }
        ExprKind::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                expression(value, offset);
            }
        }
        ExprKind::Match {
            value,
            cases,
            default,
        } => {
            expression(value, offset);
            for case in cases {
                shift(&mut case.span, offset);
                expression(&mut case.body, offset);
            }
            if let Some(default) = default {
                expression(default, offset);
            }
        }
        ExprKind::Literal(_) | ExprKind::Variable(_) | ExprKind::Unit => {}
    }
}
