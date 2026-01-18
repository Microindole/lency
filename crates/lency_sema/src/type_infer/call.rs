use super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use lency_syntax::ast::{Expr, ExprKind, Type};

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
                    // 支持调用函数类型的变量 (闭包)
                    Some(Symbol::Variable(var)) => {
                        if let Type::Function { return_type, .. } = &var.ty {
                            Ok(*return_type.clone())
                        } else {
                            Err(SemanticError::NotCallable {
                                ty: var.ty.to_string(),
                                span: span.clone(),
                            })
                        }
                    }
                    Some(Symbol::Parameter(param)) => {
                        if let Type::Function { return_type, .. } = &param.ty {
                            Ok(*return_type.clone())
                        } else {
                            Err(SemanticError::NotCallable {
                                ty: param.ty.to_string(),
                                span: span.clone(),
                            })
                        }
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
                        // Sprint 15: Special handling for Result.Ok and Result.Err
                        let is_result_builtin =
                            enum_name == "Result" && (name == "Ok" || name == "Err");

                        if enum_sym.get_variant(name).is_some() || is_result_builtin {
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
                // Check if it's a type that supports method lookup via name (Structs, Enums, or Primitives)
                let type_name_opt = match &obj_ty {
                    Type::Struct(n) => Some(n.clone()),
                    Type::Int => Some("int".to_string()),
                    Type::Bool => Some("bool".to_string()),
                    Type::String => Some("string".to_string()),
                    Type::Float => Some("float".to_string()),
                    // Sprint 15: Support Result<T,E> method calls
                    Type::Result { .. } => Some("Result".to_string()),
                    // 泛型实例化类型：使用基础名称查找方法
                    Type::Generic(base_name, _) => Some(base_name.clone()),
                    _ => None,
                };

                if let Some(type_name) = type_name_opt {
                    let symbol = self.lookup(&type_name).cloned();
                    match symbol {
                        Some(Symbol::Struct(struct_sym)) => {
                            if let Some(method) = struct_sym.get_method(name) {
                                // 对于泛型实例化类型，替换返回类型中的泛型参数
                                let return_type = if let Type::Generic(_, type_args) = &obj_ty {
                                    let mut map = std::collections::HashMap::new();
                                    for (param, arg_ty) in
                                        struct_sym.generic_params.iter().zip(type_args.iter())
                                    {
                                        map.insert(param.name.clone(), arg_ty.clone());
                                    }
                                    crate::type_infer::substitute_type(&method.return_type, &map)
                                } else {
                                    method.return_type.clone()
                                };
                                Ok(return_type)
                            } else {
                                Err(SemanticError::UndefinedMethod {
                                    class: type_name.clone(),
                                    method: name.clone(),
                                    span: span.clone(),
                                })
                            }
                        }
                        // Sprint 15: Support method calls on Enum types (e.g., Result<T,E>)
                        Some(Symbol::Enum(enum_sym)) => {
                            if let Some(method) = enum_sym.methods.get(name) {
                                // 对于泛型Result<T,E>，替换返回类型中的泛型参数
                                let return_type = if let Type::Result { ok_type, err_type } =
                                    &obj_ty
                                {
                                    let mut map = std::collections::HashMap::new();
                                    map.insert("T".to_string(), (**ok_type).clone());
                                    map.insert("E".to_string(), (**err_type).clone());
                                    crate::type_infer::substitute_type(&method.return_type, &map)
                                } else if let Type::Generic(_, type_args) = &obj_ty {
                                    let mut map = std::collections::HashMap::new();
                                    for (param, arg_ty) in
                                        enum_sym.generic_params.iter().zip(type_args.iter())
                                    {
                                        map.insert(param.name.clone(), arg_ty.clone());
                                    }
                                    crate::type_infer::substitute_type(&method.return_type, &map)
                                } else {
                                    method.return_type.clone()
                                };
                                Ok(return_type)
                            } else {
                                Err(SemanticError::UndefinedMethod {
                                    class: type_name.clone(),
                                    method: name.clone(),
                                    span: span.clone(),
                                })
                            }
                        }
                        Some(Symbol::GenericParam(_gp)) => {
                            // Copied logic for GenericParam fallback if needed, but here we matched specific types
                            // Actually, if type_name found GenericParam symbol... (e.g. shadowed)
                            // Re-use GenericParam logic?
                            // Since I extracted `type_name` from Primitives/Struct, it shouldn't be GenericParam unless I looked up "int" and found generic param "int".
                            // Assuming standard primitives, they map to Struct.
                            // If `obj_ty` was `Type::Struct`, it might be a generic param T if T resolved to Struct? No, T is Type::GenericParam.
                            // So this branch is mostly for `Type::Struct`.
                            // But wait, `Type::Struct(name)` handles generic instantiations? No, `Type::Generic`.
                            // If `obj_ty` is `Type::Struct`, it is concrete.
                            Err(SemanticError::UndefinedMethod {
                                class: type_name.clone(),
                                method: name.clone(),
                                span: span.clone(),
                            })
                        }
                        _ => Err(SemanticError::UndefinedMethod {
                            class: type_name.clone(),
                            method: name.clone(),
                            span: span.clone(),
                        }),
                    }
                } else {
                    match obj_ty {
                        Type::GenericParam(param_name) => {
                            if let Some(Symbol::GenericParam(gp)) = self.lookup(&param_name) {
                                if let Some(bound_ty) = &gp.bound {
                                    if let Type::Struct(trait_name) = bound_ty {
                                        if let Some(Symbol::Trait(trait_sym)) =
                                            self.lookup(trait_name)
                                        {
                                            if let Some(method) = trait_sym.get_method(name) {
                                                Ok(method.return_type.clone())
                                            } else {
                                                Err(SemanticError::UndefinedMethod {
                                                    class: format!("Trait {}", trait_name),
                                                    method: name.clone(),
                                                    span: span.clone(),
                                                })
                                            }
                                        } else {
                                            Err(SemanticError::UndefinedTrait {
                                                name: trait_name.clone(),
                                                span: span.clone(),
                                            })
                                        }
                                    } else {
                                        Err(SemanticError::NotCallable {
                                            ty: format!("{:?}", bound_ty),
                                            span: span.clone(),
                                        })
                                    }
                                } else {
                                    Err(SemanticError::NotCallable {
                                        ty: format!("Generic {} has no bounds", param_name),
                                        span: span.clone(),
                                    })
                                }
                            } else {
                                Err(SemanticError::UndefinedType {
                                    name: param_name.clone(),
                                    span: span.clone(),
                                })
                            }
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
            }
            _ => {
                // 复杂调用表达式（如 (f())()），暂时返回 Error
                Ok(Type::Error)
            }
        }
    }
}
