use super::ffi;
use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::AddressSpace;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 读取整个文件。当前基础 API 将无法继续处理的 I/O 失败转为 panic。
pub fn gen_read_file<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    path_expr: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    let path_ptr = generate_expr(ctx, locals, path_expr)?
        .value
        .into_pointer_value();
    let line = ctx.get_line(path_expr.span.start);
    let panic_func = ctx.panic_func.ok_or_else(|| {
        CodegenError::LLVMBuildError("panic runtime is not initialized".to_string())
    })?;
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());

    let content = ctx
        .builder
        .build_call(
            ffi::get_or_declare_read_string(ctx),
            &[path_ptr.into()],
            "file_content",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or_else(|| CodegenError::LLVMBuildError("file read returned void".to_string()))?
        .into_pointer_value();
    let read_failed = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            content,
            i8_ptr_type.const_null(),
            "file_read_failed",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    crate::runtime::gen_panic_if(
        ctx.context,
        &ctx.builder,
        panic_func,
        read_failed,
        "Failed to Read File",
        line,
    );

    Ok(CodegenValue {
        value: content.into(),
        ty: Type::String,
    })
}
