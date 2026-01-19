//! HashMap FFI Function Declarations
//!
//! HashMap Runtime FFI 函数的声明和获取

use crate::context::CodegenContext;
use crate::error::CodegenResult;
use inkwell::AddressSpace;

/// Get or declare lency_hashmap_new function
pub(super) fn get_or_declare_hashmap_new<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("lency_hashmap_new") {
        return Ok(func);
    }

    let i64_type = ctx.context.i64_type();
    let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = ptr_type.fn_type(&[i64_type.into()], false);

    Ok(ctx.module.add_function("lency_hashmap_new", fn_type, None))
}

/// Get or declare lency_hashmap_insert function
pub(super) fn get_or_declare_hashmap_insert<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("lency_hashmap_insert") {
        return Ok(func);
    }

    let i64_type = ctx.context.i64_type();
    let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = ctx
        .context
        .void_type()
        .fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);

    Ok(ctx
        .module
        .add_function("lency_hashmap_insert", fn_type, None))
}

/// Get or declare lency_hashmap_get function
pub(super) fn get_or_declare_hashmap_get<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("lency_hashmap_get") {
        return Ok(func);
    }

    let i64_type = ctx.context.i64_type();
    let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);

    Ok(ctx.module.add_function("lency_hashmap_get", fn_type, None))
}

/// Get or declare lency_hashmap_contains function
pub(super) fn get_or_declare_hashmap_contains<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("lency_hashmap_contains") {
        return Ok(func);
    }

    let i64_type = ctx.context.i64_type();
    let bool_type = ctx.context.bool_type();
    let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);

    Ok(ctx
        .module
        .add_function("lency_hashmap_contains", fn_type, None))
}

/// Get or declare lency_hashmap_remove function
pub(super) fn get_or_declare_hashmap_remove<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("lency_hashmap_remove") {
        return Ok(func);
    }

    let i64_type = ctx.context.i64_type();
    let bool_type = ctx.context.bool_type();
    let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);

    Ok(ctx
        .module
        .add_function("lency_hashmap_remove", fn_type, None))
}

/// Get or declare lency_hashmap_len function
pub(super) fn get_or_declare_hashmap_len<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("lency_hashmap_len") {
        return Ok(func);
    }

    let i64_type = ctx.context.i64_type();
    let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i64_type.fn_type(&[ptr_type.into()], false);

    Ok(ctx.module.add_function("lency_hashmap_len", fn_type, None))
}
