//! Expression Code Generation
//!
//! 表达式代码生成的主入口，将职责分散到各个子模块

mod array;
mod binary;
mod call;
mod intrinsic;
mod literal;
mod match_expr;
mod method_call;
mod string_ops;
mod struct_access;
mod struct_init;
mod unary;
mod variable;
mod vec;

use beryl_syntax::ast::{Expr, ExprKind};
use inkwell::values::{BasicValueEnum, PointerValue};
use std::collections::HashMap;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

pub struct CodegenValue<'ctx> {
    pub value: BasicValueEnum<'ctx>,
    pub ty: beryl_syntax::ast::Type,
}

/// 表达式代码生成器（保持向后兼容的公共API）
pub struct ExprGenerator<'ctx, 'a> {
    ctx: &'a CodegenContext<'ctx>,
    /// 局部变量表 (变量名 -> (指针, Beryl类型))
    locals: &'a HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
}

impl<'ctx, 'a> ExprGenerator<'ctx, 'a> {
    /// 创建表达式生成器
    pub fn new(
        ctx: &'a CodegenContext<'ctx>,
        locals: &'a HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
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
    ) -> CodegenResult<(PointerValue<'ctx>, beryl_syntax::ast::Type)> {
        // Return Type too for verification if needed
        generate_lvalue_addr(self.ctx, self.locals, expr)
    }
}

/// 内部辅助函数：生成表达式代码
fn generate_expr<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    expr: &Expr,
) -> CodegenResult<CodegenValue<'ctx>> {
    match &expr.kind {
        ExprKind::Literal(lit) => literal::gen_literal(ctx, lit),
        ExprKind::Variable(name) => variable::gen_variable(ctx, locals, name),
        ExprKind::Binary(left, op, right) => binary::gen_binary(ctx, locals, left, op, right),
        ExprKind::Unary(op, operand) => unary::gen_unary(ctx, locals, op, operand),
        ExprKind::Call { callee, args } => {
            // 检查是否为方法调用 object.method(...)
            if let ExprKind::Get { object, name } = &callee.kind {
                // 尝试作为方法调用处理
                // gen_method_call 内部会验证 object 是否为 Struct
                let line = ctx.get_line(callee.span.start);
                method_call::gen_method_call(ctx, locals, object, name, args, line)
            } else {
                call::gen_call(ctx, locals, callee, args)
            }
        }
        ExprKind::Match {
            value,
            cases,
            default,
        } => match_expr::gen_match(ctx, locals, value, cases, default.as_deref()),
        ExprKind::Print(arg) => intrinsic::gen_print(ctx, locals, arg),
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
                beryl_syntax::ast::Type::Struct(name) => name,
                beryl_syntax::ast::Type::Generic(name, _) => name,
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
        // TODO: Result 相关表达式的代码生成
        ExprKind::Try(_) | ExprKind::Ok(_) | ExprKind::Err(_) => {
            Err(CodegenError::UnsupportedExpression)
        }
    }
}

/// 内部辅助函数：生成左值地址
fn generate_lvalue_addr<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, beryl_syntax::ast::Type)>,
    expr: &Expr,
) -> CodegenResult<(PointerValue<'ctx>, beryl_syntax::ast::Type)> {
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
            let ptr = struct_access::gen_struct_member_ptr(ctx, locals, object, name, line)?;
            // Need to return type of field for verification?
            // Currently generate_lvalue_addr returns (ptr, type).
            // We need to look up field type.
            // Re-implement logic or use helper?
            // Hacking: Use struct_access logic again roughly?
            // Or better: Let gen_struct_member_ptr return (ptr, type)?
            // Changing gen_struct_member_ptr signature would affect others?
            // No, gen_member_access calls it.
            // But gen_member_access ignored return type there?
            // Let's look up type here.

            // We need struct name. Parse expr again?
            // Or just trust caller?
            // This is getting complicated.
            // Let's assume for now we can get it.

            // To get type, we need object type.
            // generate_expr(object) -> type.
            // But we might want address of object?
            // generate_lvalue_addr is for assignment: a.x = 1.

            let obj_val = generate_expr(ctx, locals, object)?;
            let struct_name = match obj_val.ty {
                beryl_syntax::ast::Type::Struct(n) => n,
                _ => return Err(CodegenError::TypeMismatch),
            };

            let field_names = ctx
                .struct_fields
                .get(&struct_name)
                .ok_or(CodegenError::TypeMismatch)?;
            let idx = field_names
                .iter()
                .position(|n| n == name)
                .ok_or(CodegenError::TypeMismatch)?;
            let field_types = ctx
                .struct_field_types
                .get(&struct_name)
                .ok_or(CodegenError::TypeMismatch)?;
            let field_ty = field_types
                .get(idx)
                .cloned()
                .ok_or(CodegenError::TypeMismatch)?;

            Ok((ptr, field_ty))
        }
        _ => Err(CodegenError::UnsupportedExpression),
    }
}
