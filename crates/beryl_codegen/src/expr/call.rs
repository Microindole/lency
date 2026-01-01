//! Function Call Code Generation
//!
//! 函数调用代码生成

use beryl_syntax::ast::{Expr, ExprKind};
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

use super::generate_expr;

/// 生成函数调用代码
pub(super) fn gen_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
    callee: &Expr,
    args: &[Expr],
) -> CodegenResult<BasicValueEnum<'ctx>> {
    // 获取函数名
    let func_name = match &callee.kind {
        ExprKind::Variable(name) => name,
        _ => return Err(CodegenError::UnsupportedExpression),
    };

    // 查找函数
    let function = ctx
        .module
        .get_function(func_name)
        .ok_or_else(|| CodegenError::FunctionNotFound(func_name.clone()))?;

    // 生成参数
    let mut arg_values = Vec::new();
    for arg in args {
        let val = generate_expr(ctx, locals, arg)?;
        arg_values.push(val.into());
    }

    // 调用函数
    let call_site = ctx
        .builder
        .build_call(function, &arg_values, "calltmp")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| CodegenError::LLVMBuildError("function returns void".to_string()))
}
