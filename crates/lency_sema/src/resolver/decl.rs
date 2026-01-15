use crate::resolver::Resolver;

use crate::symbol::{
    FunctionSymbol, GenericParamSymbol, StructSymbol, TraitMethodSignature, TraitSymbol,
};
use crate::{SemanticError, Symbol};
use lency_syntax::ast::Decl;

/// 收集顶层声明（Pass 1）
/// 收集顶层声明（Pass 1）
pub fn collect_decl(resolver: &mut Resolver, decl: &Decl) -> Vec<Decl> {
    match decl {
        Decl::Import { items, span } => {
            let mut all_new_decls = Vec::new();
            for item in items {
                let mut decls = if let Some(alias_name) = &item.alias {
                    resolver.resolve_import_as(&item.path, alias_name, span)
                } else {
                    resolver.resolve_import(&item.path, span)
                };
                all_new_decls.append(&mut decls);
            }
            all_new_decls
        }
        Decl::Var { span, name, ty, .. } => {
            let mut ty = ty.clone().unwrap_or(lency_syntax::ast::Type::Void);
            resolver.normalize_type(&mut ty);
            let sym = crate::symbol::VariableSymbol::new(name.clone(), ty, true, span.clone());
            if let Err(e) = resolver.scopes.define(crate::symbol::Symbol::Variable(sym)) {
                resolver.errors.push(e);
            }
            Vec::new()
        }
        Decl::Function {
            name,
            generic_params,
            params,
            return_type,
            span,
            ..
        } => {
            let generic_param_symbols: Vec<GenericParamSymbol> = generic_params
                .iter()
                .map(|p| GenericParamSymbol::new(p.name.clone(), p.bound.clone(), p.span.clone()))
                .collect();

            let normalized_params: Vec<_> = params
                .iter()
                .map(|p| {
                    let mut ty = p.ty.clone();
                    resolver.normalize_type_with_generics(&mut ty, &generic_param_symbols);
                    (p.name.clone(), ty)
                })
                .collect();

            let mut normalized_return_type = return_type.clone();
            resolver
                .normalize_type_with_generics(&mut normalized_return_type, &generic_param_symbols);

            let func_symbol = FunctionSymbol::new_generic(
                name.clone(),
                generic_param_symbols.clone(),
                normalized_params,
                normalized_return_type,
                span.clone(),
            );

            if let Err(e) = resolver.scopes.define(Symbol::Function(func_symbol)) {
                resolver.errors.push(e);
            }
            Vec::new()
        }

        Decl::ExternFunction {
            name,
            generic_params,
            params,
            return_type,
            span,
            ..
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

            let normalized_params: Vec<_> = params
                .iter()
                .map(|p| {
                    let mut ty = p.ty.clone();
                    resolver.normalize_type_with_generics(&mut ty, &generic_param_symbols);
                    (p.name.clone(), ty)
                })
                .collect();

            let mut normalized_return_type = return_type.clone();
            resolver
                .normalize_type_with_generics(&mut normalized_return_type, &generic_param_symbols);

            let func_symbol = FunctionSymbol::new_generic(
                name.clone(),
                generic_param_symbols.clone(),
                normalized_params,
                normalized_return_type,
                span.clone(),
            );

            if let Err(e) = resolver.scopes.define(Symbol::Function(func_symbol)) {
                resolver.errors.push(e);
            }
            Vec::new()
        }
        Decl::Struct {
            name,
            generic_params,
            fields,
            span,
            ..
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

            let mut struct_symbol = StructSymbol::new_generic(
                name.clone(),
                generic_param_symbols.clone(),
                span.clone(),
            );

            for field in fields {
                let mut field_ty = field.ty.clone();
                resolver.normalize_type_with_generics(&mut field_ty, &generic_param_symbols);
                struct_symbol.add_field(field.name.clone(), field_ty, span.clone());
            }

            if let Err(e) = resolver.scopes.define(Symbol::Struct(struct_symbol)) {
                resolver.errors.push(e);
            }
            Vec::new()
        }
        Decl::Impl { .. } => Vec::new(),
        Decl::Trait {
            name,
            generic_params,
            methods,
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

            let mut trait_symbol = if generic_param_symbols.is_empty() {
                TraitSymbol::new(name.clone(), span.clone())
            } else {
                TraitSymbol::new_generic(name.clone(), generic_param_symbols, span.clone())
            };

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

            if let Err(e) = resolver.scopes.define(Symbol::Trait(trait_symbol)) {
                resolver.errors.push(e);
            }
            Vec::new()
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
                    lency_syntax::ast::EnumVariant::Unit(n) => {
                        enum_symbol.add_variant(n.clone(), vec![])
                    }
                    lency_syntax::ast::EnumVariant::Tuple(n, types) => {
                        enum_symbol.add_variant(n.clone(), types.clone())
                    }
                }
            }

            if let Err(e) = resolver.scopes.define(Symbol::Enum(enum_symbol)) {
                resolver.errors.push(e);
            }
            Vec::new()
        }
    }
}

