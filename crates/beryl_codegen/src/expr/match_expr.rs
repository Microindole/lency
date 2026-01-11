use beryl_syntax::ast::{Expr, MatchCase, MatchPattern, Type};
use inkwell::values::PointerValue;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::CodegenError;
use crate::error::CodegenResult;
use crate::expr::{generate_expr, CodegenValue};

pub fn gen_match<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
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

    let subject_ptr = if subject_val.value.is_pointer_value() {
        subject_val.value.into_pointer_value()
    } else {
        // Primitive type (Int/Bool/Float).
        // Create a temporary alloca to store it, so we can treat it uniformly via pointer?
        // Or just handle primitives separately.
        // For primitives, we can just compare values.
        // But for consistency let's store it so gen_pattern_check can be uniform?
        // Actually gen_pattern_check matches Beryl AST MatchPattern.
        // If pattern is Literal, we compare value.
        // If pattern is Variable, we need to bind.
        // If pattern is Variant, we need GEP. This implies it MUST be a pointer (Enum).

        // Let's create a temp alloca for primitives so we can "bind" them to variables if needed (by pointer).
        // Variable binding needs a pointer in `locals`.

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
        // Beryl Sema should ensure exhaustiveness.
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

/// Recursively generate pattern checks.
/// If any check fails, branch to `mismatch_bb`.
/// If successful, populates `bindings`.
/// Control flow falls through on success.
#[allow(clippy::only_used_in_recursion)]
fn gen_pattern_check<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    pattern: &MatchPattern,
    subject_ptr: PointerValue<'ctx>,
    subject_type: &Type,
    bindings: &mut Vec<(String, PointerValue<'ctx>, Type)>,
    mismatch_bb: inkwell::basic_block::BasicBlock<'ctx>,
) -> CodegenResult<()> {
    match pattern {
        MatchPattern::Wildcard => {
            // Always matches.
            Ok(())
        }
        MatchPattern::Variable(name) => {
            // Always matches, binds variable.
            // We just store the POINTER to the subject.
            // When user uses variable, they load from this pointer.
            bindings.push((name.clone(), subject_ptr, subject_type.clone()));
            Ok(())
        }
        MatchPattern::Literal(lit) => {
            // Check equality.
            // Load value from pointer (unless subject is already loaded? No we standardized on ptr).
            // For primitive types (Int/Float/Bool), we load and compare.
            // String?

            // Assuming subject_type matches literal type (Sema checked).

            // 1. Load Subject
            // (Assuming int for now based on previous impl, but need to support others)

            let load_val = match subject_type {
                Type::Int | Type::Bool => ctx
                    .builder
                    .build_load(ctx.context.i64_type(), subject_ptr, "lit_chk_load")
                    .unwrap()
                    .into_int_value(),
                Type::Float => {
                    // Float equality is tricky?
                    // Use ordered equal `oeq`
                    let _fval = ctx
                        .builder
                        .build_load(ctx.context.f64_type(), subject_ptr, "lit_chk_fload")
                        .unwrap()
                        .into_float_value();
                    // We need to compare with literal.
                    // ...
                    return Err(CodegenError::UnsupportedFeature("Float matching".into()));
                }
                Type::String => {
                    // Call strcmp?
                    return Err(CodegenError::UnsupportedFeature("String matching".into()));
                }
                _ => return Err(CodegenError::TypeMismatch),
            };

            let lit_val = match lit {
                beryl_syntax::ast::Literal::Int(v) => {
                    ctx.context.i64_type().const_int(*v as u64, true)
                }
                beryl_syntax::ast::Literal::Bool(b) => {
                    ctx.context.bool_type().const_int(*b as u64, false)
                }
                _ => {
                    return Err(CodegenError::UnsupportedFeature(
                        "Unsupported literal in match".into(),
                    ))
                }
            };

            // Note: Bool load would result in i64 or i1?
            // build_load type needs to match ptr type.
            // If Int, i64. If Bool, i1.
            // Adjust logic above.

            let cmp = if *subject_type == Type::Bool {
                let bload = ctx
                    .builder
                    .build_load(ctx.context.bool_type(), subject_ptr, "b_load")
                    .unwrap()
                    .into_int_value();
                ctx.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, bload, lit_val, "lit_eq")
                    .unwrap()
            } else {
                // Int
                ctx.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, load_val, lit_val, "lit_eq")
                    .unwrap()
            };

            let success_bb = ctx.context.append_basic_block(
                ctx.builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap(),
                "lit_match_success",
            );

            ctx.builder
                .build_conditional_branch(cmp, success_bb, mismatch_bb)
                .unwrap();
            ctx.builder.position_at_end(success_bb);
            Ok(())
        }
        MatchPattern::Variant {
            name: variant_name,
            sub_patterns,
        } => {
            // Enum Matching.
            // subject_ptr points to { i8, [size x i8] }

            // 1. Check Tag
            // We need to know the tag index for `variant_name`.
            // The Enum Name is in `subject_type`.
            let enum_name = match subject_type {
                Type::Struct(n) => n,
                Type::Generic(n, _) => n, // Generic Enum
                _ => return Err(CodegenError::TypeMismatch),
            };

            // Look up variants info to find index
            let variants_info = ctx
                .enum_variants
                .get(enum_name)
                .ok_or(CodegenError::UndefinedStructType(enum_name.clone()))?;

            let (tag_idx, (_, field_types_ast)) = variants_info
                .iter()
                .enumerate()
                .find(|(_, (n, _))| n == variant_name)
                .ok_or(CodegenError::TypeMismatch)?; // Variant not found?

            // GEP Tag (element 0)
            let enum_struct_type = ctx.struct_types.get(enum_name).unwrap();
            let tag_ptr = ctx
                .builder
                .build_struct_gep(*enum_struct_type, subject_ptr, 0, "tag_ptr")
                .unwrap();

            let tag_val = ctx
                .builder
                .build_load(ctx.context.i64_type(), tag_ptr, "tag_val")
                .unwrap()
                .into_int_value();
            let expected_tag = ctx.context.i64_type().const_int(tag_idx as u64, false);

            let tag_cmp = ctx
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, tag_val, expected_tag, "tag_eq")
                .unwrap();

            let tag_success_bb = ctx.context.append_basic_block(
                ctx.builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap(),
                "tag_match_success",
            );
            ctx.builder
                .build_conditional_branch(tag_cmp, tag_success_bb, mismatch_bb)
                .unwrap();
            ctx.builder.position_at_end(tag_success_bb);

            // 2. Destructure Payload for Sub-patterns
            if !sub_patterns.is_empty() {
                // Bitcast payload (element 1) to variant struct layout
                let payload_arr_ptr = ctx
                    .builder
                    .build_struct_gep(*enum_struct_type, subject_ptr, 1, "payload_arr")
                    .unwrap();

                // Construct Variant Body Type { field1, field2... }
                // We need LLVM types for fields.

                // Helper to get concrete types for generic fields if needed?
                // For now assume non-generic or monomorphized.
                // Phase 4.1 assumes basic Enums.

                use crate::types::ToLLVMType;
                let mut variant_llvm_types = Vec::new();
                for ty in field_types_ast {
                    variant_llvm_types.push(ty.to_llvm_type(ctx)?);
                }
                let variant_struct_type = ctx.context.struct_type(&variant_llvm_types, false);

                let payload_typed_ptr = ctx
                    .builder
                    .build_bitcast(
                        payload_arr_ptr,
                        variant_struct_type.ptr_type(inkwell::AddressSpace::default()),
                        "payload_typed",
                    )
                    .unwrap()
                    .into_pointer_value();

                // Recurse for each field
                for (i, sub_pat) in sub_patterns.iter().enumerate() {
                    // GEP Field i
                    let field_ptr = ctx
                        .builder
                        .build_struct_gep(
                            variant_struct_type,
                            payload_typed_ptr,
                            i as u32,
                            "field_ptr",
                        )
                        .unwrap();
                    let field_type = &field_types_ast[i];

                    gen_pattern_check(
                        ctx,
                        locals,
                        sub_pat,
                        field_ptr,
                        field_type,
                        bindings,
                        mismatch_bb,
                    )?;
                }
            }

            Ok(())
        }
    }
}
