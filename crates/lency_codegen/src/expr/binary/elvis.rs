use crate::context::CodegenContext;
use crate::error::CodegenResult;
use crate::expr::{generate_expr, CodegenValue};
use inkwell::values::PointerValue;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

pub fn gen_elvis<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (PointerValue<'ctx>, Type)>,
    left: &Expr,
    right: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    let lhs_val = generate_expr(ctx, locals, left)?;
    // 非空值不需要分支，也不能留下没有终止指令的空基本块。
    if !matches!(lhs_val.ty, Type::Nullable(_)) {
        return Ok(lhs_val);
    }
    if crate::nullable::is_null_type(&lhs_val.ty) {
        return generate_expr(ctx, locals, right);
    }
    let inner_ty = match &lhs_val.ty {
        Type::Nullable(inner) => *inner.clone(),
        _ => unreachable!(),
    };
    let function = ctx
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();

    let rhs_bb = ctx.context.append_basic_block(function, "elvis_rhs");
    let value_bb = ctx.context.append_basic_block(function, "elvis_value");
    let merge_bb = ctx.context.append_basic_block(function, "elvis_merge");

    let is_not_null = crate::nullable::present(ctx, &lhs_val)?;

    ctx.builder
        .build_conditional_branch(is_not_null, value_bb, rhs_bb)
        .map_err(|e| crate::error::CodegenError::LLVMBuildError(e.to_string()))?;

    // RHS Block (Only executed if LHS is null)
    ctx.builder.position_at_end(rhs_bb);
    let rhs_val = generate_expr(ctx, locals, right)?;
    let result_ty = if lency_sema::type_infer::is_compatible(&inner_ty, &rhs_val.ty) {
        inner_ty
    } else {
        rhs_val.ty.clone()
    };
    let rhs_val =
        crate::nullable::coerce(ctx, rhs_val, &result_ty, ctx.get_line(right.span.start))?;
    let rhs_end_bb = ctx.builder.get_insert_block().unwrap();
    ctx.builder
        .build_unconditional_branch(merge_bb)
        .map_err(|e| crate::error::CodegenError::LLVMBuildError(e.to_string()))?;

    // 只在非空路径解包，空指针绝不被读取。
    ctx.builder.position_at_end(value_bb);
    let lhs_val = crate::nullable::unwrap(ctx, lhs_val, ctx.get_line(left.span.start))?;
    let lhs_val = crate::nullable::coerce(ctx, lhs_val, &result_ty, ctx.get_line(left.span.start))?;
    let current_bb = ctx.builder.get_insert_block().unwrap();
    ctx.builder
        .build_unconditional_branch(merge_bb)
        .map_err(|e| crate::error::CodegenError::LLVMBuildError(e.to_string()))?;

    // Merge Block
    ctx.builder.position_at_end(merge_bb);
    let phi = ctx
        .builder
        .build_phi(lhs_val.value.get_type(), "elvis_phi")
        .map_err(|e| crate::error::CodegenError::LLVMBuildError(e.to_string()))?;

    // If LHS is not null, use LHS.
    // Incoming from current_bb (end of LHS gen)
    phi.add_incoming(&[(&lhs_val.value, current_bb), (&rhs_val.value, rhs_end_bb)]);

    Ok(CodegenValue {
        value: phi.as_basic_value(),
        ty: result_ty,
    })
}
