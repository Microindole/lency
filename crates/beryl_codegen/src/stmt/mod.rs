//! Statement Code Generation
//!
//! 语句代码生成器，将 Beryl 语句转换为 LLVM IR

mod control_flow;

use beryl_syntax::ast::Expr;
use beryl_syntax::ast::{Stmt, Type};
use inkwell::values::PointerValue;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::ExprGenerator;

use inkwell::basic_block::BasicBlock;

/// 循环上下文，用于 break/continue 跳转
pub struct LoopContext<'ctx> {
    pub continue_block: BasicBlock<'ctx>,
    pub break_block: BasicBlock<'ctx>,
}

/// 语句代码生成器
pub struct StmtGenerator<'ctx, 'a> {
    pub(crate) ctx: &'a CodegenContext<'ctx>,
    /// 局部变量表
    pub(crate) locals: &'a mut HashMap<String, (PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    /// 循环上下文栈
    pub(crate) loop_stack: Vec<LoopContext<'ctx>>,
}

impl<'ctx, 'a> StmtGenerator<'ctx, 'a> {
    /// 创建语句生成器
    pub fn new(
        ctx: &'a CodegenContext<'ctx>,
        locals: &'a mut HashMap<String, (PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    ) -> Self {
        Self {
            ctx,
            locals,
            loop_stack: Vec::new(),
        }
    }

    /// 生成语句代码
    pub fn generate(&mut self, stmt: &Stmt) -> CodegenResult<()> {
        match stmt {
            Stmt::VarDecl {
                name, ty, value, ..
            } => self.gen_var_decl(name, ty.as_ref(), value),
            Stmt::Assignment { target, value, .. } => self.gen_assignment(target, value),
            Stmt::Return { value, .. } => self.gen_return(value.as_ref()),
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => control_flow::gen_if(self, condition, then_block, else_block.as_deref()),
            Stmt::While {
                condition, body, ..
            } => control_flow::gen_while(self, condition, body),
            Stmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => control_flow::gen_for(
                self,
                init.as_deref(),
                condition.as_ref(),
                update.as_deref(),
                body,
            ),
            Stmt::Break { .. } => control_flow::gen_break(self),
            Stmt::Continue { .. } => control_flow::gen_continue(self),
            Stmt::Expression(expr) => {
                let expr_gen = ExprGenerator::new(self.ctx, self.locals);
                expr_gen.generate(expr)?;
                Ok(())
            }
            Stmt::ForIn {
                iterator,
                iterable,
                body,
                ..
            } => control_flow::gen_for_in(self, iterator, iterable, body),
            Stmt::Block(stmts) => self.generate_block(stmts),
        }
    }

    /// 生成代码块
    pub fn generate_block(&mut self, stmts: &[Stmt]) -> CodegenResult<()> {
        for stmt in stmts {
            self.generate(stmt)?;
        }
        Ok(())
    }

    /// 生成变量声明
    fn gen_var_decl(
        &mut self,
        name: &str,
        declared_ty: Option<&Type>,
        value: &Expr,
    ) -> CodegenResult<()> {
        let expr_gen = ExprGenerator::new(self.ctx, self.locals);
        let val_wrapper = expr_gen.generate(value)?;
        let val = val_wrapper.value;

        // 若有显式类型声明，优先使用（这里假定类型检查已通过）
        // 若无，使用推导类型
        let var_ty = if let Some(ty) = declared_ty {
            ty.clone()
        } else {
            val_wrapper.ty.clone()
        };

        // 分配栈空间
        let alloca = self
            .ctx
            .builder
            .build_alloca(val.get_type(), name)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // 存储初始值
        self.ctx
            .builder
            .build_store(alloca, val)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // 记录变量（保存指针和 Beryl 类型）
        self.locals.insert(name.to_string(), (alloca, var_ty));

        Ok(())
    }

    /// 生成赋值语句
    fn gen_assignment(&mut self, target: &Expr, value: &Expr) -> CodegenResult<()> {
        // 生成目标地址（LValue）
        let expr_gen = ExprGenerator::new(self.ctx, self.locals);
        let (ptr, _ty) = expr_gen.generate_lvalue_addr(target)?;

        // 生成值
        let val_wrapper = expr_gen.generate(value)?;
        let val = val_wrapper.value;

        // 存储
        self.ctx
            .builder
            .build_store(ptr, val)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        Ok(())
    }

    /// 生成 return 语句
    fn gen_return(&mut self, value: Option<&Expr>) -> CodegenResult<()> {
        if let Some(expr) = value {
            let expr_gen = ExprGenerator::new(self.ctx, self.locals);
            let val_wrapper = expr_gen.generate(expr)?;
            self.ctx
                .builder
                .build_return(Some(&val_wrapper.value))
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        } else {
            self.ctx
                .builder
                .build_return(None)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }
        Ok(())
    }

    /// 检查基本块是否以终止指令结束
    pub(crate) fn block_ends_with_terminator(&self, bb: inkwell::basic_block::BasicBlock) -> bool {
        bb.get_terminator().is_some()
    }
}
