//! Vec Literal Code Generation

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::{Expr, Type};
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;
use std::collections::HashMap;

/// Generate code for vec![...] literals
pub fn gen_vec_literal<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    elements: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. Declare beryl_vec_new if not already declared
    let vec_new_fn = get_or_declare_vec_new(ctx)?;

    // 2. Declare beryl_vec_push if not already declared
    let vec_push_fn = get_or_declare_vec_push(ctx)?;

    // 3. Call beryl_vec_new(capacity)
    let capacity = ctx
        .context
        .i64_type()
        .const_int(elements.len() as u64, false);
    let vec_ptr = ctx
        .builder
        .build_call(vec_new_fn, &[capacity.into()], "vec")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or_else(|| CodegenError::LLVMBuildError("vec_new returned void".to_string()))?;

    // 4. For each element, call beryl_vec_push(vec, element)
    for elem in elements {
        let elem_val = generate_expr(ctx, locals, elem)?;

        // Ensure element is i64
        let elem_i64 = match elem_val.value {
            BasicValueEnum::IntValue(iv) => {
                // If it's not i64, cast it
                if iv.get_type() == ctx.context.i64_type() {
                    iv
                } else {
                    ctx.builder
                        .build_int_cast(iv, ctx.context.i64_type(), "cast_to_i64")
                        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
                }
            }
            _ => {
                return Err(CodegenError::TypeMismatch);
            }
        };

        ctx.builder
            .build_call(vec_push_fn, &[vec_ptr.into(), elem_i64.into()], "")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    // 5. Return the vec pointer
    Ok(CodegenValue {
        value: vec_ptr,
        ty: Type::Vec(Box::new(Type::Int)),
    })
}

/// Get or declare beryl_vec_new function
fn get_or_declare_vec_new<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("beryl_vec_new") {
        return Ok(func);
    }

    // declare i8* @beryl_vec_new(i64)
    let i64_type = ctx.context.i64_type();
    let vec_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = vec_ptr_type.fn_type(&[i64_type.into()], false);

    Ok(ctx.module.add_function("beryl_vec_new", fn_type, None))
}

/// Get or declare beryl_vec_push function
fn get_or_declare_vec_push<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("beryl_vec_push") {
        return Ok(func);
    }

    // declare void @beryl_vec_push(i8*, i64)
    let i64_type = ctx.context.i64_type();
    let vec_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = ctx
        .context
        .void_type()
        .fn_type(&[vec_ptr_type.into(), i64_type.into()], false);

    Ok(ctx.module.add_function("beryl_vec_push", fn_type, None))
}

/// Generate method call for Vec
pub fn gen_vec_method_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    object_val: CodegenValue<'ctx>,
    method_name: &str,
    args: &[Expr],
    inner_type: &Type,
) -> CodegenResult<CodegenValue<'ctx>> {
    let vec_ptr = object_val.value.into_pointer_value();

    match method_name {
        "push" => {
            let func = get_or_declare_vec_push(ctx)?;
            let arg_val = generate_expr(ctx, locals, &args[0])?;
            // Cast arg to i64
            let val_i64 = cast_to_i64(ctx, arg_val.value)?;
            ctx.builder
                .build_call(func, &[vec_ptr.into(), val_i64.into()], "")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            // Return void
            Ok(CodegenValue {
                value: ctx.context.i64_type().const_zero().into(), // Dummy
                ty: Type::Void,
            })
        }
        "pop" => {
            let func = get_or_declare_vec_pop(ctx)?;
            let call = ctx
                .builder
                .build_call(func, &[vec_ptr.into()], "pop_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let res_i64 = call.try_as_basic_value().left().unwrap();

            // Cast back to T
            let res_val = cast_from_i64(ctx, res_i64.into_int_value(), inner_type)?;
            Ok(CodegenValue {
                value: res_val,
                ty: inner_type.clone(),
            })
        }
        "len" => {
            let func = get_or_declare_vec_len(ctx)?;
            let call = ctx
                .builder
                .build_call(func, &[vec_ptr.into()], "len_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: call.try_as_basic_value().left().unwrap(),
                ty: Type::Int,
            })
        }
        "get" => {
            let func = get_or_declare_vec_get(ctx)?;
            let index_val = generate_expr(ctx, locals, &args[0])?;
            let index_i64 = index_val.value.into_int_value(); // Assume int

            let call = ctx
                .builder
                .build_call(func, &[vec_ptr.into(), index_i64.into()], "get_res")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let res_i64 = call.try_as_basic_value().left().unwrap();

            // Cast back
            let res_val = cast_from_i64(ctx, res_i64.into_int_value(), inner_type)?;
            Ok(CodegenValue {
                value: res_val,
                ty: inner_type.clone(),
            })
        }
        "set" => {
            let func = get_or_declare_vec_set(ctx)?;
            let index_val = generate_expr(ctx, locals, &args[0])?;
            let index_i64 = index_val.value.into_int_value();

            let val = generate_expr(ctx, locals, &args[1])?;
            let val_i64 = cast_to_i64(ctx, val.value)?;

            ctx.builder
                .build_call(
                    func,
                    &[vec_ptr.into(), index_i64.into(), val_i64.into()],
                    "",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            Ok(CodegenValue {
                value: ctx.context.i64_type().const_zero().into(),
                ty: Type::Void,
            })
        }
        _ => Err(CodegenError::FunctionNotFound(method_name.to_string())),
    }
}

