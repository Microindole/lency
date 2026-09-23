use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::types::BasicType;
use inkwell::AddressSpace;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 生成数组字面量
/// [1, 2, 3] -> 栈上分配 + 逐个存储
pub fn gen_array_literal<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
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
    let elem_lency_type = first_elem.ty.clone();

    let array_type = elem_llvm_type.array_type(elements.len() as u32);
    let array_lency_type = Type::Array {
        element_type: Box::new(elem_lency_type),
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
        ty: array_lency_type,
    })
}

/// 生成数组索引访问
/// arr[i] -> GEP + load (带边界检查)
pub fn gen_index_access<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
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

    if let Type::Vec(inner) = &array_wrapper.ty {
        // Vec 索引访问
        // call lency_vec_get
        let vec_ptr = array_val.into_pointer_value(); // %LencyVec*

        let func = crate::expr::vec::get_or_declare_vec_get(ctx)?;
        let call = ctx
            .builder
            .build_call(func, &[vec_ptr.into(), index_int.into()], "get_res")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        let res_i64 = call.try_as_basic_value().left().unwrap();

        let res_val = crate::expr::vec::cast_from_i64(ctx, res_i64.into_int_value(), inner)?;

        return Ok(CodegenValue {
            value: res_val,
            ty: *inner.clone(),
        });
    }

    if array_wrapper.ty == Type::String {
        // String Indexing: s[i] -> int (byte)
        let str_ptr = array_val.into_pointer_value();

        let i64_type = ctx.context.i64_type();
        let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
        let string_len_fn = ctx
            .module
            .get_function("lency_string_len")
            .unwrap_or_else(|| {
                let fn_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
                ctx.module.add_function("lency_string_len", fn_type, None)
            });
        let len = ctx
            .builder
            .build_call(string_len_fn, &[str_ptr.into()], "str_index_len")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| {
                CodegenError::LLVMBuildError("lency_string_len returned void".to_string())
            })?
            .into_int_value();

        let panic_func = ctx.panic_func.ok_or_else(|| {
            CodegenError::LLVMBuildError("panic runtime is not initialized".to_string())
        })?;
        crate::runtime::gen_bounds_check(
            ctx.context,
            &ctx.builder,
            panic_func,
            index_int,
            len,
            line,
        );

        // GEP i8* s, index
        let char_ptr = unsafe {
            ctx.builder
                .build_gep(ctx.context.i8_type(), str_ptr, &[index_int], "char_ptr")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        };

        // Load i8
        let char_val = ctx
            .builder
            .build_load(ctx.context.i8_type(), char_ptr, "char_val")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // Cast i8 to i64 (int)
        let int_val = ctx
            .builder
            .build_int_z_extend(
                char_val.into_int_value(),
                ctx.context.i64_type(),
                "char_to_int",
            )
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        return Ok(CodegenValue {
            value: int_val.into(),
            ty: Type::Int,
        });
    }

    // Array 逻辑
    // 获取数组类型
    let array_type = array_val.get_type();
    // 数组必须是 array type
    if !array_type.is_array_type() {
        return Err(CodegenError::UnsupportedType(
            "Indexing non-array type".into(),
        ));
    }
    let arr_ty = array_type.into_array_type();

    let array_size = arr_ty.len() as u64;

    // Lency element type
    let elem_lency_type = match array_wrapper.ty {
        Type::Array {
            element_type: inner,
            ..
        } => *inner,
        _ => {
            return Err(CodegenError::TypeMismatch);
        }
    };

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
        ty: elem_lency_type,
    })
}
