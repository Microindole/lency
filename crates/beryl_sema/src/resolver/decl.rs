use crate::resolver::Resolver;

use crate::symbol::{
    FunctionSymbol, GenericParamSymbol, StructSymbol, TraitMethodSignature, TraitSymbol,
};
use crate::{SemanticError, Symbol};
use beryl_syntax::ast::Decl;

/// 收集顶层声明（Pass 1）
pub fn collect_decl(resolver: &mut Resolver, decl: &Decl) {
    match decl {
        Decl::Function {
            name,
            generic_params,
            params,
            return_type,
            span,
            ..
        } => {
            // 创建泛型参数符号
            let generic_param_symbols: Vec<GenericParamSymbol> = generic_params
                .iter()
                .map(|p| GenericParamSymbol::new(p.name.clone(), p.bound.clone(), p.span.clone()))
                .collect();

            let func_symbol = FunctionSymbol::new_generic(
                name.clone(),
                generic_param_symbols,
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

        Decl::ExternFunction {
            name,
            generic_params,
            params,
            return_type,
            span,
            ..
        } => {
            // 创建泛型参数符号
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

            let func_symbol = FunctionSymbol::new_generic(
                name.clone(),
                generic_param_symbols,
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
        Decl::Struct {
            name,
            generic_params,
            fields,
            span,
            ..
        } => {
            // 创建泛型参数符号
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

            let mut struct_symbol =
                StructSymbol::new_generic(name.clone(), generic_param_symbols, span.clone());

            // 收集字段
            for field in fields {
                struct_symbol.add_field(field.name.clone(), field.ty.clone(), span.clone());
            }

            if let Err(e) = resolver.scopes.define(Symbol::Struct(struct_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Impl { methods: _, .. } => {
            // impl 块的方法在 collect_impl_methods 中处理
            // trait_ref 的验证在 resolve_decl 中处理
        }
        Decl::Trait {
            name,
            generic_params,
            methods,
            span,
        } => {
            // 创建泛型参数符号
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

            // 创建 Trait 符号
            let mut trait_symbol = if generic_param_symbols.is_empty() {
                TraitSymbol::new(name.clone(), span.clone())
            } else {
                TraitSymbol::new_generic(name.clone(), generic_param_symbols, span.clone())
            };

            // 收集方法签名
            for method in methods {
                let method_sig = TraitMethodSignature::new(
                    method.name.clone(),
                    method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    method.return_type.clone(),
                );
                trait_symbol.add_method(method_sig);
            }

            // 注册到符号表
            if let Err(e) = resolver.scopes.define(Symbol::Trait(trait_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Enum {
            name,
            generic_params,
            variants,
            span,
        } => {
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

            let mut enum_symbol = if generic_param_symbols.is_empty() {
                crate::symbol::EnumSymbol::new(name.clone(), span.clone())
            } else {
                crate::symbol::EnumSymbol::new_generic(
                    name.clone(),
                    generic_param_symbols,
                    span.clone(),
                )
            };

            for variant in variants {
                match variant {
                    beryl_syntax::ast::EnumVariant::Unit(n) => {
                        enum_symbol.add_variant(n.clone(), vec![])
                    }
                    beryl_syntax::ast::EnumVariant::Tuple(n, types) => {
                        enum_symbol.add_variant(n.clone(), types.clone())
                    }
                }
            }

            if let Err(e) = resolver.scopes.define(Symbol::Enum(enum_symbol)) {
                resolver.errors.push(e);
            }
        }
    }
}

/// Pass 1.5: 收集 impl 块中的方法到 StructSymbol
/// 注意：这需要在 collect_decl 之后单独调用，因为需要先收集所有 Struct
pub fn collect_impl_methods(resolver: &mut Resolver, decl: &Decl) {
    if let Decl::Impl {
        type_name,
        methods,
        span,
        ..
    } = decl
    {
        // 查找对应的 Struct
        let struct_id = resolver.scopes.lookup_id(type_name);
        if struct_id.is_none() {
            resolver.errors.push(SemanticError::UndefinedType {
                name: type_name.clone(),
                span: span.clone(),
            });
            return;
        }

        // 获取 StructSymbol 的可变引用
        let struct_id = struct_id.unwrap();
        if let Some(Symbol::Struct(struct_sym)) = resolver.scopes.get_symbol_mut(struct_id) {
            // 为每个方法创建 FunctionSymbol 并注册
            for method in methods {
                if let Decl::Function {
                    name,
                    params,
                    return_type,
                    span,
                    ..
                } = method
                {
                    let func_symbol = FunctionSymbol::new(
                        name.clone(),
                        params
                            .iter()
                            .map(|p| (p.name.clone(), p.ty.clone()))
                            .collect(),
                        return_type.clone(),
                        span.clone(),
                    );
                    struct_sym.add_method(name.clone(), func_symbol);
                }
            }
        } else {
            resolver.errors.push(SemanticError::NotAStruct {
                name: type_name.clone(),
                span: span.clone(),
            });
        }
    }
}

/// 解析声明（Pass 2）
pub fn resolve_decl(resolver: &mut Resolver, decl: &mut Decl) {
    match decl {
        Decl::Function { .. } => super::decl_impl::resolve_function(resolver, decl),
        Decl::ExternFunction { .. } => {
            // No body to resolve
        }
        Decl::Struct { .. } => super::decl_impl::resolve_struct(resolver, decl),
        Decl::Impl { .. } => super::decl_impl::resolve_impl(resolver, decl),
        Decl::Trait { .. } => super::decl_impl::resolve_trait(resolver, decl),
        Decl::Enum { .. } => super::decl_impl::resolve_enum(resolver, decl),
    }
}
