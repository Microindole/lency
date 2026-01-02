//! Variable Reference Code Generation
//!
//! 变量引用代码生成

use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::CodegenValue;
use crate::types::ToLLVMType;

/// 生成变量引用代码
pub(super) fn gen_variable<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    name: &str,
) -> CodegenResult<CodegenValue<'ctx>> {
    let (ptr, ty) = locals
        .get(name)
        .ok_or_else(|| CodegenError::UndefinedVariable(name.to_string()))?;

    // 使用保存的类型信息进行加载
    let llvm_type = ty.to_llvm_type(ctx)?;
    let val = ctx
        .builder
        .build_load(llvm_type, *ptr, name)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: val,
        ty: ty.clone(),
    })
}
