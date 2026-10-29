use super::common::load_field;
use super::ptr::gen_struct_member_ptr_val;
use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 生成安全成员访问 (?. )
pub fn gen_safe_member_access<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    object_expr: &Expr,
    field_name: &str,
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    let object_val = generate_expr(ctx, locals, object_expr)?;

    // Handle Array length safely?
    // If Array, it's non-nullable in Lency type system unless wrapped?
    // If wrapped Nullable(Array), we check null.
    // If Array, it's a pointer.

    // Common null check
    let function = ctx
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
    let safe_access_bb = ctx.context.append_basic_block(function, "safe_access");
    let safe_null_bb = ctx.context.append_basic_block(function, "safe_null");
    let merge_bb = ctx.context.append_basic_block(function, "safe_merge");

    let is_not_null = if object_val.value.is_pointer_value() {
        ctx.builder
            .build_is_not_null(object_val.value.into_pointer_value(), "is_not_null")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
    } else {
        // Not a pointer? Assume valid/true?
        // If type checking allows ?. on non-pointer, it's just true.
        ctx.context.bool_type().const_int(1, false)
    };

    ctx.builder
        .build_conditional_branch(is_not_null, safe_access_bb, safe_null_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Access Block
    ctx.builder.position_at_end(safe_access_bb);

    // For arrays
    let access_res = if matches!(object_val.ty, Type::Array { .. } | Type::Nullable(_)) {
        // We need to differentiate array length from struct field.
        // Need to check inner type.
        let inner_ty_ref = match &object_val.ty {
            Type::Nullable(t) => t.as_ref(),
            t => t,
        };
        if let Type::Array { size, .. } = inner_ty_ref {
            if field_name == "length" {
                Ok(CodegenValue {
                    value: ctx.context.i64_type().const_int(*size as u64, false).into(),
                    ty: Type::Int,
                })
            } else {
                Err(CodegenError::TypeMismatch)
            }
        } else {
            // Struct field
            let field_ptr = gen_struct_member_ptr_val(
                ctx,
                &object_val,
                object_expr.span.start,
                field_name,
                line,
            )?;
            load_field(ctx, &object_val, field_name, field_ptr)
        }
    } else if object_val.value.is_pointer_value() {
        let field_ptr =
            gen_struct_member_ptr_val(ctx, &object_val, object_expr.span.start, field_name, line)?;
        load_field(ctx, &object_val, field_name, field_ptr)
    } else {
        // Struct Value (RValue Aggregate) - use ExtractValue
        let struct_name = match &object_val.ty {
            Type::Struct(name) => name,
            _ => return Err(CodegenError::TypeMismatch),
        };

        let field_names = ctx
            .struct_fields
            .get(struct_name)
            .ok_or(CodegenError::TypeMismatch)?;
        let idx = field_names
            .iter()
            .position(|n| n == field_name)
            .ok_or(CodegenError::TypeMismatch)?;
        let field_types = ctx.struct_field_types.get(struct_name).unwrap();
        let ret_type = field_types[idx].clone();

        let val = ctx
            .builder
            .build_extract_value(
                object_val.value.into_struct_value(),
                idx as u32,
                &format!("field_{}_extract", field_name),
            )
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        Ok(CodegenValue {
            value: val,
            ty: ret_type,
        })
    };

    let valid_val = access_res?;
    let result_ty = match &valid_val.ty {
        Type::Nullable(_) => valid_val.ty.clone(),
        ty => Type::Nullable(Box::new(ty.clone())),
    };
    let valid_val = crate::nullable::coerce(ctx, valid_val, &result_ty, line)?;
    let valid_bb = ctx.builder.get_insert_block().unwrap();
    ctx.builder
        .build_unconditional_branch(merge_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Null Block
    ctx.builder.position_at_end(safe_null_bb);
    // 非空标量已转换为值单元指针；此处统一使用空指针而不是标量零。
    let null_val = valid_val.value.get_type().const_zero();
    let null_end_bb = ctx.builder.get_insert_block().unwrap();
    ctx.builder
        .build_unconditional_branch(merge_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Merge
    ctx.builder.position_at_end(merge_bb);
    let phi = ctx
        .builder
        .build_phi(valid_val.value.get_type(), "safe_phi")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    phi.add_incoming(&[(&valid_val.value, valid_bb), (&null_val, null_end_bb)]);

    Ok(CodegenValue {
        value: phi.as_basic_value(),
        ty: result_ty,
    })
}
