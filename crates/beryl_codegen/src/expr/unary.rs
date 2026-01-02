//! Unary Operation Code Generation
//!
//! 一元运算代码生成

use beryl_syntax::ast::{Expr, UnaryOp};
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

use crate::expr::{generate_expr, CodegenValue};

/// 生成一元运算代码
pub(super) fn gen_unary<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    op: &UnaryOp,
    operand: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    let val_wrapper = generate_expr(ctx, locals, operand)?;
    let val = val_wrapper.value;

    let result_val = match op {
        UnaryOp::Neg => gen_neg(ctx, val)?,
        UnaryOp::Not => gen_not(ctx, val)?,
    };

    Ok(CodegenValue {
        value: result_val,
        ty: val_wrapper.ty,
    })
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
