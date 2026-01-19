//! HashMap Code Generation
//!
//! HashMap 方法调用的代码生成

//! HashMap 具体方法实现

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::AddressSpace;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

use super::ffi::*;

/// Generate code for hashmap_int_new() intrinsic
pub fn gen_hashmap_new_call<'ctx>(ctx: &CodegenContext<'ctx>) -> CodegenResult<CodegenValue<'ctx>> {
    let func = get_or_declare_hashmap_new(ctx)?;
    let capacity = ctx.context.i64_type().const_int(16, false);

    let call = ctx
        .builder
        .build_call(func, &[capacity.into()], "hashmap")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let ptr = call.try_as_basic_value().left().unwrap();

    // 返回 int (指针作为 int 处理)
    let ptr_as_int = ctx
        .builder
        .build_ptr_to_int(ptr.into_pointer_value(), ctx.context.i64_type(), "map_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: ptr_as_int.into(),
        ty: Type::Int,
    })
}

/// Generate code for hashmap method calls via extern functions
pub fn gen_hashmap_extern_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    func_name: &str,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    match func_name {
        "hashmap_int_new" => gen_hashmap_new_call(ctx),
        "hashmap_int_insert" => {
            let func = get_or_declare_hashmap_insert(ctx)?;

            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;
            let value_val = generate_expr(ctx, locals, &args[2])?;

            // 将 map (int) 转回指针
            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            ctx.builder
                .build_call(
                    func,
                    &[map_ptr.into(), key_val.value.into(), value_val.value.into()],
                    "",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            Ok(CodegenValue {
                value: ctx.context.i64_type().const_zero().into(),
                ty: Type::Void,
            })
        }
        "hashmap_int_get" => {
            let func = get_or_declare_hashmap_get(ctx)?;

            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;

            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let call = ctx
                .builder
                .build_call(func, &[map_ptr.into(), key_val.value.into()], "get_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Int,
            })
        }
        "hashmap_int_contains" => {
            let func = get_or_declare_hashmap_contains(ctx)?;

            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;

            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let call = ctx
                .builder
                .build_call(
                    func,
                    &[map_ptr.into(), key_val.value.into()],
                    "contains_res",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Bool,
            })
        }
        "hashmap_int_remove" => {
            let func = get_or_declare_hashmap_remove(ctx)?;

            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;

            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let call = ctx
                .builder
                .build_call(func, &[map_ptr.into(), key_val.value.into()], "remove_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Bool,
            })
        }
        "hashmap_int_len" => {
            let func = get_or_declare_hashmap_len(ctx)?;

            let map_val = generate_expr(ctx, locals, &args[0])?;

            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let call = ctx
                .builder
                .build_call(func, &[map_ptr.into()], "len_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Int,
            })
        }
        "hashmap_string_new" => {
            let fn_type = ctx
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .fn_type(&[], false);
            let func = ctx
                .module
                .get_function("lency_hashmap_string_new")
                .unwrap_or_else(|| {
                    ctx.module
                        .add_function("lency_hashmap_string_new", fn_type, None)
                });
            let call = ctx
                .builder
                .build_call(func, &[], "map")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let ptr = call.try_as_basic_value().left().unwrap();
            let ptr_as_int = ctx
                .builder
                .build_ptr_to_int(ptr.into_pointer_value(), ctx.context.i64_type(), "map_int")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: ptr_as_int.into(),
                ty: Type::Int,
            })
        }
        "hashmap_string_insert" => {
            let fn_type = ctx.context.void_type().fn_type(
                &[
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                    ctx.context.i64_type().into(),
                ],
                false,
            );
            let func = ctx
                .module
                .get_function("lency_hashmap_string_insert")
                .unwrap_or_else(|| {
                    ctx.module
                        .add_function("lency_hashmap_string_insert", fn_type, None)
                });
            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;
            let value_val = generate_expr(ctx, locals, &args[2])?;
            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_call(
                    func,
                    &[map_ptr.into(), key_val.value.into(), value_val.value.into()],
                    "",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: ctx.context.i64_type().const_zero().into(),
                ty: Type::Void,
            })
        }
        "hashmap_string_get" => {
            let fn_type = ctx.context.i64_type().fn_type(
                &[
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                ],
                false,
            );
            let func = ctx
                .module
                .get_function("lency_hashmap_string_get")
                .unwrap_or_else(|| {
                    ctx.module
                        .add_function("lency_hashmap_string_get", fn_type, None)
                });
            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;
            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let call = ctx
                .builder
                .build_call(func, &[map_ptr.into(), key_val.value.into()], "get_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Int,
            })
        }
        "hashmap_string_contains" => {
            let fn_type = ctx.context.bool_type().fn_type(
                &[
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                ],
                false,
            );
            let func = ctx
                .module
                .get_function("lency_hashmap_string_contains")
                .unwrap_or_else(|| {
                    ctx.module
                        .add_function("lency_hashmap_string_contains", fn_type, None)
                });
            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;
            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let call = ctx
                .builder
                .build_call(
                    func,
                    &[map_ptr.into(), key_val.value.into()],
                    "contains_res",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Bool,
            })
        }
        "hashmap_string_remove" => {
            let fn_type = ctx.context.bool_type().fn_type(
                &[
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                    ctx.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                ],
                false,
            );
            let func = ctx
                .module
                .get_function("lency_hashmap_string_remove")
                .unwrap_or_else(|| {
                    ctx.module
                        .add_function("lency_hashmap_string_remove", fn_type, None)
                });
            let map_val = generate_expr(ctx, locals, &args[0])?;
            let key_val = generate_expr(ctx, locals, &args[1])?;
            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let call = ctx
                .builder
                .build_call(func, &[map_ptr.into(), key_val.value.into()], "remove_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Bool,
            })
        }
        "hashmap_string_len" => {
            let fn_type = ctx.context.i64_type().fn_type(
                &[ctx
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()],
                false,
            );
            let func = ctx
                .module
                .get_function("lency_hashmap_string_len")
                .unwrap_or_else(|| {
                    ctx.module
                        .add_function("lency_hashmap_string_len", fn_type, None)
                });
            let map_val = generate_expr(ctx, locals, &args[0])?;
            let map_ptr = ctx
                .builder
                .build_int_to_ptr(
                    map_val.value.into_int_value(),
                    ctx.context.i8_type().ptr_type(AddressSpace::default()),
                    "map_ptr",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let call = ctx
                .builder
                .build_call(func, &[map_ptr.into()], "len_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Int,
            })
        }
        _ => Err(CodegenError::FunctionNotFound(func_name.to_string())),
    }
}
