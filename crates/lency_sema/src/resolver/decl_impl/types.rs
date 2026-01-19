use crate::resolver::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{GenericParamSymbol, Symbol};
use lency_syntax::ast::Decl;

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
