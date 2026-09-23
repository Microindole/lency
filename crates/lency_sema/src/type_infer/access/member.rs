use super::super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use lency_syntax::ast::{Expr, ExprKind, Type};

impl<'a> TypeInferer<'a> {
    /// 推导成员访问类型
    pub(crate) fn infer_get_impl(
        &mut self,
        object: &mut Expr,
        name: &str,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // Special Case: Enum Static Access (Enum.Variant)
        let enum_access = match &object.kind {
            ExprKind::Variable(n) => Some((n.clone(), Vec::new())),
            ExprKind::GenericInstantiation { base, args } => {
                if let ExprKind::Variable(n) = &base.kind {
                    Some((n.clone(), args.clone()))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some((obj_name, args)) = enum_access {
            if let Some(Symbol::Enum(enum_sym)) = self.lookup(&obj_name) {
                if let Some(_variant_types) = enum_sym.get_variant(name) {
                    // Check Generic Arity
                    if !args.is_empty() {
                        if enum_sym.generic_params.len() != args.len() {
                            return Err(SemanticError::GenericArityMismatch {
                                name: obj_name.clone(),
                                expected: enum_sym.generic_params.len(),
                                found: args.len(),
                                span: span.clone(),
                            });
                        }
                        return Ok(Type::Generic(obj_name, args));
                    } else {
                        // Non-generic access
                        if !enum_sym.generic_params.is_empty() {
                            return Err(SemanticError::GenericArityMismatch {
                                name: obj_name.clone(),
                                expected: enum_sym.generic_params.len(),
                                found: 0,
                                span: span.clone(),
                            });
                        }
                        return Ok(Type::Struct(obj_name));
                    }
                } else {
                    return Err(SemanticError::UndefinedField {
                        class: enum_sym.name.clone(),
                        field: name.to_string(),
                        span: span.clone(),
                    });
                }
            }
        }

        // 推导对象类型
        let obj_ty = self.infer(object)?;

        match &obj_ty {
            // 结构体成员访问
            Type::Struct(struct_name) => {
                // 查找结构体定义并获取字段
                if let Some(crate::symbol::Symbol::Struct(struct_sym)) =
                    self.scopes.lookup_from(struct_name, self.current_scope)
                {
                    // 查找字段
                    if let Some(field_info) = struct_sym.get_field(name) {
                        return Ok(field_info.ty.clone());
                    } else {
                        return Err(SemanticError::UndefinedField {
                            class: struct_name.clone(),
                            field: name.to_string(),
                            span: span.clone(),
                        });
                    }
                }

                Err(SemanticError::NotAClass {
                    ty: struct_name.clone(),
                    span: span.clone(),
                })
            }

            // 泛型结构体成员访问
            Type::Generic(struct_name, args) => {
                if let Some(crate::symbol::Symbol::Struct(struct_sym)) =
                    self.scopes.lookup_from(struct_name, self.current_scope)
                {
                    if let Some(field_info) = struct_sym.get_field(name) {
                        // 构建泛型替换表
                        if struct_sym.generic_params.len() != args.len() {
                            return Err(SemanticError::GenericArityMismatch {
                                name: struct_name.clone(),
                                expected: struct_sym.generic_params.len(),
                                found: args.len(),
                                span: span.clone(),
                            });
                        }

                        let mut subst_map = std::collections::HashMap::new();
                        for (param, arg) in struct_sym.generic_params.iter().zip(args.iter()) {
                            subst_map.insert(param.name.clone(), arg.clone());
                        }

                        return Ok(crate::type_infer::substitute_type(
                            &field_info.ty,
                            &subst_map,
                        ));
                    } else {
                        return Err(SemanticError::UndefinedField {
                            class: struct_name.clone(),
                            field: name.to_string(),
                            span: span.clone(),
                        });
                    }
                }
                Err(SemanticError::NotAClass {
                    ty: struct_name.clone(),
                    span: span.clone(),
                })
            }

            // 数组的 .length 属性
            Type::Array { .. } => {
                if name == "length" {
                    Ok(Type::Int)
                } else {
                    Err(SemanticError::UndefinedField {
                        class: "Array".to_string(), // Array is not technicaly a class, but error msg fits
                        field: name.to_string(),
                        span: span.clone(),
                    })
                }
            }
            Type::Nullable(_inner) => {
                // 可空类型需要先检查 null
                Err(SemanticError::PossibleNullAccess {
                    ty: obj_ty.to_string(),
                    span: span.clone(),
                })
            }
            _ => Err(SemanticError::NotAClass {
                ty: obj_ty.to_string(),
                span: span.clone(),
            }),
        }
    }

    /// 推导安全成员访问类型 (?. )
    pub(crate) fn infer_safe_get_impl(
        &mut self,
        object: &mut Expr,
        name: &str,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let obj_ty = self.infer(object)?;

        // 用于查找成员的实际类型 (unwrap nullable)
        let inner_ty = match &obj_ty {
            Type::Nullable(inner) => inner.as_ref(),
            _ => &obj_ty, // 如果不是 nullable，也可以使用 ?. (只是冗余)
        };

        match inner_ty {
            Type::Struct(struct_name) => {
                if let Some(crate::symbol::Symbol::Struct(struct_sym)) =
                    self.scopes.lookup_from(struct_name, self.current_scope)
                {
                    if let Some(field_info) = struct_sym.get_field(name) {
                        // 结果必须是 nullable
                        match &field_info.ty {
                            Type::Nullable(_) => return Ok(field_info.ty.clone()),
                            _ => return Ok(Type::Nullable(Box::new(field_info.ty.clone()))),
                        }
                    } else {
                        return Err(SemanticError::UndefinedField {
                            class: struct_name.clone(),
                            field: name.to_string(),
                            span: span.clone(),
                        });
                    }
                }
                Err(SemanticError::NotAClass {
                    ty: struct_name.clone(),
                    span: span.clone(),
                })
            }
            // 数组的 .length 属性
            Type::Array { .. } => {
                if name == "length" {
                    // length is Int, result is Int?
                    Ok(Type::Nullable(Box::new(Type::Int)))
                } else {
                    Err(SemanticError::UndefinedField {
                        class: "Array".to_string(),
                        field: name.to_string(),
                        span: span.clone(),
                    })
                }
            }
            _ => Err(SemanticError::NotAClass {
                ty: obj_ty.to_string(),
                span: span.clone(),
            }),
        }
    }
}
