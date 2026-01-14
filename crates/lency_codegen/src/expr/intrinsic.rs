//! Intrinsic Functions Code Generation
//!
//! 内置函数代码生成：print
//! 文件 I/O 函数已移至 file_io 模块

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::vec::{cast_from_i64, get_or_declare_vec_get, get_or_declare_vec_len};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::AddressSpace;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

// 重新导出 file_io 模块的函数
pub use super::file_io::{gen_read_file, gen_write_file};

/// 生成 Print 内建函数调用
pub fn gen_print<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    arg: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    let arg_val = generate_expr(ctx, locals, arg)?;

    // 1. 打印值内容
    gen_print_value_impl(ctx, arg_val.value, &arg_val.ty)?;

    // 2. 打印换行符
    gen_print_newline(ctx)?;

    Ok(CodegenValue {
        value: ctx.context.i64_type().const_int(0, false).into(),
        ty: Type::Void,
    })
}

/// 打印值内容（不换行）
fn gen_print_value_impl<'ctx>(
    ctx: &CodegenContext<'ctx>,
    value: inkwell::values::BasicValueEnum<'ctx>,
    ty: &Type,
) -> CodegenResult<()> {
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let i64_type = ctx.context.i64_type();

    // Helper to get printf
    let get_printf = || {
        ctx.module.get_function("printf").unwrap_or_else(|| {
            let fn_type = i64_type.fn_type(&[i8_ptr_type.into()], true);
            ctx.module.add_function("printf", fn_type, None)
        })
    };

    match ty {
        Type::Int => {
            let printf_fn = get_printf();
            let format_str = ctx
                .builder
                .build_global_string_ptr("%lld", "int_fmt")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            ctx.builder
                .build_call(
                    printf_fn,
                    &[format_str.as_pointer_value().into(), value.into()],
                    "print_int",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
        Type::Float => {
            let printf_fn = get_printf();
            let format_str = ctx
                .builder
                .build_global_string_ptr("%f", "float_fmt")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            ctx.builder
                .build_call(
                    printf_fn,
                    &[format_str.as_pointer_value().into(), value.into()],
                    "print_float",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
        Type::Bool => {
            let printf_fn = get_printf();
            let format_str = ctx
                .builder
                .build_global_string_ptr("%s", "bool_fmt")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let true_str = ctx
                .builder
                .build_global_string_ptr("true", "true_str")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let false_str = ctx
                .builder
                .build_global_string_ptr("false", "false_str")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let bool_val = value.into_int_value();
            let str_val = ctx
                .builder
                .build_select(
                    bool_val,
                    true_str.as_pointer_value(),
                    false_str.as_pointer_value(),
                    "bool_str",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            ctx.builder
                .build_call(
                    printf_fn,
                    &[format_str.as_pointer_value().into(), str_val.into()],
                    "print_bool",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
        Type::String => {
            let printf_fn = get_printf();
            let format_str = ctx
                .builder
                .build_global_string_ptr("%s", "str_fmt")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            ctx.builder
                .build_call(
                    printf_fn,
                    &[format_str.as_pointer_value().into(), value.into()],
                    "print_str",
                )
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
        Type::Vec(inner_type) => {
            // Print "["
            gen_print_str_literal(ctx, "[")?;

            let vec_ptr = value.into_pointer_value();
            let len_fn = get_or_declare_vec_len(ctx)?;
            let get_fn = get_or_declare_vec_get(ctx)?;

            // Get length
            let len_call = ctx
                .builder
                .build_call(len_fn, &[vec_ptr.into()], "len")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let len = len_call
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_int_value();

            // Loop Construction
            let func = ctx
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();
            let entry_bb = ctx.builder.get_insert_block().unwrap(); // Save entry block for PHI

            let loop_bb = ctx.context.append_basic_block(func, "print_loop");
            let body_bb = ctx.context.append_basic_block(func, "print_body");
            let end_bb = ctx.context.append_basic_block(func, "print_end");

            ctx.builder.build_unconditional_branch(loop_bb).unwrap();

            // LOOP BB
            ctx.builder.position_at_end(loop_bb);
            let i_phi = ctx.builder.build_phi(ctx.context.i64_type(), "i").unwrap();
            let i = i_phi.as_basic_value().into_int_value();

            let cmp = ctx
                .builder
                .build_int_compare(inkwell::IntPredicate::SLT, i, len, "loop_cond")
                .unwrap();
            ctx.builder
                .build_conditional_branch(cmp, body_bb, end_bb)
                .unwrap();

            // BODY BB
            ctx.builder.position_at_end(body_bb);

            // Get element
            let get_call = ctx
                .builder
                .build_call(get_fn, &[vec_ptr.into(), i.into()], "elem_raw")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            let elem_i64 = get_call
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_int_value();

            // Cast and Print element
            let elem_val = cast_from_i64(ctx, elem_i64, inner_type)?;
            gen_print_value_impl(ctx, elem_val, inner_type)?;

            // Print separator logic
            let next_i = ctx
                .builder
                .build_int_add(i, ctx.context.i64_type().const_int(1, false), "next_i")
                .unwrap();

            let is_last = ctx
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, next_i, len, "is_last")
                .unwrap();

            let sep_bb = ctx.context.append_basic_block(func, "print_sep");
            let cont_bb = ctx.context.append_basic_block(func, "print_cont");

            ctx.builder
                .build_conditional_branch(is_last, cont_bb, sep_bb)
                .unwrap();

            // Separator BB
            ctx.builder.position_at_end(sep_bb);
            gen_print_str_literal(ctx, ", ")?;
            ctx.builder.build_unconditional_branch(cont_bb).unwrap();

            // Continue BB
            ctx.builder.position_at_end(cont_bb);
            ctx.builder.build_unconditional_branch(loop_bb).unwrap();

            // PHI Setup
            // Incoming 0 from entry_bb
            // Incoming next_i from cont_bb
            i_phi.add_incoming(&[
                (&ctx.context.i64_type().const_int(0, false), entry_bb),
                (&next_i, cont_bb),
            ]);

            // END BB
            ctx.builder.position_at_end(end_bb);
            gen_print_str_literal(ctx, "]")?;
        }
        _ => {
            // 其他类型打印其类型名称或占位符
            gen_print_str_literal(ctx, "<unknown>")?;
        }
    }
    Ok(())
}

fn gen_print_newline<'ctx>(ctx: &CodegenContext<'ctx>) -> CodegenResult<()> {
    gen_print_str_literal(ctx, "\n")
}

fn gen_print_str_literal<'ctx>(ctx: &CodegenContext<'ctx>, s: &str) -> CodegenResult<()> {
    let i8_ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
    let i64_type = ctx.context.i64_type();

    let printf_fn = ctx.module.get_function("printf").unwrap_or_else(|| {
        let fn_type = i64_type.fn_type(&[i8_ptr_type.into()], true);
        ctx.module.add_function("printf", fn_type, None)
    });

    let format_str = ctx
        .builder
        .build_global_string_ptr("%s", "str_fmt")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let val_str = ctx
        .builder
        .build_global_string_ptr(s, "literal")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    ctx.builder
        .build_call(
            printf_fn,
            &[
                format_str.as_pointer_value().into(),
                val_str.as_pointer_value().into(),
            ],
            "print_lit",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    Ok(())
}
