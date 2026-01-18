use crate::resolver::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{FunctionSymbol, GenericParamSymbol, ParameterSymbol, Symbol};
use crate::SemanticError;
use lency_syntax::ast::{Decl, Type};

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

        for (i, param) in params.iter_mut().enumerate() {
            resolver.normalize_type(&mut param.ty);
            resolver.resolve_type(&param.ty, span);
            let param_symbol =
                ParameterSymbol::new(param.name.clone(), param.ty.clone(), span.clone(), i);
            if let Err(e) = resolver.scopes.define(Symbol::Parameter(param_symbol)) {
                resolver.errors.push(e);
            }
        }

        resolver.normalize_type(return_type);
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
            resolver.normalize_type(&mut field.ty);
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
        // Helper to extract base name from Type
        let target_name = match type_name {
            Type::Struct(name) => name.clone(),
            Type::Generic(name, _) => name.clone(),
            Type::Int => "int".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Float => "float".to_string(),
            Type::Result { .. } => "Result".to_string(), // Sprint 15: 支持为Result<T,E>定义impl
            _ => "unknown".to_string(),
        };

        let struct_id = resolver.scopes.lookup_id(&target_name);
        if struct_id.is_none() {
            resolver.errors.push(SemanticError::UndefinedType {
                name: target_name.clone(),
                span: span.clone(),
            });
            return;
        }

        let struct_id = struct_id.unwrap();
        // Sprint 15: Allow impl for both Struct and Enum (e.g., Result<T,E>)
        let is_valid_target = matches!(
            resolver.scopes.get_symbol(struct_id),
            Some(Symbol::Struct(_)) | Some(Symbol::Enum(_))
        );
        if !is_valid_target {
            resolver.errors.push(SemanticError::NotAStruct {
                name: target_name.clone(),
                span: span.clone(),
            });
            return;
        }

        if let Some(trait_ty) = trait_ref {
            let trait_name_str = match trait_ty {
                Type::Struct(name) => name.clone(),
                Type::Generic(name, _) => name.clone(),
                _ => "unknown_trait".to_string(),
            };

            if let Some(trait_id) = resolver.scopes.lookup_id(&trait_name_str) {
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
                                trait_name: trait_name_str.clone(),
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
                                // Prepare Generic Substitution Map
                                let mut subst_map = std::collections::HashMap::new();
                                if let Type::Generic(_, args) = trait_ty {
                                    for (i, arg) in args.iter().enumerate() {
                                        if i < trait_sym.generic_params.len() {
                                            subst_map.insert(
                                                trait_sym.generic_params[i].name.clone(),
                                                arg.clone(),
                                            );
                                        }
                                    }
                                }

                                // Helper to substitute
                                let substitute = |ty: &Type| -> Type {
                                    match ty {
                                        Type::Generic(name, args) if args.is_empty() => {
                                            if let Some(replacement) = subst_map.get(name) {
                                                replacement.clone()
                                            } else {
                                                ty.clone()
                                            }
                                        }
                                        Type::Struct(name) => {
                                            if let Some(replacement) = subst_map.get(name) {
                                                replacement.clone()
                                            } else {
                                                ty.clone()
                                            }
                                        }
                                        Type::Generic(_name, _args) => {
                                            // Handle nested generics e.g. List<T> -> List<int>
                                            // Recursion needed? For Phase 2 simple substitution is likely enough
                                            // Assuming args can be substituted
                                            // Simple specific handling for standard generic types
                                            ty.clone()
                                        }
                                        _ => ty.clone(),
                                    }
                                };

                                let expected_ret = substitute(&trait_sig.return_type);
                                if return_type != &expected_ret {
                                    resolver.errors.push(
                                        SemanticError::TraitMethodSignatureMismatch {
                                            trait_name: trait_name_str.clone(),
                                            method_name: method_name.clone(),
                                            expected: expected_ret.to_string(),
                                            found: return_type.to_string(),
                                            span: span.clone(),
                                        },
                                    );
                                    continue;
                                }

                                if params.len() != trait_sig.params.len() {
                                    resolver.errors.push(
                                        SemanticError::TraitMethodSignatureMismatch {
                                            trait_name: trait_name_str.clone(),
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
                                    let expected_param = substitute(trait_param_ty);
                                    if param.ty != expected_param {
                                        resolver.errors.push(
                                            SemanticError::TraitMethodSignatureMismatch {
                                                trait_name: trait_name_str.clone(),
                                                method_name: method_name.clone(),
                                                expected: expected_param.to_string(),
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
                    let trait_generic_params = trait_sym.generic_params.clone();
                    for method in methods.iter() {
                        if let Decl::Function {
                            name: method_name,
                            params,
                            return_type,
                            span: method_span,
                            ..
                        } = method
                        {
                            // Normalize types for Symbol using generic params
                            let normalized_params: Vec<_> = params
                                .iter()
                                .map(|p| {
                                    let mut ty = p.ty.clone();
                                    resolver.normalize_type_with_generics(
                                        &mut ty,
                                        &trait_generic_params,
                                    );
                                    (p.name.clone(), ty)
                                })
                                .collect();

                            let mut normalized_return_type = return_type.clone();
                            resolver.normalize_type_with_generics(
                                &mut normalized_return_type,
                                &trait_generic_params,
                            );

                            let func_sym = FunctionSymbol::new(
                                method_name.clone(),
                                normalized_params,
                                normalized_return_type,
                                method_span.clone(),
                            );

                            // Sprint 15: Support adding methods to both Struct and Enum
                            match resolver.scopes.get_symbol_mut(struct_id) {
                                Some(Symbol::Struct(ref mut struct_sym)) => {
                                    struct_sym.add_method(method_name.clone(), func_sym);
                                }
                                Some(Symbol::Enum(ref mut enum_sym)) => {
                                    enum_sym.methods.insert(method_name.clone(), func_sym);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } else {
                resolver.errors.push(SemanticError::UndefinedTrait {
                    name: trait_name_str.clone(),
                    span: span.clone(),
                });
            }
        } else {
            // 普通 impl 块 (无 trait): 也需要注册方法到 StructSymbol
            // Coerce AST Generic Params to Symbols for normalization
            let generic_param_symbols: Vec<GenericParamSymbol> = generic_params
                .iter()
                .map(|p| GenericParamSymbol::new(p.name.clone(), p.bound.clone(), p.span.clone()))
                .collect();

            for method in methods.iter() {
                if let Decl::Function {
                    name: method_name,
                    params,
                    return_type,
                    span: method_span,
                    ..
                } = method
                {
                    // Normalize types using normalize_type_with_generics
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

                    let func_sym = FunctionSymbol::new(
                        method_name.clone(),
                        normalized_params,
                        normalized_return_type,
                        method_span.clone(),
                    );

                    // Sprint 15: Support adding methods to both Struct and Enum
                    match resolver.scopes.get_symbol_mut(struct_id) {
                        Some(Symbol::Struct(ref mut struct_sym)) => {
                            struct_sym.add_method(method_name.clone(), func_sym);
                        }
                        Some(Symbol::Enum(ref mut enum_sym)) => {
                            enum_sym.methods.insert(method_name.clone(), func_sym);
                        }
                        _ => {}
                    }
                }
            }
        }

        for method in methods {
            if let Decl::Function {
                params,
                return_type,
                body,
                span,
                ..
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

                // Make `this` type generic-aware if needed
                // Currently using implicit type_name assumption
                // NOTE: Using type_name.clone() which is now Type!
                // We need Type for ParameterSymbol
                let mut this_type = type_name.clone();
                resolver.normalize_type(&mut this_type);

                let this_param =
                    ParameterSymbol::new("this".to_string(), this_type, span.clone(), 0);
                if let Err(e) = resolver.scopes.define(Symbol::Parameter(this_param)) {
                    resolver.errors.push(e);
                }

                for (i, param) in params.iter_mut().enumerate() {
                    resolver.normalize_type(&mut param.ty);
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

                // CRITICAL: Normalize return type in AST!
                resolver.normalize_type(return_type);
                resolver.resolve_type(return_type, span);

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
            resolver.normalize_type(&mut method.return_type);
            resolver.resolve_type(&method.return_type, span);
            for param in &mut method.params {
                resolver.normalize_type(&mut param.ty);
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
                lency_syntax::ast::EnumVariant::Unit(_) => {}
                lency_syntax::ast::EnumVariant::Tuple(_, types) => {
                    for ty in types {
                        resolver.normalize_type(ty);
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
