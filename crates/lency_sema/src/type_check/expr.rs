use super::TypeChecker;
use crate::error::SemanticError;
use crate::symbol::{FunctionSymbol, Symbol};
use crate::type_infer::{is_compatible, substitute_type};
use lency_syntax::ast::{Expr, ExprKind, Type};
use std::collections::HashMap;

/// 检查函数调用
pub fn check_call(
    checker: &mut TypeChecker,
    callee: &mut Expr,
    args: &mut [Expr],
    span: &std::ops::Range<usize>,
) -> Result<Type, SemanticError> {
    // 解析被调用者 (包括泛型实例化)
    let (func, is_method, subst_map) = match &mut callee.kind {
        ExprKind::GenericInstantiation {
            base,
            args: type_args,
        } => {
            // 泛型函数调用: func::<T>(...)
            match &mut base.kind {
                ExprKind::Variable(name) => {
                    match checker.scopes.lookup(name) {
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

                            // Build substitution map
                            let mut map = HashMap::new();
                            for (param, arg_ty) in f.generic_params.iter().zip(type_args.iter()) {
                                map.insert(param.name.as_str().to_string(), arg_ty.clone());
                            }
                            (f.clone(), false, map)
                        }
                        _ => {
                            return Err(SemanticError::NotCallable {
                                ty: name.clone(),
                                span: span.clone(),
                            })
                        }
                    }
                }
                _ => {
                    return Err(SemanticError::NotCallable {
                        ty: "complex generic expression".to_string(),
                        span: span.clone(),
                    })
                }
            }
        }
        ExprKind::Variable(name) => {
            // 普通函数调用
            match checker.scopes.lookup(name) {
                Some(Symbol::Function(f)) => (f.clone(), false, HashMap::new()),
                Some(Symbol::Struct(s)) => {
                    // 构造函数
                    let func_sym = FunctionSymbol {
                        name: name.clone(),
                        params: s
                            .fields
                            .iter()
                            .map(|(fname, finfo)| (fname.clone(), finfo.ty.clone()))
                            .collect(),
                        return_type: Type::Struct(name.clone()),
                        generic_params: s.generic_params.clone(), // Struct generic params
                        span: s.span.clone(),
                        is_public: true, // Constructors are usually public or match struct visibility
                    };
                    (func_sym, false, HashMap::new())
                }
                _ => {
                    return Err(SemanticError::NotCallable {
                        ty: name.clone(),
                        span: span.clone(),
                    });
                }
            }
        }
        ExprKind::Get { object, name } => {
            // 方法调用处理
            let obj_type = checker.infer_type(object)?;

            // Check if it's a type that supports method lookup via name (Structs or Primitives)
            let type_name_opt = match &obj_type {
                Type::Struct(n) => Some(n.clone()),
                Type::Int => Some("int".to_string()),
                Type::Bool => Some("bool".to_string()),
                Type::String => Some("string".to_string()),
                Type::Float => Some("float".to_string()),
                // 泛型实例化类型：使用 mangle 后的名称查找方法
                Type::Generic(base_name, _) => Some(base_name.clone()),
                _ => None,
            };

            if let Some(type_name) = type_name_opt {
                // 查找符号定义
                let symbol = checker.scopes.lookup(&type_name).cloned();

                match symbol {
                    Some(Symbol::Struct(struct_sym)) => {
                        if let Some(method) = struct_sym.get_method(name) {
                            let mut func = method.clone();
                            // Fix 'this' type for primitives
                            let this_type = match &obj_type {
                                Type::Struct(n) => Type::Struct(n.clone()),
                                _ => obj_type.clone(),
                            };
                            func.params.insert(0, ("this".to_string(), this_type));

                            // 对于泛型实例化类型，构建泛型参数替换映射
                            let subst_map = if let Type::Generic(_, type_args) = &obj_type {
                                let mut map = HashMap::new();
                                for (param, arg_ty) in
                                    struct_sym.generic_params.iter().zip(type_args.iter())
                                {
                                    map.insert(param.name.clone(), arg_ty.clone());
                                }
                                map
                            } else {
                                HashMap::new()
                            };

                            (func, true, subst_map)
                        } else {
                            return Err(SemanticError::UndefinedMethod {
                                class: type_name.clone(),
                                method: name.clone(),
                                span: span.clone(),
                            });
                        }
                    }
                    Some(Symbol::GenericParam(_)) => {
                        return Err(SemanticError::UndefinedType {
                            name: type_name.clone(),
                            span: object.span.clone(),
                        });
                    }
                    _ => {
                        return Err(SemanticError::UndefinedType {
                            name: type_name.clone(),
                            span: object.span.clone(),
                        });
                    }
                }
            } else {
                match obj_type {
                    Type::GenericParam(param_name) => {
                        // 泛型参数方法调用: t.foo() where T: Trait
                        if let Some(Symbol::GenericParam(gp_sym)) =
                            checker.scopes.lookup(&param_name)
                        {
                            if let Some(bound_ty) = &gp_sym.bound {
                                // 解析约束类型
                                // 目前假设 bound 是 Type::Struct(TraitName) 形式（由 Parser 生成）
                                if let Type::Struct(trait_name) = bound_ty {
                                    if let Some(Symbol::Trait(trait_sym)) =
                                        checker.scopes.lookup(trait_name)
                                    {
                                        if let Some(trait_method) = trait_sym.get_method(name) {
                                            // 从 Trait 方法签名构造 FunctionSymbol
                                            // 需要添加隐式 this 参数，类型为 GenericParam(T)
                                            let mut params = trait_method.params.clone();
                                            params.insert(
                                                0,
                                                (
                                                    "this".to_string(),
                                                    Type::GenericParam(param_name.clone()),
                                                ),
                                            );

                                            let func_sym = FunctionSymbol {
                                                name: trait_method.name.clone(),
                                                params,
                                                return_type: trait_method.return_type.clone(),
                                                generic_params: vec![], // Trait 方法特定的泛型参数？暂不支持
                                                span: trait_sym.span.clone(), // 使用 Trait 的 span 作为近似
                                                is_public: true, // Trait 方法通过接口总是可见的
                                            };
                                            (func_sym, true, HashMap::new())
                                        } else {
                                            return Err(SemanticError::UndefinedMethod {
                                                class: format!("Trait {}", trait_name),
                                                method: name.clone(),
                                                span: span.clone(),
                                            });
                                        }
                                    } else {
                                        return Err(SemanticError::UndefinedTrait {
                                            name: trait_name.clone(),
                                            span: span.clone(),
                                        });
                                    }
                                } else {
                                    return Err(SemanticError::NotCallable {
                                        ty: format!("Bounded type {:?}", bound_ty),
                                        span: span.clone(),
                                    });
                                }
                            } else {
                                return Err(SemanticError::NotCallable {
                                    ty: format!("Generic {} has no bounds", param_name),
                                    span: span.clone(),
                                });
                            }
                        } else {
                            return Err(SemanticError::UndefinedType {
                                name: param_name,
                                span: object.span.clone(),
                            });
                        }
                    }
                    Type::Vec(inner_type) => {
                        // Vec 内置方法处理
                        match name.as_str() {
                            "push" => {
                                // push(val)
                                if args.len() != 1 {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "push".to_string(),
                                        expected: 1,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                let arg_ty = checker.infer_type(&mut args[0])?;
                                if !is_compatible(&inner_type, &arg_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: inner_type.to_string(),
                                        found: arg_ty.to_string(),
                                        span: args[0].span.clone(),
                                    });
                                }
                                return Ok(Type::Void);
                            }
                            "pop" => {
                                // pop() -> T
                                if !args.is_empty() {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "pop".to_string(),
                                        expected: 0,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                return Ok(*inner_type);
                            }
                            "len" => {
                                // len() -> int
                                if !args.is_empty() {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "len".to_string(),
                                        expected: 0,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                return Ok(Type::Int);
                            }
                            "get" => {
                                // get(index) -> T
                                if args.len() != 1 {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "get".to_string(),
                                        expected: 1,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                let arg_ty = checker.infer_type(&mut args[0])?;
                                if !is_compatible(&Type::Int, &arg_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: "int".to_string(),
                                        found: arg_ty.to_string(),
                                        span: args[0].span.clone(),
                                    });
                                }
                                return Ok(*inner_type);
                            }
                            "set" => {
                                // set(index, val) -> void
                                if args.len() != 2 {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "set".to_string(),
                                        expected: 2,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                let index_ty = checker.infer_type(&mut args[0])?;
                                if !is_compatible(&Type::Int, &index_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: "int".to_string(),
                                        found: index_ty.to_string(),
                                        span: args[0].span.clone(),
                                    });
                                }
                                let val_ty = checker.infer_type(&mut args[1])?;
                                if !is_compatible(&inner_type, &val_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: inner_type.to_string(),
                                        found: val_ty.to_string(),
                                        span: args[1].span.clone(),
                                    });
                                }
                                return Ok(Type::Void);
                            }
                            _ => {
                                return Err(SemanticError::UndefinedMethod {
                                    class: format!("Vec<{}>", inner_type),
                                    method: name.clone(),
                                    span: span.clone(),
                                });
                            }
                        }
                    }
                    _ => {
                        return Err(SemanticError::NotAStruct {
                            name: obj_type.to_string(),
                            span: object.span.clone(),
                        });
                    }
                }
            }
        }
        _ => {
            // 复杂调用表达式暂不支持
            return Ok(Type::Error);
        }
    };

    // 检查参数数量
    // 如果是方法调用，定义中有隐式 this 参数，所以 args.len() + 1 应该等于 params.len()
    let expected_args = if is_method {
        func.params.len() - 1
    } else {
        func.params.len()
    };

    if args.len() != expected_args {
        return Err(SemanticError::ArgumentCountMismatch {
            name: func.name.clone(),
            expected: expected_args,
            found: args.len(),
            span: span.clone(),
        });
    }

    // 检查每个参数类型
    let skip_count = if is_method { 1 } else { 0 };
    let params_iter = func.params.iter().skip(skip_count);

    for (arg, (_, param_ty)) in args.iter_mut().zip(params_iter) {
        let arg_ty = checker.infer_type(arg)?;
        // 关键：检查参数前先替换其中的泛型参数
        let expected_ty = substitute_type(param_ty, &subst_map);

        if !is_compatible(&expected_ty, &arg_ty) {
            checker.errors.push(SemanticError::TypeMismatch {
                expected: expected_ty.to_string(),
                found: arg_ty.to_string(),
                span: arg.span.clone(), // Use arg.span
            });
        }
    }

    // 返回类型的泛型替换
    Ok(substitute_type(&func.return_type, &subst_map))
}
