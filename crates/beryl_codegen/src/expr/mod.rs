//! Expression Code Generation
//!
//! 表达式代码生成的主入口，将职责分散到各个子模块

mod array;
mod binary;
mod call;
mod intrinsic;
mod literal;
mod match_expr;
mod string_ops;
mod unary;
mod variable;

use beryl_syntax::ast::{Expr, ExprKind};
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// 表达式代码生成器（保持向后兼容的公共API）
pub struct ExprGenerator<'ctx, 'a> {
    ctx: &'a CodegenContext<'ctx>,
    /// 局部变量表 (变量名 -> (指针, LLVM类型))
    locals: &'a HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
}

impl<'ctx, 'a> ExprGenerator<'ctx, 'a> {
    /// 创建表达式生成器
    pub fn new(
        ctx: &'a CodegenContext<'ctx>,
        locals: &'a HashMap<
            String,
            (
                inkwell::values::PointerValue<'ctx>,
                inkwell::types::BasicTypeEnum<'ctx>,
            ),
        >,
    ) -> Self {
        Self { ctx, locals }
    }

    /// 生成表达式代码（主分发方法）
    pub fn generate(&self, expr: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        generate_expr(self.ctx, self.locals, expr)
    }
}

/// 内部辅助函数：生成表达式代码
///
/// 这个函数被各个子模块使用，用于递归生成子表达式
fn generate_expr<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
    expr: &Expr,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match &expr.kind {
        ExprKind::Literal(lit) => literal::gen_literal(ctx, lit),
        ExprKind::Variable(name) => variable::gen_variable(ctx, locals, name),
        ExprKind::Binary(left, op, right) => binary::gen_binary(ctx, locals, left, op, right),
        ExprKind::Unary(op, operand) => unary::gen_unary(ctx, locals, op, operand),
        ExprKind::Call { callee, args } => call::gen_call(ctx, locals, callee, args),
        ExprKind::Match {
            value,
            cases,
            default,
        } => match_expr::gen_match(ctx, locals, value, cases, default.as_deref()),
        ExprKind::Print(arg) => intrinsic::gen_print(ctx, locals, arg),
        ExprKind::Array(elements) => array::gen_array_literal(ctx, locals, elements),
        ExprKind::Index { array, index } => array::gen_index_access(ctx, locals, array, index),
        ExprKind::Get { object, name } => array::gen_get_property(ctx, locals, object, name),
        ExprKind::StructLiteral { .. } => {
            // 简化实现：返回 null 指针
            // 完整实现需要：1. 分配结构体内存 2. 初始化字段 3. 返回指针
            let ptr_type = ctx
                .context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::default());
            Ok(ptr_type.const_null().into())
        }
        _ => Err(CodegenError::UnsupportedExpression),
    }
}
