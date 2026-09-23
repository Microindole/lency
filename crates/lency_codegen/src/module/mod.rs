//! Module Code Generation
//!
//! 模块代码生成器，负责生成整个程序
//! 逻辑分布：
//! - types.rs: 负责类型注册和 Struct/Enum Body 生成
//! - functions.rs: 负责函数声明、Globals 和函数体生成

use lency_syntax::ast::Program;

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

        // 4. 定义 Enum Body (必须在 Struct Body 之后，以便计算大小)
        self.define_enum_bodies(program)?;

        // 5. 第一遍：声明所有函数（支持前向引用）和 Globals
        self.declare_functions(program)?;

        // 6. 第二遍：生成函数体
        self.generate_function_bodies(program)?;

        // 7. Generate main wrapper (entry point)
        self.generate_main_wrapper()?;

        Ok(())
    }
}
