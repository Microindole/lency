//! Method Call Code Generation
//!
//! 处理方法调用：object.method(args)

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::values::PointerValue;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 生成方法调用代码
pub fn gen_method_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (PointerValue<'ctx>, Type)>,
    object: &Expr,
    method_name: &str,
    args: &[Expr],
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 0. Check for Enum Constructor Call (Enum.Variant(args))
    let enum_check = match &object.kind {
        lency_syntax::ast::ExprKind::Variable(name) => Some((name.clone(), vec![])),
        lency_syntax::ast::ExprKind::GenericInstantiation { base, args } => {
            if let lency_syntax::ast::ExprKind::Variable(name) = &base.kind {
                Some((name.clone(), args.clone()))
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some((name, generic_args)) = enum_check {
        if ctx.enum_types.contains(&name) {
            // It's an Enum Constructor!
            // If generic args are present, we need to mangle the name (e.g. Option<int> -> Option_int)
            let struct_name = if !generic_args.is_empty() {
                lency_monomorph::mangling::mangle_type(&Type::Generic(name.clone(), generic_args))
            } else {
                name.clone()
            };

            let ctor_name = format!("{}_{}", struct_name, method_name);
            let function = ctx
                .module
                .get_function(&ctor_name)
                .ok_or_else(|| CodegenError::FunctionNotFound(ctor_name.clone()))?;

            let mut compiled_args = Vec::with_capacity(args.len());
            for arg in args {
                let arg_val = generate_expr(ctx, locals, arg)?;
                compiled_args.push(arg_val.value.into());
            }

            let call_site = ctx
                .builder
                .build_call(function, &compiled_args, "call_ctor")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let val = call_site
                .try_as_basic_value()
                .left()
                .ok_or(CodegenError::LLVMBuildError(
                    "Constructor returned void".into(),
                ))?;

            return Ok(CodegenValue {
                value: val,
                ty: Type::Struct(struct_name),
            });
        }
    }

    // 1. 计算对象表达式
    let object_val = generate_expr(ctx, locals, object)?;

    // 2. 根据对象类型分发
    // Clone type to avoid borrowing object_val, so we can move it
    let object_type = object_val.ty.clone();

    match object_type {
        Type::Vec(inner) => crate::expr::vec::gen_vec_method_call(
            ctx,
            locals,
            object_val,
            method_name,
            args,
            &inner,
        ),
        // Primitive types: int, string, bool
        Type::Int | Type::String | Type::Bool => {
            // 获取类型名称用于 mangling
            let type_name = match &object_val.ty {
                Type::Int => "int",
                Type::String => "string",
                Type::Bool => "bool",
                _ => unreachable!(),
            };

            // 构建 mangled name: int_hash, string_eq 等
            let mangled_name = format!("{}_{}", type_name, method_name);

            // 查找函数
            let function = ctx
                .module
                .get_function(&mangled_name)
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成参数列表
            // 对于 primitive types，第一个参数是 this 的值（不是指针）
            let mut compiled_args = Vec::with_capacity(args.len() + 1);

            // 将 this 值作为第一个参数
            compiled_args.push(object_val.value.into());

            // 添加其他参数
            for arg in args {
                let arg_val = generate_expr(ctx, locals, arg)?;
                compiled_args.push(arg_val.value.into());
            }

            // 获取返回类型
            let return_type = ctx
                .function_signatures
                .get(&mangled_name)
                .cloned()
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成调用
            let call_site = ctx
                .builder
                .build_call(function, &compiled_args, "call_method")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            // 处理返回值
            let val = call_site.try_as_basic_value().left();

            if let Some(v) = val {
                Ok(CodegenValue {
                    value: v,
                    ty: return_type,
                })
            } else {
                // Void 返回，生成 dummy 值
                let dummy = ctx.context.bool_type().const_int(0, false).into();
                Ok(CodegenValue {
                    value: dummy,
                    ty: Type::Void,
                })
            }
        }
        Type::Struct(name) => {
            // 获取 this 指针
            let this_ptr = if object_val.value.is_pointer_value() {
                object_val.value.into_pointer_value()
            } else {
                // 如果是值（右值结构体），分配临时空间
                let struct_type = object_val.value.get_type();
                let alloca = ctx
                    .builder
                    .build_alloca(struct_type, "this_tmp")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                ctx.builder
                    .build_store(alloca, object_val.value)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                alloca
            };

            let struct_name = name.clone();

            // 运行时 Null 检查
            if let Some(panic_func) = ctx.panic_func {
                crate::runtime::gen_null_check(
                    ctx.context,
                    &ctx.builder,
                    panic_func,
                    this_ptr,
                    line,
                );
            }

            // Sprint 15: Option 内置方法支持
            if struct_name.starts_with("Option__") {
                if let Some(res) = crate::expr::option::gen_option_builtin_method(
                    ctx,
                    locals,
                    this_ptr,
                    method_name,
                    args,
                    &struct_name,
                    &[], // Generic args not preserved in Codegen Type::Struct, passing empty
                )? {
                    return Ok(res);
                }
            }

            // Sprint 15: Result 内置方法支持 (Monomorphized)
            if let Some(suffix) = struct_name.strip_prefix("Result__") {
                // Try to parse T and E from "Result__{T}_{E}"
                // Use simplified heuristic: split at last "_" assuming E is "Error" or simple type

                // Heuristic: Split at last underscore
                // E.g. int_Error -> ok=int, err=Error
                // Vec_int_Error -> ok=Vec_int, err=Error (Assuming mangling preserves underscores)
                // Note: This is fragile if T ends with _ or E has _.
                // But generally E is "Error" in Lency std or simple.
                // If parsing fails or is ambiguous, we might have issues with unwrap return type.

                let (ok_part, err_part) = if let Some(idx) = suffix.rfind('_') {
                    (&suffix[..idx], &suffix[idx + 1..])
                } else {
                    (suffix, "Error") // Fallback
                };

                let ok_type = match ok_part {
                    "int" => Type::Int,
                    "bool" => Type::Bool,
                    "string" => Type::String,
                    "float" => Type::Float,
                    "void" => Type::Void,
                    x => Type::Struct(x.to_string()),
                };

                let err_type = match err_part {
                    "Error" => Type::Struct("Error".to_string()),
                    x => Type::Struct(x.to_string()),
                };

                if let Some(res) = crate::expr::result::gen_result_builtin_method(
                    ctx,
                    locals,
                    this_ptr,
                    method_name,
                    args,
                    &ok_type,
                    &err_type,
                    line,
                )? {
                    return Ok(res);
                }
            }

            // 构建 mangled name
            let mangled_name = format!("{}_{}", struct_name, method_name);

            // 查找函数
            let function = ctx
                .module
                .get_function(&mangled_name)
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成参数列表
            let mut compiled_args = Vec::with_capacity(args.len() + 1);

            // 将 this_ptr 作为第一个参数
            compiled_args.push(this_ptr.into());

            // 添加其他参数
            for arg in args {
                let arg_val = generate_expr(ctx, locals, arg)?;
                compiled_args.push(arg_val.value.into());
            }

            // 获取返回类型
            let return_type = ctx
                .function_signatures
                .get(&mangled_name)
                .cloned()
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成调用
            let call_site = ctx
                .builder
                .build_call(function, &compiled_args, "call_method")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            // 处理返回值
            let val = call_site.try_as_basic_value().left();

            if let Some(v) = val {
                Ok(CodegenValue {
                    value: v,
                    ty: return_type,
                })
            } else {
                // Void 返回，生成 dummy 值
                let dummy = ctx.context.bool_type().const_int(0, false).into();
                Ok(CodegenValue {
                    value: dummy,
                    ty: Type::Void,
                })
            }
        }
        Type::Result { ok_type, err_type } => {
            // Result 类型方法调用
            // Result 使用指针语义，类似 Struct
            let this_ptr = if object_val.value.is_pointer_value() {
                object_val.value.into_pointer_value()
            } else {
                return Err(CodegenError::TypeMismatch);
            };

            // 尝试使用内置方法实现
            if let Some(result) = crate::expr::result::gen_result_builtin_method(
                ctx,
                locals,
                this_ptr,
                method_name,
                args,
                &ok_type,
                &err_type,
                line,
            )? {
                return Ok(result);
            }

            // Fallback: 查找编译的方法函数
            // 构建 mangled 方法名：Result__int_Error_unwrap_or
            let result_type_mangled = lency_monomorph::mangling::mangle_type(&Type::Result {
                ok_type: ok_type.clone(),
                err_type: err_type.clone(),
            });
            let mangled_name = format!("{}_{}", result_type_mangled, method_name);

            // 查找函数
            let function = ctx
                .module
                .get_function(&mangled_name)
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成参数列表
            let mut compiled_args = Vec::with_capacity(args.len() + 1);

            // 将 this_ptr 作为第一个参数
            compiled_args.push(this_ptr.into());

            // 添加其他参数
            for arg in args {
                let arg_val = generate_expr(ctx, locals, arg)?;
                compiled_args.push(arg_val.value.into());
            }

            // 获取返回类型
            let return_type = ctx
                .function_signatures
                .get(&mangled_name)
                .cloned()
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成调用
            let call_site = ctx
                .builder
                .build_call(function, &compiled_args, "call_result_method")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            // 处理返回值
            let val = call_site.try_as_basic_value().left();

            if let Some(v) = val {
                Ok(CodegenValue {
                    value: v,
                    ty: return_type,
                })
            } else {
                // Void 返回，生成 dummy 值
                let dummy = ctx.context.bool_type().const_int(0, false).into();
                Ok(CodegenValue {
                    value: dummy,
                    ty: Type::Void,
                })
            }
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}
