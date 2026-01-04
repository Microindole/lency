//! Struct Access Code Generation
//!
//! 处理结构体成员访问：point.x

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::{Expr, Type};
use inkwell::values::PointerValue;
use std::collections::HashMap;

/// 生成成员指针（LValue）
/// 用于赋值或读取
pub fn gen_struct_member_ptr<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    object_expr: &Expr,
    field_name: &str,
    line: u32,
) -> CodegenResult<PointerValue<'ctx>> {
    // 1. 计算对象表达式
    let object_val = generate_expr(ctx, locals, object_expr)?;

    // 2. 必须是指针类型（结构体是通过指针传递的）
    if !object_val.value.is_pointer_value() {
        return Err(CodegenError::UnsupportedType(
            "Field access on non-pointer value".to_string(),
        ));
    }
    let ptr_val = object_val.value.into_pointer_value();

    // 运行时 Null 检查
    if let Some(panic_func) = ctx.panic_func {
        crate::runtime::gen_null_check(ctx.context, &ctx.builder, panic_func, ptr_val, line);
    }

    // 3. 获取结构体名称和 LLVM 类型
    let struct_name = match &object_val.ty {
        Type::Struct(name) => name,

        _ => {
            return Err(CodegenError::UnsupportedType(format!(
                "Field access on non-struct type: {:?}",
                object_val.ty
            )))
        }
    };

    let struct_type = ctx.struct_types.get(struct_name).ok_or_else(|| {
        CodegenError::UnsupportedType(format!("Unknown struct '{}'", struct_name))
    })?;

    // 6. 查找字段索引
    let field_names = ctx.struct_fields.get(struct_name).ok_or_else(|| {
        CodegenError::UnsupportedType(format!("Unknown struct '{}'", struct_name))
    })?;

    let index = field_names
        .iter()
        .position(|n| n == field_name)
        .ok_or_else(|| {
            CodegenError::UnsupportedType(format!(
                "Struct '{}' has no field '{}'",
                struct_name, field_name
            ))
        })?;

    // 7. 生成 GEP
    let field_ptr = ctx
        .builder
        .build_struct_gep(
            *struct_type,
            ptr_val,
            index as u32,
            &format!("field_{}_ptr", field_name),
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(field_ptr)
}

/// 内部辅助：从已有 CodegenValue 生成成员指针
pub fn gen_struct_member_ptr_val<'ctx>(
    ctx: &CodegenContext<'ctx>,
    object_val: &CodegenValue<'ctx>,
    object_span_start: usize, // Needed for line info
    field_name: &str,
    line: u32,
) -> CodegenResult<PointerValue<'ctx>> {
    // 2. 必须是指针类型（结构体是通过指针传递的）
    if !object_val.value.is_pointer_value() {
        return Err(CodegenError::UnsupportedType(
            "Field access on non-pointer value".to_string(),
        ));
    }
    let ptr_val = object_val.value.into_pointer_value();

    // 运行时 Null 检查 (Caller might have done it, e.g. safe access. But if not optional, standard check)
    // Wait, this function shouldn't check null if it's called from safe access!
    // But gen_struct_member_ptr (unsafe) calls it.
    // So we should have a flag? Or separate logic.
    // "gen_struct_member_ptr_val_unchecked"?
    // Or just "gen_struct_member_ptr_val" and let caller handle check.
    // Standard access needs check. Safe access needs check (but branches).
    // So raw GEP generation should not check.

    // 3. 获取结构体名称和 LLVM 类型
    let struct_name = match &object_val.ty {
        Type::Struct(name) => name,
        Type::Nullable(inner) => match &**inner {
            Type::Struct(name) => name,
            _ => return Err(CodegenError::TypeMismatch),
        },
        _ => {
            return Err(CodegenError::UnsupportedType(format!(
                "Field access on non-struct type: {:?}",
                object_val.ty
            )))
        }
    };

    let struct_type = ctx.struct_types.get(struct_name).ok_or_else(|| {
        CodegenError::UnsupportedType(format!("Unknown struct '{}'", struct_name))
    })?;

    // 6. 查找字段索引
    let field_names = ctx.struct_fields.get(struct_name).ok_or_else(|| {
        CodegenError::UnsupportedType(format!("Unknown struct '{}'", struct_name))
    })?;

    let index = field_names
        .iter()
        .position(|n| n == field_name)
        .ok_or_else(|| {
            CodegenError::UnsupportedType(format!(
                "Struct '{}' has no field '{}'",
                struct_name, field_name
            ))
        })?;

    // 7. 生成 GEP
    let field_ptr = ctx
        .builder
        .build_struct_gep(
            *struct_type,
            ptr_val,
            index as u32,
            &format!("field_{}_ptr", field_name),
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(field_ptr)
}

