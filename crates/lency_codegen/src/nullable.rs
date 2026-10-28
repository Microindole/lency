//! 可空值的统一存储与取值边界。指针值直接使用空指针，标量使用堆上值单元。

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::CodegenValue;
use crate::types::ToLLVMType;
use inkwell::types::BasicType;
use inkwell::values::IntValue;
use lency_syntax::ast::Type;

pub(crate) fn is_null_type(ty: &Type) -> bool {
    matches!(ty, Type::Nullable(inner) if matches!(**inner, Type::Error | Type::Void))
}

pub(crate) fn present<'ctx>(
    ctx: &CodegenContext<'ctx>,
    value: &CodegenValue<'ctx>,
) -> CodegenResult<IntValue<'ctx>> {
    if matches!(value.ty, Type::Nullable(_)) {
        ctx.builder
            .build_is_not_null(value.value.into_pointer_value(), "nullable.present")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
    } else {
        Ok(ctx.context.bool_type().const_all_ones())
    }
}

/// 仅在语义分析已允许使用非空值的边界调用，并保留运行时保护。
pub(crate) fn unwrap<'ctx>(
    ctx: &CodegenContext<'ctx>,
    value: CodegenValue<'ctx>,
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    let Type::Nullable(inner) = value.ty else {
        return Ok(value);
    };
    let ptr = value.value.into_pointer_value();
    if let Some(panic) = ctx.panic_func {
        crate::runtime::gen_null_check(ctx.context, &ctx.builder, panic, ptr, line);
    }
    let llvm_type = inner.to_llvm_type(ctx)?;
    let result = if llvm_type.is_pointer_type() {
        value.value
    } else {
        ctx.builder
            .build_load(llvm_type, ptr, "nullable.value")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
    };
    Ok(CodegenValue {
        value: result,
        ty: *inner,
    })
}

pub(crate) fn coerce<'ctx>(
    ctx: &CodegenContext<'ctx>,
    value: CodegenValue<'ctx>,
    expected: &Type,
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    if &value.ty == expected {
        return Ok(value);
    }
    if let Type::Nullable(inner) = expected {
        let target = expected.to_llvm_type(ctx)?.into_pointer_type();
        if is_null_type(&value.ty) {
            return Ok(CodegenValue {
                value: target.const_null().into(),
                ty: expected.clone(),
            });
        }
        if matches!(value.ty, Type::Nullable(_)) {
            return Err(CodegenError::TypeMismatch);
        }
        let payload = coerce(ctx, value, inner, line)?;
        let llvm_type = inner.to_llvm_type(ctx)?;
        let ptr = if llvm_type.is_pointer_type() {
            payload.value.into_pointer_value()
        } else {
            // 不能使用栈地址：可空值可以作为函数结果或结构体字段逃逸。
            let malloc = ctx
                .module
                .get_function("lency_alloc")
                .ok_or_else(|| CodegenError::FunctionNotFound("lency_alloc".into()))?;
            let size = llvm_type.size_of().ok_or(CodegenError::TypeMismatch)?;
            let allocation = ctx
                .builder
                .build_call(malloc, &[size.into()], "nullable.box")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
                .try_as_basic_value()
                .left()
                .ok_or(CodegenError::TypeMismatch)?
                .into_pointer_value();
            if let Some(panic) = ctx.panic_func {
                crate::runtime::gen_null_check(ctx.context, &ctx.builder, panic, allocation, line);
            }
            ctx.builder
                .build_store(allocation, payload.value)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            allocation
        };
        return Ok(CodegenValue {
            value: ptr.into(),
            ty: expected.clone(),
        });
    }
    if matches!(value.ty, Type::Nullable(_)) {
        return coerce(ctx, unwrap(ctx, value, line)?, expected, line);
    }
    if value.ty == Type::Int && *expected == Type::Float {
        let converted = ctx
            .builder
            .build_signed_int_to_float(
                value.value.into_int_value(),
                ctx.context.f64_type(),
                "int.to.float",
            )
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        return Ok(CodegenValue {
            value: converted.into(),
            ty: Type::Float,
        });
    }
    // 集合的元素类型已由语义分析验证，LLVM 表示在此必须一致。
    if value.value.get_type() == expected.to_llvm_type(ctx)? {
        return Ok(CodegenValue {
            value: value.value,
            ty: expected.clone(),
        });
    }
    Err(CodegenError::TypeMismatch)
}
