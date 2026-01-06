use crate::resolver::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{FunctionSymbol, GenericParamSymbol, ParameterSymbol, StructSymbol};
use crate::{SemanticError, Symbol};
use beryl_syntax::ast::{Decl, Type};

/// 收集顶层声明（Pass 1）
pub fn collect_decl(resolver: &mut Resolver, decl: &Decl) {
    match decl {
        Decl::Function {
            name,
            generic_params,
            params,
            return_type,
            span,
            ..
        } => {
            // 创建泛型参数符号
            let generic_param_symbols: Vec<GenericParamSymbol> = generic_params
                .iter()
                .map(|p| GenericParamSymbol::new(p.clone(), span.clone()))
                .collect();

            let func_symbol = FunctionSymbol::new_generic(
                name.clone(),
                generic_param_symbols,
                params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect(),
                return_type.clone(),
                span.clone(),
            );

            if let Err(e) = resolver.scopes.define(Symbol::Function(func_symbol)) {
                resolver.errors.push(e);
            }
        }

        Decl::ExternFunction {
            name,
            generic_params,
            params,
            return_type,
            span,
            ..
        } => {
            // 创建泛型参数符号
            let generic_param_symbols: Vec<GenericParamSymbol> = generic_params
                .iter()
                .map(|p| GenericParamSymbol::new(p.clone(), span.clone()))
                .collect();

            let func_symbol = FunctionSymbol::new_generic(
                name.clone(),
                generic_param_symbols,
                params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect(),
                return_type.clone(),
                span.clone(),
            );

            if let Err(e) = resolver.scopes.define(Symbol::Function(func_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Struct {
            name,
            generic_params,
            fields,
            span,
            ..
        } => {
            // 创建泛型参数符号
            let generic_param_symbols: Vec<GenericParamSymbol> = generic_params
                .iter()
                .map(|p| GenericParamSymbol::new(p.clone(), span.clone()))
                .collect();

            let mut struct_symbol =
                StructSymbol::new_generic(name.clone(), generic_param_symbols, span.clone());

            // 收集字段
            for field in fields {
                struct_symbol.add_field(field.name.clone(), field.ty.clone(), span.clone());
            }

            if let Err(e) = resolver.scopes.define(Symbol::Struct(struct_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Impl {
            type_name,
            methods,
            span,
            ..
        } => {
            // 查找对应的 Struct
            let struct_id = resolver.scopes.lookup_id(type_name);
            if struct_id.is_none() {
                resolver.errors.push(SemanticError::UndefinedType {
                    name: type_name.clone(),
                    span: span.clone(),
                });
                return;
            }

            // 获取 StructSymbol 的可变引用
            let struct_id = struct_id.unwrap();
            if let Some(Symbol::Struct(struct_sym)) = resolver.scopes.get_symbol_mut(struct_id) {
                // 为每个方法创建 FunctionSymbol 并注册
                for method in methods {
                    if let Decl::Function {
                        name,
                        params,
                        return_type,
                        span,
                        ..
                    } = method
                    {
                        let func_symbol = FunctionSymbol::new(
                            name.clone(),
                            params
                                .iter()
                                .map(|p| (p.name.clone(), p.ty.clone()))
                                .collect(),
                            return_type.clone(),
                            span.clone(),
                        );
                        struct_sym.add_method(name.clone(), func_symbol);
                    }
                }
            } else {
                resolver.errors.push(SemanticError::NotAStruct {
                    name: type_name.clone(),
                    span: span.clone(),
                });
            }
        }
    }
}

/// 解析声明（Pass 2）
pub fn resolve_decl(resolver: &mut Resolver, decl: &Decl) {
    match decl {
        Decl::Function {
            name: _,
            generic_params,
            params,
            return_type,
            body,
            span,
            ..
        } => {
            // 进入函数作用域
            resolver.scopes.enter_scope(ScopeKind::Function);

            // 注册泛型参数到作用域
            // 这样在函数体中可以识别 T, U 等类型参数
            for gp in generic_params {
                let gp_symbol = GenericParamSymbol::new(gp.clone(), span.clone());
                if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                    resolver.errors.push(e);
                }
            }

            // 注册并验证参数
            for (i, param) in params.iter().enumerate() {
                // 验证参数类型
                resolver.resolve_type(&param.ty, span);

                let param_symbol =
                    ParameterSymbol::new(param.name.clone(), param.ty.clone(), span.clone(), i);
                if let Err(e) = resolver.scopes.define(Symbol::Parameter(param_symbol)) {
                    resolver.errors.push(e);
                }
            }

            // 验证返回类型
            resolver.resolve_type(return_type, span);

            // 解析函数体
            for stmt in body {
                resolver.resolve_stmt(stmt);
            }

            // 退出函数作用域
            resolver.scopes.exit_scope();
        }

        Decl::ExternFunction { .. } => {
            // No body to resolve
        }
        Decl::Struct {
            name: _,
            generic_params,
            fields,
            span,
            ..
        } => {
            // 如果是泛型结构体，创建临时作用域来注册泛型参数
            let has_generics = !generic_params.is_empty();
            if has_generics {
                resolver.scopes.enter_scope(ScopeKind::Block);
                // 注册泛型参数到作用域
                for gp in generic_params {
                    let gp_symbol = GenericParamSymbol::new(gp.clone(), span.clone());
                    if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                        resolver.errors.push(e);
                    }
                }
            }

            // 验证字段类型
            for field in fields {
                resolver.resolve_type(&field.ty, span);
            }

            if has_generics {
                resolver.scopes.exit_scope();
            }
        }
        Decl::Impl {
            type_name,
            generic_params,
            methods,
            span,
            ..
        } => {
            // 验证 Struct 存在
            let struct_id = resolver.scopes.lookup_id(type_name);
            if struct_id.is_none() {
                resolver.errors.push(SemanticError::UndefinedType {
                    name: type_name.clone(),
                    span: span.clone(),
                });
                return;
            }

            let struct_id = struct_id.unwrap();
            if !matches!(
                resolver.scopes.get_symbol(struct_id),
                Some(Symbol::Struct(_))
            ) {
                resolver.errors.push(SemanticError::NotAStruct {
                    name: type_name.clone(),
                    span: span.clone(),
                });
                return;
            }

            // 解析每个方法（添加隐式 this 参数和泛型参数）
            for method in methods {
                if let Decl::Function {
                    params, body, span, ..
                } = method
                {
                    // 进入方法作用域
                    resolver.scopes.enter_scope(ScopeKind::Function);

                    // 注册impl块的泛型参数到作用域
                    // 这样方法可以使用 impl<T> 声明的类型参数
                    for gp in generic_params {
                        let gp_symbol = GenericParamSymbol::new(gp.clone(), span.clone());
                        if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                            resolver.errors.push(e);
                        }
                    }

                    // 注册隐式 this 参数
                    let this_type = Type::Struct(type_name.clone());
                    let this_param = ParameterSymbol::new(
                        "this".to_string(),
                        this_type,
                        span.clone(),
                        0, // this 是第一个参数
                    );
                    if let Err(e) = resolver.scopes.define(Symbol::Parameter(this_param)) {
                        resolver.errors.push(e);
                    }

                    // 注册其他参数（索引从 1 开始）
                    for (i, param) in params.iter().enumerate() {
                        // 验证参数类型（在 Impl 块中，参数类型可能引用了 impl 的泛型参数）
                        resolver.resolve_type(&param.ty, span);

                        let param_symbol = ParameterSymbol::new(
                            param.name.clone(),
                            param.ty.clone(),
                            span.clone(),
                            i + 1, // this 是 0，所以从 1 开始
                        );
                        if let Err(e) = resolver.scopes.define(Symbol::Parameter(param_symbol)) {
                            resolver.errors.push(e);
                        }
                    }

                    // 解析方法体
                    for stmt in body {
                        resolver.resolve_stmt(stmt);
                    }

                    // 退出方法作用域
                    resolver.scopes.exit_scope();
                }
            }
        }
    }
}
