//! Literal Code Generation
//!
//! 字面量代码生成

use beryl_syntax::ast::{Literal, Type};
use inkwell::AddressSpace;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::expr::CodegenValue;

/// 生成字面量代码
pub(super) fn gen_literal<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lit: &Literal,
) -> CodegenResult<CodegenValue<'ctx>> {
    match lit {
        Literal::Int(val) => {
            let int_type = ctx.context.i64_type();
            Ok(CodegenValue {
                value: int_type.const_int(*val as u64, false).into(),
                ty: Type::Int,
            })
        }
        Literal::Float(val) => {
            let float_type = ctx.context.f64_type();
            Ok(CodegenValue {
                value: float_type.const_float(*val).into(),
                ty: Type::Float,
            })
        }
        Literal::Bool(val) => {
            let bool_type = ctx.context.bool_type();
            Ok(CodegenValue {
                value: bool_type.const_int(*val as u64, false).into(),
                ty: Type::Bool,
            })
        }
        Literal::String(s) => {
            let str_val = ctx
                .builder
                .build_global_string_ptr(s, "str")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(CodegenValue {
                value: str_val.as_pointer_value().into(),
                ty: Type::String,
            })
        }
        Literal::Null => {
            let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
            Ok(CodegenValue {
                value: ptr_type.const_null().into(),
                ty: Type::Nullable(Box::new(Type::Void)), // Placeholder
            })
        }
    }
}
