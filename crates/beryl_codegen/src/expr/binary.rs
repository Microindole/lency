//! Binary Operation Code Generation
//!
//! 二元运算代码生成

use beryl_syntax::ast::{BinaryOp, Expr};
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

use super::{generate_expr, string_ops};

/// 生成二元运算代码
pub(super) fn gen_binary<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let lhs = generate_expr(ctx, locals, left)?;
    let rhs = generate_expr(ctx, locals, right)?;

    match op {
        BinaryOp::Add => gen_add(ctx, lhs, rhs),
        BinaryOp::Sub => gen_sub(ctx, lhs, rhs),
        BinaryOp::Mul => gen_mul(ctx, lhs, rhs),
        BinaryOp::Div => gen_div(ctx, lhs, rhs),
        BinaryOp::Mod => gen_mod(ctx, lhs, rhs),
        BinaryOp::Eq => gen_eq(ctx, lhs, rhs),
        BinaryOp::Neq => gen_neq(ctx, lhs, rhs),
        BinaryOp::Lt => gen_lt(ctx, lhs, rhs),
        BinaryOp::Gt => gen_gt(ctx, lhs, rhs),
        BinaryOp::Leq => gen_leq(ctx, lhs, rhs),
        BinaryOp::Geq => gen_geq(ctx, lhs, rhs),
        BinaryOp::And => gen_and(ctx, lhs, rhs),
        BinaryOp::Or => gen_or(ctx, lhs, rhs),
    }
}

fn gen_add<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_add(l, r, "addtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_add(l, r, "addtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        // int + float -> float (类型提升)
        (BasicValueEnum::IntValue(l), BasicValueEnum::FloatValue(r)) => {
            let l_float = ctx
                .builder
                .build_signed_int_to_float(l, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_add(l_float, r, "addtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        // float + int -> float (类型提升)
        (BasicValueEnum::FloatValue(l), BasicValueEnum::IntValue(r)) => {
            let r_float = ctx
                .builder
                .build_signed_int_to_float(r, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_add(l, r_float, "addtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        // 字符串连接
        (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
            string_ops::concat(ctx, l, r)
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_sub<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_sub(l, r, "subtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_sub(l, r, "subtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        // 类型提升
        (BasicValueEnum::IntValue(l), BasicValueEnum::FloatValue(r)) => {
            let l_float = ctx
                .builder
                .build_signed_int_to_float(l, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_sub(l_float, r, "subtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        (BasicValueEnum::FloatValue(l), BasicValueEnum::IntValue(r)) => {
            let r_float = ctx
                .builder
                .build_signed_int_to_float(r, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_sub(l, r_float, "subtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_mul<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_mul(l, r, "multmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_mul(l, r, "multmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        // 类型提升
        (BasicValueEnum::IntValue(l), BasicValueEnum::FloatValue(r)) => {
            let l_float = ctx
                .builder
                .build_signed_int_to_float(l, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_mul(l_float, r, "multmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        (BasicValueEnum::FloatValue(l), BasicValueEnum::IntValue(r)) => {
            let r_float = ctx
                .builder
                .build_signed_int_to_float(r, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_mul(l, r_float, "multmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_div<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_signed_div(l, r, "divtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_div(l, r, "divtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        // 类型提升
        (BasicValueEnum::IntValue(l), BasicValueEnum::FloatValue(r)) => {
            let l_float = ctx
                .builder
                .build_signed_int_to_float(l, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_div(l_float, r, "divtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        (BasicValueEnum::FloatValue(l), BasicValueEnum::IntValue(r)) => {
            let r_float = ctx
                .builder
                .build_signed_int_to_float(r, ctx.context.f64_type(), "itof")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_div(l, r_float, "divtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_mod<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_signed_rem(l, r, "modtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_eq<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_compare(IntPredicate::EQ, l, r, "eqtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eqtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_neq<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_compare(IntPredicate::NE, l, r, "netmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_compare(inkwell::FloatPredicate::ONE, l, r, "netmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_lt<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_compare(IntPredicate::SLT, l, r, "lttmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_compare(inkwell::FloatPredicate::OLT, l, r, "lttmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_gt<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_compare(IntPredicate::SGT, l, r, "gttmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_compare(inkwell::FloatPredicate::OGT, l, r, "gttmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_leq<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_compare(IntPredicate::SLE, l, r, "letmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_compare(inkwell::FloatPredicate::OLE, l, r, "letmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_geq<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_int_compare(IntPredicate::SGE, l, r, "getmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_compare(inkwell::FloatPredicate::OGE, l, r, "getmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_and<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_and(l, r, "andtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_or<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => ctx
            .builder
            .build_or(l, r, "ortmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}
