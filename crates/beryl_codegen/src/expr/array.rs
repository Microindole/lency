use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::{Expr, Type};
use inkwell::types::BasicType;
use std::collections::HashMap;

/// 生成数组字面量
/// [1, 2, 3] -> 栈上分配 + 逐个存储
pub fn gen_array_literal<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    elements: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    if elements.is_empty() {
        // For an empty array literal `[]`, its type is ambiguous without context.
        // Sema should ideally infer this or require explicit type annotation.
        // For now, we'll return an error as we can't determine the element type for LLVM.
        return Err(CodegenError::UnsupportedExpression);
    }

    // 生成所有元素的值
    let mut element_codegens = Vec::new();
    for elem in elements {
        element_codegens.push(generate_expr(ctx, locals, elem)?);
    }

    // 元素类型（假设所有元素类型相同，Sema 已验证）
    // element_codegens[0] is CodegenValue
    let first_elem = &element_codegens[0];
    let elem_llvm_type = first_elem.value.get_type();
    let elem_beryl_type = first_elem.ty.clone();

    let array_type = elem_llvm_type.array_type(elements.len() as u32);
    let array_beryl_type = Type::Array {
        element_type: Box::new(elem_beryl_type),
        size: elements.len(),
    };

    // 在栈上分配数组
    let array_alloca = ctx
        .builder
        .build_alloca(array_type, "array_literal")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 逐个存储元素
    for (i, val_wrapper) in element_codegens.iter().enumerate() {
        let value = val_wrapper.value;
        // GEP: array_ptr, 0, i
        let indices = [
            ctx.context.i64_type().const_int(0, false),
            ctx.context.i64_type().const_int(i as u64, false),
        ];
        let elem_ptr = unsafe {
            ctx.builder
                .build_gep(
                    array_type,
                    array_alloca,
                    &indices,
                    &format!("elem_{}_ptr", i),
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        };

        ctx.builder
            .build_store(elem_ptr, value)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 加载整个数组作为值返回
    let val = ctx
        .builder
        .build_load(array_type, array_alloca, "array_value")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: val,
        ty: array_beryl_type,
    })
}

/// 生成数组索引访问
/// arr[i] -> GEP + load (带边界检查)
pub fn gen_index_access<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    array_expr: &Expr,
    index_expr: &Expr,
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 生成数组和索引
    let array_wrapper = generate_expr(ctx, locals, array_expr)?;
    let index_wrapper = generate_expr(ctx, locals, index_expr)?;

    let array_val = array_wrapper.value;
    let index_val = index_wrapper.value;

    // 确保索引是整数
    let index_int = index_val.into_int_value();

    // 获取数组类型
    let array_type = array_val.get_type();

    // 数组必须是 array type
    let arr_ty = array_type.into_array_type();

    let array_size = arr_ty.len() as u64;

    // Beryl element type
    let elem_beryl_type = match array_wrapper.ty {
        Type::Array {
            element_type: inner,
            ..
        } => *inner,
        _ => {
            return Err(CodegenError::UnsupportedType(
                "Indexing non-array type".into(),
            ))
        }
    };

    // === 边界检查 ===
    // if (index < 0 || index >= size) { panic }

    // 1. index >= 0 (对于 i64，检查符号位)
    // === 边界检查 ===
    if let Some(panic_func) = ctx.panic_func {
        let len_val = ctx.context.i64_type().const_int(array_size, false);
        crate::runtime::gen_bounds_check(
            ctx.context,
            &ctx.builder,
            panic_func,
            index_int,
            len_val,
            line,
        );
    }

    // 需要先将数组存到栈上（因为 array_val 是值）
    let array_alloca = ctx
        .builder
        .build_alloca(arr_ty, "array_temp")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    ctx.builder
        .build_store(array_alloca, array_val)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // GEP: array_ptr, 0, index
    let indices = [ctx.context.i64_type().const_int(0, false), index_int];
    let elem_ptr = unsafe {
        ctx.builder
            .build_gep(arr_ty, array_alloca, &indices, "elem_ptr")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
    };

    // Load element
    let elem_type = arr_ty.get_element_type();
    let val = ctx
        .builder
        .build_load(elem_type, elem_ptr, "elem_value")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: val,
        ty: elem_beryl_type,
    })
}
