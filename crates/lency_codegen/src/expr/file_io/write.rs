use super::ffi;
use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::AddressSpace;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 写入整个文件。当前基础 API 将无法继续处理的 I/O 失败转为 panic。
pub fn gen_write_file<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    path_expr: &Expr,
    content_expr: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    let path_ptr = generate_expr(ctx, locals, path_expr)?
        .value
        .into_pointer_value();
    let content_ptr = generate_expr(ctx, locals, content_expr)?
        .value
        .into_pointer_value();
    let line = ctx.get_line(path_expr.span.start);
    let panic_func = ctx.panic_func.ok_or_else(|| {
        CodegenError::LLVMBuildError("panic runtime is not initialized".to_string())
    })?;
    let i64_type = ctx.context.i64_type();
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());

    let file_handle = ctx
        .builder
        .build_call(
            ffi::get_or_declare_open(ctx),
            &[path_ptr.into(), i64_type.const_int(1, false).into()],
            "write_handle",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or_else(|| CodegenError::LLVMBuildError("file_open returned void".to_string()))?
        .into_pointer_value();
    let open_failed = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            file_handle,
            i8_ptr_type.const_null(),
            "file_open_failed",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    crate::runtime::gen_panic_if(
        ctx.context,
        &ctx.builder,
        panic_func,
        open_failed,
        "Failed to Open File for Writing",
        line,
    );

    let bytes_written = ctx
        .builder
        .build_call(
            ffi::get_or_declare_write(ctx),
            &[file_handle.into(), content_ptr.into()],
            "bytes_written",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or_else(|| CodegenError::LLVMBuildError("file_write returned void".to_string()))?
        .into_int_value();
    ctx.builder
        .build_call(ffi::get_or_declare_close(ctx), &[file_handle.into()], "")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let write_failed = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            bytes_written,
            i64_type.const_all_ones(),
            "file_write_failed",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    crate::runtime::gen_panic_if(
        ctx.context,
        &ctx.builder,
        panic_func,
        write_failed,
        "Failed to Write File",
        line,
    );

    Ok(CodegenValue {
        value: ctx.context.bool_type().const_zero().into(),
        ty: Type::Void,
    })
}
