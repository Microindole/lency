//! Type Inference
//!
//! 类型推导模块，处理 Lency 中 `var x = 10` 这种省略类型声明的情况。
//! 遵循 "Crystal Clear" 哲学：推导规则透明可预测。
//!
//! 模块结构遵循开闭原则 (OCP)，将不同类型的表达式推导逻辑分拆到子模块中。

mod access;
mod call;
mod control;
mod literal;
mod operators;

#[cfg(test)]
mod tests;

use crate::error::SemanticError;
use crate::operators::{BinaryOpRegistry, UnaryOpRegistry};
use crate::scope::{ScopeId, ScopeStack};
use crate::symbol::Symbol;
use lency_syntax::ast::{Expr, ExprKind, Type};

/// 类型推导器
///
/// 注意：TypeInferer 需要知道当前所在的作用域，
/// 因为变量查找需要从正确的作用域开始。
pub struct TypeInferer<'a> {
    pub(crate) scopes: &'a mut ScopeStack,
    /// 当前作用域 ID（由调用者设置，用于正确的符号查找）
    pub(crate) current_scope: ScopeId,
    /// 二元运算符注册表
    pub(crate) binary_ops: BinaryOpRegistry,
    /// 一元运算符注册表
    pub(crate) unary_ops: UnaryOpRegistry,
}

impl<'a> TypeInferer<'a> {
    pub fn new(scopes: &'a mut ScopeStack) -> Self {
        Self {
            current_scope: scopes.current_scope(),
            scopes,
            binary_ops: BinaryOpRegistry::new(),
            unary_ops: UnaryOpRegistry::new(),
        }
    }

    /// 创建一个指定作用域的推导器
    pub fn with_scope(scopes: &'a mut ScopeStack, scope_id: ScopeId) -> Self {
        Self {
            scopes,
            current_scope: scope_id,
            binary_ops: BinaryOpRegistry::new(),
            unary_ops: UnaryOpRegistry::new(),
        }
    }

    /// 设置当前作用域
    pub fn set_scope(&mut self, scope_id: ScopeId) {
        self.current_scope = scope_id;
    }

