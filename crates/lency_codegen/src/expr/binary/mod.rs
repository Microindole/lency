//! Binary Operation Code Generation
//!
//! 二元运算代码生成

use lency_syntax::ast::{BinaryOp, Expr};
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::CodegenResult;

use crate::expr::{generate_expr, CodegenValue};
use lency_syntax::ast::Type;

pub mod arithmetic;
pub mod comparison;
pub mod logical;
mod nullable;

pub mod elvis;

use arithmetic::{gen_add, gen_div, gen_mod, gen_mul, gen_sub};
use comparison::{gen_eq, gen_geq, gen_gt, gen_leq, gen_lt, gen_neq};
use elvis::gen_elvis;
use logical::gen_logical;

/// 生成二元运算代码
pub(super) fn gen_binary<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    // Short-circuiting operators
    if matches!(op, BinaryOp::Elvis) {
        return gen_elvis(ctx, locals, left, right);
    }
    if matches!(op, BinaryOp::And | BinaryOp::Or) {
        return gen_logical(ctx, locals, left, op, right);
    }

    let lhs_wrapper = generate_expr(ctx, locals, left)?;
    let rhs_wrapper = generate_expr(ctx, locals, right)?;

    if matches!(op, BinaryOp::Eq | BinaryOp::Neq)
        && (matches!(lhs_wrapper.ty, Type::Nullable(_))
            || matches!(rhs_wrapper.ty, Type::Nullable(_)))
    {
        return nullable::compare(ctx, lhs_wrapper, rhs_wrapper, op);
    }
    let lhs_wrapper = crate::nullable::unwrap(ctx, lhs_wrapper, ctx.get_line(left.span.start))?;
    let rhs_wrapper = crate::nullable::unwrap(ctx, rhs_wrapper, ctx.get_line(right.span.start))?;

    let lhs_val = lhs_wrapper.value;
    let rhs_val = rhs_wrapper.value;

    let result_val = match op {
        BinaryOp::Add => gen_add(ctx, lhs_val, rhs_val),
        BinaryOp::Sub => gen_sub(ctx, lhs_val, rhs_val),
        BinaryOp::Mul => gen_mul(ctx, lhs_val, rhs_val),
        BinaryOp::Div => gen_div(ctx, lhs_val, rhs_val, ctx.get_line(right.span.start)),
        BinaryOp::Mod => gen_mod(ctx, lhs_val, rhs_val, ctx.get_line(right.span.start)),
        BinaryOp::Eq => gen_eq(ctx, lhs_val, rhs_val, &lhs_wrapper.ty),
        BinaryOp::Neq => gen_neq(ctx, lhs_val, rhs_val, &lhs_wrapper.ty),
        BinaryOp::Lt => gen_lt(ctx, lhs_val, rhs_val),
        BinaryOp::Gt => gen_gt(ctx, lhs_val, rhs_val),
        BinaryOp::Leq => gen_leq(ctx, lhs_val, rhs_val),
        BinaryOp::Geq => gen_geq(ctx, lhs_val, rhs_val),
        BinaryOp::And | BinaryOp::Or | BinaryOp::Elvis => {
            unreachable!("short-circuit operators handled before eager evaluation")
        }
    }?;

    let result_ty = match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            lhs_wrapper.ty
        }
        _ => Type::Bool,
    };

    Ok(CodegenValue {
        value: result_val,
        ty: result_ty,
    })
}
