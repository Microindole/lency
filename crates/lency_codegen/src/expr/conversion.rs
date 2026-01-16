//! Type Conversion Code Generation
//!
//! 类型转换函数的代码生成

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::AddressSpace;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// Check if function name is a type conversion function
pub fn is_type_conversion_fn(name: &str) -> bool {
    matches!(
        name,
        "int_to_string"
            | "float_to_string"
            | "parse_int"
            | "parse_float"
            | "file_exists"
            | "is_dir"
    )
}

/// Generate code for type conversion function calls
pub fn gen_type_conversion_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    func_name: &str,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    match func_name {
        "int_to_string" => gen_int_to_string(ctx, locals, args),
        "float_to_string" => gen_float_to_string(ctx, locals, args),
        "parse_int" => gen_parse_int(ctx, locals, args),
        "parse_float" => gen_parse_float(ctx, locals, args),
        "file_exists" => gen_file_exists(ctx, locals, args),
        "is_dir" => gen_is_dir(ctx, locals, args),
        _ => Err(CodegenError::FunctionNotFound(func_name.to_string())),
    }
}

/// Generate code for int_to_string(int) -> string
fn gen_int_to_string<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    if args.len() != 1 {
        return Err(CodegenError::UnsupportedFeature(
            "int_to_string expects 1 argument".to_string(),
        ));
    }

    // Declare or get lency_int_to_string FFI function
    let func = if let Some(f) = ctx.module.get_function("lency_int_to_string") {
        f
    } else {
        let i64_type = ctx.context.i64_type();
        let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
        let fn_type = ptr_type.fn_type(&[i64_type.into()], false);
        ctx.module
            .add_function("lency_int_to_string", fn_type, None)
    };

    // Evaluate argument
    let arg_val = generate_expr(ctx, locals, &args[0])?;
    if arg_val.ty != Type::Int {
        return Err(CodegenError::TypeMismatch);
    }

    // Call FFI
    let call = ctx
        .builder
        .build_call(func, &[arg_val.value.into()], "int_to_str")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result = call.try_as_basic_value().left().unwrap();

    Ok(CodegenValue {
        value: result,
        ty: Type::String,
    })
}

/// Generate code for float_to_string(float) -> string
fn gen_float_to_string<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    if args.len() != 1 {
        return Err(CodegenError::UnsupportedFeature(
            "float_to_string expects 1 argument".to_string(),
        ));
    }

    // Declare or get lency_float_to_string FFI function
    let func = if let Some(f) = ctx.module.get_function("lency_float_to_string") {
        f
    } else {
        let f64_type = ctx.context.f64_type();
        let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
        let fn_type = ptr_type.fn_type(&[f64_type.into()], false);
        ctx.module
            .add_function("lency_float_to_string", fn_type, None)
    };

    // Evaluate argument
    let arg_val = generate_expr(ctx, locals, &args[0])?;
    if arg_val.ty != Type::Float {
        return Err(CodegenError::TypeMismatch);
    }

    // Call FFI
    let call = ctx
        .builder
        .build_call(func, &[arg_val.value.into()], "float_to_str")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result = call.try_as_basic_value().left().unwrap();

    Ok(CodegenValue {
        value: result,
        ty: Type::String,
    })
}

/// Generate code for parse_int(string) -> int
fn gen_parse_int<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    if args.len() != 1 {
        return Err(CodegenError::UnsupportedFeature(
            "parse_int expects 1 argument".to_string(),
        ));
    }

    // Declare or get lency_parse_int FFI function
    let func = if let Some(f) = ctx.module.get_function("lency_parse_int") {
        f
    } else {
        let i64_type = ctx.context.i64_type();
        let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
        let fn_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        ctx.module.add_function("lency_parse_int", fn_type, None)
    };

    // Evaluate argument
    let arg_val = generate_expr(ctx, locals, &args[0])?;
    if arg_val.ty != Type::String {
        return Err(CodegenError::TypeMismatch);
    }

    // Allocate is_ok flag (we ignore it for now, assume success)
    let _i32_type = ctx.context.i32_type();
    let is_ok_ptr = ctx
        .builder
        .build_alloca(_i32_type, "is_ok")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Call FFI
    let call = ctx
        .builder
        .build_call(
            func,
            &[arg_val.value.into(), is_ok_ptr.into()],
            "parse_int_res",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result = call.try_as_basic_value().left().unwrap();

    Ok(CodegenValue {
        value: result,
        ty: Type::Int,
    })
}

