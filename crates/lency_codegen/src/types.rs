//! Type Mapping
//!
//! Lency 类型到 LLVM 类型的映射

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::AddressSpace;
use lency_syntax::ast::Type;

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

            // 结构体类型: struct Point -> %Point* (结构体通过指针传递)
            Type::Struct(name) => {
                if let Some(struct_type) = context.struct_types.get(name) {
                    Ok(struct_type
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum())
                } else {
                    Err(CodegenError::UndefinedStructType(name.clone()))
                }
            }

            // Vec 类型: Vec<T> -> %LencyVec* (运行时指针)
            Type::Vec(_) => Ok(context
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),

            // 泛型参数: T, U
            // 在单态化之前不应该遇到这种类型
            Type::GenericParam(_) => Err(CodegenError::UnsupportedType(
                "generic parameters should be resolved before codegen".to_string(),
            )),

            // 泛型类型
            Type::Generic(_, _) => Err(CodegenError::UnsupportedType(
                "generics not yet supported".to_string(),
            )),

            // Result 类型: 使用命名结构体 Result__ok_err
            // Sprint 15: 为Result创建opaque struct type使用mangled name
            Type::Result { ok_type, err_type } => {
                // Use mangle_type for consistent naming
                let result_ty = Type::Result {
                    ok_type: ok_type.clone(),
                    err_type: err_type.clone(),
                };
                let mangled_name = lency_monomorph::mangling::mangle_type(&result_ty);

                // FIXME: 首先检查是否已经在 context.struct_types 中注册
                // 如果已注册，直接返回已有的类型，避免重复创建导致类型不匹配
                if let Some(existing_struct) = context.struct_types.get(&mangled_name) {
                    return Ok(existing_struct
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum());
                }

                // 如果未注册，创建新的struct type（但不注册，因为这里是&引用）
                // 注意：这种情况下可能会导致问题，因为后续无法找到这个类型
                // 正确的做法是在 module.rs 中提前注册所有 Result 类型
                let mut field_types: Vec<BasicTypeEnum> = Vec::new();

                // 1. is_ok 标志位 (i1)
                field_types.push(context.context.bool_type().as_basic_type_enum());

                // 2. ok_value (如果不是 void)
                if !matches!(**ok_type, Type::Void) {
                    field_types.push(ok_type.to_llvm_type(context)?);
                }

                // 3. err_value (如果不是 void - 虽然 Error 通常不是 Void)
                if !matches!(**err_type, Type::Void) {
                    field_types.push(err_type.to_llvm_type(context)?);
                }

                // 创建命名结构体 (使用mangled name)
                let struct_type = context.context.opaque_struct_type(&mangled_name);
                struct_type.set_body(&field_types, false);

                // Result使用指针语义传递
                Ok(struct_type
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum())
            }

            // 函数类型: int(int, int) -> function pointer
            Type::Function {
                param_types,
                return_type,
            } => {
                let param_llvm_types: Result<Vec<_>, _> = param_types
                    .iter()
                    .map(|t| t.to_llvm_type(context).map(|ty| ty.into()))
                    .collect();
                let param_llvm_types = param_llvm_types?;

                let fn_type = if matches!(**return_type, Type::Void) {
                    context
                        .context
                        .void_type()
                        .fn_type(&param_llvm_types, false)
                } else {
                    let ret = return_type.to_llvm_type(context)?;
                    ret.fn_type(&param_llvm_types, false)
                };

                Ok(fn_type
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum())
            }

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
    use lency_sema::types::TypeInfo;
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
