use super::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{ClassSymbol, FunctionSymbol, ParameterSymbol, Symbol};
use beryl_syntax::ast::Decl;

/// 收集顶层声明（Pass 1）
pub fn collect_decl(resolver: &mut Resolver, decl: &Decl) {
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

            if let Err(e) = resolver.scopes.define(Symbol::Function(func_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Class {
            name,
            generics,
            fields,
            span,
            ..
        } => {
            let mut class_symbol = ClassSymbol::new(name.clone(), generics.clone(), span.clone());

            // 收集字段
            for field in fields {
                class_symbol.add_field(
                    field.name.clone(),
                    field.ty.clone(),
                    span.clone(), // 使用类的 span（理想情况下应该用字段的 span）
                );
            }

            if let Err(e) = resolver.scopes.define(Symbol::Class(class_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::ExternFunction {
            name,
            params,
            return_type,
            span,
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

            if let Err(e) = resolver.scopes.define(Symbol::Function(func_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Struct { name, fields, span } => {
            let mut struct_symbol = crate::symbol::StructSymbol::new(name.clone(), span.clone());

            // 收集字段
            for field in fields {
                struct_symbol.add_field(field.name.clone(), field.ty.clone(), span.clone());
            }

            if let Err(e) = resolver.scopes.define(Symbol::Struct(struct_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Impl {
            type_name, methods, ..
        } => {
            // TODO: Register methods (Phase 2)
            let _ = (type_name, methods);
        }
    }
}

/// 解析声明（Pass 2）
pub fn resolve_decl(resolver: &mut Resolver, decl: &Decl) {
    match decl {
        Decl::Function {
            name: _,
            params,
            body,
            span,
            ..
        } => {
            // 进入函数作用域
            resolver.scopes.enter_scope(ScopeKind::Function);

            // 注册参数
            for (i, param) in params.iter().enumerate() {
                let param_symbol =
                    ParameterSymbol::new(param.name.clone(), param.ty.clone(), span.clone(), i);
                if let Err(e) = resolver.scopes.define(Symbol::Parameter(param_symbol)) {
                    resolver.errors.push(e);
                }
            }

            // 解析函数体
            for stmt in body {
                resolver.resolve_stmt(stmt);
            }

            // 退出函数作用域
            resolver.scopes.exit_scope();
        }
        Decl::Class { methods, .. } => {
            // 进入类作用域
            resolver.scopes.enter_scope(ScopeKind::Class);

            // 解析方法
            for method in methods {
                resolver.resolve_decl(method);
            }

            // 退出类作用域
            resolver.scopes.exit_scope();
        }
        Decl::ExternFunction { .. } => {
            // No body to resolve
        }
        Decl::Struct { .. } => {
            // TODO: Resolve struct fields (Phase 2)
        }
        Decl::Impl { methods, .. } => {
            // TODO: Resolve impl methods (Phase 2)
            for method in methods {
                resolve_decl(resolver, method);
            }
        }
    }
}
