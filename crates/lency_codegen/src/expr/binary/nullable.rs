use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::CodegenValue;
use lency_syntax::ast::{BinaryOp, Type};

pub(super) fn compare<'ctx>(
    ctx: &CodegenContext<'ctx>,
    left: CodegenValue<'ctx>,
    right: CodegenValue<'ctx>,
    op: &BinaryOp,
) -> CodegenResult<CodegenValue<'ctx>> {
    if crate::nullable::is_null_type(&left.ty) || crate::nullable::is_null_type(&right.ty) {
        let other = if crate::nullable::is_null_type(&left.ty) {
            &right
        } else {
            &left
        };
        let present = crate::nullable::present(ctx, other)?;
        let value = if *op == BinaryOp::Eq {
            ctx.builder
                .build_not(present, "nullable.absent")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
        } else {
            present
        };
        return Ok(CodegenValue {
            value: value.into(),
            ty: Type::Bool,
        });
    }
    let lhs_present = crate::nullable::present(ctx, &left)?;
    let rhs_present = crate::nullable::present(ctx, &right)?;
    let both_present = ctx
        .builder
        .build_and(lhs_present, rhs_present, "nullable.both")
        .unwrap();
    let same_presence = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            lhs_present,
            rhs_present,
            "nullable.same_presence",
        )
        .unwrap();
    let function = ctx
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
    let compare = ctx.context.append_basic_block(function, "nullable.compare");
    let absent = ctx.context.append_basic_block(function, "nullable.absent");
    let merge = ctx.context.append_basic_block(function, "nullable.merge");
    ctx.builder
        .build_conditional_branch(both_present, compare, absent)
        .unwrap();
    ctx.builder.position_at_end(compare);
    let mut left = crate::nullable::unwrap(ctx, left, 0)?;
    let mut right = crate::nullable::unwrap(ctx, right, 0)?;
    if left.ty == Type::Float && right.ty == Type::Int {
        right = crate::nullable::coerce(ctx, right, &Type::Float, 0)?;
    } else if left.ty == Type::Int && right.ty == Type::Float {
        left = crate::nullable::coerce(ctx, left, &Type::Float, 0)?;
    }
    let equal = super::comparison::gen_eq(ctx, left.value, right.value, &left.ty)?;
    let compare_end = ctx.builder.get_insert_block().unwrap();
    ctx.builder.build_unconditional_branch(merge).unwrap();
    ctx.builder.position_at_end(absent);
    ctx.builder.build_unconditional_branch(merge).unwrap();
    ctx.builder.position_at_end(merge);
    let phi = ctx
        .builder
        .build_phi(ctx.context.bool_type(), "nullable.equal")
        .unwrap();
    phi.add_incoming(&[(&equal, compare_end), (&same_presence, absent)]);
    let value = phi.as_basic_value().into_int_value();
    let value = if *op == BinaryOp::Neq {
        ctx.builder.build_not(value, "nullable.not_equal").unwrap()
    } else {
        value
    };
    Ok(CodegenValue {
        value: value.into(),
        ty: Type::Bool,
    })
}
