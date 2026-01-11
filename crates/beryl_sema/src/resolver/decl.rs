use crate::resolver::Resolver;
use crate::scope::ScopeKind;
use crate::symbol::{
    FunctionSymbol, GenericParamSymbol, ParameterSymbol, StructSymbol, TraitMethodSignature,
    TraitSymbol,
};
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
                .map(|p| GenericParamSymbol::new(p.name.clone(), p.bound.clone(), p.span.clone()))
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
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

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
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

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
        Decl::Impl { methods: _, .. } => {
            // impl 块的方法在 collect_impl_methods 中处理
            // trait_ref 的验证在 resolve_decl 中处理
        }
        Decl::Trait {
            name,
            generic_params,
            methods,
            span,
        } => {
            // 创建泛型参数符号
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

            // 创建 Trait 符号
            let mut trait_symbol = if generic_param_symbols.is_empty() {
                TraitSymbol::new(name.clone(), span.clone())
            } else {
                TraitSymbol::new_generic(name.clone(), generic_param_symbols, span.clone())
            };

            // 收集方法签名
            for method in methods {
                let method_sig = TraitMethodSignature::new(
                    method.name.clone(),
                    method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    method.return_type.clone(),
                );
                trait_symbol.add_method(method_sig);
            }

            // 注册到符号表
            if let Err(e) = resolver.scopes.define(Symbol::Trait(trait_symbol)) {
                resolver.errors.push(e);
            }
        }
        Decl::Enum {
            name,
            generic_params,
            variants,
            span,
        } => {
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            let generic_param_symbols = gps;

            let mut enum_symbol = if generic_param_symbols.is_empty() {
                crate::symbol::EnumSymbol::new(name.clone(), span.clone())
            } else {
                crate::symbol::EnumSymbol::new_generic(
                    name.clone(),
                    generic_param_symbols,
                    span.clone(),
                )
            };

            for variant in variants {
                match variant {
                    beryl_syntax::ast::EnumVariant::Unit(n) => {
                        enum_symbol.add_variant(n.clone(), vec![])
                    }
                    beryl_syntax::ast::EnumVariant::Tuple(n, types) => {
                        enum_symbol.add_variant(n.clone(), types.clone())
                    }
                }
            }

            if let Err(e) = resolver.scopes.define(Symbol::Enum(enum_symbol)) {
                resolver.errors.push(e);
            }
        }
    }
}

