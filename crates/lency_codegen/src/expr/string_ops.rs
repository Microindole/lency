//! String Operations Code Generation
//!
//! 字符串操作代码生成，包含 C 运行时函数声明

use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// 生成字符串连接代码
pub(super) fn concat<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lhs: PointerValue<'ctx>,
    rhs: PointerValue<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let strlen_fn = get_or_declare_strlen(ctx);
    let malloc_fn = get_or_declare_malloc(ctx);
    let strcpy_fn = get_or_declare_strcpy(ctx);
    let strcat_fn = get_or_declare_strcat(ctx);

    let len1 = ctx
        .builder
        .build_call(strlen_fn, &[lhs.into()], "len1")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    let len2 = ctx
        .builder
        .build_call(strlen_fn, &[rhs.into()], "len2")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    let total_len = ctx
        .builder
        .build_int_add(len1, len2, "total_len")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let total_len_plus_one = ctx
        .builder
        .build_int_add(
            total_len,
            ctx.context.i64_type().const_int(1, false),
            "total_len_p1",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let result_ptr = ctx
        .builder
        .build_call(malloc_fn, &[total_len_plus_one.into()], "concat_result")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_pointer_value();

    ctx.builder
        .build_call(strcpy_fn, &[result_ptr.into(), lhs.into()], "")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    ctx.builder
        .build_call(strcat_fn, &[result_ptr.into(), rhs.into()], "")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(result_ptr.into())
}

fn get_or_declare_strlen<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("strlen") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = ctx.context.i64_type().fn_type(&[i8_ptr_type.into()], false);
    ctx.module.add_function("strlen", fn_type, None)
}

fn get_or_declare_malloc<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("lency_alloc") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i8_ptr_type.fn_type(&[ctx.context.i64_type().into()], false);
    ctx.module.add_function("lency_alloc", fn_type, None)
}

fn get_or_declare_strcpy<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("strcpy") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
    ctx.module.add_function("strcpy", fn_type, None)
}

fn get_or_declare_strcat<'ctx>(ctx: &CodegenContext<'ctx>) -> FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("strcat") {
        return func;
    }
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
    ctx.module.add_function("strcat", fn_type, None)
}

// ============== Sprint 12: 字符串内置函数 ==============

use super::CodegenValue;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 生成 len(string) -> int
pub fn gen_len<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    arg: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    use super::generate_expr;

    let arg_val = generate_expr(ctx, locals, arg)?;
    let str_ptr = arg_val.value.into_pointer_value();

    // 声明 lency_string_len
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let i64_type = ctx.context.i64_type();

    let string_len_fn = ctx
        .module
        .get_function("lency_string_len")
        .unwrap_or_else(|| {
            let fn_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
            ctx.module.add_function("lency_string_len", fn_type, None)
        });

    let result = ctx
        .builder
        .build_call(string_len_fn, &[str_ptr.into()], "str_len")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_string_len returned void".to_string(),
        ))?;

    Ok(CodegenValue {
        value: result,
        ty: Type::Int,
    })
}

/// 生成 trim(string) -> string
pub fn gen_trim<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    arg: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    use super::generate_expr;

    let arg_val = generate_expr(ctx, locals, arg)?;
    let str_ptr = arg_val.value.into_pointer_value();

    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());

    let string_trim_fn = ctx
        .module
        .get_function("lency_string_trim")
        .unwrap_or_else(|| {
            let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
            ctx.module.add_function("lency_string_trim", fn_type, None)
        });

    let result = ctx
        .builder
        .build_call(string_trim_fn, &[str_ptr.into()], "str_trim")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_string_trim returned void".to_string(),
        ))?;

    Ok(CodegenValue {
        value: result,
        ty: Type::String,
    })
}

/// 生成 split(string, string) -> Vec<string>
pub fn gen_split<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    str_arg: &Expr,
    delim: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    use super::generate_expr;

    let str_val = generate_expr(ctx, locals, str_arg)?;
    let str_ptr = str_val.value.into_pointer_value();

    let delim_val = generate_expr(ctx, locals, delim)?;
    let delim_ptr = delim_val.value.into_pointer_value();

    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());

    let string_split_fn = ctx
        .module
        .get_function("lency_string_split")
        .unwrap_or_else(|| {
            let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
            ctx.module.add_function("lency_string_split", fn_type, None)
        });

    let result = ctx
        .builder
        .build_call(
            string_split_fn,
            &[str_ptr.into(), delim_ptr.into()],
            "str_split",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_string_split returned void".to_string(),
        ))?;

    Ok(CodegenValue {
        value: result,
        ty: Type::Vec(Box::new(Type::String)),
    })
}

