use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::string_ops;
use inkwell::values::BasicValueEnum;

pub fn gen_add<'ctx>(
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

pub fn gen_sub<'ctx>(
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

pub fn gen_mul<'ctx>(
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

pub fn gen_div<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    line: u32,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
            let panic_func = ctx.panic_func.ok_or_else(|| {
                CodegenError::LLVMBuildError("panic runtime is not initialized".to_string())
            })?;
            let int_type = r.get_type();
            let is_zero = ctx
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    r,
                    int_type.const_zero(),
                    "div_by_zero",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            crate::runtime::gen_panic_if(
                ctx.context,
                &ctx.builder,
                panic_func,
                is_zero,
                "Integer Division by Zero",
                line,
            );

            let min_value = int_type.const_int(1_u64 << (int_type.get_bit_width() - 1), false);
            let minus_one = int_type.const_all_ones();
            let lhs_is_min = ctx
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, l, min_value, "div_lhs_is_min")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let rhs_is_minus_one = ctx
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    r,
                    minus_one,
                    "div_rhs_is_minus_one",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let overflows = ctx
                .builder
                .build_and(lhs_is_min, rhs_is_minus_one, "div_overflow")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            crate::runtime::gen_panic_if(
                ctx.context,
                &ctx.builder,
                panic_func,
                overflows,
                "Integer Division Overflow",
                line,
            );

            ctx.builder
                .build_int_signed_div(l, r, "divtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
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

pub fn gen_mod<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    line: u32,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match (lhs, rhs) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
            let panic_func = ctx.panic_func.ok_or_else(|| {
                CodegenError::LLVMBuildError("panic runtime is not initialized".to_string())
            })?;
            let is_zero = ctx
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    r,
                    r.get_type().const_zero(),
                    "rem_by_zero",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            crate::runtime::gen_panic_if(
                ctx.context,
                &ctx.builder,
                panic_func,
                is_zero,
                "Integer Remainder by Zero",
                line,
            );
            // 最小整数除以 -1 的商不可表示，但余数为零；避免交给 LLVM 的未定义路径。
            let minimum = l.get_type().const_int(1_u64 << 63, false);
            let is_minimum = ctx
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, l, minimum, "rem.minimum")
                .unwrap();
            let is_minus_one = ctx
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    r,
                    r.get_type().const_all_ones(),
                    "rem.minus_one",
                )
                .unwrap();
            let special = ctx
                .builder
                .build_and(is_minimum, is_minus_one, "rem.special")
                .unwrap();
            let divisor = ctx
                .builder
                .build_select(special, r.get_type().const_int(1, false), r, "rem.divisor")
                .unwrap()
                .into_int_value();
            ctx.builder
                .build_int_signed_rem(l, divisor, "modtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => ctx
            .builder
            .build_float_rem(l, r, "modtmp")
            .map(Into::into)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string())),
        (BasicValueEnum::IntValue(l), BasicValueEnum::FloatValue(r)) => {
            let l = ctx
                .builder
                .build_signed_int_to_float(l, ctx.context.f64_type(), "rem.promoted")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_rem(l, r, "modtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        (BasicValueEnum::FloatValue(l), BasicValueEnum::IntValue(r)) => {
            let r = ctx
                .builder
                .build_signed_int_to_float(r, ctx.context.f64_type(), "rem.promoted")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            ctx.builder
                .build_float_rem(l, r, "modtmp")
                .map(Into::into)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}
