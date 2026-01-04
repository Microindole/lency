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
    // TODO: array might be passed as generic ptr if we change array repr. For now array literals are alloca'd array types.

    // 正常结构体字段访问
    let field_ptr = gen_struct_member_ptr(ctx, locals, object_expr, field_name, line)?;

    // Load
    // We need element type for build_load.
    // field_ptr is a pointer.
    // From step 81 (in prev function), we got field_ptr via GEP using struct_type.
    let struct_name_str = match &object_val.ty {
        Type::Struct(name) => name,

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
    let field_types = ctx
        .struct_field_types
        .get(struct_name_str)
        .ok_or(CodegenError::TypeMismatch)?;
    let ret_type = field_types
        .get(idx)
        .cloned()
        .ok_or(CodegenError::TypeMismatch)?;

    // We need LLVM type for load.
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
