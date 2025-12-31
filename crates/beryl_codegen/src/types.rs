//! Type Mapping
//!
//! Beryl 类型到 LLVM 类型的映射

use crate::error::{CodegenError, CodegenResult};
use beryl_syntax::ast::Type;
use inkwell::context::Context;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::AddressSpace;

/// Type 到 LLVM 类型的转换 trait
pub trait ToLLVMType<'ctx> {
    /// 转换为 LLVM BasicTypeEnum
    fn to_llvm_type(&self, context: &'ctx Context) -> CodegenResult<BasicTypeEnum<'ctx>>;
}

impl<'ctx> ToLLVMType<'ctx> for Type {
    fn to_llvm_type(&self, context: &'ctx Context) -> CodegenResult<BasicTypeEnum<'ctx>> {
        match self {
            // 基础类型映射
            Type::Int => Ok(context.i64_type().as_basic_type_enum()),
            Type::Float => Ok(context.f64_type().as_basic_type_enum()),
            Type::Bool => Ok(context.bool_type().as_basic_type_enum()),

            // 字符串用 i8* 表示 (C 风格字符串指针)
            Type::String => Ok(context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),

            // Void 不是 basic type，调用者应该特殊处理
            Type::Void => Err(CodegenError::UnsupportedType(
                "void is not a basic type".to_string(),
            )),

            // 可空类型用指针表示
            Type::Nullable(inner) => {
                let inner_type = inner.to_llvm_type(context)?;
                Ok(inner_type
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum())
            }

            // 类类型用指针表示
            Type::Class(name) => {
                // 暂时用 opaque pointer (i8*) 表示类实例
                // 未来可以实现完整的类布局
                Ok(context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum())
            }

            // 泛型类型（如 List<int>）暂不支持
            Type::Generic(name, _) => Err(CodegenError::UnsupportedType(format!(
                "generic type {} not yet supported",
                name
            ))),

            // 错误类型不应出现在 codegen 阶段
            Type::Error => Err(CodegenError::UnsupportedType(
                "error type should not reach codegen".to_string(),
            )),
        }
    }
}

/// 检查类型是否为整数类型
pub fn is_int_type(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::Bool)
}

/// 检查类型是否为浮点类型
pub fn is_float_type(ty: &Type) -> bool {
    matches!(ty, Type::Float)
}

/// 检查类型是否为数值类型
pub fn is_numeric_type(ty: &Type) -> bool {
    is_int_type(ty) || is_float_type(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numeric() {
        assert!(is_numeric_type(&Type::Int));
        assert!(is_numeric_type(&Type::Float));
        assert!(!is_numeric_type(&Type::String));
        assert!(!is_numeric_type(&Type::Bool));
    }

    #[test]
    fn test_to_llvm_type() {
        let context = Context::create();

        // 测试基础类型
        assert!(Type::Int.to_llvm_type(&context).is_ok());
        assert!(Type::Float.to_llvm_type(&context).is_ok());
        assert!(Type::Bool.to_llvm_type(&context).is_ok());
        assert!(Type::String.to_llvm_type(&context).is_ok());

        // Void 应该报错
        assert!(Type::Void.to_llvm_type(&context).is_err());
    }
}
