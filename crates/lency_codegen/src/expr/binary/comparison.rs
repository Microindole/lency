use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use lency_syntax::ast::Type;

pub fn gen_eq<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    lhs_ty: &Type,
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
        (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
            // 检查是否为字符串类型，使用 strcmp 进行内容比较
            if matches!(lhs_ty, Type::String) {
                // 获取或声明 strcmp 函数
                let strcmp_fn = ctx.module.get_function("strcmp").unwrap_or_else(|| {
                    let i32_type = ctx.context.i32_type();
                    let i8_ptr_type = ctx
                        .context
                        .i8_type()
                        .ptr_type(inkwell::AddressSpace::default());
                    let fn_type =
                        i32_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
                    ctx.module.add_function(
                        "strcmp",
                        fn_type,
                        Some(inkwell::module::Linkage::External),
                    )
                });

                // 调用 strcmp(lhs, rhs)
                let call = ctx
                    .builder
                    .build_call(strcmp_fn, &[l.into(), r.into()], "strcmp_result")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                let strcmp_result = call.try_as_basic_value().left().unwrap().into_int_value();

                // strcmp 返回 0 表示相等
                let zero = ctx.context.i32_type().const_int(0, false);
                ctx.builder
                    .build_int_compare(IntPredicate::EQ, strcmp_result, zero, "streqtmp")
                    .map(Into::into)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
            } else {
                // 非字符串指针：比较指针地址
                let l_int = ctx
                    .builder
                    .build_ptr_to_int(l, ctx.context.i64_type(), "lhs_ptr_int")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                let r_int = ctx
                    .builder
                    .build_ptr_to_int(r, ctx.context.i64_type(), "rhs_ptr_int")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                ctx.builder
                    .build_int_compare(IntPredicate::EQ, l_int, r_int, "eqtmp")
                    .map(Into::into)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
            }
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}

pub fn gen_neq<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    lhs_ty: &Type,
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
        (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
            // 检查是否为字符串类型
            if matches!(lhs_ty, Type::String) {
                // 获取或声明 strcmp 函数
                let strcmp_fn = ctx.module.get_function("strcmp").unwrap_or_else(|| {
                    let i32_type = ctx.context.i32_type();
                    let i8_ptr_type = ctx
                        .context
                        .i8_type()
                        .ptr_type(inkwell::AddressSpace::default());
                    let fn_type =
                        i32_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
                    ctx.module.add_function(
                        "strcmp",
                        fn_type,
                        Some(inkwell::module::Linkage::External),
                    )
                });

                let call = ctx
                    .builder
                    .build_call(strcmp_fn, &[l.into(), r.into()], "strcmp_result")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                let strcmp_result = call.try_as_basic_value().left().unwrap().into_int_value();

                // strcmp 返回非 0 表示不相等
                let zero = ctx.context.i32_type().const_int(0, false);
                ctx.builder
                    .build_int_compare(IntPredicate::NE, strcmp_result, zero, "strnetmp")
                    .map(Into::into)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
            } else {
                let l_int = ctx
                    .builder
                    .build_ptr_to_int(l, ctx.context.i64_type(), "lhs_ptr_int")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                let r_int = ctx
                    .builder
                    .build_ptr_to_int(r, ctx.context.i64_type(), "rhs_ptr_int")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                ctx.builder
                    .build_int_compare(IntPredicate::NE, l_int, r_int, "netmp")
                    .map(Into::into)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
            }
        }
        _ => Err(CodegenError::TypeMismatch),
    }
}

pub fn gen_lt<'ctx>(
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

pub fn gen_gt<'ctx>(
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

pub fn gen_leq<'ctx>(
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

pub fn gen_geq<'ctx>(
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
