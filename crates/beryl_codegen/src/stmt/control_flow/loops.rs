use crate::error::{CodegenError, CodegenResult};
use crate::expr::ExprGenerator;
use beryl_syntax::ast::{Expr, Stmt};

use crate::stmt::{LoopContext, StmtGenerator};

/// 生成 while 循环
pub fn gen_while<'ctx, 'a>(
    gen: &mut StmtGenerator<'ctx, 'a>,
    condition: &Expr,
    body: &[Stmt],
) -> CodegenResult<()> {
    // 获取当前函数
    let function = gen
        .ctx
        .builder
        .get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| CodegenError::LLVMBuildError("not in a function".to_string()))?;

    // 创建基本块
    let cond_bb = gen.ctx.context.append_basic_block(function, "while.cond");
    let body_bb = gen.ctx.context.append_basic_block(function, "while.body");
    let after_bb = gen.ctx.context.append_basic_block(function, "while.end");

    // 跳转到条件块
    gen.ctx
        .builder
        .build_unconditional_branch(cond_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 条件块
    gen.ctx.builder.position_at_end(cond_bb);
    let expr_gen = ExprGenerator::new(gen.ctx, gen.locals);
    let cond_val = expr_gen.generate(condition)?;
    let cond_int = cond_val.value.into_int_value();
    gen.ctx
        .builder
        .build_conditional_branch(cond_int, body_bb, after_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 循环体
    gen.ctx.builder.position_at_end(body_bb);

    // Push loop context
    gen.loop_stack.push(LoopContext {
        continue_block: cond_bb,
        break_block: after_bb,
    });

    gen.generate_block(body)?;

    // Pop loop context
    gen.loop_stack.pop();

    let current_body = gen.ctx.builder.get_insert_block().unwrap();
    if !gen.block_ends_with_terminator(current_body) {
        gen.ctx
            .builder
            .build_unconditional_branch(cond_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 循环后
    gen.ctx.builder.position_at_end(after_bb);

    Ok(())
}

/// 生成 for 循环
pub fn gen_for<'ctx, 'a>(
    gen: &mut StmtGenerator<'ctx, 'a>,
    init: Option<&Stmt>,
    condition: Option<&Expr>,
    update: Option<&Stmt>,
    body: &[Stmt],
) -> CodegenResult<()> {
    // 获取当前函数
    let function = gen
        .ctx
        .builder
        .get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| CodegenError::LLVMBuildError("not in a function".to_string()))?;

    // 1. 生成初始化（在当前块）
    if let Some(init_stmt) = init {
        gen.generate(init_stmt)?;
    }

    // 2. 创建基本块
    let cond_bb = gen.ctx.context.append_basic_block(function, "for.cond");
    let body_bb = gen.ctx.context.append_basic_block(function, "for.body");
    let inc_bb = gen.ctx.context.append_basic_block(function, "for.inc");
    let after_bb = gen.ctx.context.append_basic_block(function, "for.end");

    // 3. 跳转到条件块
    gen.ctx
        .builder
        .build_unconditional_branch(cond_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 4. 条件块
    gen.ctx.builder.position_at_end(cond_bb);
    if let Some(cond) = condition {
        let expr_gen = ExprGenerator::new(gen.ctx, gen.locals);
        let cond_val = expr_gen.generate(cond)?;
        let cond_int = cond_val.value.into_int_value();
        gen.ctx
            .builder
            .build_conditional_branch(cond_int, body_bb, after_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    } else {
        // 无条件时，直接进入循环体（无限循环）
        gen.ctx
            .builder
            .build_unconditional_branch(body_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 5. 循环体块
    gen.ctx.builder.position_at_end(body_bb);

    // Push loop context
    gen.loop_stack.push(LoopContext {
        continue_block: inc_bb,
        break_block: after_bb,
    });

    gen.generate_block(body)?;

    // Pop loop context
    gen.loop_stack.pop();

    let current_body = gen.ctx.builder.get_insert_block().unwrap();
    if !gen.block_ends_with_terminator(current_body) {
        gen.ctx
            .builder
            .build_unconditional_branch(inc_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 6. 增量块
    gen.ctx.builder.position_at_end(inc_bb);
    if let Some(upd) = update {
        gen.generate(upd)?;
    }
    gen.ctx
        .builder
        .build_unconditional_branch(cond_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 7. 循环结束块
    gen.ctx.builder.position_at_end(after_bb);

    Ok(())
}

/// 生成 break 语句
pub fn gen_break(gen: &mut StmtGenerator) -> CodegenResult<()> {
    let context = gen.loop_stack.last().ok_or_else(|| {
        CodegenError::LLVMBuildError(
            "break statement outside loop (should be caught by sema)".to_string(),
        )
    })?;

    gen.ctx
        .builder
        .build_unconditional_branch(context.break_block)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(())
}

/// 生成 continue 语句
pub fn gen_continue(gen: &mut StmtGenerator) -> CodegenResult<()> {
    let context = gen.loop_stack.last().ok_or_else(|| {
        CodegenError::LLVMBuildError(
            "continue statement outside loop (should be caught by sema)".to_string(),
        )
    })?;

    gen.ctx
        .builder
        .build_unconditional_branch(context.continue_block)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(())
}
