//! Closure Code Generation
//!
//! 闭包代码生成 - 将闭包提升为匿名顶层函数

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::CodegenValue;
use crate::types::ToLLVMType;
use beryl_syntax::ast::{Expr, Param, Type};
use inkwell::AddressSpace;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 全局闭包计数器，用于生成唯一函数名
static CLOSURE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 生成闭包表达式
/// 策略：将闭包提升为匿名顶层函数，返回函数指针
pub fn gen_closure<'ctx>(
    ctx: &CodegenContext<'ctx>,
    _locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)>,
    params: &[Param],
    body: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. 生成唯一函数名
    let closure_id = CLOSURE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let fn_name = format!("__closure_{}", closure_id);

    // 2. 构建参数类型
    let param_types: Result<Vec<_>, _> = params
        .iter()
        .map(|p| p.ty.to_llvm_type(ctx).map(|ty| ty.into()))
        .collect();
    let param_types = param_types?;

    // 3. 推导返回类型 (从 body 表达式推导)
    //    注意：此时 body 应该已经被类型检查过
    //    这里我们需要从 body 生成代码来获取返回值类型
    //    简化处理：先创建函数，再生成函数体

    // 4. 创建 LLVM 函数 (先假设返回 i64，后续会被 phi 节点覆盖)
    //    更好的方式是第一遍推导返回类型，但这里简化为 void 或 int
    let ret_type = ctx.context.i64_type(); // 简化：假设闭包返回 int
    let fn_type = ret_type.fn_type(&param_types, false);
    let function = ctx.module.add_function(&fn_name, fn_type, None);

    // 5. 创建函数入口块
    let entry = ctx.context.append_basic_block(function, "entry");

    // 保存当前 builder 位置
    let current_block = ctx.builder.get_insert_block();

    ctx.builder.position_at_end(entry);

    // 6. 设置局部变量 (参数)
    let mut closure_locals: HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)> =
        HashMap::new();

    // 注入返回类型
    let null_ptr = ctx
        .context
        .i8_type()
        .ptr_type(AddressSpace::default())
        .const_null();
    closure_locals.insert("__return_type".to_string(), (null_ptr, Type::Int));

    for (i, param) in params.iter().enumerate() {
        let param_value = function
            .get_nth_param(i as u32)
            .ok_or_else(|| CodegenError::LLVMBuildError(format!("missing param {}", i)))?;
        param_value.set_name(&param.name);

        let param_type = param.ty.to_llvm_type(ctx)?;
        let alloca = ctx
            .builder
            .build_alloca(param_type, &param.name)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        ctx.builder
            .build_store(alloca, param_value)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        closure_locals.insert(param.name.clone(), (alloca, param.ty.clone()));
    }

    // 7. 生成闭包体 (作为单个表达式)
    let body_value = crate::expr::generate_expr(ctx, &closure_locals, body)?;

    // 8. 生成 return
    ctx.builder
        .build_return(Some(&body_value.value))
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 9. 恢复 builder 位置
    if let Some(block) = current_block {
        ctx.builder.position_at_end(block);
    }

    // 10. 返回函数指针
    let fn_ptr = function.as_global_value().as_pointer_value();

    Ok(CodegenValue {
        value: fn_ptr.into(),
        ty: Type::Function {
            param_types: params.iter().map(|p| p.ty.clone()).collect(),
            return_type: Box::new(body_value.ty),
        },
    })
}
