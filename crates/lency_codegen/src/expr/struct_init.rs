//! Struct Initialization Code Generation
//!
//! 处理结构体字面量：Point { x: 1, y: 2 }

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 生成结构体字面量
pub fn gen_struct_literal<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    type_name: &str,
    fields: &[(String, Expr)],
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. 获取结构体类型
    let struct_type = ctx.struct_types.get(type_name).ok_or_else(|| {
        CodegenError::UnsupportedType(format!("Struct '{}' not found", type_name))
    })?;

    // 2. 获取字段顺序
    let field_names = ctx.struct_fields.get(type_name).ok_or_else(|| {
        CodegenError::UnsupportedType(format!("Struct '{}' fields not found", type_name))
    })?;

    // 3. 通过运行时分配，以便匹配手动释放接口。
    // size_of 返回的是 Option<IntValue>，因为如果是 opaque 可能没有 size。但我们已经定义了 body。
    let size = struct_type.size_of().ok_or_else(|| {
        CodegenError::LLVMBuildError(format!("Struct '{}' has no size (opaque?)", type_name))
    })?;

    let malloc = ctx.module.get_function("lency_alloc").ok_or_else(|| {
        CodegenError::LLVMBuildError("lency_alloc function not found".to_string())
    })?;

    let malloc_call = ctx
        .builder
        .build_call(malloc, &[size.into()], "malloc_struct")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let raw_ptr = malloc_call
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_alloc returned void".to_string(),
        ))?
        .into_pointer_value();

    // 4. 转换为结构体指针。
    let struct_ptr_type = struct_type.ptr_type(inkwell::AddressSpace::default());
    let struct_ptr = ctx
        .builder
        .build_bitcast(raw_ptr, struct_ptr_type, "struct_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .into_pointer_value();

    // 5. 初始化字段
    // 将提供的字段值映射到它们在结构体中的位置
    let mut provided_values = HashMap::new();
    for (name, expr) in fields {
        let val_wrapper = generate_expr(ctx, locals, expr)?;
        let field_index = field_names
            .iter()
            .position(|field| field == name)
            .ok_or(CodegenError::TypeMismatch)?;
        let expected = &ctx
            .struct_field_types
            .get(type_name)
            .ok_or(CodegenError::TypeMismatch)?[field_index];
        let val_wrapper =
            crate::nullable::coerce(ctx, val_wrapper, expected, ctx.get_line(expr.span.start))?;
        provided_values.insert(name, val_wrapper.value);
    }

    // 按定义顺序遍历字段并 store
    for (i, field_name) in field_names.iter().enumerate() {
        if let Some(val) = provided_values.get(field_name) {
            // GEP to field address
            // struct_ptr is Pointer to Struct.
            // GEP needs to dereference that pointer?
            // build_struct_gep takes the pointer to the struct.
            let field_ptr = ctx
                .builder
                .build_struct_gep(
                    *struct_type,
                    struct_ptr,
                    i as u32,
                    &format!("field_{}", field_name),
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            ctx.builder
                .build_store(field_ptr, *val)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        } else {
            return Err(CodegenError::TypeMismatch);
        }
    }

    Ok(CodegenValue {
        value: struct_ptr.into(),
        ty: Type::Struct(type_name.to_string()),
    })
}
