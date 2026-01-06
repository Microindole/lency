use crate::context::CodegenContext;
use crate::error::CodegenResult;
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::{Expr, Type};
use inkwell::values::{BasicValueEnum, PointerValue};
use std::collections::HashMap;

pub fn gen_elvis<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (PointerValue<'ctx>, Type)>,
    left: &Expr,
    right: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    let function = ctx
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();

    let rhs_bb = ctx.context.append_basic_block(function, "elvis_rhs");
    let merge_bb = ctx.context.append_basic_block(function, "elvis_merge");

    // Start with LHS
    // But wait, we are already in a block. We should generate LHS in current block?
    // No, standard flow:
    // 1. Generate LHS code (might span blocks).
    // 2. Check LHS result != Null.
    // 3. Branch.

    // Evaluate LHS
    let lhs_val = generate_expr(ctx, locals, left)?;

    // Check if null
    let is_not_null = match lhs_val.value {
        BasicValueEnum::PointerValue(ptr) => ctx
            .builder
            .build_is_not_null(ptr, "is_not_null")
            .map_err(|e| crate::error::CodegenError::LLVMBuildError(e.to_string()))?,
        _ => {
            // Non-pointer types (e.g. Int) are implicitly non-null for now unless wrapped.
            // If Int?, it should be wrapped. But we agreed to focus on Pointers.
            // If it's not a pointer, assume not null? Or error?
            // "i64 0" is not null.
            // For MVP, assume reference types.
            // If trivial true, we can optimize, but let's assume Pointers.
            return Ok(lhs_val);
        }
    };

    // Current block might have changed during generate_expr
    let current_bb = ctx.builder.get_insert_block().unwrap();

    ctx.builder
        .build_conditional_branch(is_not_null, merge_bb, rhs_bb)
        .map_err(|e| crate::error::CodegenError::LLVMBuildError(e.to_string()))?;

    // RHS Block (Only executed if LHS is null)
    ctx.builder.position_at_end(rhs_bb);
    let rhs_val = generate_expr(ctx, locals, right)?;
    let rhs_end_bb = ctx.builder.get_insert_block().unwrap();
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

    // Type logic: Result type is union/common type.
    // We reuse logic from Sema (right is compatible with inner).
    // If LHS is T?, result is T.
    // If LHS is T, result is T.
    // We assume Type Checker verified compatibility.
    // The Phi type is LLVM type of LHS/RHS (which should match: pointer).

    // Result Type: Unwrapped LHS?
    // If LHS is T?, and result is non-null, is it T?
    // Beryl's runtime type (LLVM type) for T and T? is likely same (Pointer).
    // So explicit unwrap is just type metadata change.

    let result_ty = match lhs_val.ty {
        Type::Nullable(inner) => *inner,
        t => t,
    };

    Ok(CodegenValue {
        value: phi.as_basic_value(),
        ty: result_ty,
    })
}
