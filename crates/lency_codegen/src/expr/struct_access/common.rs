use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::CodegenValue;
use crate::types::ToLLVMType;
use inkwell::values::PointerValue;
use lency_syntax::ast::Type;

/// 辅助：加载字段
pub fn load_field<'ctx>(
    ctx: &CodegenContext<'ctx>,
    object_val: &CodegenValue<'ctx>,
    field_name: &str,
    field_ptr: PointerValue<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let ret_type = field_type(ctx, &object_val.ty, field_name)?;
    let llvm_ret_type = ret_type.to_llvm_type(ctx)?;
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

/// 读取成员类型元数据，不执行接收者表达式。
pub(super) fn field_type(
    ctx: &CodegenContext<'_>,
    object_ty: &Type,
    field_name: &str,
) -> CodegenResult<Type> {
    let struct_name_str = match object_ty {
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
    ctx.struct_field_types
        .get(struct_name_str)
        .and_then(|fields| fields.get(idx))
        .cloned()
        .ok_or(CodegenError::TypeMismatch)
}