// Helpers for declaring missing functions
pub(crate) fn get_or_declare_vec_pop<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("beryl_vec_pop") {
        return Ok(func);
    }
    let i64_type = ctx.context.i64_type();
    let vec_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i64_type.fn_type(&[vec_ptr_type.into()], false);
    Ok(ctx.module.add_function("beryl_vec_pop", fn_type, None))
}

pub(crate) fn get_or_declare_vec_len<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("beryl_vec_len") {
        return Ok(func);
    }
    let i64_type = ctx.context.i64_type();
    let vec_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i64_type.fn_type(&[vec_ptr_type.into()], false); // Takes const ptr? in lib.rs logic it takes const ptr.
    Ok(ctx.module.add_function("beryl_vec_len", fn_type, None))
}

pub(crate) fn get_or_declare_vec_get<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("beryl_vec_get") {
        return Ok(func);
    }
    let i64_type = ctx.context.i64_type();
    let vec_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i64_type.fn_type(&[vec_ptr_type.into(), i64_type.into()], false);
    Ok(ctx.module.add_function("beryl_vec_get", fn_type, None))
}

pub(crate) fn get_or_declare_vec_set<'ctx>(
    ctx: &CodegenContext<'ctx>,
) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
    if let Some(func) = ctx.module.get_function("beryl_vec_set") {
        return Ok(func);
    }
    let i64_type = ctx.context.i64_type();
    let vec_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = ctx.context.void_type().fn_type(
        &[vec_ptr_type.into(), i64_type.into(), i64_type.into()],
        false,
    );
    Ok(ctx.module.add_function("beryl_vec_set", fn_type, None))
}

pub(crate) fn cast_to_i64<'ctx>(
    ctx: &CodegenContext<'ctx>,
    val: BasicValueEnum<'ctx>,
) -> CodegenResult<inkwell::values::IntValue<'ctx>> {
    if val.is_pointer_value() {
        Ok(ctx
            .builder
            .build_ptr_to_int(val.into_pointer_value(), ctx.context.i64_type(), "ptr2int")
            .unwrap())
    } else if val.is_int_value() {
        let iv = val.into_int_value();
        if iv.get_type() == ctx.context.i64_type() {
            Ok(iv)
        } else {
            Ok(ctx
                .builder
                .build_int_cast(iv, ctx.context.i64_type(), "cast")
                .unwrap())
        }
    } else {
        // float?
        if val.is_float_value() {
            Ok(ctx
                .builder
                .build_bitcast(val.into_float_value(), ctx.context.i64_type(), "f2i")
                .unwrap()
                .into_int_value())
        } else {
            Err(CodegenError::TypeMismatch)
        }
    }
}

pub(crate) fn cast_from_i64<'ctx>(
    ctx: &CodegenContext<'ctx>,
    val: inkwell::values::IntValue<'ctx>,
    ty: &Type,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match ty {
        Type::Int => Ok(val.into()),
        Type::Float => Ok(ctx
            .builder
            .build_bitcast(val, ctx.context.f64_type(), "i2f")
            .unwrap()),
        Type::String | Type::Struct(_) | Type::Vec(_) | Type::Array { .. } => {
            // Pointer
            let ptr_ty = ctx.context.i8_type().ptr_type(AddressSpace::default());
            Ok(ctx
                .builder
                .build_int_to_ptr(val, ptr_ty, "i2ptr")
                .unwrap()
                .into())
        }
        Type::Bool => Ok(ctx
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                val,
                ctx.context.i64_type().const_zero(),
                "i2bool",
            )
            .unwrap()
            .into()),
        _ => Err(CodegenError::TypeMismatch),
    }
}
