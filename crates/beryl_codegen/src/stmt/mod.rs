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
    pub(crate) locals:
        &'a mut HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>)>,
    /// 循环上下文栈
    pub(crate) loop_stack: Vec<LoopContext<'ctx>>,
}

impl<'ctx, 'a> StmtGenerator<'ctx, 'a> {
    /// 创建语句生成器
    pub fn new(
        ctx: &'a CodegenContext<'ctx>,
        locals: &'a mut HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>)>,
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
        _declared_ty: Option<&Type>,
        value: &Expr,
    ) -> CodegenResult<()> {
        let expr_gen = ExprGenerator::new(self.ctx, self.locals);
        let val = expr_gen.generate(value)?;

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

        // 记录变量（保存指针和类型）
        self.locals
            .insert(name.to_string(), (alloca, val.get_type()));

        Ok(())
    }

    /// 生成赋值语句
    fn gen_assignment(&mut self, target: &Expr, value: &Expr) -> CodegenResult<()> {
        // 获取目标变量名
        let var_name = match &target.kind {
            beryl_syntax::ast::ExprKind::Variable(name) => name,
            _ => return Err(CodegenError::UnsupportedExpression),
        };

        // 查找变量
        let (ptr, _) = self
            .locals
            .get(var_name)
            .ok_or_else(|| CodegenError::UndefinedVariable(var_name.clone()))?;

        // 生成值
        let expr_gen = ExprGenerator::new(self.ctx, self.locals);
        let val = expr_gen.generate(value)?;

        // 存储
        self.ctx
            .builder
            .build_store(*ptr, val)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        Ok(())
    }

    /// 生成 return 语句
    fn gen_return(&mut self, value: Option<&Expr>) -> CodegenResult<()> {
        if let Some(expr) = value {
            let expr_gen = ExprGenerator::new(self.ctx, self.locals);
            let val = expr_gen.generate(expr)?;
            self.ctx
                .builder
                .build_return(Some(&val))
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
