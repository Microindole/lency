use crate::resolver::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{FunctionSymbol, GenericParamSymbol, ParameterSymbol, Symbol};
use crate::SemanticError;
use beryl_syntax::ast::{Decl, Type};

pub fn resolve_function(resolver: &mut Resolver, decl: &mut Decl) {
    if let Decl::Function {
        name: _,
        generic_params,
        params,
        return_type,
        body,
        span,
        ..
    } = decl
    {
        resolver.scopes.enter_scope(ScopeKind::Function);

        let mut gps = Vec::new();
        for param in generic_params {
            gps.push(GenericParamSymbol::new(
                param.name.clone(),
                param.bound.clone(),
                param.span.clone(),
            ));
        }
        for gp_symbol in gps {
            if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                resolver.errors.push(e);
            }
        }

        for (i, param) in params.iter().enumerate() {
            resolver.resolve_type(&param.ty, span);
            let param_symbol =
                ParameterSymbol::new(param.name.clone(), param.ty.clone(), span.clone(), i);
            if let Err(e) = resolver.scopes.define(Symbol::Parameter(param_symbol)) {
                resolver.errors.push(e);
            }
        }

        resolver.resolve_type(return_type, span);

        for stmt in body {
            resolver.resolve_stmt(stmt);
        }

        resolver.scopes.exit_scope();
    }
}

pub fn resolve_struct(resolver: &mut Resolver, decl: &mut Decl) {
    if let Decl::Struct {
        name: _,
        generic_params,
        fields,
        span,
        ..
    } = decl
    {
        let has_generics = !generic_params.is_empty();
        if has_generics {
            resolver.scopes.enter_scope(ScopeKind::Block);
            for gp in generic_params {
                let gp_symbol =
                    GenericParamSymbol::new(gp.name.clone(), gp.bound.clone(), gp.span.clone());
                if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                    resolver.errors.push(e);
                }
            }
        }

        for field in fields {
            resolver.resolve_type(&field.ty, span);
        }

        if has_generics {
            resolver.scopes.exit_scope();
        }
    }
}

