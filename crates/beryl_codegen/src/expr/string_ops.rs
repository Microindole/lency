//! String Operations Code Generation
//!
//! 字符串操作代码生成，包含 C 运行时函数声明

use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// 生成字符串连接代码
pub(super) fn concat<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: PointerValue<'ctx>,
    rhs: PointerValue<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let strlen_fn = get_or_declare_strlen(ctx);
    let malloc_fn = get_or_declare_malloc(ctx);
    let strcpy_fn = get_or_declare_strcpy(ctx);
    let strcat_fn = get_or_declare_strcat(ctx);

    let len1 = ctx
        .builder
        .build_call(strlen_fn, &[lhs.into()], "len1")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    let len2 = ctx
        .builder
        .build_call(strlen_fn, &[rhs.into()], "len2")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    let total_len = ctx
        .builder
        .build_int_add(len1, len2, "total_len")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let total_len_plus_one = ctx
        .builder
        .build_int_add(
            total_len,
            ctx.context.i64_type().const_int(1, false),
            "total_len_p1",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result_ptr = ctx
        .builder
        .build_call(malloc_fn, &[total_len_plus_one.into()], "concat_result")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_pointer_value();

    ctx.builder
        .build_call(strcpy_fn, &[result_ptr.into(), lhs.into()], "")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    ctx.builder
        .build_call(strcat_fn, &[result_ptr.into(), rhs.into()], "")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(result_ptr.into())
}

fn get_or_declare_strlen<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("strlen") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = ctx.context.i64_type().fn_type(&[i8_ptr_type.into()], false);
    ctx.module.add_function("strlen", fn_type, None)
}

fn get_or_declare_malloc<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("malloc") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i8_ptr_type.fn_type(&[ctx.context.i64_type().into()], false);
    ctx.module.add_function("malloc", fn_type, None)
}

fn get_or_declare_strcpy<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("strcpy") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
    ctx.module.add_function("strcpy", fn_type, None)
}

fn get_or_declare_strcat<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("strcat") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
    ctx.module.add_function("strcat", fn_type, None)
}
