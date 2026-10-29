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
    pub function_signatures: std::collections::HashMap<String, lency_syntax::ast::Type>,
    /// 参数类型用于在调用边界保持可空值及数值表示一致。
    pub function_param_types: std::collections::HashMap<String, Vec<lency_syntax::ast::Type>>,
    /// 非空读取细化绑定到具体存储，不能被同名局部变量继承。
    pub(crate) read_refinements:
        std::cell::RefCell<std::collections::HashMap<String, inkwell::values::PointerValue<'ctx>>>,
    /// Struct Field Types (ordered) - used to recover Lency Type from field access
    /// Struct Field Types (ordered) - used to recover Lency Type from field access
    pub struct_field_types: std::collections::HashMap<String, Vec<lency_syntax::ast::Type>>,

    // Enum Metadata (Sprint 10)
    /// Set of Enum Names
    pub enum_types: std::collections::HashSet<String>,
    /// Map EnumName -> Ordered Variants [(Name, Types)]
    pub enum_variants:
        std::collections::HashMap<String, Vec<(String, Vec<lency_syntax::ast::Type>)>>,

    /// Global Variable Types
    pub global_var_types: std::collections::HashMap<String, lency_syntax::ast::Type>,

    /// Runtime Panic Function
    pub panic_func: Option<inkwell::values::FunctionValue<'ctx>>,
    pub source_map: lency_syntax::source_map::SourceMap,
    pub(crate) trace_strings:
        std::cell::RefCell<std::collections::HashMap<String, inkwell::values::PointerValue<'ctx>>>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// 创建新的代码生成上下文
    ///
    /// # Arguments
    /// * `context` - LLVM Context 引用
    /// * `module_name` - 模块名称
    /// * `source` - 源代码 (可选, 用于调试信息)
    pub fn new(context: &'ctx Context, module_name: &str, source: Option<&str>) -> Self {
        Self {
            context,
            module: context.create_module(module_name),
            builder: context.create_builder(),
            struct_types: std::collections::HashMap::new(),
            struct_fields: std::collections::HashMap::new(),
            function_signatures: std::collections::HashMap::new(),
            function_param_types: std::collections::HashMap::new(),
            read_refinements: std::cell::RefCell::new(std::collections::HashMap::new()),
            struct_field_types: std::collections::HashMap::new(),
            enum_types: std::collections::HashSet::new(),
            enum_variants: std::collections::HashMap::new(),
            global_var_types: std::collections::HashMap::new(),
            panic_func: None,
            source_map: lency_syntax::source_map::SourceMap::main(
                module_name,
                source.unwrap_or(""),
            ),
            trace_strings: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// 获取字节偏移对应的行号 (1-based)
    pub fn get_line(&self, byte_offset: usize) -> u32 {
        self.source_map.location(byte_offset).line
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
