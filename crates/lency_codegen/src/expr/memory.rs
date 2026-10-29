//! 手动释放保持原始引用表示，不对已细化的可空标量进行解包。

use super::{generate_expr, variable, CodegenValue};
use crate::{CodegenContext, CodegenError, CodegenResult};
use inkwell::values::PointerValue;
use lency_syntax::ast::{Expr, ExprKind, Type};
use std::collections::HashMap;

pub(super) fn gen_free<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (PointerValue<'ctx>, Type)>,
    arg: &Expr,
    offset: usize,
) -> CodegenResult<CodegenValue<'ctx>> {
    let value = if let ExprKind::Variable(name) = &arg.kind {
        variable::gen_variable(ctx, locals, name)?
    } else {
        generate_expr(ctx, locals, arg)?
    };
    if !value.value.is_pointer_value() {
        return Err(CodegenError::TypeMismatch);
    }
    let pointer = ctx
        .context
        .i8_type()
        .ptr_type(inkwell::AddressSpace::default());
    let free = ctx.module.get_function("lency_free").unwrap_or_else(|| {
        ctx.module.add_function(
            "lency_free",
            ctx.context.void_type().fn_type(&[pointer.into()], false),
            None,
        )
    });
    crate::trace::location(ctx, offset)?;
    ctx.builder
        .build_call(free, &[value.value.into()], "")
        .map_err(|error| CodegenError::LLVMBuildError(error.to_string()))?;
    Ok(CodegenValue {
        value: ctx.context.bool_type().const_zero().into(),
        ty: Type::Void,
    })
}
