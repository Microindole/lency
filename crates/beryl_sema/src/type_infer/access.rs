use super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use beryl_syntax::ast::{Expr, ExprKind, Literal, Type};

impl<'a> TypeInferer<'a> {
    /// 推导变量类型
    pub(crate) fn infer_variable(
        &self,
        name: &str,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // 1. 优先检查 Flow Analysis Refinements (Smart Casts)
        if let Some(refined_type) = self.scopes.lookup_refinement(name) {
            return Ok(refined_type);
        }

        match self.lookup(name) {
            Some(symbol) => {
                match symbol.ty() {
                    Some(ty) => Ok(ty.clone()),
                    None => {
                        // 函数名不是值类型
                        if let Symbol::Function(func) = symbol {
                            // 返回函数类型的占位（暂时用 Void 表示）
                            // 未来可扩展为 FunctionType
                            Ok(func.return_type.clone())
                        } else {
                            Ok(Type::Error)
                        }
                    }
                }
            }
            None => {
                // 尝试隐式 this 访问
                if let Some(this_sym) = self.lookup("this") {
                    if let Some(Type::Struct(struct_name)) = this_sym.ty() {
                        // 查找结构体定义
                        if let Some(crate::symbol::Symbol::Struct(struct_def)) =
                            self.lookup(struct_name)
                        {
                            if let Some(field) = struct_def.get_field(name) {
                                return Ok(field.ty.clone());
                            }
                        }
                    }
                }

                Err(SemanticError::UndefinedVariable {
                    name: name.to_string(),
                    span: span.clone(),
                })
            }
        }
    }

    /// 推导成员访问类型
    pub(crate) fn infer_get(
        &self,
        object: &mut Expr,
        field_name: &str,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
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
                    if let Some(field_info) = struct_sym.get_field(field_name) {
                        return Ok(field_info.ty.clone());
                    } else {
                        return Err(SemanticError::UndefinedField {
                            class: struct_name.clone(),
                            field: field_name.to_string(),
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
                    if let Some(field_info) = struct_sym.get_field(field_name) {
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
                            field: field_name.to_string(),
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
                if field_name == "length" {
                    Ok(Type::Int)
                } else {
                    Err(SemanticError::UndefinedField {
                        class: "Array".to_string(), // Array is not technicaly a class, but error msg fits
                        field: field_name.to_string(),
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
    pub(crate) fn infer_safe_get(
        &self,
        object: &mut Expr,
        field_name: &str,
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
                    if let Some(field_info) = struct_sym.get_field(field_name) {
                        // 结果必须是 nullable
                        match &field_info.ty {
                            Type::Nullable(_) => return Ok(field_info.ty.clone()),
                            _ => return Ok(Type::Nullable(Box::new(field_info.ty.clone()))),
                        }
                    } else {
                        return Err(SemanticError::UndefinedField {
                            class: struct_name.clone(),
                            field: field_name.to_string(),
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
                if field_name == "length" {
                    // length is Int, result is Int?
                    Ok(Type::Nullable(Box::new(Type::Int)))
                } else {
                    Err(SemanticError::UndefinedField {
                        class: "Array".to_string(),
                        field: field_name.to_string(),
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

    /// 推导数组字面量类型
    pub(crate) fn infer_array(
        &self,
        elements: &mut [Expr],
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        if elements.is_empty() {
            // 空数组需要类型注解
            return Err(SemanticError::CannotInferType {
                name: "array literal".to_string(),
                span: span.clone(),
            });
        }

        // 推导第一个元素的类型作为数组元素类型
        let first_ty = self.infer(&mut elements[0])?;

        // 检查所有元素类型一致
        for elem in elements.iter_mut().skip(1) {
            let elem_ty = self.infer(elem)?;
            if elem_ty != first_ty {
                return Err(SemanticError::TypeMismatch {
                    expected: first_ty.to_string(),
                    found: elem_ty.to_string(),
                    span: elem.span.clone(),
                });
            }
        }

        // 返回固定大小数组类型: [T; N]
        Ok(Type::Array {
            element_type: Box::new(first_ty),
            size: elements.len(),
        })
    }

    /// 推导数组索引类型
    pub(crate) fn infer_index(
        &self,
        array: &mut Expr,
        index: &mut Expr,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let array_ty = self.infer(array)?;
        let index_ty = self.infer(index)?;

        // 索引必须是 int 类型
        if index_ty != Type::Int {
            return Err(SemanticError::TypeMismatch {
                expected: "int".to_string(),
                found: index_ty.to_string(),
                span: index.span.clone(),
            });
        }

        // 编译期边界检查：如果索引是常量，检查是否越界
        if let ExprKind::Literal(Literal::Int(idx_val)) = &index.kind {
            if let Type::Array { size, .. } = &array_ty {
                // 检查负数索引
                if *idx_val < 0 {
                    return Err(SemanticError::ArrayIndexOutOfBounds {
                        index: *idx_val,
                        size: *size,
                        span: index.span.clone(),
                    });
                }

                // 检查越界
                let idx_usize = *idx_val as usize;
                if idx_usize >= *size {
                    return Err(SemanticError::ArrayIndexOutOfBounds {
                        index: *idx_val,
                        size: *size,
                        span: index.span.clone(),
                    });
                }
            }
        }

        // 数组类型检查
        match &array_ty {
            Type::Array { element_type, .. } => Ok((**element_type).clone()),
            Type::Nullable(inner) => match **inner {
                Type::Array {
                    ref element_type, ..
                } => Ok((**element_type).clone()),
                Type::Generic(ref name, ref args) if name == "List" && !args.is_empty() => {
                    Ok(args[0].clone())
                }
                _ => Err(SemanticError::TypeMismatch {
                    expected: "array or list".to_string(),
                    found: array_ty.to_string(),
                    span: span.clone(),
                }),
            },
            Type::Generic(name, args) if name == "List" && !args.is_empty() => {
                // 动态数组 List<T>
                Ok(args[0].clone())
            }
            _ => Err(SemanticError::TypeMismatch {
                expected: "array or list".to_string(),
                found: array_ty.to_string(),
                span: span.clone(),
            }),
        }
    }
}
