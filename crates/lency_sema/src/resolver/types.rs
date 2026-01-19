use super::Resolver;
use crate::error::SemanticError;
use crate::symbol::{GenericParamSymbol, Symbol};
use lency_syntax::ast::{Span, Type};

/// Normalize types (e.g., Vec<T> -> Type::Vec(T))
pub fn normalize_type(resolver: &mut Resolver, ty: &mut Type) {
    match ty {
        Type::Generic(name, args) if name == "Vec" => {
            if args.len() != 1 {
                return;
            }
            let inner = args.remove(0);
            *ty = Type::Vec(Box::new(inner));
            if let Type::Vec(inner) = ty {
                normalize_type(resolver, inner);
            }
        }
        Type::Generic(_, args) => {
            for arg in args {
                normalize_type(resolver, arg);
            }
        }
        Type::Nullable(inner) => {
            normalize_type(resolver, inner);
        }
        Type::Vec(inner) => {
            normalize_type(resolver, inner);
        }
        Type::Array { element_type, .. } => {
            normalize_type(resolver, element_type);
        }
        Type::Result { ok_type, err_type } => {
            normalize_type(resolver, ok_type);
            normalize_type(resolver, err_type);
        }
        Type::Struct(name) => {
            if let Some(Symbol::GenericParam(_)) = resolver.scopes.lookup(name) {
                *ty = Type::GenericParam(name.clone());
            }
        }
        _ => {}
    }
}

/// Normalize types with explicit list of known generic params
pub fn normalize_type_with_generics(
    resolver: &mut Resolver,
    ty: &mut Type,
    generics: &[GenericParamSymbol],
) {
    match ty {
        Type::Generic(name, args) if name == "Vec" => {
            if args.len() != 1 {
                return;
            }
            let inner = args.remove(0);
            *ty = Type::Vec(Box::new(inner));
            if let Type::Vec(inner) = ty {
                normalize_type_with_generics(resolver, inner, generics);
            }
        }
        Type::Generic(_, args) => {
            for arg in args {
                normalize_type_with_generics(resolver, arg, generics);
            }
        }
        Type::Nullable(inner) => {
            normalize_type_with_generics(resolver, inner, generics);
        }
        Type::Vec(inner) => {
            normalize_type_with_generics(resolver, inner, generics);
        }
        Type::Array { element_type, .. } => {
            normalize_type_with_generics(resolver, element_type, generics);
        }
        Type::Result { ok_type, err_type } => {
            normalize_type_with_generics(resolver, ok_type, generics);
            normalize_type_with_generics(resolver, err_type, generics);
        }
        Type::Struct(name) => {
            if generics.iter().any(|gp| &gp.name == name) {
                *ty = Type::GenericParam(name.clone());
            } else {
                if let Some(Symbol::GenericParam(_)) = resolver.scopes.lookup(name) {
                    *ty = Type::GenericParam(name.clone());
                }
            }
        }
        _ => {}
    }
}

/// 验证类型引用（包括泛型参数检查）
pub fn resolve_type(resolver: &mut Resolver, ty: &Type, span: &Span) {
    match ty {
        Type::Generic(name, args) => {
            let sym = resolver.scopes.lookup(name);
            match sym {
                Some(Symbol::Struct(s)) => {
                    if s.generic_params.len() != args.len() {
                        resolver.errors.push(SemanticError::GenericArityMismatch {
                            name: name.clone(),
                            expected: s.generic_params.len(),
                            found: args.len(),
                            span: span.clone(),
                        });
                    }
                }
                Some(Symbol::Trait(t)) => {
                    if t.generic_params.len() != args.len() {
                        resolver.errors.push(SemanticError::GenericArityMismatch {
                            name: name.clone(),
                            expected: t.generic_params.len(),
                            found: args.len(),
                            span: span.clone(),
                        });
                    }
                }
                Some(_) => {
                    resolver.errors.push(SemanticError::NotAGenericType {
                        name: name.clone(),
                        span: span.clone(),
                    });
                }
                None => {
                    resolver.errors.push(SemanticError::UndefinedType {
                        name: name.clone(),
                        span: span.clone(),
                    });
                }
            }
            for arg in args {
                resolve_type(resolver, arg, span);
            }
        }
        Type::Result { ok_type, err_type } => {
            resolve_type(resolver, ok_type, span);
            resolve_type(resolver, err_type, span);
        }
        Type::Struct(name) => match resolver.scopes.lookup(name) {
            Some(Symbol::Struct(s)) => {
                if !s.generic_params.is_empty() {
                    resolver.errors.push(SemanticError::GenericArityMismatch {
                        name: name.clone(),
                        expected: s.generic_params.len(),
                        found: 0,
                        span: span.clone(),
                    });
                }
            }
            Some(Symbol::GenericParam(_)) => {}
            Some(Symbol::Trait(t)) => {
                if !t.generic_params.is_empty() {
                    resolver.errors.push(SemanticError::GenericArityMismatch {
                        name: name.clone(),
                        expected: t.generic_params.len(),
                        found: 0,
                        span: span.clone(),
                    });
                }
            }
            Some(Symbol::Enum(e)) => {
                if !e.generic_params.is_empty() {
                    resolver.errors.push(SemanticError::GenericArityMismatch {
                        name: name.clone(),
                        expected: e.generic_params.len(),
                        found: 0,
                        span: span.clone(),
                    });
                }
            }
            Some(_) => {
                resolver.errors.push(SemanticError::UndefinedType {
                    name: name.clone(),
                    span: span.clone(),
                });
            }
            None => {
                resolver.errors.push(SemanticError::UndefinedType {
                    name: name.clone(),
                    span: span.clone(),
                });
            }
        },
        Type::Vec(inner)
        | Type::Array {
            element_type: inner,
            ..
        }
        | Type::Nullable(inner) => {
            resolve_type(resolver, inner, span);
        }
        _ => {}
    }
}
