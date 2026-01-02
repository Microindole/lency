//! LLVM Context Management
//!
//! 封装 LLVM 的 Context、Module、Builder，简化代码生成过程

use crate::error::CodegenError;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

/// LLVM 代码生成上下文
///
/// 持有 LLVM 的核心组件，避免在各个生成器之间传递多个参数
pub struct CodegenContext<'ctx> {
    /// LLVM Context
    pub context: &'ctx Context,
    /// LLVM Module
    pub module: Module<'ctx>,
    /// LLVM IR Builder
    pub builder: Builder<'ctx>,
    /// Named Struct Types
    pub struct_types: std::collections::HashMap<String, inkwell::types::StructType<'ctx>>,
    /// Struct Field Names (ordered) - used to map field name to index
    pub struct_fields: std::collections::HashMap<String, Vec<String>>,
    /// Function Return Types - used for type propagation
    pub function_signatures: std::collections::HashMap<String, beryl_syntax::ast::Type>,
    /// Struct Field Types (ordered) - used to recover Beryl Type from field access
    pub struct_field_types: std::collections::HashMap<String, Vec<beryl_syntax::ast::Type>>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// 创建新的代码生成上下文
    ///
    /// # Arguments
    /// * `context` - LLVM Context 引用
    /// * `module_name` - 模块名称
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self {
            context,
            module: context.create_module(module_name),
            builder: context.create_builder(),
            struct_types: std::collections::HashMap::new(),
            struct_fields: std::collections::HashMap::new(),
            function_signatures: std::collections::HashMap::new(),
            struct_field_types: std::collections::HashMap::new(),
        }
    }

    /// 获取模块的 LLVM IR 字符串表示
    pub fn print_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// 验证模块的正确性
    pub fn verify(&self) -> Result<(), CodegenError> {
        self.module
            .verify()
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))
    }
}
