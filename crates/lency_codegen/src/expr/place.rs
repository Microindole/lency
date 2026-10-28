//! 赋值目标表示：真实存储地址或运行时集合槽位，计算目标后才计算右侧。

use super::{generate_expr, generate_lvalue_addr, CodegenValue};
use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::types::ToLLVMType;
use inkwell::values::{IntValue, PointerValue};
use lency_syntax::ast::{Expr, ExprKind, Type};
use std::collections::HashMap;

type Locals<'ctx> = HashMap<String, (PointerValue<'ctx>, Type)>;

enum Place<'ctx> {
    Memory(PointerValue<'ctx>, Type),
    Vector(PointerValue<'ctx>, IntValue<'ctx>, Type),
}

fn resolve<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &Locals<'ctx>,
    target: &Expr,
) -> CodegenResult<Place<'ctx>> {
    let ExprKind::Index { array, index } = &target.kind else {
        let (ptr, ty) = generate_lvalue_addr(ctx, locals, target)?;
        return Ok(Place::Memory(ptr, ty));
    };
    let line = ctx.get_line(target.span.start);
    let (array_storage, array_value) = if matches!(
        array.kind,
        ExprKind::Variable(_) | ExprKind::Get { .. } | ExprKind::Index { .. }
    ) {
        match resolve(ctx, locals, array)? {
            Place::Memory(ptr, ty @ Type::Array { .. }) => (Some((ptr, ty)), None),
            Place::Memory(ptr, ty) => {
                let value = ctx
                    .builder
                    .build_load(ty.to_llvm_type(ctx)?, ptr, "index.base")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                if let Type::Nullable(inner) = &ty {
                    if matches!(**inner, Type::Array { .. }) {
                        let payload = value.into_pointer_value();
                        if let Some(panic) = ctx.panic_func {
                            crate::runtime::gen_null_check(
                                ctx.context,
                                &ctx.builder,
                                panic,
                                payload,
                                line,
                            );
                        }
                        (Some((payload, *inner.clone())), None)
                    } else {
                        (None, Some(CodegenValue { value, ty }))
                    }
                } else {
                    (None, Some(CodegenValue { value, ty }))
                }
            }
            Place::Vector(vector, index, ty) => {
                let get = super::vec::get_or_declare_vec_get(ctx)?;
                let raw = ctx
                    .builder
                    .build_call(get, &[vector.into(), index.into()], "index.nested")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .ok_or(CodegenError::TypeMismatch)?
                    .into_int_value();
                let value = super::vec::cast_from_i64(ctx, raw, &ty)?;
                (None, Some(CodegenValue { value, ty }))
            }
        }
    } else {
        (None, Some(generate_expr(ctx, locals, array)?))
    };

    let index_value = generate_expr(ctx, locals, index)?;
    let index_value = crate::nullable::coerce(ctx, index_value, &Type::Int, line)?
        .value
        .into_int_value();
    if let Some((ptr, Type::Array { element_type, size })) = array_storage {
        let length = ctx.context.i64_type().const_int(size as u64, false);
        check_bounds(ctx, index_value, length, line)?;
        let ty = Type::Array {
            element_type: element_type.clone(),
            size,
        }
        .to_llvm_type(ctx)?;
        let address = unsafe {
            ctx.builder
                .build_gep(
                    ty,
                    ptr,
                    &[ctx.context.i64_type().const_zero(), index_value],
                    "index.address",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        };
        return Ok(Place::Memory(address, *element_type));
    }
    let value = crate::nullable::unwrap(ctx, array_value.ok_or(CodegenError::TypeMismatch)?, line)?;
    if let Type::Vec(inner) = value.ty {
        let vector = value.value.into_pointer_value();
        let len = super::vec::get_or_declare_vec_len(ctx)?;
        let length = ctx
            .builder
            .build_call(len, &[vector.into()], "index.length")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or(CodegenError::TypeMismatch)?
            .into_int_value();
        check_bounds(ctx, index_value, length, line)?;
        return Ok(Place::Vector(vector, index_value, *inner));
    }
    Err(CodegenError::UnsupportedExpression)
}

fn check_bounds<'ctx>(
    ctx: &CodegenContext<'ctx>,
    index: IntValue<'ctx>,
    len: IntValue<'ctx>,
    line: u32,
) -> CodegenResult<()> {
    let panic = ctx
        .panic_func
        .ok_or_else(|| CodegenError::LLVMBuildError("panic runtime missing".into()))?;
    crate::runtime::gen_bounds_check(ctx.context, &ctx.builder, panic, index, len, line);
    Ok(())
}

pub(super) fn index_address<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &Locals<'ctx>,
    target: &Expr,
) -> CodegenResult<(PointerValue<'ctx>, Type)> {
    match resolve(ctx, locals, target)? {
        Place::Memory(ptr, ty) => Ok((ptr, ty)),
        Place::Vector(..) => Err(CodegenError::UnsupportedExpression),
    }
}

pub(super) fn assign<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &Locals<'ctx>,
    target: &Expr,
    expression: &Expr,
) -> CodegenResult<()> {
    let place = resolve(ctx, locals, target)?;
    let ty = match &place {
        Place::Memory(_, ty) | Place::Vector(_, _, ty) => ty,
    };
    let value = generate_expr(ctx, locals, expression)?;
    let value = crate::nullable::coerce(ctx, value, ty, ctx.get_line(expression.span.start))?.value;
    match place {
        Place::Memory(ptr, _) => {
            ctx.builder
                .build_store(ptr, value)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
        Place::Vector(vector, index, _) => {
            let value = super::vec::cast_to_i64(ctx, value)?;
            let set = super::vec::get_or_declare_vec_set(ctx)?;
            ctx.builder
                .build_call(set, &[vector.into(), index.into(), value.into()], "")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
    }
    Ok(())
}
