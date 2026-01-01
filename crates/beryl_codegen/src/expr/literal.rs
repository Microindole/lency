//! Literal Code Generation
//!
//! 字面量代码生成

use beryl_syntax::ast::Literal;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// 生成字面量代码
pub(super) fn gen_literal<'ctx>(
    ctx: &CodegenContext<'ctx>,
    lit: &Literal,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match lit {
        Literal::Int(val) => {
            let int_type = ctx.context.i64_type();
            Ok(int_type.const_int(*val as u64, false).into())
        }
        Literal::Float(val) => {
            let float_type = ctx.context.f64_type();
            Ok(float_type.const_float(*val).into())
        }
        Literal::Bool(val) => {
            let bool_type = ctx.context.bool_type();
            Ok(bool_type.const_int(*val as u64, false).into())
        }
        Literal::String(s) => {
            let str_val = ctx
                .builder
                .build_global_string_ptr(s, "str")
                .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?;
            Ok(str_val.as_pointer_value().into())
        }
        Literal::Null => {
            let ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::default());
            Ok(ptr_type.const_null().into())
        }
    }
}