/// Pass 1.5: 收集 impl 块中的方法到 StructSymbol
/// 注意：这需要在 collect_decl 之后单独调用，因为需要先收集所有 Struct
pub fn collect_impl_methods(resolver: &mut Resolver, decl: &Decl) {
    if let Decl::Impl {
        type_name,
        methods,
        span,
        ..
    } = decl
    {
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

/// 解析声明（Pass 2）
pub fn resolve_decl(resolver: &mut Resolver, decl: &mut Decl) {
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
            // 泛型参数处理
            let mut gps = Vec::new();
            for param in generic_params {
                gps.push(GenericParamSymbol::new(
                    param.name.clone(),
                    param.bound.clone(),
                    param.span.clone(),
                ));
            }
            for gp_symbol in gps {
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
                    let gp_symbol =
                        GenericParamSymbol::new(gp.name.clone(), gp.bound.clone(), gp.span.clone());
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
            trait_ref,
            type_name,
            generic_params,
            methods,
            span,
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

            // 如果是 impl Trait for Type，验证 Trait 存在并检查方法
            if let Some(trait_name) = trait_ref {
                if let Some(trait_id) = resolver.scopes.lookup_id(trait_name) {
                    if let Some(Symbol::Trait(trait_sym)) = resolver.scopes.get_symbol(trait_id) {
                        // 检查是否实现了 Trait 的所有方法
                        let impl_method_names: Vec<&str> = methods
                            .iter()
                            .filter_map(|m| {
                                if let Decl::Function { name, .. } = m {
                                    Some(name.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for trait_method in &trait_sym.methods {
                            if !impl_method_names.contains(&trait_method.name.as_str()) {
                                resolver.errors.push(SemanticError::MissingTraitMethod {
                                    trait_name: trait_name.clone(),
                                    method_name: trait_method.name.clone(),
                                    span: span.clone(),
                                });
                            }
                        }

                        // 验证方法签名匹配（参数类型、返回类型）
                        for method in methods.iter() {
                            if let Decl::Function {
                                name: method_name,
                                params,
                                return_type,
                                span,
                                ..
                            } = method
                            {
                                // 查找对应的 Trait 方法签名
                                if let Some(trait_sig) =
                                    trait_sym.methods.iter().find(|m| m.name == *method_name)
                                {
                                    // 1. 检查返回类型
                                    if return_type != &trait_sig.return_type {
                                        resolver.errors.push(
                                            SemanticError::TraitMethodSignatureMismatch {
                                                trait_name: trait_name.clone(),
                                                method_name: method_name.clone(),
                                                expected: trait_sig.return_type.to_string(),
                                                found: return_type.to_string(),
                                                span: span.clone(),
                                            },
                                        );
                                        continue;
                                    }

                                    // 2. 检查参数数量
                                    if params.len() != trait_sig.params.len() {
                                        resolver.errors.push(
                                            SemanticError::TraitMethodSignatureMismatch {
                                                trait_name: trait_name.clone(),
                                                method_name: method_name.clone(),
                                                expected: format!(
                                                    "{} params",
                                                    trait_sig.params.len()
                                                ),
                                                found: format!("{} params", params.len()),
                                                span: span.clone(),
                                            },
                                        );
                                        continue;
                                    }

                                    // 3. 检查参数类型
                                    for (i, param) in params.iter().enumerate() {
                                        let (_, trait_param_ty) = &trait_sig.params[i];
                                        if &param.ty != trait_param_ty {
                                            resolver.errors.push(
                                                SemanticError::TraitMethodSignatureMismatch {
                                                    trait_name: trait_name.clone(),
                                                    method_name: method_name.clone(),
                                                    expected: trait_param_ty.to_string(),
                                                    found: param.ty.to_string(),
                                                    span: span.clone(),
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    resolver.errors.push(SemanticError::UndefinedTrait {
                        name: trait_name.clone(),
                        span: span.clone(),
                    });
                }
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
                    for gp in generic_params.iter() {
                        if let Some(bound) = &gp.bound {
                            resolver.resolve_type(bound, &gp.span);
                        }
                        let gp_symbol = GenericParamSymbol::new(
                            gp.name.clone(),
                            gp.bound.clone(),
                            gp.span.clone(),
                        );
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

        Decl::Trait {
            generic_params,
            methods,
            span,
            ..
        } => {
            // 解析 Trait 定义中的类型

            // 1. 进入泛型作用域 (如果有)
            let has_generics = !generic_params.is_empty();
            if has_generics {
                resolver.scopes.enter_scope(ScopeKind::Block);
                for gp in generic_params {
                    if let Some(bound) = &gp.bound {
                        resolver.resolve_type(bound, &gp.span);
                    }
                    let gp_symbol =
                        GenericParamSymbol::new(gp.name.clone(), gp.bound.clone(), gp.span.clone());
                    if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                        resolver.errors.push(e);
                    }
                }
            }

            // 2. 解析每个方法的签名类型
            for method in methods {
                // 验证返回类型
                resolver.resolve_type(&method.return_type, span);

                // 验证参数类型
                for param in &method.params {
                    resolver.resolve_type(&param.ty, span);
                }
            }

            // 3. 退出作用域
            if has_generics {
                resolver.scopes.exit_scope();
            }
        }
        Decl::Enum {
            variants,
            generic_params,
            span,
            ..
        } => {
            // 如果是泛型枚举，创建临时作用域来注册泛型参数
            let has_generics = !generic_params.is_empty();
            if has_generics {
                resolver.scopes.enter_scope(ScopeKind::Block);
                // 注册泛型参数到作用域
                for gp in generic_params {
                    if let Some(bound) = &gp.bound {
                        resolver.resolve_type(bound, &gp.span);
                    }
                    let gp_symbol =
                        GenericParamSymbol::new(gp.name.clone(), gp.bound.clone(), gp.span.clone());
                    if let Err(e) = resolver.scopes.define(Symbol::GenericParam(gp_symbol)) {
                        resolver.errors.push(e);
                    }
                }
            }

            // 验证变体中的类型
            for variant in variants {
                match variant {
                    beryl_syntax::ast::EnumVariant::Unit(_) => {}
                    beryl_syntax::ast::EnumVariant::Tuple(_, types) => {
                        for ty in types {
                            resolver.resolve_type(ty, span);
                        }
                    }
                }
            }

            if has_generics {
                resolver.scopes.exit_scope();
            }
        }
    }
}