/// 生成成员访问（RValue）
pub fn gen_member_access<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    object_expr: &Expr,
    field_name: &str,
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    let object_val = generate_expr(ctx, locals, object_expr)?;

    // 特殊处理数组 length
    if let Type::Array { size, .. } = &object_val.ty {
        if field_name == "length" {
            return Ok(CodegenValue {
                value: ctx
                    .context
                    .i64_type()
                    .const_int((*size) as u64, false)
                    .into(),
                ty: Type::Int,
            });
        }
    }

    // Null check for standard access
    if object_val.value.is_pointer_value() {
        if let Some(panic_func) = ctx.panic_func {
            crate::runtime::gen_null_check(
                ctx.context,
                &ctx.builder,
                panic_func,
                object_val.value.into_pointer_value(),
                line,
            );
        }
    }

    let field_ptr =
        gen_struct_member_ptr_val(ctx, &object_val, object_expr.span.start, field_name, line)?;

    // Load logic...
    load_field(ctx, &object_val, field_name, field_ptr)
}

/// 辅助：加载字段
fn load_field<'ctx>(
    ctx: &CodegenContext<'ctx>,
    object_val: &CodegenValue<'ctx>,
    field_name: &str,
    field_ptr: PointerValue<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    // Get return type logic (duplicated from before, can extract)
    let struct_name_str = match &object_val.ty {
        Type::Struct(name) => name,
        Type::Nullable(inner) => match &**inner {
            Type::Struct(n) => n,
            _ => return Err(CodegenError::TypeMismatch),
        },
        _ => return Err(CodegenError::TypeMismatch),
    };

    let field_names = ctx
        .struct_fields
        .get(struct_name_str)
        .ok_or(CodegenError::TypeMismatch)?;
    let idx = field_names
        .iter()
        .position(|n| n == field_name)
        .ok_or(CodegenError::TypeMismatch)?;
    let ret_type = ctx.struct_field_types.get(struct_name_str).unwrap()[idx].clone();

    let llvm_ret_type = crate::types::ToLLVMType::to_llvm_type(&ret_type, ctx)?;
    let load = ctx
        .builder
        .build_load(
            llvm_ret_type,
            field_ptr,
            &format!("field_{}_val", field_name),
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: load,
        ty: ret_type,
    })
}

/// 生成安全成员访问 (?. )
pub fn gen_safe_member_access<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    object_expr: &Expr,
    field_name: &str,
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    let object_val = generate_expr(ctx, locals, object_expr)?;

    // Handle Array length safely?
    // If Array, it's non-nullable in Beryl type system unless wrapped?
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
    } else {
        let field_ptr =
            gen_struct_member_ptr_val(ctx, &object_val, object_expr.span.start, field_name, line)?;
        load_field(ctx, &object_val, field_name, field_ptr)
    };

    let valid_val = access_res?;
    let valid_bb = ctx.builder.get_insert_block().unwrap();
    ctx.builder
        .build_unconditional_branch(merge_bb)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Null Block
    ctx.builder.position_at_end(safe_null_bb);
    // Return null of correct type.
    // valid_val.ty is T. We need Nullable(T) compatible.
    // LLVM null pointer matches.
    let null_val = valid_val.value.get_type().const_zero(); // Correct logic for ptrs/ints(0).
                                                            // For Int? we might need special handling if boxed. Assuming pointers.
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

    let result_ty = match valid_val.ty {
        Type::Nullable(_) => valid_val.ty,
        t => Type::Nullable(Box::new(t)),
    };

    Ok(CodegenValue {
        value: phi.as_basic_value(),
        ty: result_ty,
    })
}
