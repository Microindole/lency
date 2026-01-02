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

            // 结构体类型
            Type::Struct(name) => {
                // 查找已注册的结构体类型
                if let Some(struct_type) = context.struct_types.get(name) {
                    // 结构体通过指针传递
                    Ok(struct_type
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum())
                } else {
                    // 如果找不到（可能是因为前向引用或者尚未定义），暂时先返回 i8*
                    // 在 Pass 1 应该已经全部定义了，所以这里找不到通常是错误
                    // 但为了鲁棒性，或者支持某些边缘情况，我们也可以报错
                    // 鉴于我们是按顺序生成的，如果找不到，说明是声明顺序问题或逻辑错误。
                    // 但考虑到 module 生成分两遍，body 生成时应该都能找到了。
                    Err(CodegenError::UnsupportedType(format!(
                        "Struct type '{}' not found in context",
                        name
                    )))
                }
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
        let codegen_ctx = CodegenContext::new(&context, "test");

        // 测试基础类型
        assert!(Type::Int.to_llvm_type(&codegen_ctx).is_ok());
        assert!(Type::Float.to_llvm_type(&codegen_ctx).is_ok());
        assert!(Type::Bool.to_llvm_type(&codegen_ctx).is_ok());
        assert!(Type::String.to_llvm_type(&codegen_ctx).is_ok());

        // Void 应该报错
        assert!(Type::Void.to_llvm_type(&codegen_ctx).is_err());
    }
}
