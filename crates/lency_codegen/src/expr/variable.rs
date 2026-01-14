//! Variable Reference Code Generation
//!
//! 变量引用代码生成

use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::CodegenValue;
use crate::types::ToLLVMType;
use inkwell::types::BasicType;

/// 生成变量引用代码
pub(super) fn gen_variable<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    name: &str,
) -> CodegenResult<CodegenValue<'ctx>> {
    let (ptr, ty) = match locals.get(name) {
        Some(val) => val,
        None => {
            // 尝试隐式 this 访问
            if let Some((this_ptr, this_ty)) = locals.get("this") {
                if let lency_syntax::ast::Type::Struct(struct_name) = this_ty {
                    // 1. 加载 this 指针 (Struct*)
                    // locals 中存储的是 alloca 的地址，所以需要先 load 出来
                    let this_llvm_ty = this_ty.to_llvm_type(ctx)?;
                    let this_val = ctx
                        .builder
                        .build_load(this_llvm_ty, *this_ptr, "this")
                        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
                        .into_pointer_value();

                    // 2. 查找字段信息
                    if let Some(field_names) = ctx.struct_fields.get(struct_name) {
                        if let Some(index) = field_names.iter().position(|n| n == name) {
                            let struct_llvm_type = ctx.struct_types.get(struct_name).unwrap();

                            // 3. GEP 获取字段地址
                            let field_ptr = ctx
                                .builder
                                .build_struct_gep(
                                    *struct_llvm_type,
                                    this_val,
                                    index as u32,
                                    &format!("field_{}_ptr", name),
                                )
                                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                            // 4. 获取字段类型
                            let field_types = ctx.struct_field_types.get(struct_name).unwrap();
                            let field_ty = &field_types[index];

                            // 5. Load 字段值
                            let field_llvm_ty = field_ty.to_llvm_type(ctx)?;
                            let val = ctx
                                .builder
                                .build_load(
                                    field_llvm_ty,
                                    field_ptr,
                                    &format!("field_{}_val", name),
                                )
                                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                            return Ok(CodegenValue {
                                value: val,
                                ty: field_ty.clone(),
                            });
                        }
                    }
                }
            }

            // 尝试全局变量
            if let Some(global) = ctx.module.get_global(name) {
                if let Some(ty) = ctx.global_var_types.get(name) {
                    let ptr = global.as_pointer_value();
                    let llvm_type = ty.to_llvm_type(ctx)?;
                    let val = ctx
                        .builder
                        .build_load(llvm_type, ptr, name)
                        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                    return Ok(CodegenValue {
                        value: val,
                        ty: ty.clone(),
                    });
                }
            }

            return Err(CodegenError::UndefinedVariable(name.to_string()));
        }
    };

    // 使用保存的类型信息进行加载
    // 对于 this 变量（方法中），它是指向结构体的指针，需要使用指针类型加载
    let llvm_type = if name == "this" {
        if let lency_syntax::ast::Type::Struct(_) = ty {
            ty.to_llvm_type(ctx)?
                .ptr_type(inkwell::AddressSpace::default())
                .into()
        } else {
            ty.to_llvm_type(ctx)?
        }
    } else {
        ty.to_llvm_type(ctx)?
    };
    let val = ctx
        .builder
        .build_load(llvm_type, *ptr, name)
        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

    Ok(CodegenValue {
        value: val,
        ty: ty.clone(),
    })
}
