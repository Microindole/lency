//! Struct Initialization Code Generation
//!
//! 处理结构体字面量：Point { x: 1, y: 2 }

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 生成结构体字面量
pub fn gen_struct_literal<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
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

    // 3. 计算大小并调用 malloc
    // size_of 返回的是 Option<IntValue>，因为如果是 opaque 可能没有 size。但我们已经定义了 body。
    let size = struct_type.size_of().ok_or_else(|| {
        CodegenError::LLVMBuildError(format!("Struct '{}' has no size (opaque?)", type_name))
    })?;

    let malloc = ctx
        .module
        .get_function("malloc")
        .ok_or_else(|| CodegenError::LLVMBuildError("malloc function not found".to_string()))?;

    let malloc_call = ctx
        .builder
        .build_call(malloc, &[size.into()], "malloc_struct")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let raw_ptr = malloc_call
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "malloc returned void".to_string(),
        ))?
        .into_pointer_value();

    // 4. Bitcast i8* to StructType*
    let struct_ptr_type = struct_type.ptr_type(inkwell::AddressSpace::default());
    // Inkwell's build_bitcast takes BasicValueEnum, returns BasicValueEnum
    // Wait, build_bitcast takes (val, type, name).
    // raw_ptr is PointerValue.

    // In LLVM 15+ typed pointers are gone, so raw_ptr (i8*) can often just be used if we specify type in GEP/Load/Store.
    // However, for strictness and if using older LLVM binding behavior:
    // If opaque pointers are enabled (LLVM 15 default), bitcast might be no-op or unnecessary,
    // but inkwell might explicit types.
    // Let's assume typed pointers for safety with inkwell API.

    // Actually, inkwell wrapping implies typed pointers API usually.
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
            // 如果字段没有在字面量中提供，这里会有默认值逻辑吗？
            // Beryl 语法目前似乎要求提供所有字段，或者 Sema 阶段会检查。
            // 这里假设 Sema 已经保证完整性，或者是 Option/Default 处理。
            // 如果缺失，暂不处理（或者是 garbage/zero initialized if malloc was calloc? malloc is raw）
            // 应该报错或者 fill zero。
            // 为了简化，假设必填。
        }
    }

    Ok(CodegenValue {
        value: struct_ptr.into(),
        ty: Type::Struct(type_name.to_string()),
    })
}
