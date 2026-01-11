use super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use beryl_syntax::ast::{Expr, ExprKind, Type};

impl<'a> TypeInferer<'a> {
    /// 推导函数调用类型
    /// 推导函数调用类型
    pub(crate) fn infer_call(
        &mut self,
        callee: &mut Expr,
        _args: &mut [Expr],
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        match &mut callee.kind {
            ExprKind::Variable(name) => {
                match self.lookup(name) {
                    Some(Symbol::Function(func)) => Ok(func.return_type.clone()),
                    Some(Symbol::Struct(s)) => {
                        // Constructor
                        Ok(Type::Struct(s.name.clone()))
                    }
                    Some(_) => Err(SemanticError::NotCallable {
                        ty: name.clone(),
                        span: span.clone(),
                    }),
                    None => Err(SemanticError::UndefinedFunction {
                        name: name.clone(),
                        span: span.clone(),
                    }),
                }
            }
            ExprKind::GenericInstantiation {
                base,
                args: type_args,
            } => {
                // 泛型函数调用
                if let ExprKind::Variable(name) = &base.kind {
                    match self.lookup(name) {
                        Some(Symbol::Function(f)) => {
                            // Check generic arg count
                            if f.generic_params.len() != type_args.len() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    name: format!("{} generic args", name),
                                    expected: f.generic_params.len(),
                                    found: type_args.len(),
                                    span: span.clone(),
                                });
                            }
                            // Subst map
                            let mut map = std::collections::HashMap::new();
                            for (param, arg_ty) in f.generic_params.iter().zip(type_args.iter()) {
                                map.insert(param.name.as_str().to_string(), arg_ty.clone());
                            }
                            // Substitute return type
                            Ok(crate::type_infer::substitute_type(&f.return_type, &map))
                        }
                        _ => Err(SemanticError::NotCallable {
                            ty: "generic instantiation".into(),
                            span: span.clone(),
                        }),
                    }
                } else {
                    Err(SemanticError::NotCallable {
                        ty: "complex generic instantiation".into(),
                        span: span.clone(),
                    })
                }
            }
            ExprKind::Get { object, name } => {
                // 1. Check Enum Constructor: Enum.Variant(...)
                let enum_access = match &object.kind {
                    ExprKind::Variable(name) => Some((name.clone(), Vec::new())),
                    ExprKind::GenericInstantiation { base, args } => {
                        if let ExprKind::Variable(name) = &base.kind {
                            Some((name.clone(), args.clone()))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some((enum_name, args)) = enum_access {
                    if let Some(Symbol::Enum(enum_sym)) = self.lookup(&enum_name) {
                        if enum_sym.get_variant(name).is_some() {
                            // Check Generic Arity
                            if !args.is_empty() {
                                if enum_sym.generic_params.len() != args.len() {
                                    return Err(SemanticError::GenericArityMismatch {
                                        name: enum_name.clone(),
                                        expected: enum_sym.generic_params.len(),
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                return Ok(Type::Generic(enum_name, args));
                            } else {
                                if !enum_sym.generic_params.is_empty() {
                                    return Err(SemanticError::GenericArityMismatch {
                                        name: enum_name.clone(),
                                        expected: enum_sym.generic_params.len(),
                                        found: 0,
                                        span: span.clone(),
                                    });
                                }
                                return Ok(Type::Struct(enum_name));
                            }
                        }
                    }
                }

                // 2. Check Method Call: obj.method(...)
                let obj_ty = self.infer(object)?;
                match obj_ty {
                    Type::Struct(struct_name) => {
                        // Check mangled name lookup: Struct_Method
                        let mangled = format!("{}_{}", struct_name, name);
                        if let Some(Symbol::Function(f)) = self.lookup(&mangled) {
                            return Ok(f.return_type.clone());
                        }

                        Err(SemanticError::UndefinedMethod {
                            class: struct_name,
                            method: name.clone(),
                            span: span.clone(),
                        })
                    }
                    Type::Vec(inner) => match name.as_str() {
                        "push" | "set" => Ok(Type::Void),
                        "pop" | "get" => Ok(*inner),
                        "len" => Ok(Type::Int),
                        _ => Err(SemanticError::UndefinedMethod {
                            class: "Vec".to_string(),
                            method: name.clone(),
                            span: span.clone(),
                        }),
                    },
                    _ => Err(SemanticError::NotCallable {
                        ty: obj_ty.to_string(),
                        span: span.clone(),
                    }),
                }
            }
            _ => {
                // 复杂调用表达式（如 (f())()），暂时返回 Error
                Ok(Type::Error)
            }
        }
    }
}
