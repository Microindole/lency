//! Function Code Generation
//!
//! 函数代码生成器

use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
use lency_syntax::ast::{Decl, Type};
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::stmt::StmtGenerator;
use crate::types::ToLLVMType;

/// 函数代码生成器
pub struct FunctionGenerator<'ctx, 'a> {
    ctx: &'a CodegenContext<'ctx>,
}

impl<'ctx, 'a> FunctionGenerator<'ctx, 'a> {
    /// 创建函数生成器
    pub fn new(ctx: &'a CodegenContext<'ctx>) -> Self {
        Self { ctx }
    }

    /// 根据类型名称字符串获取 Type enum
    /// 用于处理 impl 块中的 this 参数类型
    fn type_name_to_type(name: &str) -> Type {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "string" => Type::String,
            "bool" => Type::Bool,
            _ => Type::Struct(name.to_string()),
        }
    }

    /// 检查类型名称是否为 primitive 类型
    fn is_primitive_type_name(name: &str) -> bool {
        matches!(name, "int" | "float" | "string" | "bool")
    }

    /// 生成函数代码
    pub fn generate(
        &mut self,
        decl: &Decl,
        llvm_name_override: Option<&str>,
        struct_name_context: Option<&str>,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        let Decl::Function {
            name,
            params,
            return_type,
            body,
            ..
        } = decl
        else {
            return Err(CodegenError::NotAFunction);
        };

        // 获取函数（应该已经在声明阶段创建）
        let llvm_name = llvm_name_override.unwrap_or(name);
        let function = self
            .ctx
            .module
            .get_function(llvm_name)
            .ok_or_else(|| CodegenError::FunctionNotFound(llvm_name.to_string()))?;

        // 如果函数体为空（extern 函数），直接返回
        if body.is_empty() {
            return Ok(function);
        }

        // 创建入口基本块
        let entry = self.ctx.context.append_basic_block(function, "entry");
        self.ctx.builder.position_at_end(entry);

        // 局部变量表
        let mut locals: HashMap<String, (inkwell::values::PointerValue<'ctx>, Type)> =
            HashMap::new();

        // 注入当前函数的返回类型，供 ExprGenerator (如 gen_try) 使用
        // 使用特殊的 key "__return_type"
        // PointerValue 使用 null，因为这里只关注 Type
        // 注意：PointerValue 必须有类型。这里随便用 i8* null。
        let null_ptr = self
            .ctx
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default())
            .const_null();
        locals.insert("__return_type".to_string(), (null_ptr, return_type.clone()));

        // 检测是否是方法（通过 struct_name_context 或 mangled name 格式）
        // 如果提供了 struct_name_context，则明确是方法
        let struct_name_opt = if let Some(s) = struct_name_context {
            Some(s.to_string())
        } else if llvm_name_override.is_some()
            && llvm_name.contains('_')
            && !llvm_name.starts_with("__")
            && llvm_name != name
        {
            // Fallback: 从函数名推断 struct 名称：StructName_methodName
            // 注意：这种推断在 StructName 包含下划线时会失效 (例如 my_io__Module)
            // 所以首选 struct_name_context
            llvm_name.split('_').next().map(|s| s.to_string())
        } else {
            None
        };

        let mut param_offset = 0;

        // 如果是方法，先处理 this 参数
        if let Some(type_name_str) = struct_name_opt {
            let this_value = function.get_nth_param(0).ok_or_else(|| {
                CodegenError::LLVMBuildError("missing this parameter".to_string())
            })?;

            // 将类型名称转为 Type enum
            let this_type = Self::type_name_to_type(&type_name_str);
            let is_primitive = Self::is_primitive_type_name(&type_name_str);

            if is_primitive {
                // Primitive 类型：this 是值类型，直接分配并存储
                let this_llvm_type = this_type.to_llvm_type(self.ctx)?;
                let this_alloca = self
                    .ctx
                    .builder
                    .build_alloca(this_llvm_type, "this")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                self.ctx
                    .builder
                    .build_store(this_alloca, this_value)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                locals.insert("this".to_string(), (this_alloca, this_type));
            } else {
                // Struct 类型：this 是指针类型
                let this_ptr_llvm_type = this_type
                    .to_llvm_type(self.ctx)?
                    .ptr_type(inkwell::AddressSpace::default());
                let this_alloca = self
                    .ctx
                    .builder
                    .build_alloca(this_ptr_llvm_type, "this")
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                self.ctx
                    .builder
                    .build_store(this_alloca, this_value)
                    .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

                locals.insert("this".to_string(), (this_alloca, this_type));
            }

            param_offset = 1; // 后续参数从索引 1 开始
        }

        // 为其他参数分配空间并存储
        for (i, param) in params.iter().enumerate() {
            let param_value = function
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| CodegenError::LLVMBuildError(format!("missing parameter {}", i)))?;

            let param_type = param.ty.to_llvm_type(self.ctx)?;
            let alloca = self
                .ctx
                .builder
                .build_alloca(param_type, &param.name)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            self.ctx
                .builder
                .build_store(alloca, param_value)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

            locals.insert(param.name.clone(), (alloca, param.ty.clone()));
        }

        // 生成函数体
        let mut stmt_gen = StmtGenerator::new(self.ctx, &mut locals, return_type);
        stmt_gen.generate_block(body)?;

        // 如果是 void 函数且没有显式 return，添加隐式 return
        if *return_type == Type::Void {
            let last_bb = self.ctx.builder.get_insert_block();
            if let Some(bb) = last_bb {
                if bb.get_terminator().is_none() {
                    self.ctx
                        .builder
                        .build_return(None)
                        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
                }
            }
        }

        Ok(function)
    }

    /// 声明函数（不生成函数体）
    pub fn declare(
        &self,
        name: &str,
        params: &[lency_syntax::ast::Param],
        return_type: &Type,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        // 构建参数类型列表
        let mut param_types = Vec::new();
        for param in params {
            // 对于方法的 this 参数，使用指针类型（结构体通过指针传递）
            let param_ty = if param.name == "this" {
                if let Type::Struct(_) = &param.ty {
                    // this 参数使用指针类型
                    param
                        .ty
                        .to_llvm_type(self.ctx)?
                        .ptr_type(inkwell::AddressSpace::default())
                        .into()
                } else {
                    param.ty.to_llvm_type(self.ctx)?
                }
            } else {
                param.ty.to_llvm_type(self.ctx)?
            };
            param_types.push(param_ty.into());
        }

        // 构建函数类型
        let fn_type = if *return_type == Type::Void {
            self.ctx.context.void_type().fn_type(&param_types, false)
        } else {
            let ret_ty = return_type.to_llvm_type(self.ctx)?;
            ret_ty.fn_type(&param_types, false)
        };

        // 添加函数到模块
        let function = self.ctx.module.add_function(name, fn_type, None);

        // 设置参数名称
        for (i, param) in params.iter().enumerate() {
            if let Some(param_value) = function.get_nth_param(i as u32) {
                param_value.set_name(&param.name);
            }
        }

        Ok(function)
    }
}
