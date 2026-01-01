//! Unary Operation Code Generation
//!
//! 一元运算代码生成

use beryl_syntax::ast::{Expr, UnaryOp};
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

use super::generate_expr;

/// 生成一元运算代码
pub(super) fn gen_unary<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
    op: &UnaryOp,
    operand: &Expr,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let val = generate_expr(ctx, locals, operand)?;

    match op {
        UnaryOp::Neg => gen_neg(ctx, val),
        UnaryOp::Not => gen_not(ctx, val),
    }
}

fn gen_neg<'ctx>(
    ctx: &CodegenContext<'ctx>,
    val: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match val {
        BasicValueEnum::IntValue(v) => ctx
            .builder
            .build_int_neg(v, "negtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        BasicValueEnum::FloatValue(v) => ctx
            .builder
            .build_float_neg(v, "negtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}

fn gen_not<'ctx>(
    ctx: &CodegenContext<'ctx>,
    val: BasicValueEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match val {
        BasicValueEnum::IntValue(v) => ctx
            .builder
            .build_not(v, "nottmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        _ => Err(CodegenError::TypeMismatch),
    }
}