/// Generate code for parse_float(string) -> float
fn gen_parse_float<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    if args.len() != 1 {
        return Err(CodegenError::UnsupportedFeature(
            "parse_float expects 1 argument".to_string(),
        ));
    }

    // Declare or get lency_parse_float FFI function
    let func = if let Some(f) = ctx.module.get_function("lency_parse_float") {
        f
    } else {
        let f64_type = ctx.context.f64_type();
        let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
        let fn_type = f64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        ctx.module.add_function("lency_parse_float", fn_type, None)
    };

    // Evaluate argument
    let arg_val = generate_expr(ctx, locals, &args[0])?;
    if arg_val.ty != Type::String {
        return Err(CodegenError::TypeMismatch);
    }

    // Allocate is_ok flag (we ignore it for now, assume success)
    let _i32_type = ctx.context.i32_type();
    let is_ok_ptr = ctx
        .builder
        .build_alloca(_i32_type, "is_ok")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Call FFI
    let call = ctx
        .builder
        .build_call(
            func,
            &[arg_val.value.into(), is_ok_ptr.into()],
            "parse_float_res",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result = call.try_as_basic_value().left().unwrap();

    Ok(CodegenValue {
        value: result,
        ty: Type::Float,
    })
}

/// Generate code for file_exists(string) -> bool
fn gen_file_exists<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    if args.len() != 1 {
        return Err(CodegenError::UnsupportedFeature(
            "file_exists expects 1 argument".to_string(),
        ));
    }

    // Declare or get lency_file_exists FFI function
    let func = if let Some(f) = ctx.module.get_function("lency_file_exists") {
        f
    } else {
        let i64_type = ctx.context.i64_type();
        let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
        let fn_type = i64_type.fn_type(&[ptr_type.into()], false);
        ctx.module.add_function("lency_file_exists", fn_type, None)
    };

    // Evaluate argument
    let arg_val = generate_expr(ctx, locals, &args[0])?;
    if arg_val.ty != Type::String {
        return Err(CodegenError::TypeMismatch);
    }

    // Call FFI
    let call = ctx
        .builder
        .build_call(func, &[arg_val.value.into()], "file_exists_res")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result = call.try_as_basic_value().left().unwrap().into_int_value();

    // Convert i64 to bool
    let bool_result = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            result,
            ctx.context.i64_type().const_zero(),
            "to_bool",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: bool_result.into(),
        ty: Type::Bool,
    })
}

/// Generate code for is_dir(string) -> bool
fn gen_is_dir<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    if args.len() != 1 {
        return Err(CodegenError::UnsupportedFeature(
            "is_dir expects 1 argument".to_string(),
        ));
    }

    // Declare or get lency_file_is_dir FFI function
    let func = if let Some(f) = ctx.module.get_function("lency_file_is_dir") {
        f
    } else {
        let i64_type = ctx.context.i64_type();
        let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
        let fn_type = i64_type.fn_type(&[ptr_type.into()], false);
        ctx.module.add_function("lency_file_is_dir", fn_type, None)
    };

    // Evaluate argument
    let arg_val = generate_expr(ctx, locals, &args[0])?;
    if arg_val.ty != Type::String {
        return Err(CodegenError::TypeMismatch);
    }

    // Call FFI
    let call = ctx
        .builder
        .build_call(func, &[arg_val.value.into()], "is_dir_res")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result = call.try_as_basic_value().left().unwrap().into_int_value();

    // Convert i64 to bool
    let bool_result = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            result,
            ctx.context.i64_type().const_zero(),
            "to_bool",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: bool_result.into(),
        ty: Type::Bool,
    })
}
