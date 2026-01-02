use crate::error::{CodegenError, CodegenResult};
use crate::expr::ExprGenerator;
use beryl_syntax::ast::{Expr, Stmt};

use crate::stmt::StmtGenerator;

/// 生成 if 语句
pub fn gen_if<'ctx, 'a>(
    gen: &mut StmtGenerator<'ctx, 'a>,
    condition: &Expr,
    then_block: &[Stmt],
    else_block: Option<&[Stmt]>,
) -> CodegenResult<()> {
    // 生成条件
    let expr_gen = ExprGenerator::new(gen.ctx, gen.locals);
    let cond_wrapper = expr_gen.generate(condition)?;
    let cond_int = cond_wrapper.value.into_int_value();

    // 获取当前函数
    let function = gen
        .ctx
        .builder
        .get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| CodegenError::LLVMBuildError("not in a function".to_string()))?;

    // 创建基本块
    let then_bb = gen.ctx.context.append_basic_block(function, "then");
    let else_bb = gen.ctx.context.append_basic_block(function, "else");
    let merge_bb = gen.ctx.context.append_basic_block(function, "ifcont");

    // 条件跳转
    gen.ctx
        .builder
        .build_conditional_branch(cond_int, then_bb, else_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Then 分支
    gen.ctx.builder.position_at_end(then_bb);
    gen.generate_block(then_block)?;
    // 如果 then 分支没有返回，跳转到 merge
    let current_then = gen.ctx.builder.get_insert_block().unwrap();
    if !gen.block_ends_with_terminator(current_then) {
        gen.ctx
            .builder
            .build_unconditional_branch(merge_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // Else 分支
    gen.ctx.builder.position_at_end(else_bb);
    if let Some(else_stmts) = else_block {
        gen.generate_block(else_stmts)?;
    }
    // 如果 else 分支没有返回，跳转到 merge
    let current_else = gen.ctx.builder.get_insert_block().unwrap();
    if !gen.block_ends_with_terminator(current_else) {
        gen.ctx
            .builder
            .build_unconditional_branch(merge_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 合并点
    gen.ctx.builder.position_at_end(merge_bb);

    Ok(())
}
