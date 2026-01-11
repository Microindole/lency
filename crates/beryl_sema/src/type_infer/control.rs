use super::{is_compatible, TypeInferer};
use crate::error::SemanticError;
use crate::symbol::Symbol;
use beryl_syntax::ast::{Expr, MatchCase, MatchPattern, Type};

impl<'a> TypeInferer<'a> {
    pub(crate) fn infer_match(
        &mut self,
        value: &mut Expr,
        cases: &mut [MatchCase],
        _default: Option<&mut Expr>, // Deprecated/Unused
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let value_ty = self.infer(value)?;

        // Ensure we can match on this type (Int, Bool, String, Enum, Struct?)
        // Design doc: Match expression must return value.
        // Cases must have same return type.

        let mut ret_ty = Type::Error;
        let mut first = true;

        for case in cases.iter_mut() {
            // Enter scope for pattern bindings
            let scope_id = self.scopes.enter_scope(crate::scope::ScopeKind::Block);
            let parent_scope = self.current_scope;
            self.current_scope = scope_id;

            let pat_result = self.check_pattern(&case.pattern, &value_ty, &case.span);
            if let Err(e) = pat_result {
                self.scopes.exit_scope();
                self.current_scope = parent_scope;
                return Err(e);
            }

            let body_ty = self.infer(case.body.as_mut());
            self.scopes.exit_scope(); // Exit scope after inferring body
            self.current_scope = parent_scope;
            let body_ty = body_ty?;

            if first {
                ret_ty = body_ty;
                first = false;
            } else if !is_compatible(&ret_ty, &body_ty) {
                return Err(SemanticError::TypeMismatch {
                    expected: ret_ty.to_string(),
                    found: body_ty.to_string(),
                    span: case.body.span.clone(),
                });
            }
        }

        // Exhaustiveness check (Basic)
        // Check if there is a Wildcard or Variable pattern at top level
        // Or if all Enum variants are covered (requires collecting tags)
        // For Phase 4.1: Just warn if no wildcard?
        // Or assume user handles it.
        // If ret_ty is inferred, we assume OK.

        if cases.is_empty() {
            return Err(SemanticError::TypeMismatch {
                expected: "non-empty match".to_string(),
                found: "empty match".to_string(),
                span: span.clone(),
            });
        }

        Ok(ret_ty)
    }

    fn check_pattern(
        &mut self,
        pattern: &MatchPattern,
        target_ty: &Type,
        span: &std::ops::Range<usize>,
    ) -> Result<(), SemanticError> {
        match pattern {
            MatchPattern::Literal(lit) => {
                let pat_ty = self.infer_literal(lit);
                if pat_ty != *target_ty {
                    return Err(SemanticError::TypeMismatch {
                        expected: target_ty.to_string(),
                        found: pat_ty.to_string(),
                        span: span.clone(),
                    });
                }
                Ok(())
            }
            MatchPattern::Wildcard => Ok(()),
            MatchPattern::Variable(name) => {
                // Bind variable 'name' with type 'target_ty'
                // Check if shadowing? Shadowing allowed in new scope.
                let var_sym = crate::symbol::VariableSymbol::new(
                    name.clone(),
                    target_ty.clone(),
                    false, // immutable binding
                    span.clone(),
                );

                self.scopes.define(Symbol::Variable(var_sym))?;
                Ok(())
            }
            MatchPattern::Variant { name, sub_patterns } => {
                // Check if target_ty is Enum
                // Could be Type::Struct(enum_name) or Type::Generic(enum_name, args)
                let (enum_name, generic_args) = match target_ty {
                    Type::Struct(n) => (n, vec![]),
                    Type::Generic(n, args) => (n, args.clone()),
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: "Enum type".to_string(),
                            found: target_ty.to_string(),
                            span: span.clone(),
                        });
                    }
                };

                // Lookup Enum and Variant Info (Clone to avoid holding borrow)
                let (enum_generic_params, variant_field_types) =
                    if let Some(Symbol::Enum(e)) = self.lookup(enum_name) {
                        if let Some(types) = e.get_variant(name) {
                            (e.generic_params.clone(), types.clone())
                        } else {
                            return Err(SemanticError::UndefinedField {
                                class: enum_name.clone(),
                                field: name.clone(),
                                span: span.clone(),
                            });
                        }
                    } else {
                        return Err(SemanticError::UndefinedType {
                            name: enum_name.clone(),
                            span: span.clone(),
                        });
                    };

                // Check arity
                if sub_patterns.len() != variant_field_types.len() {
                    // Make nice error
                    return Err(SemanticError::TypeMismatch {
                        expected: format!("{} fields", variant_field_types.len()),
                        found: format!("{} patterns", sub_patterns.len()),
                        span: span.clone(),
                    });
                }

                // Check sub-patterns recursively
                // Need to substitute generics in field_types!
                // Map: Enum Generic Params -> generic_args

                for (i, sub_pat) in sub_patterns.iter().enumerate() {
                    let field_ty = &variant_field_types[i];

                    // Subst
                    let concrete_field_ty = if !generic_args.is_empty() {
                        self.substitute_generics(field_ty, &enum_generic_params, &generic_args)
                    } else {
                        field_ty.clone()
                    };

                    self.check_pattern(sub_pat, &concrete_field_ty, span)?;
                }

                Ok(())
            }
        }
    }

    // Helper for substitution
    // TODO: move to a shared utility
    fn substitute_generics(
        &self,
        ty: &Type,
        params: &[crate::symbol::GenericParamSymbol],
        args: &[Type],
    ) -> Type {
        match ty {
            Type::Struct(name) => {
                // Check if name matches any param
                for (i, param) in params.iter().enumerate() {
                    if *name == param.name && i < args.len() {
                        return args[i].clone();
                    }
                }
                Type::Struct(name.clone())
            }
            Type::Generic(name, inner_args) => {
                // Substitute args recursively
                let new_args: Vec<Type> = inner_args
                    .iter()
                    .map(|arg| self.substitute_generics(arg, params, args))
                    .collect();
                Type::Generic(name.clone(), new_args)
            }
            Type::Vec(inner) => Type::Vec(Box::new(self.substitute_generics(inner, params, args))),
            Type::Array { element_type, size } => Type::Array {
                element_type: Box::new(self.substitute_generics(element_type, params, args)),
                size: *size,
            },
            Type::Nullable(inner) => {
                Type::Nullable(Box::new(self.substitute_generics(inner, params, args)))
            }
            // ... handle other types
            _ => ty.clone(),
        }
    }
}
