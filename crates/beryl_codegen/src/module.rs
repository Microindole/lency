//! Module Code Generation
//!
//! 模块代码生成器，负责生成整个程序

use beryl_syntax::ast::{Decl, Program};

use crate::context::CodegenContext;
use crate::error::CodegenResult;
use crate::function::FunctionGenerator;

/// 模块代码生成器
pub struct ModuleGenerator<'ctx, 'a> {
    ctx: &'a CodegenContext<'ctx>,
}

impl<'ctx, 'a> ModuleGenerator<'ctx, 'a> {
    /// 创建模块生成器
    pub fn new(ctx: &'a CodegenContext<'ctx>) -> Self {
        Self { ctx }
    }

    /// 生成整个程序
    pub fn generate(&mut self, program: &Program) -> CodegenResult<()> {
        let func_gen = FunctionGenerator::new(self.ctx);

        // 第一遍：声明所有函数（支持前向引用）
        for decl in &program.decls {
            match decl {
                Decl::Function {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    func_gen.declare(name, params, return_type)?;
                }
                Decl::Class { .. } => {
                    // 类定义暂不支持，跳过
                    continue;
                }
                Decl::ExternFunction {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    func_gen.declare(name, params, return_type)?;
                }
                Decl::Struct { .. } => {
                    // TODO: Struct codegen (Phase 2)
                }
                Decl::Impl { .. } => {
                    // TODO: Impl codegen (Phase 2)
                }
            }
        }

        // 第二遍：生成函数体
        let mut func_gen = FunctionGenerator::new(self.ctx);
        for decl in &program.decls {
            match decl {
                Decl::Function { .. } => {
                    func_gen.generate(decl)?;
                }
                Decl::Class { .. } | Decl::ExternFunction { .. } => {
                    // 类定义暂不支持，跳过
                    // 外部函数没有体，跳过
                    continue;
                }
                Decl::Struct { .. } => {
                    // TODO: Struct codegen (Phase 2)
                }
                Decl::Impl { .. } => {
                    // TODO: Impl codegen (Phase 2)
                }
            }
        }

        Ok(())
    }
}
