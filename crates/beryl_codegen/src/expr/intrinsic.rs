use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::{Expr, Type};
use inkwell::AddressSpace;
use std::collections::HashMap;

/// 生成 Print 内建函数调用
pub fn gen_print<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    arg: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. 生成参数值
    let val_wrapper = generate_expr(ctx, locals, arg)?;
    let val = val_wrapper.value;
    let ty = val_wrapper.ty;

    let format_str = match ty {
        Type::Int => "%ld\n", // Beryl Int is i64
        Type::Float => "%f\n",
        Type::Bool => "%d\n", // Print bool as 0/1 for now
        Type::String => "%s\n",
        Type::Array { .. } => "[Array]\n", // Placeholder
        Type::Struct(_) => "[Struct]\n",   // Placeholder
        Type::Vec(_) => "[Vec]\n",         // Placeholder

        Type::Generic(_, _) => "[Generic]\n",
        Type::Void => "\n",
        Type::Nullable(_) => "[Nullable]\n",
        Type::Error => "%d\n", // Fallback
    };

    // Special handling for bool (i1) to print "true"/"false" is harder without AST Type info.
    // Let's stick to %d for bools (0/1) or improve later.

    // 构建 printf 调用
    // 构建 printf 调用
    // declare i32 @printf(i8*, ...)
    let i32_type = ctx.context.i32_type();
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());

    // 获取或创建 printf 声明
    let printf_func = if let Some(func) = ctx.module.get_function("printf") {
        func
    } else {
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        ctx.module.add_function("printf", printf_type, None)
    };

    // 创建 format string 全局变量
    let fmt_global_name = format!("fmt.{}", format_str.replace("%", "").replace("\\n", "nl"));
    // Optimization: cache strings?
    let fmt_str_val = ctx
        .builder
        .build_global_string_ptr(format_str, &fmt_global_name)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .as_pointer_value();

    // Call printf
    // Handle float promotion to double for printf (varargs)
    let arg_val = if val.get_type().is_float_type() {
        // float -> double
        // Beryl float is f64, so it's already double.
        val
    } else {
        val
    };

    ctx.builder
        .build_call(
            printf_func,
            &[fmt_str_val.into(), arg_val.into()],
            "printf_call",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Print returns void
    // But Codegen expects BasicValueEnum.
    // We can return a dummy value or change generate_expr signature?
    // generate_expr returns BasicValueEnum.
    // Let's return const int 0.
    // Return value
    Ok(CodegenValue {
        value: ctx.context.i64_type().const_int(0, false).into(),
        ty: Type::Void,
    })
}
