use crate::error::{CodegenError, CodegenResult};
use crate::expr::ExprGenerator;
use lency_syntax::ast::{BinaryOp, Expr, ExprKind, Literal, Stmt};

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
    let cond_wrapper = crate::nullable::unwrap(
        gen.ctx,
        cond_wrapper,
        gen.ctx.get_line(condition.span.start),
    )?;
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
    gen.with_local_scope(|gen| {
        if let ExprKind::Binary(left, BinaryOp::Neq, right) = &condition.kind {
            let name = match (&left.kind, &right.kind) {
                (ExprKind::Variable(name), ExprKind::Literal(Literal::Null))
                | (ExprKind::Literal(Literal::Null), ExprKind::Variable(name)) => Some(name),
                _ => None,
            };
            if let Some(name) = name {
                let storage = gen.locals.get(name).map(|(ptr, _)| *ptr).or_else(|| {
                    gen.ctx
                        .module
                        .get_global(name)
                        .map(|global| global.as_pointer_value())
                });
                if let Some(storage) = storage {
                    gen.ctx
                        .read_refinements
                        .borrow_mut()
                        .insert(name.clone(), storage);
                }
            }
        }
        gen.generate_block(then_block)
    })?;
    // 如果 then 分支没有返回，跳转到 merge
    let current_then = gen.ctx.builder.get_insert_block().unwrap();
    let then_terminated = gen.block_ends_with_terminator(current_then);
    if !then_terminated {
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
    let else_terminated = gen.block_ends_with_terminator(current_else);
    if !else_terminated {
        gen.ctx
            .builder
            .build_unconditional_branch(merge_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 合并点
    if then_terminated && else_terminated {
        merge_bb.remove_from_function().map_err(|_| {
            CodegenError::LLVMBuildError("failed to remove unreachable if merge".to_string())
        })?;
        gen.ctx.builder.position_at_end(current_else);
    } else {
        gen.ctx.builder.position_at_end(merge_bb);
    }

    Ok(())
}
