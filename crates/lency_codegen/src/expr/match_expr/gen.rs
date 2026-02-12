use crate::context::CodegenContext;
use crate::error::CodegenError;
use crate::error::CodegenResult;
use crate::expr::match_expr::pattern::gen_pattern_check;
use crate::expr::{generate_expr, CodegenValue};

use lency_syntax::ast::{Expr, MatchCase, Type};
use std::collections::HashMap;

pub fn gen_match<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    value: &Expr,
    cases: &[MatchCase],
    default: Option<&Expr>,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. Evaluate logic
    // We use a "Sequential Check" strategy (linear decision).
    // Future optimization: Use switch for top-level Enum/Int.

    let subject_val = generate_expr(ctx, locals, value)?;

    // We need the address of the subject for GEP (accessing fields).
    // If subject is already an LValue (Variable, Get), we might want its address.
    // generate_expr returns a loaded Value.
    // For Enums/Structs (aggregate types), they are pointers in LLVM.
    // So subject_val.value SHOULD be a PointerValue to the struct/enum.
    // For primitive Int/Bool, it is an IntValue.

    let subject_ptr = if subject_val.value.is_pointer_value()
        && !matches!(subject_val.ty, Type::String | Type::Vec(_))
    {
        subject_val.value.into_pointer_value()
    } else {
        // Primitive type (Int/Bool/Float) OR String/Vec (which are pointers but treated as values).
        // Create a temporary alloca to store it.

        // Inline create_entry_block_alloca logic:
        let builder = ctx.context.create_builder();
        let entry = ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
            .get_first_basic_block()
            .unwrap();
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }
        let alloca = builder
            .build_alloca(subject_val.value.get_type(), "match_subject_tmp")
            .unwrap();

        ctx.builder.build_store(alloca, subject_val.value).unwrap();
        alloca
    };

    // Type of the result of the match expression (inferred from first case or default)
    // We need to know the result type to build PHI.
    // We assume semantic analysis has ensured all arms return compatible types.
    // We'll peek at the first arm generation to determine type, or use inferred type passed down?
    // We don't have inferred type passed down here.
    // We rely on generating the first block to see the type.

    // Prepare blocks
    let current_func = ctx
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();

    let merge_bb = ctx.context.append_basic_block(current_func, "match_merge");

    let mut incoming_values = Vec::new();
    let mut result_type = Type::Void;
    let mut result_llvm_type = None;

    // We will chain checks: Case 0 Test -> Case 0 Body -> Merge
    //                       |
    //                       v
    //                       Case 1 Test -> ...

    let mut next_check_bb = ctx.context.append_basic_block(current_func, "case_0_check");
    ctx.builder
        .build_unconditional_branch(next_check_bb)
        .unwrap();

    for (i, case) in cases.iter().enumerate() {
        let check_bb = next_check_bb;
        let body_bb = ctx
            .context
            .append_basic_block(current_func, &format!("case_{}_body", i));

        // Create next check block (or default/unreachable if last)
        let is_last = i == cases.len() - 1;
        let next_bb_name = if is_last {
            // If we have default expr, that's the "next".
            // If not, and we fall through, it's unreachable (assuming exhaustiveness)
            "match_fallback"
        } else {
            &format!("case_{}_check", i + 1)
        };
        let next_bb = ctx.context.append_basic_block(current_func, next_bb_name);
        next_check_bb = next_bb; // updating for next iteration

        // -----------------------------------------------------
        // Generate Check Logic in check_bb
        // -----------------------------------------------------
        ctx.builder.position_at_end(check_bb);

        // We need a list of bindings that this pattern will generate if successful.
        // We can't modify `locals` yet because we might fail the check.
        // So `gen_pattern_check` returns a list of (name, ptr, type).

        let mut bindings = Vec::new();
        gen_pattern_check(
            ctx,
            locals, // Needed for looking up constants? Or type info?
            &case.pattern,
            subject_ptr,
            &subject_val.ty,
            &mut bindings,
            next_bb, // Jump here if fail
        )?;

        // If gen_pattern_check returns, it means "if we are still here, we potentially matched".
        // If it fully matched (e.g. strict boolean true), it didn't branch to mismatch yet?
        // Wait, `gen_pattern_check` needs to generate branches on failure.
        // If it returns successfully, it means "checked and passed (or emitted checks)".
        // Wait, we need to AND the checks together?
        // Actually `gen_pattern_check` as designed in Plan: "If !=, br mismatch_bb".
        // So if it returns, we flow to success.

        // So:
        // gen_pattern_check(... mismatch_bb=next_bb)
        // ctx.builder.build_unconditional_branch(body_bb);

        // Note: gen_pattern_check might have multiple checks.
        // e.g. case Some(1): check tag -> next_bb, check val -> next_bb.
        // Success: Branch to body
        ctx.builder.build_unconditional_branch(body_bb).unwrap();

        // -----------------------------------------------------
        // Generate Body in body_bb
        // -----------------------------------------------------
        ctx.builder.position_at_end(body_bb);

        // Construct new locals for body
        let mut body_locals = locals.clone();
        for (name, ptr, ty) in bindings {
            body_locals.insert(name, (ptr, ty));
        }

        let body_val = generate_expr(ctx, &body_locals, &case.body)?;

        // Record result type from first arm
        if result_llvm_type.is_none() {
            result_type = body_val.ty.clone();
            result_llvm_type = Some(body_val.value.get_type());
        }

        // Capture result
        incoming_values.push((body_val.value, body_bb));

        // Jump to merge
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_unconditional_branch(merge_bb).unwrap();
        }
    }

    // -----------------------------------------------------
    // Handle Fallback (Default / Exhaustive End) in next_check_bb
    // -----------------------------------------------------
    ctx.builder.position_at_end(next_check_bb);

    // Check if we have a provided default expression (legacy AST?)
    // Or if the last case covered everything.
    // The `cases` loop connects the last failure to `next_check_bb`.

    if let Some(def) = default {
        let def_val = generate_expr(ctx, locals, def)?;
        if result_llvm_type.is_none() {
            result_type = def_val.ty.clone();
            result_llvm_type = Some(def_val.value.get_type());
        }
        incoming_values.push((def_val.value, next_check_bb));
        ctx.builder.build_unconditional_branch(merge_bb).unwrap();
    } else {
        // No default. If we reach here, it's a runtime mismatch error (or undefined).
        // Lency Sema should ensure exhaustiveness.
        // For safety, let's panic or unreachable.
        // If we want to be nice: panic("Pattern matching failed").
        // For now: unreachable.
        ctx.builder.build_unreachable().unwrap();
    }

    // -----------------------------------------------------
    // Merge Block
    // -----------------------------------------------------
    ctx.builder.position_at_end(merge_bb);

    if incoming_values.is_empty() {
        // Should not happen if cases > 0 or default exists
        return Err(CodegenError::LLVMBuildError("Empty match".to_string()));
    }

    if let Some(phi_ty) = result_llvm_type {
        let phi = ctx.builder.build_phi(phi_ty, "match_result").unwrap();
        for (v, bb) in incoming_values {
            phi.add_incoming(&[(&v, bb)]);
        }

        Ok(CodegenValue {
            value: phi.as_basic_value(),
            ty: result_type,
        })
    } else {
        // Void/Never result?
        // If result type is void, we return first void?
        // Let's create dummy void.
        Ok(CodegenValue {
            value: ctx.context.i64_type().const_int(0, false).into(), // Dummy
            ty: Type::Void,
        })
    }
}