/// Pass 1.5: 收集 impl 块中的方法到 StructSymbol
/// 注意：这需要在 collect_decl 之后单独调用，因为需要先收集所有 Struct
pub fn collect_impl_methods(resolver: &mut Resolver, decl: &Decl) {
    if let Decl::Impl {
        type_name,
        methods,
        generic_params,
        span,
        ..
    } = decl
    {
        // Extract symbol name from Type
        let target_name = match type_name {
            lency_syntax::ast::Type::Struct(name) => name.clone(),
            lency_syntax::ast::Type::Generic(name, _) => name.clone(),
            lency_syntax::ast::Type::Int => "int".to_string(),
            lency_syntax::ast::Type::Bool => "bool".to_string(),
            lency_syntax::ast::Type::String => "string".to_string(),
            lency_syntax::ast::Type::Float => "float".to_string(),
            _ => type_name.to_string(), // Fallback
        };

        // 查找对应的 Struct
        let struct_id = resolver.scopes.lookup_id(&target_name);
        if struct_id.is_none() {
            resolver.errors.push(SemanticError::UndefinedType {
                name: target_name,
                span: span.clone(),
            });
            return;
        }

        // Collect generic param symbols from Impl block
        let generic_param_symbols: Vec<GenericParamSymbol> = generic_params
            .iter()
            .map(|p| GenericParamSymbol::new(p.name.clone(), p.bound.clone(), p.span.clone()))
            .collect();

        // 1. Pre-process methods (Normalize types using resolver)
        // This requires mutable access to resolver, but NOT to struct_sym (scopes)
        let mut methods_to_add = Vec::new();
        for method in methods {
            if let Decl::Function {
                name,
                params,
                return_type,
                span,
                ..
            } = method
            {
                let normalized_params: Vec<_> = params
                    .iter()
                    .map(|p| {
                        let mut ty = p.ty.clone();
                        resolver.normalize_type_with_generics(&mut ty, &generic_param_symbols);
                        (p.name.clone(), ty)
                    })
                    .collect();

                let mut normalized_return_type = return_type.clone();
                resolver.normalize_type_with_generics(
                    &mut normalized_return_type,
                    &generic_param_symbols,
                );

                let func_symbol = FunctionSymbol::new(
                    name.clone(),
                    normalized_params,
                    normalized_return_type,
                    span.clone(),
                );
                methods_to_add.push((name.clone(), func_symbol));
            }
        }

        // 2. Add to Struct Symbol (Requires mutable access to scopes)
        // Now we can borrow resolver.scopes mutably without conflict
        let struct_id = struct_id.unwrap();
        if let Some(Symbol::Struct(struct_sym)) = resolver.scopes.get_symbol_mut(struct_id) {
            for (name, func_symbol) in methods_to_add {
                struct_sym.add_method(name, func_symbol);
            }
        } else {
            resolver.errors.push(SemanticError::NotAStruct {
                name: target_name,
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
        Decl::Var { value, .. } => {
            // Resolve initialization expression
            resolver.resolve_expr(value);
        }
        Decl::Import { .. } => {} // Noop for now
    }
}
