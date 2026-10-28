//! 源码调用栈插桩，不依赖 LLVM 优化后的本机栈形状。

use crate::{CodegenContext, CodegenError, CodegenResult};
use inkwell::values::PointerValue;

fn string<'ctx>(ctx: &CodegenContext<'ctx>, value: &str) -> CodegenResult<PointerValue<'ctx>> {
    if let Some(pointer) = ctx.trace_strings.borrow().get(value).copied() {
        return Ok(pointer);
    }
    let pointer = ctx
        .builder
        .build_global_string_ptr(value, "trace.text")
        .map_err(|error| CodegenError::LLVMBuildError(error.to_string()))?
        .as_pointer_value();
    ctx.trace_strings.borrow_mut().insert(value.into(), pointer);
    Ok(pointer)
}

pub(crate) fn enter(ctx: &CodegenContext<'_>, name: &str, offset: usize) -> CodegenResult<()> {
    let source = ctx.source_map.location(offset);
    let name = string(ctx, name)?;
    let file = string(ctx, &source.file)?;
    let function = ctx
        .module
        .get_function("lency_trace_enter")
        .ok_or(CodegenError::TypeMismatch)?;
    ctx.builder
        .build_call(
            function,
            &[
                name.into(),
                file.into(),
                ctx.context
                    .i32_type()
                    .const_int(source.line as u64, false)
                    .into(),
                ctx.context
                    .i32_type()
                    .const_int(source.column as u64, false)
                    .into(),
            ],
            "",
        )
        .map_err(|error| CodegenError::LLVMBuildError(error.to_string()))?;
    Ok(())
}

pub(crate) fn location(ctx: &CodegenContext<'_>, offset: usize) -> CodegenResult<()> {
    let source = ctx.source_map.location(offset);
    let file = string(ctx, &source.file)?;
    let function = ctx
        .module
        .get_function("lency_trace_location")
        .ok_or(CodegenError::TypeMismatch)?;
    ctx.builder
        .build_call(
            function,
            &[
                file.into(),
                ctx.context
                    .i32_type()
                    .const_int(source.line as u64, false)
                    .into(),
                ctx.context
                    .i32_type()
                    .const_int(source.column as u64, false)
                    .into(),
            ],
            "",
        )
        .map_err(|error| CodegenError::LLVMBuildError(error.to_string()))?;
    Ok(())
}

pub(crate) fn leave(ctx: &CodegenContext<'_>) -> CodegenResult<()> {
    let function = ctx
        .module
        .get_function("lency_trace_leave")
        .ok_or(CodegenError::TypeMismatch)?;
    ctx.builder
        .build_call(function, &[], "")
        .map_err(|error| CodegenError::LLVMBuildError(error.to_string()))?;
    Ok(())
}
