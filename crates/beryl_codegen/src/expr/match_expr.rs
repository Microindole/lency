use beryl_syntax::ast::{Expr, MatchCase, MatchPattern};
use inkwell::values::BasicValueEnum as LLVMValue;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::CodegenError;
use crate::error::CodegenResult;
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::Type;

pub fn gen_match<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    value: &Expr,
    cases: &[MatchCase],
    default: Option<&Expr>,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. Generate condition value
    let cond_val_wrapper = generate_expr(ctx, locals, value)?;
    let cond_int = match cond_val_wrapper.value {
        LLVMValue::IntValue(v) => v,
        _ => return Err(CodegenError::TypeMismatch),
    };

    // 2. Create basic blocks
    let current_bb = ctx.builder.get_insert_block().unwrap();
    let function = current_bb.get_parent().unwrap();

    let merge_bb = ctx.context.append_basic_block(function, "match.merge");
    let default_bb = ctx.context.append_basic_block(function, "match.default");

    // Pre-allocate blocks for cases
    let mut case_blocks = Vec::with_capacity(cases.len());
    for i in 0..cases.len() {
        case_blocks.push(
            ctx.context
                .append_basic_block(function, &format!("match.case.{}", i)),
        );
    }

    // 3. Build switch instruction
    // We need to collect (IntValue, BasicBlock) pairs
    let mut switch_cases = Vec::with_capacity(cases.len());

    for (i, case) in cases.iter().enumerate() {
        let pattern_val = match &case.pattern {
            MatchPattern::Literal(beryl_syntax::ast::Literal::Int(v)) => {
                ctx.context.i64_type().const_int(*v as u64, true)
            }
            _ => {
                return Err(CodegenError::UnsupportedFeature(
                    "Only int literals supported in match".to_string(),
                ))
            }
        };
        switch_cases.push((pattern_val, case_blocks[i]));
    }

    ctx.builder
        .build_switch(cond_int, default_bb, &switch_cases)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 4. Generate code for cases
    let mut incoming_values: Vec<(LLVMValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
        Vec::new();
    let mut result_type = Type::Void; // Placeholder

    for (i, case) in cases.iter().enumerate() {
        ctx.builder.position_at_end(case_blocks[i]);
        let body_wrapper = generate_expr(ctx, locals, &case.body)?;
        incoming_values.push((body_wrapper.value, case_blocks[i]));
        if i == 0 {
            result_type = body_wrapper.ty;
        }

        // Branch to merge
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
    }

    // 5. Generate code for default
    ctx.builder.position_at_end(default_bb);
    if let Some(def_expr) = default {
        let def_wrapper = generate_expr(ctx, locals, def_expr)?;
        incoming_values.push((def_wrapper.value, default_bb));
        if cases.is_empty() {
            result_type = def_wrapper.ty;
        }
    }

    // Always flow from default_bb to merge_bb if default_bb is reached (via switch default)
    // If default expression present, we generated code properly.
    // If not, we are here in default_bb. We must terminate.
    // However, if we don't have a value for default path, we can't join PHI.
    // If default is None, we need a dummy value or `unreachable`?
    // If we build_unreachable, then merge_bb has one less incoming block.

    if default.is_some() {
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
    } else {
        // No default case provided by user.
        // In codegen we treat it as unreachable?
        // Yes, assuming exhaustiveness checked elsewhere or just undefined behavior.
        ctx.builder
            .build_unreachable()
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        // Note: If we build unreachable, this block (default_bb) does not branch to merge_bb.
        // So merge_bb's PHI should NOT include (value, default_bb).
    }

    // 6. Merge Block & PHI
    ctx.builder.position_at_end(merge_bb);

    if incoming_values.is_empty() {
        // No cases?
        return Err(CodegenError::LLVMBuildError(
            "Empty match not supported".to_string(),
        ));
    }

    let phi = ctx
        .builder
        .build_phi(incoming_values[0].0.get_type(), "match.result")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    for (val, bb) in incoming_values {
        phi.add_incoming(&[(&val, bb)]);
    }

    Ok(CodegenValue {
        value: phi.as_basic_value(),
        ty: result_type,
    })
}
