//! Function Call Code Generation
//!
//! 函数调用代码生成

use lency_syntax::ast::{Expr, ExprKind};
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

use crate::expr::{generate_expr, CodegenValue};
use crate::types::ToLLVMType;
use inkwell::types::BasicType;
use lency_syntax::ast::Type;

/// 生成函数调用代码
pub(super) fn gen_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    callee: &Expr,
    args: &[Expr],
) -> CodegenResult<CodegenValue<'ctx>> {
    // 获取函数名
    let func_name = match &callee.kind {
        ExprKind::Variable(name) => name,
        _ => return Err(CodegenError::UnsupportedExpression),
    };

    // Intrinsics keep ordinary Call AST nodes. Their names are registered as
    // global function symbols by sema; only this lowering boundary is special.
    match (func_name.as_str(), args) {
        ("print", [arg]) => return super::intrinsic::gen_print(ctx, locals, arg),
        ("read_file", [path]) => return super::intrinsic::gen_read_file(ctx, locals, path),
        ("write_file", [path, content]) => {
            return super::intrinsic::gen_write_file(ctx, locals, path, content)
        }
        ("len", [arg]) => return super::string_ops::gen_len(ctx, locals, arg),
        ("trim", [arg]) => return super::string_ops::gen_trim(ctx, locals, arg),
        ("split", [value, delimiter]) => {
            return super::string_ops::gen_split(ctx, locals, value, delimiter)
        }
        ("join", [values, separator]) => {
            return super::string_ops::gen_join(ctx, locals, values, separator)
        }
        ("substr", [value, start, len]) => {
            return super::string_ops::gen_substr(ctx, locals, value, start, len)
        }
        ("char_to_string", [arg]) => {
            return super::string_ops::gen_char_to_string(ctx, locals, arg)
        }
        ("format", [template, values]) => {
            return super::string_ops::gen_format(ctx, locals, template, values)
        }
        ("panic", [message]) => return super::intrinsic::gen_panic(ctx, locals, message),
        (name, _) if is_intrinsic_name(name) => {
            return Err(CodegenError::UnsupportedFeature(format!(
                "invalid argument count for intrinsic {name}"
            )))
        }
        _ => {}
    }

    // 检查是否为 hashmap extern 函数
    if super::hashmap::is_hashmap_extern(func_name) {
        return super::hashmap::gen_hashmap_extern_call(ctx, locals, func_name, args);
    }

    // 检查是否为类型转换函数
    if super::conversion::is_type_conversion_fn(func_name) {
        return super::conversion::gen_type_conversion_call(ctx, locals, func_name, args);
    }

    // 生成参数
    let mut arg_values = Vec::new();
    for arg in args {
        let val_wrapper = generate_expr(ctx, locals, arg)?;
        arg_values.push(val_wrapper.value.into());
    }

    // 检查是否为函数指针变量。冻结语法不提供闭包字面量，但保留函数类型调用。
    if let Some((ptr, var_ty)) = locals.get(func_name) {
        if let Type::Function {
            param_types,
            return_type,
        } = var_ty
        {
            // 获取函数指针的 LLVM 类型
            let fn_ptr_type = var_ty.to_llvm_type(ctx)?;

            // 加载函数指针
            let fn_ptr = ctx
                .builder
                .build_load(fn_ptr_type, *ptr, "fn_ptr_load")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
                .into_pointer_value();

            // 构建函数类型
            let param_llvm_types: Result<Vec<_>, _> = param_types
                .iter()
                .map(|t| t.to_llvm_type(ctx).map(|ty| ty.into()))
                .collect();
            let param_llvm_types = param_llvm_types?;

            let fn_type = if matches!(**return_type, Type::Void) {
                ctx.context.void_type().fn_type(&param_llvm_types, false)
            } else {
                let ret = return_type.to_llvm_type(ctx)?;
                ret.fn_type(&param_llvm_types, false)
            };

            // 间接调用
            let call_site = ctx
                .builder
                .build_indirect_call(fn_type, fn_ptr, &arg_values, "function_pointer_call")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let val = call_site.try_as_basic_value().left();

            if let Some(v) = val {
                return Ok(CodegenValue {
                    value: v,
                    ty: *return_type.clone(),
                });
            } else {
                let dummy = ctx.context.bool_type().const_int(0, false).into();
                return Ok(CodegenValue {
                    value: dummy,
                    ty: Type::Void,
                });
            }
        }
    }

    // 查找直接函数
    let function = ctx
        .module
        .get_function(func_name)
        .ok_or_else(|| CodegenError::FunctionNotFound(func_name.clone()))?;

    // 获取函数返回类型
    let return_type = ctx
        .function_signatures
        .get(func_name)
        .cloned()
        .ok_or_else(|| CodegenError::FunctionNotFound(func_name.clone()))?;

    // 调用函数
    let call_site = ctx
        .builder
        .build_call(function, &arg_values, "calltmp")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let val = call_site.try_as_basic_value().left();

    if let Some(v) = val {
        Ok(CodegenValue {
            value: v,
            ty: return_type,
        })
    } else {
        let dummy = ctx.context.bool_type().const_int(0, false).into();
        Ok(CodegenValue {
            value: dummy,
            ty: Type::Void,
        })
    }
}

fn is_intrinsic_name(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "read_file"
            | "write_file"
            | "len"
            | "trim"
            | "split"
            | "join"
            | "substr"
            | "char_to_string"
            | "format"
            | "panic"
    )
}
