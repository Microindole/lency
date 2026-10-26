use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::values::PointerValue;
use lency_syntax::ast::{BinaryOp, Expr, Type};
use std::collections::HashMap;

/// 逻辑运算从左到右求值，仅在左侧不能确定结果时执行右侧。
pub fn gen_logical<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (PointerValue<'ctx>, Type)>,
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    let lhs = generate_expr(ctx, locals, left)?;
    let lhs = crate::nullable::unwrap(ctx, lhs, ctx.get_line(left.span.start))?;
    if lhs.ty != Type::Bool {
        return Err(CodegenError::TypeMismatch);
    }
    let lhs_value = lhs.value.into_int_value();
    let lhs_end = ctx
        .builder
        .get_insert_block()
        .ok_or_else(|| CodegenError::LLVMBuildError("logical expression outside a block".into()))?;
    let function = lhs_end.get_parent().ok_or_else(|| {
        CodegenError::LLVMBuildError("logical expression outside a function".into())
    })?;
    let rhs_block = ctx.context.append_basic_block(function, "logical.rhs");
    let merge = ctx.context.append_basic_block(function, "logical.merge");
    let (on_true, on_false) = match op {
        BinaryOp::And => (rhs_block, merge),
        BinaryOp::Or => (merge, rhs_block),
        _ => return Err(CodegenError::UnsupportedExpression),
    };
    ctx.builder
        .build_conditional_branch(lhs_value, on_true, on_false)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    ctx.builder.position_at_end(rhs_block);
    let rhs = generate_expr(ctx, locals, right)?;
    let rhs = crate::nullable::unwrap(ctx, rhs, ctx.get_line(right.span.start))?;
    if rhs.ty != Type::Bool {
        return Err(CodegenError::TypeMismatch);
    }
    let rhs_value = rhs.value.into_int_value();
    // 嵌套逻辑运算和运行时检查可能改变右侧实际结束块。
    let rhs_end = ctx.builder.get_insert_block().unwrap();
    ctx.builder
        .build_unconditional_branch(merge)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    ctx.builder.position_at_end(merge);
    let phi = ctx
        .builder
        .build_phi(ctx.context.bool_type(), "logical.result")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    phi.add_incoming(&[(&lhs_value, lhs_end), (&rhs_value, rhs_end)]);
    Ok(CodegenValue {
        value: phi.as_basic_value(),
        ty: Type::Bool,
    })
}
