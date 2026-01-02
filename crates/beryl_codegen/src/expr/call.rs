//! Function Call Code Generation
//!
//! 函数调用代码生成

use beryl_syntax::ast::{Expr, ExprKind};
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::Type;

/// 生成函数调用代码
pub(super) fn gen_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    callee: &Expr,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
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
    // 生成参数
    let mut arg_values = Vec::new();
    for arg in args {
        let val_wrapper = generate_expr(ctx, locals, arg)?;
        arg_values.push(val_wrapper.value.into());
    }

    // 获取函数返回类型
    let return_type = ctx
        .function_signatures
        .get(func_name)
        .cloned()
        .ok_or_else(|| CodegenError::FunctionNotFound(func_name.clone()))?;

    // 调用函数
    let call_site = ctx
        .builder
        .build_call(function, &arg_values, "calltmp")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let val = call_site.try_as_basic_value().left();

    if let Some(v) = val {
        Ok(CodegenValue {
            value: v,
            ty: return_type,
        })
    } else {
        // Void return, but we need to return something?
        // Usually void functions don't return value in expressions.
        // But CodegenValue expects a value.
        // We can use a dummy value/type or handle Void specially.
        // For now, let's return a dummy Int(0) if it's explicitly Void.
        // Or if the usage expects a value, it will be an error.

        let dummy = ctx.context.bool_type().const_int(0, false).into();
        Ok(CodegenValue {
            value: dummy,
            ty: Type::Void,
        })
    }
}
