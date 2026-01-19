use crate::resolver::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{GenericParamSymbol, ParameterSymbol, Symbol};
use lency_syntax::ast::Decl;

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
