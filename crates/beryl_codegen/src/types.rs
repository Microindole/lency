//! Type Mapping
//!
//! Beryl 类型到 LLVM 类型的映射

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use beryl_syntax::ast::Type;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::AddressSpace;

/// Type 到 LLVM 类型的转换 trait
pub trait ToLLVMType<'ctx> {
    /// 转换为 LLVM BasicTypeEnum
    fn to_llvm_type(&self, context: &CodegenContext<'ctx>) -> CodegenResult<BasicTypeEnum<'ctx>>;
}

impl<'ctx> ToLLVMType<'ctx> for Type {
    fn to_llvm_type(&self, context: &CodegenContext<'ctx>) -> CodegenResult<BasicTypeEnum<'ctx>> {
        match self {
            // 基础类型映射
            Type::Int => Ok(context.context.i64_type().as_basic_type_enum()),
            Type::Float => Ok(context.context.f64_type().as_basic_type_enum()),
            Type::Bool => Ok(context.context.bool_type().as_basic_type_enum()),

            // 字符串用 i8* 表示 (C 风格字符串指针)
            Type::String => Ok(context
                .context
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

            // 数组类型: [T; N] -> [N x T]
            Type::Array { element_type, size } => {
                let elem_type = element_type.to_llvm_type(context)?;
                Ok(elem_type.array_type(*size as u32).as_basic_type_enum())
            }

            // 结构体类型: struct Point -> %Point*
            Type::Struct(name) => {
                if let Some(struct_type) = context.struct_types.get(name) {
                    Ok(struct_type
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum())
                } else {
                    Err(CodegenError::UndefinedStructType(name.clone()))
                }
            }

            // Vec 类型: Vec -> %BerylVec* (运行时指针)
            Type::Vec => Ok(context
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),

            // 泛型类型
            Type::Generic(_, _) => Err(CodegenError::UnsupportedType(
                "generics not yet supported".to_string(),
            )),

            Type::Error => Err(CodegenError::UnsupportedType("error type".to_string())),
        }
    }
}

/// 检查类型是否为整数类型
pub fn is_int_type(ty: &Type) -> bool {
    matches!(ty, Type::Int)
}

/// 检查类型是否为浮点类型
pub fn is_float_type(ty: &Type) -> bool {
    matches!(ty, Type::Float)
}

/// 检查类型是否为数值类型
///
/// 使用 TypeInfo trait 进行统一判断，避免硬编码
pub fn is_numeric_type(ty: &Type) -> bool {
    use beryl_sema::types::TypeInfo;
    ty.is_numeric()
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

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
        let codegen_ctx = CodegenContext::new(&context, "test", None);

        // 测试基础类型
        assert!(Type::Int.to_llvm_type(&codegen_ctx).is_ok());
        assert!(Type::Float.to_llvm_type(&codegen_ctx).is_ok());
        assert!(Type::Bool.to_llvm_type(&codegen_ctx).is_ok());
        assert!(Type::String.to_llvm_type(&codegen_ctx).is_ok());

        // Void 应该报错
        assert!(Type::Void.to_llvm_type(&codegen_ctx).is_err());
    }
}
