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
                    // 空向量暂定为 Vec<void>
                    return Ok(Type::Vec(Box::new(Type::Void)));
                }

                let mut common_type = self.infer(&mut elements[0])?;

                for elem in elements.iter_mut().skip(1) {
                    let elem_ty = self.infer(elem)?;

                    if common_type == elem_ty {
                        continue;
                    }

                    // 类型提升规则
                    if common_type == Type::Int && elem_ty == Type::Float {
                        common_type = Type::Float;
                    } else if common_type == Type::Float && elem_ty == Type::Int {
                        // Keep Float
                    } else {
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
            // Result 相关表达式
            ExprKind::Ok(inner) => {
                // Ok(x) 的类型是 Result<typeof(x), Error>
                let inner_ty = self.infer(inner)?;
                Ok(Type::Result {
                    ok_type: Box::new(inner_ty),
                    err_type: Box::new(Type::Struct("Error".to_string())),
                })
            }
            ExprKind::Err(inner) => {
                // Err(msg) 的类型需要知道 ok_type，暂时返回 Result<void, Error>
                self.infer(inner)?;
                Ok(Type::Result {
                    ok_type: Box::new(Type::Void),
                    err_type: Box::new(Type::Struct("Error".to_string())),
                })
            }
            // 闭包
            ExprKind::Closure { params, body } => {
                // 进入闭包作用域
                let scope_id = self.scopes.enter_scope(crate::scope::ScopeKind::Function);
                let parent_scope = self.current_scope;
                self.current_scope = scope_id;

                // 注册参数
                for (i, param) in params.iter().enumerate() {
                    let param_sym = crate::symbol::ParameterSymbol::new(
                        param.name.clone(),
                        param.ty.clone(),
                        expr.span.clone(),
                        i,
                    );
                    let _ = self.scopes.define(Symbol::Parameter(param_sym));
                }

                // 推导闭包体类型
                let body_ty = self.infer(body)?;

                self.scopes.exit_scope();
                self.current_scope = parent_scope;

                // 返回函数类型
                Ok(Type::Function {
                    param_types: params.iter().map(|p| p.ty.clone()).collect(),
                    return_type: Box::new(body_ty),
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
