use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::{generate_expr, CodegenValue};
use crate::types::ToLLVMType;
use lency_syntax::ast::{Expr, Type};
use std::collections::HashMap;

/// 生成 Ok(val) 构造器
pub fn gen_ok<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    inner: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. 生成内部值
    let val_wrapper = generate_expr(ctx, locals, inner)?;
    let ok_val = val_wrapper.value;
    let ok_ty = val_wrapper.ty;

    // 2. 构造 Result 类型: Result<T, Error>
    let result_ty = Type::Result {
        ok_type: Box::new(ok_ty.clone()),
        err_type: Box::new(Type::Struct("Error".to_string())),
    };

    // 3. 获取已注册的 Result struct type
    let mangled_name = lency_monomorph::mangling::mangle_type(&result_ty);
    let struct_type = *ctx
        .struct_types
        .get(&mangled_name)
        .ok_or_else(|| crate::error::CodegenError::UndefinedStructType(mangled_name.clone()))?;

    // 获取 Result pointer type
    let result_ptr_type = struct_type.ptr_type(inkwell::AddressSpace::default());

    // 4. Malloc
    let size = struct_type.size_of().ok_or(CodegenError::LLVMBuildError(
        "Failed to get size of Result type".to_string(),
    ))?;

    let malloc = ctx
        .module
        .get_function("malloc")
        .ok_or(CodegenError::LLVMBuildError("malloc not found".to_string()))?;

    let malloc_call = ctx
        .builder
        .build_call(malloc, &[size.into()], "malloc_result")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let raw_ptr = malloc_call
        .try_as_basic_value()
        .left()
        .ok_or(CodegenError::LLVMBuildError(
            "malloc returned void".to_string(),
        ))?
        .into_pointer_value();

    // 5. Cast and Store
    let result_ptr = ctx
        .builder
        .build_pointer_cast(raw_ptr, result_ptr_type, "result_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Store is_ok = true (1)
    let is_ok_ptr = ctx
        .builder
        .build_struct_gep(struct_type, result_ptr, 0, "is_ok_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    ctx.builder
        .build_store(is_ok_ptr, ctx.context.bool_type().const_int(1, false))
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Store ok_value (index 1)
    if !matches!(ok_ty, Type::Void) {
        let val_ptr = ctx
            .builder
            .build_struct_gep(struct_type, result_ptr, 1, "ok_val_ptr")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        ctx.builder
            .build_store(val_ptr, ok_val)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    }

    Ok(CodegenValue {
        value: result_ptr.into(),
        ty: result_ty,
    })
}

/// 生成 Err(msg) 构造器
pub fn gen_err<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    inner: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. 生成内部值 (Error object or fields)
    let val_wrapper = generate_expr(ctx, locals, inner)?;
    let err_val = val_wrapper.value;
    // Assume inner is Error struct pointer (Type::Struct("Error"))

    // 2. 构造 Result 类型: Result<void, Error>
    let result_ty = Type::Result {
        ok_type: Box::new(Type::Void),
        err_type: Box::new(Type::Struct("Error".to_string())),
    };

    // 3. 获取已注册的 Result struct type
    let mangled_name = lency_monomorph::mangling::mangle_type(&result_ty);
    let struct_type = *ctx
        .struct_types
        .get(&mangled_name)
        .ok_or_else(|| crate::error::CodegenError::UndefinedStructType(mangled_name.clone()))?;

    // 获取 Result pointer type
    let result_ptr_type = struct_type.ptr_type(inkwell::AddressSpace::default());

    // 4. Malloc
    let size = struct_type.size_of().ok_or(CodegenError::LLVMBuildError(
        "Failed to get size of Result type".to_string(),
    ))?;

    let malloc = ctx
        .module
        .get_function("malloc")
        .ok_or(CodegenError::LLVMBuildError("malloc not found".to_string()))?;

    let malloc_call = ctx
        .builder
        .build_call(malloc, &[size.into()], "malloc_result_err")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    let raw_ptr = malloc_call
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_pointer_value();

    // 5. Store
    let result_ptr = ctx
        .builder
        .build_pointer_cast(raw_ptr, result_ptr_type, "result_err_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Store is_ok = false (0)
    let is_ok_ptr = ctx
        .builder
        .build_struct_gep(struct_type, result_ptr, 0, "is_ok_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    ctx.builder
        .build_store(is_ok_ptr, ctx.context.bool_type().const_int(0, false))
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // Store err_value (index 1) -- because ok_val is void and skipped
    let err_ptr = ctx
        .builder
        .build_struct_gep(struct_type, result_ptr, 1, "err_val_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    ctx.builder
        .build_store(err_ptr, err_val)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: result_ptr.into(),
        ty: result_ty,
    })
}

/// 生成 ? 运算符逻辑
pub fn gen_try<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    inner: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    // 1. 生成 result 表达式
    let val_wrapper = generate_expr(ctx, locals, inner)?;
    let result_ptr = val_wrapper.value.into_pointer_value();

    // 解析 Result 类型以获取 ok_type
    let ok_type = match &val_wrapper.ty {
        Type::Result { ok_type, .. } => *ok_type.clone(),
        _ => return Err(CodegenError::TypeMismatch),
    };

    // 2. 获取已注册的 Result struct type
    let mangled_name = lency_monomorph::mangling::mangle_type(&val_wrapper.ty);
    let struct_type = *ctx
        .struct_types
        .get(&mangled_name)
        .ok_or_else(|| crate::error::CodegenError::UndefinedStructType(mangled_name.clone()))?;

    let is_ok_ptr = ctx
        .builder
        .build_struct_gep(struct_type, result_ptr, 0, "is_ok_check")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    let is_ok = ctx
        .builder
        .build_load(ctx.context.bool_type(), is_ok_ptr, "is_ok")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        .into_int_value();

    // 3. 创建 BasicBlocks
    // 获取当前函数
    let current_block = ctx
        .builder
        .get_insert_block()
        .ok_or(CodegenError::LLVMBuildError("No insert block".to_string()))?;
    let parent_func = current_block
        .get_parent()
        .ok_or(CodegenError::LLVMBuildError(
            "No parent function".to_string(),
        ))?;

    // 我们不需要显式的 then_block，因为 success case 直接跳到 cont_block 继续执行
    let else_block = ctx.context.append_basic_block(parent_func, "try_err");
    let cont_block = ctx.context.append_basic_block(parent_func, "try_cont"); // Continue execution

    // If ok -> cont (unwrap), else -> else_block (early return)
    ctx.builder
        .build_conditional_branch(is_ok, cont_block, else_block)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 4. Implement Else (Early Return)
    ctx.builder.position_at_end(else_block);

    // 获取 error 值
    let err_idx = if matches!(ok_type, Type::Void) { 1 } else { 2 };
    let err_val_ptr = ctx
        .builder
        .build_struct_gep(struct_type, result_ptr, err_idx, "err_val_ptr")
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    let err_val = ctx
        .builder
        .build_load(
            Type::Struct("Error".to_string()).to_llvm_type(ctx)?,
            err_val_ptr,
            "err_val",
        )
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    // 构造返回值并 return
    // 从 locals 中获取 return_type
    let (_, func_ret_type) = locals
        .get("__return_type")
        .ok_or(CodegenError::LLVMBuildError(
            "Missing return type in locals".to_string(),
        ))?;

    // 如果函数返回 Result<T, Error>，我们需要构造 Err(err_val)
    if let Type::Result {
        ok_type: ret_ok_type,
        ..
    } = func_ret_type
    {
        // 获取返回类型的 Result struct type
        let ret_mangled_name = lency_monomorph::mangling::mangle_type(func_ret_type);
        let ret_struct_type = *ctx.struct_types.get(&ret_mangled_name).ok_or_else(|| {
            crate::error::CodegenError::UndefinedStructType(ret_mangled_name.clone())
        })?;

        // Malloc
        let size = ret_struct_type
            .size_of()
            .ok_or(CodegenError::LLVMBuildError(
                "Failed to get size of Result type".to_string(),
            ))?;
        let malloc = ctx
            .module
            .get_function("malloc")
            .ok_or(CodegenError::LLVMBuildError("malloc not found".to_string()))?;
        let malloc_call = ctx
            .builder
            .build_call(malloc, &[size.into()], "malloc_try_ret")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        let raw_ptr = malloc_call
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Cast
        let ret_ptr_type = func_ret_type.to_llvm_type(ctx)?.into_pointer_type();
        let ret_ptr = ctx
            .builder
            .build_pointer_cast(raw_ptr, ret_ptr_type, "try_ret_ptr")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // Store is_ok = 0
        let is_ok_ret_ptr = ctx
            .builder
            .build_struct_gep(ret_struct_type, ret_ptr, 0, "is_ok_ret")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        ctx.builder
            .build_store(is_ok_ret_ptr, ctx.context.bool_type().const_int(0, false))
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // Store err (last field)
        let err_ret_idx = if matches!(**ret_ok_type, Type::Void) {
            1
        } else {
            2
        };
        let err_ret_ptr = ctx
            .builder
            .build_struct_gep(ret_struct_type, ret_ptr, err_ret_idx, "err_ret_ptr")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        ctx.builder
            .build_store(err_ret_ptr, err_val)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // Return
        ctx.builder
            .build_return(Some(&ret_ptr))
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
    } else {
        return Err(CodegenError::TypeMismatch); // ? used in non-Result function
    }

    // 5. Implement Then (Unwrap)
    ctx.builder.position_at_end(cont_block);

    // 如果 ok 是 void，返回 void (dummy?)
    if matches!(ok_type, Type::Void) {
        // How to represent void value? Expr generate returns CodegenValue with Type::Void?
        // CodegenValue has BasicValueEnum.
        // We can use a dummy value (e.g. const struct{}) or just handle it as special case to not rely on value.
        // But generate_expr needs to return something.
        // Lency void is typically not used as value.
        // Let's return const int 0 as dummy.
        // Or if Lency has Unit type.
        // For now, let's just return a dummy i1 0.
        Ok(CodegenValue {
            value: ctx.context.bool_type().const_int(0, false).into(),
            ty: Type::Void,
        })
    } else {
        // Load ok val
        let ok_idx = 1;
        let ok_val_ptr = ctx
            .builder
            .build_struct_gep(struct_type, result_ptr, ok_idx, "ok_val_unwrap")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        let ok_val = ctx
            .builder
            .build_load(ok_type.to_llvm_type(ctx)?, ok_val_ptr, "ok_val")
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        Ok(CodegenValue {
            value: ok_val,
            ty: ok_type,
        })
    }
}