pub fn resolve_impl(resolver: &mut Resolver, decl: &mut Decl) {
    if let Decl::Impl {
        trait_ref,
        type_name,
        generic_params,
        methods,
        span,
        ..
    } = decl
    {
        let struct_id = resolver.scopes.lookup_id(type_name);
        if struct_id.is_none() {
            resolver.errors.push(SemanticError::UndefinedType {
                name: type_name.clone(),
                span: span.clone(),
            });
            return;
        }

        let struct_id = struct_id.unwrap();
        if !matches!(
            resolver.scopes.get_symbol(struct_id),
            Some(Symbol::Struct(_))
        ) {
            resolver.errors.push(SemanticError::NotAStruct {
                name: type_name.clone(),
                span: span.clone(),
            });
            return;
        }

        if let Some(trait_name) = trait_ref {
            if let Some(trait_id) = resolver.scopes.lookup_id(trait_name) {
                if let Some(Symbol::Trait(trait_sym)) = resolver.scopes.get_symbol(trait_id) {
                    let impl_method_names: Vec<&str> = methods
                        .iter()
                        .filter_map(|m| {
                            if let Decl::Function { name, .. } = m {
                                Some(name.as_str())
                            } else {
                                None
                            }
                        })
                        .collect();

                    for trait_method in &trait_sym.methods {
                        if !impl_method_names.contains(&trait_method.name.as_str()) {
                            resolver.errors.push(SemanticError::MissingTraitMethod {
                                trait_name: trait_name.clone(),
                                method_name: trait_method.name.clone(),
                                span: span.clone(),
                            });
                        }
                    }

                    for method in methods.iter() {
                        if let Decl::Function {
                            name: method_name,
                            params,
                            return_type,
                            span,
                            ..
                        } = method
                        {
                            if let Some(trait_sig) =
                                trait_sym.methods.iter().find(|m| m.name == *method_name)
                            {
                                if return_type != &trait_sig.return_type {
                                    resolver.errors.push(
                                        SemanticError::TraitMethodSignatureMismatch {
                                            trait_name: trait_name.clone(),
                                            method_name: method_name.clone(),
                                            expected: trait_sig.return_type.to_string(),
                                            found: return_type.to_string(),
                                            span: span.clone(),
                                        },
                                    );
                                    continue;
                                }

                                if params.len() != trait_sig.params.len() {
                                    resolver.errors.push(
                                        SemanticError::TraitMethodSignatureMismatch {
                                            trait_name: trait_name.clone(),
                                            method_name: method_name.clone(),
                                            expected: format!("{} params", trait_sig.params.len()),
                                            found: format!("{} params", params.len()),
                                            span: span.clone(),
                                        },
                                    );
                                    continue;
                                }

                                for (i, param) in params.iter().enumerate() {
                                    let (_, trait_param_ty) = &trait_sig.params[i];
                                    if &param.ty != trait_param_ty {
                                        resolver.errors.push(
                                            SemanticError::TraitMethodSignatureMismatch {
                                                trait_name: trait_name.clone(),
                                                method_name: method_name.clone(),
                                                expected: trait_param_ty.to_string(),
                                                found: param.ty.to_string(),
                                                span: span.clone(),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }

                    // 注册 trait 方法到 StructSymbol
                    // 这样 u.greet() 可以直接调用
                    for method in methods.iter() {
                        if let Decl::Function {
                            name: method_name,
                            params,
                            return_type,
                            span: method_span,
                            ..
                        } = method
                        {
                            let func_sym = FunctionSymbol::new(
                                method_name.clone(),
                                params
                                    .iter()
                                    .map(|p| (p.name.clone(), p.ty.clone()))
                                    .collect(),
                                return_type.clone(),
                                method_span.clone(),
                            );

                            // 更新 StructSymbol 添加方法
                            if let Some(Symbol::Struct(ref mut struct_sym)) =
                                resolver.scopes.get_symbol_mut(struct_id)
                            {
                                struct_sym.add_method(method_name.clone(), func_sym);
                            }
                        }
                    }
                }
            } else {
                resolver.errors.push(SemanticError::UndefinedTrait {
                    name: trait_name.clone(),
                    span: span.clone(),
                });
            }
        } else {
            // 普通 impl 块 (无 trait): 也需要注册方法到 StructSymbol
            for method in methods.iter() {
                if let Decl::Function {
                    name: method_name,
                    params,
                    return_type,
                    span: method_span,
                    ..
                } = method
                {
                    let func_sym = FunctionSymbol::new(
                        method_name.clone(),
                        params
                            .iter()
                            .map(|p| (p.name.clone(), p.ty.clone()))
                            .collect(),
                        return_type.clone(),
                        method_span.clone(),
                    );

                    if let Some(Symbol::Struct(ref mut struct_sym)) =
                        resolver.scopes.get_symbol_mut(struct_id)
                    {
                        struct_sym.add_method(method_name.clone(), func_sym);
                    }
                }
            }
        }

        for method in methods {
            if let Decl::Function {
                params, body, span, ..
            } = method
            {
                resolver.scopes.enter_scope(ScopeKind::Function);

                for gp in generic_params.iter() {
                    if let Some(bound) = &gp.bound {
                        resolver.resolve_type(bound, &gp.span);
                    }
                    let gp_symbol =
                        GenericParamSymbol::new(gp.name.clone(), gp.bound.clone(), gp.span.clone());
                    if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                        resolver.errors.push(e);
                    }
                }

                let this_type = Type::Struct(type_name.clone());
                let this_param =
                    ParameterSymbol::new("this".to_string(), this_type, span.clone(), 0);
                if let Err(e) = resolver.scopes.define(Symbol::Parameter(this_param)) {
                    resolver.errors.push(e);
                }

                for (i, param) in params.iter().enumerate() {
                    resolver.resolve_type(&param.ty, span);

                    let param_symbol = ParameterSymbol::new(
                        param.name.clone(),
                        param.ty.clone(),
                        span.clone(),
                        i + 1,
                    );
                    if let Err(e) = resolver.scopes.define(Symbol::Parameter(param_symbol)) {
                        resolver.errors.push(e);
                    }
                }

                for stmt in body {
                    resolver.resolve_stmt(stmt);
                }

                resolver.scopes.exit_scope();
            }
        }
    }
}

pub fn resolve_trait(resolver: &mut Resolver, decl: &mut Decl) {
    if let Decl::Trait {
        generic_params,
        methods,
        span,
        ..
    } = decl
    {
        let has_generics = !generic_params.is_empty();
        if has_generics {
            resolver.scopes.enter_scope(ScopeKind::Block);
            for gp in generic_params {
                if let Some(bound) = &gp.bound {
                    resolver.resolve_type(bound, &gp.span);
                }
                let gp_symbol =
                    GenericParamSymbol::new(gp.name.clone(), gp.bound.clone(), gp.span.clone());
                if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                    resolver.errors.push(e);
                }
            }
        }

        for method in methods {
            resolver.resolve_type(&method.return_type, span);
            for param in &method.params {
                resolver.resolve_type(&param.ty, span);
            }
        }

        if has_generics {
            resolver.scopes.exit_scope();
        }
    }
}

pub fn resolve_enum(resolver: &mut Resolver, decl: &mut Decl) {
    if let Decl::Enum {
        variants,
        generic_params,
        span,
        ..
    } = decl
    {
        let has_generics = !generic_params.is_empty();
        if has_generics {
            resolver.scopes.enter_scope(ScopeKind::Block);
            for gp in generic_params {
                if let Some(bound) = &gp.bound {
                    resolver.resolve_type(bound, &gp.span);
                }
                let gp_symbol =
                    GenericParamSymbol::new(gp.name.clone(), gp.bound.clone(), gp.span.clone());
                if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                    resolver.errors.push(e);
                }
            }
        }

        for variant in variants {
            match variant {
                beryl_syntax::ast::EnumVariant::Unit(_) => {}
                beryl_syntax::ast::EnumVariant::Tuple(_, types) => {
                    for ty in types {
                        resolver.resolve_type(ty, span);
                    }
                }
            }
        }

        if has_generics {
            resolver.scopes.exit_scope();
        }
    }
}
