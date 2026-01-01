use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::generate_expr;
use beryl_syntax::ast::Expr;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;
use std::collections::HashMap;

/// 生成 Print 内建函数调用
pub fn gen_print<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
    arg: &Expr,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    // 1. 生成参数值
    let val = generate_expr(ctx, locals, arg)?;

    // 2. 推断参数类型 (Codegen 阶段没有 AST Type 信息，需要从 LLVM Value 推断或 Sema 传递?
    //    Sema 已经检查过了，但 Codegen 这里只有 Expr AST 和 locals map。
    //    Value 只有 LLVM Type。
    //    我们需要知道 AST Type 才能选择正确的 format string。
    //    不幸的是，Expr AST 节点在 AST 阶段并没有附带推断出的 Type。
    //    在此架构下，我们只能根据 LLVM Type 猜测，或者在 AST 中存储 Type。
    //
    //    Current Architecture Limitation: AST does not store Types on Expr nodes after Sema.
    //    Workaround: Check LLVM Type.

    let llvm_type = val.get_type();
    let format_str = if llvm_type.is_int_type() {
        "%d\n"
    } else if llvm_type.is_float_type() {
        "%f\n"
    } else if llvm_type.is_pointer_type() {
        // Assume string for now (i8*)
        // To distinguish string vs others, strict typing needed.
        // For Beryl MVP, only Strings are pointers besides classes.
        "%s\n"
    } else {
        // Bool is i1 (int type in LLVM often i1 or i8)
        // Wait, is_int_type() returns true for i1.
        "%d\n"
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
    let arg_val = if llvm_type.is_float_type() {
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
    Ok(ctx.context.i64_type().const_int(0, false).into())
}