/// 生成 join(Vec<string>, string) -> string
pub fn gen_join<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    vec_arg: &Expr,
    sep: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    use super::generate_expr;

    let vec_val = generate_expr(ctx, locals, vec_arg)?;
    let vec_ptr = vec_val.value.into_pointer_value();

    let sep_val = generate_expr(ctx, locals, sep)?;
    let sep_ptr = sep_val.value.into_pointer_value();

    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());

    let string_join_fn = ctx
        .module
        .get_function("lency_string_join")
        .unwrap_or_else(|| {
            let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
            ctx.module.add_function("lency_string_join", fn_type, None)
        });

    let result = ctx
        .builder
        .build_call(
            string_join_fn,
            &[vec_ptr.into(), sep_ptr.into()],
            "str_join",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_string_join returned void".to_string(),
        ))?;

    Ok(CodegenValue {
        value: result,
        ty: Type::String,
    })
}

/// 生成 substr(string, int, int) -> string
pub fn gen_substr<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    str_arg: &Expr,
    start: &Expr,
    len: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    use super::generate_expr;

    let str_val = generate_expr(ctx, locals, str_arg)?;
    let str_ptr = str_val.value.into_pointer_value();

    let start_val = generate_expr(ctx, locals, start)?;
    let start_int = start_val.value.into_int_value();

    let len_val = generate_expr(ctx, locals, len)?;
    let len_int = len_val.value.into_int_value();

    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let i64_type = ctx.context.i64_type();

    let string_substr_fn = ctx
        .module
        .get_function("lency_string_substr")
        .unwrap_or_else(|| {
            let fn_type = i8_ptr_type.fn_type(
                &[i8_ptr_type.into(), i64_type.into(), i64_type.into()],
                false,
            );
            ctx.module
                .add_function("lency_string_substr", fn_type, None)
        });

    let result = ctx
        .builder
        .build_call(
            string_substr_fn,
            &[str_ptr.into(), start_int.into(), len_int.into()],
            "str_substr",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_string_substr returned void".to_string(),
        ))?;

    Ok(CodegenValue {
        value: result,
        ty: Type::String,
    })
}

/// 生成 char_to_string(int) -> string
/// 将字符码 (ASCII/Unicode) 转换为单字符字符串
pub fn gen_char_to_string<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    arg: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    use super::generate_expr;

    let arg_val = generate_expr(ctx, locals, arg)?;
    let char_code = arg_val.value.into_int_value();

    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let i64_type = ctx.context.i64_type();

    // 声明 C runtime 函数: char* lency_char_to_string(int64_t char_code)
    let char_to_string_fn = ctx
        .module
        .get_function("lency_char_to_string")
        .unwrap_or_else(|| {
            let fn_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
            ctx.module
                .add_function("lency_char_to_string", fn_type, None)
        });

    let result = ctx
        .builder
        .build_call(char_to_string_fn, &[char_code.into()], "char_str")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_char_to_string returned void".to_string(),
        ))?;

    Ok(CodegenValue {
        value: result,
        ty: Type::String,
    })
}

/// 生成 format(string, Vec<string>) -> string
pub fn gen_format<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    template: &Expr,
    args: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    use super::generate_expr;

    let template_val = generate_expr(ctx, locals, template)?;
    let template_ptr = template_val.value.into_pointer_value();

    let args_val = generate_expr(ctx, locals, args)?;
    let args_ptr = args_val.value.into_pointer_value();

    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());

    let string_format_fn = ctx
        .module
        .get_function("lency_string_format")
        .unwrap_or_else(|| {
            let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
            ctx.module
                .add_function("lency_string_format", fn_type, None)
        });

    let result = ctx
        .builder
        .build_call(
            string_format_fn,
            &[template_ptr.into(), args_ptr.into()],
            "str_format",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "lency_string_format returned void".to_string(),
        ))?;

    Ok(CodegenValue {
        value: result,
        ty: Type::String,
    })
}
