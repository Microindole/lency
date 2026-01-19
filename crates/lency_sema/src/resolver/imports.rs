use super::decl;
use super::Resolver;
use crate::error::SemanticError;
use lency_syntax::ast::{Decl, Expr, ExprKind, Span, Type};

/// 解析导入模块
pub fn resolve_import(
    resolver: &mut Resolver,
    path_components: &[String],
    span: &Span,
) -> Vec<Decl> {
    // 1. 构建路径
    let mut path_buf = resolver.root_dir.clone();
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
    if resolver.visited_modules.contains(&path_buf) {
        return Vec::new();
    }

    // 3. 将当前模块加入栈
    resolver.visited_modules.insert(path_buf.clone());

    // 4. 读取并解析文件
    println!("Importing module: {:?}", path_buf);
    let source = match std::fs::read_to_string(&path_buf) {
        Ok(s) => s,
        Err(e) => {
            let err = SemanticError::ImportError {
                message: format!("failed to read file '{:?}': {}", path_buf, e),
                span: span.clone(),
            };
            resolver.errors.push(err);
            return Vec::new();
        }
    };

    // 5. Parse
    match lency_syntax::parser::parse(&source) {
        Ok(prog) => {
            // 6. 递归收集 (Pass 1)
            let mut all_synthetics = Vec::new();
            for decl in &prog.decls {
                let mut synthetics = decl::collect_decl(resolver, decl);
                all_synthetics.append(&mut synthetics);
            }

            // Store program
            let mut prog = prog;
            prog.decls.append(&mut all_synthetics);
            resolver.loaded_programs.push(prog);

            Vec::new()
        }
        Err(e) => {
            let err = SemanticError::ImportError {
                message: format!("parse error in '{:?}': {:?}", path_buf, e),
                span: span.clone(),
            };
            resolver.errors.push(err);
            Vec::new()
        }
    }
}

/// 解析带别名的导入 `import path as alias`
pub fn resolve_import_as(
    resolver: &mut Resolver,
    path_components: &[String],
    alias: &str,
    span: &Span,
) -> Vec<Decl> {
    // 1. 构建路径
    let mut path_buf = resolver.root_dir.clone();
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

    let already_loaded = resolver.visited_modules.contains(&path_buf);

    if !already_loaded {
        resolver.visited_modules.insert(path_buf.clone());
        println!("Importing module as '{}': {:?}", alias, path_buf);
    }

    let source = match std::fs::read_to_string(&path_buf) {
        Ok(s) => s,
        Err(e) => {
            resolver.errors.push(SemanticError::ImportError {
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
                type_name: Type::Struct(struct_name.clone()),
                generic_params: Vec::new(),
                methods,
            });

            // Globals (if not loaded)
            synthetic_decls.append(&mut other_decls);

            // Global Var
            synthetic_decls.push(Decl::Var {
                span: span.clone(),
                name: alias.to_string(),
                ty: Some(Type::Struct(struct_name.clone())),
                value: Expr {
                    kind: ExprKind::StructLiteral {
                        type_: Type::Struct(struct_name.clone()),
                        fields: Vec::new(),
                    },
                    span: span.clone(),
                },
            });

            // Recursively collect symbols for synthetics
            let mut deep_synthetics = Vec::new();
            for decl in &synthetic_decls {
                let mut nested = decl::collect_decl(resolver, decl);
                deep_synthetics.append(&mut nested);
            }

            synthetic_decls.append(&mut deep_synthetics);

            synthetic_decls
        }
        Err(e) => {
            resolver.errors.push(SemanticError::ImportError {
                message: format!("parse error in '{:?}': {:?}", path_buf, e),
                span: span.clone(),
            });
            Vec::new()
        }
    }
}
