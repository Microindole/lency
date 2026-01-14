//! Method Call Code Generation
//!
//! 处理方法调用：object.method(args)

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use inkwell::values::PointerValue;
use lency_syntax::ast::{Expr, Type};
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
        lency_syntax::ast::ExprKind::Variable(name) => Some(name.clone()),
        lency_syntax::ast::ExprKind::GenericInstantiation { base, .. } => {
            if let lency_syntax::ast::ExprKind::Variable(name) = &base.kind {
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

    // 1. 计算对象表达式
    let object_val = generate_expr(ctx, locals, object)?;

    // 2. 根据对象类型分发
    // Clone type to avoid borrowing object_val, so we can move it
    let object_type = object_val.ty.clone();

    match object_type {
        Type::Vec(inner) => crate::expr::vec::gen_vec_method_call(
            ctx,
            locals,
            object_val,
            method_name,
            args,
            &inner,
        ),
        Type::Struct(name) => {
            // 获取 this 指针
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

            let struct_name = name.clone();

            // 运行时 Null 检查
            if let Some(panic_func) = ctx.panic_func {
                crate::runtime::gen_null_check(
                    ctx.context,
                    &ctx.builder,
                    panic_func,
                    this_ptr,
                    line,
                );
            }

            // 构建 mangled name
            let mangled_name = format!("{}_{}", struct_name, method_name);

            // 查找函数
            let function = ctx
                .module
                .get_function(&mangled_name)
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成参数列表
            let mut compiled_args = Vec::with_capacity(args.len() + 1);

            // 将 this_ptr 作为第一个参数
            compiled_args.push(this_ptr.into());

            // 添加其他参数
            for arg in args {
                let arg_val = generate_expr(ctx, locals, arg)?;
                compiled_args.push(arg_val.value.into());
            }

            // 获取返回类型
            let return_type = ctx
                .function_signatures
                .get(&mangled_name)
                .cloned()
                .ok_or_else(|| CodegenError::FunctionNotFound(mangled_name.clone()))?;

            // 生成调用
            let call_site = ctx
                .builder
                .build_call(function, &compiled_args, "call_method")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            // 处理返回值
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
        _ => Err(CodegenError::TypeMismatch),
    }
}

// 移除 gen_method_call_fallback，不再需要
#[allow(dead_code)]
fn _unused() {}
