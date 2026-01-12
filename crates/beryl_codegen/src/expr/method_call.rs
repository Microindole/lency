//! Method Call Code Generation
//!
//! 处理方法调用：object.method(args)

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use beryl_syntax::ast::{Expr, Type};
use inkwell::values::PointerValue;
use std::collections::HashMap;

/// 生成方法调用代码
pub fn gen_method_call<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (PointerValue<'ctx>, Type)>,
    object: &Expr,
    method_name: &str,
    args: &[Expr],
    line: u32,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 0. Check for Enum Constructor Call (Enum.Variant(args))
    let enum_check = match &object.kind {
        beryl_syntax::ast::ExprKind::Variable(name) => Some(name.clone()),
        beryl_syntax::ast::ExprKind::GenericInstantiation { base, .. } => {
            if let beryl_syntax::ast::ExprKind::Variable(name) = &base.kind {
                Some(name.clone())
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(name) = enum_check {
        if ctx.enum_types.contains(&name) {
            // It's an Enum Constructor!
            let ctor_name = format!("{}_{}", name, method_name);
            let function = ctx
                .module
                .get_function(&ctor_name)
                .ok_or_else(|| CodegenError::FunctionNotFound(ctor_name.clone()))?;

            let mut compiled_args = Vec::with_capacity(args.len());
            for arg in args {
                let arg_val = generate_expr(ctx, locals, arg)?;
                compiled_args.push(arg_val.value.into());
            }

            let call_site = ctx
                .builder
                .build_call(function, &compiled_args, "call_ctor")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            let val = call_site
                .try_as_basic_value()
                .left()
                .ok_or(CodegenError::LLVMBuildError(
                    "Constructor returned void".into(),
                ))?;

            return Ok(CodegenValue {
                value: val,
                ty: Type::Struct(name.clone()),
            });
        }
    }

    // 1. 生成对象表达式
    let object_val = generate_expr(ctx, locals, object)?;

    // 2. 获取 Struct 类型名称
    // 2. 特殊处理 Vec 方法调用
    if let Type::Vec(inner) = &object_val.ty {
        let inner_ty = inner.as_ref().clone();
        return crate::expr::vec::gen_vec_method_call(
            ctx,
            locals,
            object_val,
            method_name,
            args,
            &inner_ty,
        );
    }

    // 2. 获取 Struct 类型名称
    let struct_name = match &object_val.ty {
        Type::Struct(name) => name,
        _ => return Err(CodegenError::TypeMismatch),
    };

    // 3. 获取 this 指针
    // 由于 Beryl 中 Struct 总是通过指针传递（即使是按值语义），
    // 这里的 value 应该要么是 PointerValue（如果是左值指针），
    // 要么是 StructValue（如果是右值加载出的结构体）。
    let this_ptr = if object_val.value.is_pointer_value() {
        object_val.value.into_pointer_value()
    } else {
        // 如果是值（右值结构体），分配临时空间
        let struct_type = object_val.value.get_type();
        let alloca = ctx
            .builder
            .build_alloca(struct_type, "this_tmp")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        ctx.builder
            .build_store(alloca, object_val.value)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        alloca
    };

    // 运行时 Null 检查
    if let Some(panic_func) = ctx.panic_func {
        crate::runtime::gen_null_check(ctx.context, &ctx.builder, panic_func, this_ptr, line);
    }

    // 4. 构建 mangled name
    let mangled_name = format!("{}_{}", struct_name, method_name);

    // 5. 查找函数
    let function = ctx
        .module
        .get_function(&mangled_name)
        .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

    // 6. 生成参数列表
    let mut compiled_args = Vec::with_capacity(args.len() + 1);

    // Determines how to pass 'this'
    let first_param = function.get_first_param();
    if let Some(param) = first_param {
        let expected_type = param.get_type();

        let this_arg = if expected_type.is_pointer_type() {
            // Function expects pointer
            if object_val.value.is_pointer_value() {
                object_val.value
            } else {
                // Have value, need pointer -> Alloca
                let struct_type = object_val.value.get_type();
                let alloca = ctx
                    .builder
                    .build_alloca(struct_type, "this_tmp")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                ctx.builder
                    .build_store(alloca, object_val.value)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                // Null check if we just created it? It's non-null.
                // If object_val was pointer, we invoke check below (if enabled) but here we store value so it is valid logic.
                alloca.into()
            }
        } else {
            // Function expects value
            if object_val.value.is_pointer_value() {
                // Have pointer, need value -> Load
                let load = ctx
                    .builder
                    .build_load(
                        expected_type,
                        object_val.value.into_pointer_value(),
                        "this_load",
                    )
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                load
            } else {
                // Have value, expect value -> Direct
                object_val.value
            }
        };

        // Null Check logic (original) - only if we have pointer?
        // Reuse logic if pointer?
        if this_arg.is_pointer_value() {
            if let Some(panic_func) = ctx.panic_func {
                crate::runtime::gen_null_check(
                    ctx.context,
                    &ctx.builder,
                    panic_func,
                    this_arg.into_pointer_value(),
                    line,
                );
            }
        }

        compiled_args.push(this_arg.into());
    } else {
        // Should not happen for methods? Or static methods?
        // If no params, ignore 'this'.
    }

    // 添加其他参数
    for arg in args {
        let arg_val = generate_expr(ctx, locals, arg)?;
        compiled_args.push(arg_val.value.into());
    }

    // 7. 获取返回类型
    let return_type = ctx
        .function_signatures
        .get(&mangled_name)
        .cloned()
        .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

    // 8. 生成调用
    let call_site = ctx
        .builder
        .build_call(function, &compiled_args, "call_method")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 9. 处理返回值
    let val = call_site.try_as_basic_value().left();

    if let Some(v) = val {
        Ok(CodegenValue {
            value: v,
            ty: return_type,
        })
    } else {
        // Void 返回，生成 dummy 值
        let dummy = ctx.context.bool_type().const_int(0, false).into();
        Ok(CodegenValue {
            value: dummy,
            ty: Type::Void,
        })
    }
}
