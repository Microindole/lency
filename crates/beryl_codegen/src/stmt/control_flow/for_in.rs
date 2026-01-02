use crate::error::{CodegenError, CodegenResult};
use crate::expr::ExprGenerator;
use beryl_syntax::ast::{Expr, Stmt};

use crate::stmt::{LoopContext, StmtGenerator};

/// 生成 for-in 循环
pub fn gen_for_in<'ctx, 'a>(
    gen: &mut StmtGenerator<'ctx, 'a>,
    iterator: &str,
    iterable: &Expr,
    body: &[Stmt],
) -> CodegenResult<()> {
    // 获取当前函数
    let function = gen
        .ctx
        .builder
        .get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| CodegenError::LLVMBuildError("not in a function".to_string()))?;

    // 1. Evaluate iterable
    let expr_gen = ExprGenerator::new(gen.ctx, gen.locals);
    let array_val = expr_gen.generate(iterable)?;

    // Safety check: must be array type (Sema ensured this)
    if !array_val.value.is_array_value() {
        return Err(CodegenError::LLVMBuildError(
            "For-in iterable must be an array".to_string(),
        ));
    }
    let array_type = array_val.value.get_type().into_array_type();
    let size = array_type.len() as u64;

    // Store array temporary on stack (to allow GEP)
    let array_alloca = gen
        .ctx
        .builder
        .build_alloca(array_type, "for_arr_temp")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    gen.ctx
        .builder
        .build_store(array_alloca, array_val.value)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 2. Index variable (alloca)
    let i64_type = gen.ctx.context.i64_type();
    let idx_alloca = gen
        .ctx
        .builder
        .build_alloca(i64_type, "for_idx")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    gen.ctx
        .builder
        .build_store(idx_alloca, i64_type.const_int(0, false))
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 3. Loop Blocks
    let cond_bb = gen.ctx.context.append_basic_block(function, "forin.cond");
    let body_bb = gen.ctx.context.append_basic_block(function, "forin.body");
    let inc_bb = gen.ctx.context.append_basic_block(function, "forin.inc");
    let after_bb = gen.ctx.context.append_basic_block(function, "forin.end");

    gen.ctx
        .builder
        .build_unconditional_branch(cond_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 4. Condition
    gen.ctx.builder.position_at_end(cond_bb);
    let curr_idx = gen
        .ctx
        .builder
        .build_load(i64_type, idx_alloca, "curr_idx")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .into_int_value();
    let cond = gen
        .ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::SLT,
            curr_idx,
            i64_type.const_int(size, false),
            "loop_cond",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    gen.ctx
        .builder
        .build_conditional_branch(cond, body_bb, after_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 5. Body
    gen.ctx.builder.position_at_end(body_bb);

    // Load element
    let zero = i64_type.const_int(0, false);
    let elem_ptr = unsafe {
        gen.ctx
            .builder
            .build_gep(array_type, array_alloca, &[zero, curr_idx], "elem_ptr")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
    };
    let elem_val = gen
        .ctx
        .builder
        .build_load(array_type.get_element_type(), elem_ptr, "elem_val")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Create iterator variable local
    let iter_alloca = gen
        .ctx
        .builder
        .build_alloca(array_type.get_element_type(), iterator)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    gen.ctx
        .builder
        .build_store(iter_alloca, elem_val)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Add to locals (Handle shadowing)
    let elem_ty = match array_val.ty {
        beryl_syntax::ast::Type::Array { element_type, .. } => *element_type,
        _ => return Err(CodegenError::TypeMismatch),
    };

    let old_local = gen
        .locals
        .insert(iterator.to_string(), (iter_alloca, elem_ty));

    // Push Loop Context
    gen.loop_stack.push(LoopContext {
        continue_block: inc_bb,
        break_block: after_bb,
    });

    // Generate Body
    gen.generate_block(body)?;

    gen.loop_stack.pop();

    // Restore locals
    if let Some(old) = old_local {
        gen.locals.insert(iterator.to_string(), old);
    } else {
        gen.locals.remove(iterator);
    }

    let current_body = gen.ctx.builder.get_insert_block().unwrap();
    if !gen.block_ends_with_terminator(current_body) {
        gen.ctx
            .builder
            .build_unconditional_branch(inc_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 6. Increment
    gen.ctx.builder.position_at_end(inc_bb);
    let next_idx = gen
        .ctx
        .builder
        .build_int_add(curr_idx, i64_type.const_int(1, false), "next_idx")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    gen.ctx
        .builder
        .build_store(idx_alloca, next_idx)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    gen.ctx
        .builder
        .build_unconditional_branch(cond_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 7. End
    gen.ctx.builder.position_at_end(after_bb);

    Ok(())
}
