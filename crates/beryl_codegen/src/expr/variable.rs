//! Variable Reference Code Generation
//!
//! 变量引用代码生成

use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// 生成变量引用代码
pub(super) fn gen_variable<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
    name: &str,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let (ptr, llvm_type) = locals
        .get(name)
        .ok_or_else(|| CodegenError::UndefinedVariable(name.to_string()))?;

    // 使用保存的类型信息进行加载
    ctx.builder
        .build_load(*llvm_type, *ptr, name)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
}
