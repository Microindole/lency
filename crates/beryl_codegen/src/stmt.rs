//! Statement Code Generation
//!
//! 语句代码生成器，将 Beryl 语句转换为 LLVM IR

use beryl_syntax::ast::Expr;
use beryl_syntax::ast::{Stmt, Type};
use inkwell::values::PointerValue;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::ExprGenerator;

/// 语句代码生成器
pub struct StmtGenerator<'ctx, 'a> {
    ctx: &'a CodegenContext<'ctx>,
    /// 局部变量表 (变量名 -> (指针, LLVM类型))
    locals: &'a mut HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>)>,
}

impl<'ctx, 'a> StmtGenerator<'ctx, 'a> {
    /// 创建语句生成器
    pub fn new(
        ctx: &'a CodegenContext<'ctx>,
        locals: &'a mut HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>)>,
    ) -> Self {
        Self { ctx, locals }
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
            } => self.gen_if(condition, then_block, else_block.as_deref()),
            Stmt::While {
                condition, body, ..
            } => self.gen_while(condition, body),
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

    /// 生成 if 语句
    fn gen_if(
        &mut self,
        condition: &Expr,
        then_block: &[Stmt],
        else_block: Option<&[Stmt]>,
    ) -> CodegenResult<()> {
        // 生成条件
        let expr_gen = ExprGenerator::new(self.ctx, self.locals);
        let cond_val = expr_gen.generate(condition)?;
        let cond_int = cond_val.into_int_value();

        // 获取当前函数
        let function = self
            .ctx
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::LLVMBuildError("not in a function".to_string()))?;

        // 创建基本块
        let then_bb = self.ctx.context.append_basic_block(function, "then");
        let else_bb = self.ctx.context.append_basic_block(function, "else");
        let merge_bb = self.ctx.context.append_basic_block(function, "ifcont");

        // 条件跳转
        self.ctx
            .builder
            .build_conditional_branch(cond_int, then_bb, else_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // Then 分支
        self.ctx.builder.position_at_end(then_bb);
        self.generate_block(then_block)?;
        // 如果 then 分支没有返回，跳转到 merge
        if !self.block_ends_with_terminator(then_bb) {
            self.ctx
                .builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }

        // Else 分支
        self.ctx.builder.position_at_end(else_bb);
        if let Some(else_stmts) = else_block {
            self.generate_block(else_stmts)?;
        }
        // 如果 else 分支没有返回，跳转到 merge
        if !self.block_ends_with_terminator(else_bb) {
            self.ctx
                .builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }

        // 合并点
        self.ctx.builder.position_at_end(merge_bb);

        Ok(())
    }

    /// 生成 while 循环
    fn gen_while(&mut self, condition: &Expr, body: &[Stmt]) -> CodegenResult<()> {
        // 获取当前函数
        let function = self
            .ctx
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::LLVMBuildError("not in a function".to_string()))?;

        // 创建基本块
        let cond_bb = self.ctx.context.append_basic_block(function, "while.cond");
        let body_bb = self.ctx.context.append_basic_block(function, "while.body");
        let after_bb = self.ctx.context.append_basic_block(function, "while.end");

        // 跳转到条件块
        self.ctx
            .builder
            .build_unconditional_branch(cond_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // 条件块
        self.ctx.builder.position_at_end(cond_bb);
        let expr_gen = ExprGenerator::new(self.ctx, self.locals);
        let cond_val = expr_gen.generate(condition)?;
        let cond_int = cond_val.into_int_value();
        self.ctx
            .builder
            .build_conditional_branch(cond_int, body_bb, after_bb)
            .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;

        // 循环体
        self.ctx.builder.position_at_end(body_bb);
        self.generate_block(body)?;
        if !self.block_ends_with_terminator(body_bb) {
            self.ctx
                .builder
                .build_unconditional_branch(cond_bb)
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
        }

        // 循环后
        self.ctx.builder.position_at_end(after_bb);

        Ok(())
    }

    /// 检查基本块是否以终止指令结束
    fn block_ends_with_terminator(&self, bb: inkwell::basic_block::BasicBlock) -> bool {
        bb.get_terminator().is_some()
    }
}