    /// 查找符号（从当前作用域向上）
    fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.scopes.lookup_from(name, self.current_scope)
    }

    /// 推导表达式的类型
    pub fn infer(&mut self, expr: &mut Expr) -> Result<Type, SemanticError> {
        match &mut expr.kind {
            ExprKind::Literal(lit) => Ok(self.infer_literal(lit)),

            ExprKind::Unit => Ok(Type::Void),

            ExprKind::Variable(name) => self.infer_variable(name, &expr.span),

            ExprKind::Binary(left, op, right) => self.infer_binary(left, op, right, &expr.span),

            ExprKind::Unary(op, operand) => self.infer_unary(op, operand, &expr.span),

            ExprKind::Call { callee, args } => self.infer_call(callee, args, &expr.span),

            ExprKind::Get { object, name } => self.infer_get(object, name, &expr.span),

            ExprKind::SafeGet { object, name } => self.infer_safe_get(object, name, &expr.span),

            ExprKind::Array(elements) => self.infer_array(elements, &expr.span),

            ExprKind::Index { array, index } => self.infer_index(array, index, &expr.span),

            ExprKind::Match {
                value,
                cases,
                default,
            } => self.infer_match(value, cases, default.as_deref_mut(), &expr.span),

            ExprKind::Print(print_expr) => {
                self.infer(print_expr)?;
                Ok(Type::Void)
            }

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
                    if let crate::symbol::Symbol::Struct(s) = sym {
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
                    // 空向量暂定为 Vec<void>，在兼容性检查时特殊处理？
                    // 或者暂不支持空向量字面量（因为无法推导类型）
                    // 更好的方式是让 is_compatible 允许 Vec<void> 赋值给 Vec<T> (如果 T != void)
                    // 但目前简单起见，返回 Vec<void>
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
                // To support `var f = func::<int>`, we need function types.
                // Since we don't have first-class function types yet, we return Error or a placeholder.
                // However, generic instantiations are usually part of a call.
                // If we encounter one detached, it's invalid unless we support func pointers.
                Err(SemanticError::NotCallable {
                    ty: "Generic function usage as value not supported".into(),
                    span: expr.span.clone(),
                })
            }
            // Result 相关表达式
            ExprKind::Try(inner) => {
                // expr? 解包 Result，返回 ok_type
                let inner_ty = self.infer(inner)?;
                match inner_ty {
                    Type::Result { ok_type, .. } => Ok(*ok_type),
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "Result<T, E>".to_string(),
                        found: inner_ty.to_string(),
                        span: expr.span.clone(),
                    }),
                }
            }
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
                // TODO: 通过上下文推导 ok_type
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
            // File I/O intrinsics (Sprint 12)
            ExprKind::ReadFile(path) => {
                // 验证 path 是 string
                let path_ty = self.infer(path)?;
                if path_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: path_ty.to_string(),
                        span: path.span.clone(),
                    });
                }
                // 返回 string! (Result<string, Error>)
                Ok(Type::Result {
                    ok_type: Box::new(Type::String),
                    err_type: Box::new(Type::Struct("Error".to_string())),
                })
            }
            ExprKind::WriteFile(path, content) => {
                // 验证 path 和 content 都是 string
                let path_ty = self.infer(path)?;
                let content_ty = self.infer(content)?;
                if path_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: path_ty.to_string(),
                        span: path.span.clone(),
                    });
                }
                if content_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: content_ty.to_string(),
                        span: content.span.clone(),
                    });
                }
                // 返回 void! (Result<void, Error>)
                Ok(Type::Result {
                    ok_type: Box::new(Type::Void),
                    err_type: Box::new(Type::Struct("Error".to_string())),
                })
            }
            // 字符串内置函数 (Sprint 12)
            ExprKind::Len(arg) => {
                // len(string) -> int
                let arg_ty = self.infer(arg)?;
                if arg_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: arg_ty.to_string(),
                        span: arg.span.clone(),
                    });
                }
                Ok(Type::Int)
            }
            ExprKind::Trim(arg) => {
                // trim(string) -> string
                let arg_ty = self.infer(arg)?;
                if arg_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: arg_ty.to_string(),
                        span: arg.span.clone(),
                    });
                }
                Ok(Type::String)
            }
            ExprKind::Split(str_arg, delim) => {
                // split(string, string) -> Vec<string>
                let str_ty = self.infer(str_arg)?;
                let delim_ty = self.infer(delim)?;
                if str_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: str_ty.to_string(),
                        span: str_arg.span.clone(),
                    });
                }
                if delim_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: delim_ty.to_string(),
                        span: delim.span.clone(),
                    });
                }
                Ok(Type::Vec(Box::new(Type::String)))
            }
            ExprKind::Join(vec_arg, sep) => {
                // join(Vec<string>, string) -> string
                let vec_ty = self.infer(vec_arg)?;
                let sep_ty = self.infer(sep)?;
                // 检查是否为 Vec<string>
                match &vec_ty {
                    Type::Vec(inner) if **inner == Type::String => {}
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: "Vec<string>".to_string(),
                            found: vec_ty.to_string(),
                            span: vec_arg.span.clone(),
                        });
                    }
                }
                if sep_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: sep_ty.to_string(),
                        span: sep.span.clone(),
                    });
                }
                Ok(Type::String)
            }
            ExprKind::Substr(str_arg, start, len_arg) => {
                // substr(string, int, int) -> string
                let str_ty = self.infer(str_arg)?;
                let start_ty = self.infer(start)?;
                let len_ty = self.infer(len_arg)?;
                if str_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: str_ty.to_string(),
                        span: str_arg.span.clone(),
                    });
                }
                if start_ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        found: start_ty.to_string(),
                        span: start.span.clone(),
                    });
                }
                if len_ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        found: len_ty.to_string(),
                        span: len_arg.span.clone(),
                    });
                }
                Ok(Type::String)
            }
        }
    }
}

/// 检查两个类型是否兼容（用于赋值）
pub fn is_compatible(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        // 完全相同
        (a, b) if a == b => true,

        // int 可以隐式转为 float（Lency 设计决策：这是唯一允许的隐式转换）
        (Type::Float, Type::Int) => true,

        // null 字面量 (Type::Nullable(Type::Error)) 可以赋给任何可空类型
        (Type::Nullable(_), Type::Nullable(inner)) if matches!(**inner, Type::Error) => true,

        // 可空类型可以接受非空类型
        (Type::Nullable(inner), actual) => is_compatible(inner, actual),

        // Vec 兼容性: Invariant (T must match T)
        // 特例：允许 Vec<void> (空字面量) 赋值给任意 Vec<T> ? 不，空字面量推导为 Vec<void> 比较危险。
        // 目前坚持 Invariant。如果需要支持空向量，可能需要更复杂的流式类型推导。
        // 简单处理：如果 T == Void (empty literal)，视为兼容。
        (Type::Vec(t1), Type::Vec(t2)) => {
            if matches!(**t2, Type::Void) {
                true
            } else {
                t1 == t2
            }
        }

        // Result 兼容性: Result<void, E> (来自 Err 构造器) 可以匹配任意 Result<T, E>
        (
            Type::Result {
                ok_type: expected_ok,
                err_type: expected_err,
            },
            Type::Result {
                ok_type: actual_ok,
                err_type: actual_err,
            },
        ) => {
            // 如果 actual_ok 是 Void (来自 Err 构造器)，视为兼容
            if matches!(**actual_ok, Type::Void) {
                is_compatible(expected_err, actual_err)
            } else {
                is_compatible(expected_ok, actual_ok) && is_compatible(expected_err, actual_err)
            }
        }

        // Error 类型用于错误恢复，总是兼容
        (Type::Error, _) | (_, Type::Error) => true,

        _ => false,
    }
}

/// 替换类型中的泛型参数 (简单的局部实现)
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
