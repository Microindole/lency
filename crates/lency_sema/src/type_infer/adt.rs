use crate::error::SemanticError;
use crate::symbol::Symbol;
use crate::type_infer::{is_compatible, TypeInferer};
use lency_syntax::ast::{Expr, ExprKind, Type};

impl<'a> TypeInferer<'a> {
    pub(crate) fn infer_adt(&mut self, expr: &mut Expr) -> Result<Type, SemanticError> {
        match &mut expr.kind {
            ExprKind::StructLiteral { type_, fields } => {
                // 解构类型名称和泛型参数
                let (type_name, generic_args) = match type_ {
                    Type::Struct(name) => (name, Vec::new()),
                    Type::Generic(name, args) => (name, args.clone()),
                    _ => {
                        return Err(SemanticError::UndefinedType {
                            name: type_.to_string(),
                            span: expr.span.clone(),
                        });
                    }
                };

                // Get struct info first (Clone to avoid holding borrow during inference)
                let struct_data = self.lookup(type_name).and_then(|sym| {
                    if let Symbol::Struct(s) = sym {
                        Some((s.generic_params.clone(), s.fields.clone()))
                    } else {
                        None
                    }
                });

                if let Some((struct_params, struct_fields)) = struct_data {
                    // 检查泛型参数数量
                    if struct_params.len() != generic_args.len() {
                        return Err(SemanticError::GenericArityMismatch {
                            name: type_name.clone(),
                            expected: struct_params.len(),
                            found: generic_args.len(),
                            span: expr.span.clone(),
                        });
                    }

                    // 构建泛型替换表
                    let mut subst_map = std::collections::HashMap::new();
                    for (param, arg) in struct_params.iter().zip(generic_args.iter()) {
                        subst_map.insert(param.name.clone(), arg.clone());
                    }

                    // 检查所有字段
                    for (field_name, field_expr) in fields {
                        // 验证字段存在
                        if let Some(field_info) = struct_fields.get(field_name) {
                            // 推导字段值的类型
                            let expr_ty = self.infer(field_expr)?;

                            // 获取期望类型并应用泛型替换
                            let expected_ty = substitute_type(&field_info.ty, &subst_map);

                            if !is_compatible(&expected_ty, &expr_ty) {
                                return Err(SemanticError::TypeMismatch {
                                    expected: expected_ty.to_string(),
                                    found: expr_ty.to_string(),
                                    span: field_expr.span.clone(),
                                });
                            }
                        } else {
                            return Err(SemanticError::UndefinedField {
                                class: type_name.clone(),
                                field: field_name.clone(),
                                span: field_expr.span.clone(),
                            });
                        }
                    }
                    Ok(type_.clone())
                } else {
                    Err(SemanticError::UndefinedType {
                        name: type_name.clone(),
                        span: expr.span.clone(),
                    })
                }
            }
            ExprKind::VecLiteral(elements) => {
                if elements.is_empty() {
                    return Ok(Type::Vec(Box::new(Type::Void)));
                }

                let mut common_type = self.infer(&mut elements[0])?;

                for elem in elements.iter_mut().skip(1) {
                    let elem_ty = self.infer(elem)?;

                    if common_type == elem_ty {
                        continue;
                    }

                    if common_type == Type::Int && elem_ty == Type::Float {
                        common_type = Type::Float;
                    } else if common_type != Type::Float || elem_ty != Type::Int {
                        return Err(SemanticError::TypeMismatch {
                            expected: common_type.to_string(),
                            found: elem_ty.to_string(),
                            span: elem.span.clone(),
                        });
                    }
                }
                Ok(Type::Vec(Box::new(common_type)))
            }
            ExprKind::GenericInstantiation { base: _, args: _ } => {
                Err(SemanticError::NotCallable {
                    ty: "Generic function usage as value not supported".into(),
                    span: expr.span.clone(),
                })
            }
            _ => unreachable!("Not an ADT expression"),
        }
    }
}

/// 替换类型中的泛型参数
pub(crate) fn substitute_type(
    ty: &Type,
    mapping: &std::collections::HashMap<String, Type>,
) -> Type {
    match ty {
        Type::GenericParam(name) => {
            if let Some(concrete) = mapping.get(name) {
                concrete.clone()
            } else {
                ty.clone()
            }
        }
        Type::Generic(name, args) => {
            let new_args = args
                .iter()
                .map(|arg| substitute_type(arg, mapping))
                .collect();
            Type::Generic(name.clone(), new_args)
        }
        Type::Vec(inner) => Type::Vec(Box::new(substitute_type(inner, mapping))),
        Type::Array { element_type, size } => Type::Array {
            element_type: Box::new(substitute_type(element_type, mapping)),
            size: *size,
        },
        Type::Nullable(inner) => Type::Nullable(Box::new(substitute_type(inner, mapping))),
        Type::Struct(name) => {
            if let Some(concrete) = mapping.get(name) {
                concrete.clone()
            } else {
                ty.clone()
            }
        }
        _ => ty.clone(),
    }
}
