//! Expression Code Generation
//!
//! 表达式代码生成的主入口，将职责分散到各个子模块

mod array;
mod binary;
mod call;
pub mod conversion;
mod file_io;
pub mod hashmap;
mod intrinsic;
mod literal;
mod match_expr;
mod memory;
mod method_call;
mod place;
mod string_ops;
mod struct_access;
mod struct_init;
mod unary;
mod variable;
mod vec;

use inkwell::values::{BasicValueEnum, PointerValue};
use lency_syntax::ast::{Expr, ExprKind};
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

pub struct CodegenValue<'ctx> {
    pub value: BasicValueEnum<'ctx>,
    pub ty: lency_syntax::ast::Type,
}

/// 表达式代码生成器（保持向后兼容的公共API）
pub struct ExprGenerator<'ctx, 'a> {
    ctx: &'a CodegenContext<'ctx>,
    /// 局部变量表 (变量名 -> (指针, Lency类型))
    locals: &'a HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
}

impl<'ctx, 'a> ExprGenerator<'ctx, 'a> {
    /// 创建表达式生成器
    pub fn new(
        ctx: &'a CodegenContext<'ctx>,
        locals: &'a HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    ) -> Self {
        Self { ctx, locals }
    }

    /// 生成表达式代码（主分发方法）
    pub fn generate(&self, expr: &Expr) -> CodegenResult<CodegenValue<'ctx>> {
        generate_expr(self.ctx, self.locals, expr)
    }

    /// 生成左值地址（用于赋值）
    pub fn generate_lvalue_addr(
        &self,
        expr: &Expr,
    ) -> CodegenResult<(PointerValue<'ctx>, lency_syntax::ast::Type)> {
        // Return Type too for verification if needed
        generate_lvalue_addr(self.ctx, self.locals, expr)
    }

    pub fn generate_assignment(&self, target: &Expr, value: &Expr) -> CodegenResult<()> {
        place::assign(self.ctx, self.locals, target, value)
    }
}

/// 内部辅助函数：生成表达式代码
fn generate_expr<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    expr: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    crate::trace::location(ctx, expr.span.start)?;
    match &expr.kind {
        ExprKind::Literal(lit) => literal::gen_literal(ctx, lit),
        ExprKind::Unit => Ok(CodegenValue {
            value: ctx.context.bool_type().const_int(0, false).into(),
            ty: lency_syntax::ast::Type::Void,
        }),
        ExprKind::Variable(name) => {
            let value = variable::gen_variable(ctx, locals, name)?;
            let storage = locals.get(name).map(|(ptr, _)| *ptr).or_else(|| {
                ctx.module
                    .get_global(name)
                    .map(|global| global.as_pointer_value())
            });
            let refined =
                storage.is_some() && ctx.read_refinements.borrow().get(name).copied() == storage;
            if refined {
                crate::nullable::unwrap(ctx, value, ctx.get_line(expr.span.start))
            } else {
                Ok(value)
            }
        }
        ExprKind::Binary(left, op, right) => binary::gen_binary(ctx, locals, left, op, right),
        ExprKind::Unary(op, operand) => unary::gen_unary(ctx, locals, op, operand),
        ExprKind::Call { callee, args } => {
            // 检查是否为方法调用 object.method(...)
            if let ExprKind::Get { object, name } = &callee.kind {
                // 尝试作为方法调用处理
                // gen_method_call 内部会验证 object 是否为 Struct
                method_call::gen_method_call(ctx, locals, object, name, args, callee.span.start)
            } else {
                call::gen_call(ctx, locals, callee, args)
            }
        }
        ExprKind::Match {
            value,
            cases,
            default,
        } => match_expr::gen_match(ctx, locals, value, cases, default.as_deref()),
        ExprKind::Array(elements) => array::gen_array_literal(ctx, locals, elements),
        ExprKind::Index { array, index } => {
            let line = ctx.get_line(expr.span.start);
            array::gen_index_access(ctx, locals, array, index, line)
        }
        ExprKind::Get { object, name } => {
            let line = ctx.get_line(expr.span.start);
            struct_access::gen_member_access(ctx, locals, object, name, line)
        }
        ExprKind::SafeGet { object, name } => {
            let line = ctx.get_line(expr.span.start);
            struct_access::gen_safe_member_access(ctx, locals, object, name, line)
        }
        ExprKind::StructLiteral { type_, fields } => {
            let type_name = match type_ {
                lency_syntax::ast::Type::Struct(name) => name,
                lency_syntax::ast::Type::Generic(name, _) => name,
                _ => {
                    return Err(CodegenError::UnsupportedType(format!(
                        "Invalid struct literal type: {:?}",
                        type_
                    )))
                }
            };
            struct_init::gen_struct_literal(ctx, locals, type_name, fields)
        }
        ExprKind::VecLiteral(elements) => vec::gen_vec_literal(ctx, locals, elements),
        ExprKind::GenericInstantiation { .. } => {
            unreachable!("GenericInstantiation (turbo-fish) should be monomorphized before codegen")
        }
    }
}

/// 内部辅助函数：生成左值地址
fn generate_lvalue_addr<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    expr: &Expr,
) -> CodegenResult<(PointerValue<'ctx>, lency_syntax::ast::Type)> {
    match &expr.kind {
        ExprKind::GenericInstantiation { .. } => {
            unreachable!("GenericInstantiation (turbo-fish) should be monomorphized before codegen")
        }
        ExprKind::Variable(name) => {
            let (ptr, ty) = locals
                .get(name)
                .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))?;
            Ok((*ptr, ty.clone()))
        }
        ExprKind::Get { object, name } => {
            let line = ctx.get_line(expr.span.start);
            struct_access::gen_struct_member_ptr(ctx, locals, object, name, line)
        }
        ExprKind::Index { .. } => place::index_address(ctx, locals, expr),
        _ => Err(CodegenError::UnsupportedExpression),
    }
}
