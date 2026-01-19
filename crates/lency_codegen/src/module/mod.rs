//! Module Code Generation
//!
//! 模块代码生成器，负责生成整个程序
//! 逻辑分布：
//! - types.rs: 负责类型注册（Struct/Enum/Result）和 Struct/Enum Body 生成
//! - functions.rs: 负责函数声明、Globals 和函数体生成

use lency_syntax::ast::{Program, Type};

use crate::context::CodegenContext;
use crate::error::CodegenResult;

mod functions;
mod types;

/// 模块代码生成器
pub struct ModuleGenerator<'ctx, 'a> {
    pub(crate) ctx: &'a mut CodegenContext<'ctx>,
}

impl<'ctx, 'a> ModuleGenerator<'ctx, 'a> {
    /// 创建模块生成器
    pub fn new(ctx: &'a mut CodegenContext<'ctx>) -> Self {
        Self { ctx }
    }

    /// 生成整个程序
    pub fn generate(&mut self, program: &Program) -> CodegenResult<()> {
        // 1. 注入运行时函数 (__lency_panic, printf, exit, malloc)
        self.inject_runtime()?;

        // 2. 第零遍：注册类型 (opaque)
        self.register_opaque_types(program)?;

        // 3. 第0.5遍：定义 Struct Body
        self.define_struct_bodies(program)?;

        // 4. Sprint 15: 预注册 intrinsic 函数使用的 Result 类型 (read_file 依赖 Error struct)
        // 此时 Error struct 已经注册完成
        let read_file_result = Type::Result {
            ok_type: Box::new(Type::String),
            err_type: Box::new(Type::Struct("Error".to_string())),
        };
        self.register_result_type(&read_file_result)?;
        // write_file 返回 Result<void, Error> 会被 register_result_type 自动注册

        // 5. 第0.6遍：定义 Enum Body (必须在 Struct Body 之后，以便计算大小)
        self.define_enum_bodies(program)?;

        // 6. 第一遍：声明所有函数（支持前向引用）和 Globals
        self.declare_functions(program)?;

        // 7. 第二遍：生成函数体
        self.generate_function_bodies(program)?;

        // 8. Generate main wrapper (entry point)
        self.generate_main_wrapper()?;

        Ok(())
    }
}
