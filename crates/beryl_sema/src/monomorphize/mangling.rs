//! Name Mangling for Monomorphization
//!
//! 负责将泛型类型转换为唯一的字符串标识符。
//! 规则：
//! - 基础类型保持不变: int, string
//! - 泛型实例化: Name__Arg1_Arg2 (e.g., Box<int> -> Box__int)
//! - 嵌套泛型递归处理: Vec<Box<int>> -> Vec__Box__int

use beryl_syntax::ast::Type;

pub fn mangle_type(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::String => "string".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Void => "void".to_string(),

        // 结构体名直接使用（假设非泛型）
        Type::Struct(name) => name.clone(),

        // 泛型实例化: Box<T> -> Box__T
        Type::Generic(name, args) => {
            let encoded_args: Vec<String> = args.iter().map(mangle_type).collect();
            format!("{}__{}", name, encoded_args.join("_"))
        }

        // Vec<T> -> Vec__T (Beryl 的 Vec 实际上可以视为 Generic("Vec", [T]))
        Type::Vec(inner) => {
            format!("Vec__{}", mangle_type(inner))
        }

        // Array [N]T -> Array__T__N
        Type::Array { element_type, size } => {
            format!("Array__{}__{}", mangle_type(element_type), size)
        }

        // Nullable T? -> T__opt
        Type::Nullable(inner) => {
            format!("{}__{}", mangle_type(inner), "opt")
        }

        // GenericParam T -> T (应该已经被替换了，如果在 mangling 时遇到，说明是在特化过程中)
        Type::GenericParam(name) => name.clone(),

        // Result<T, E> -> Result__T__E
        Type::Result { ok_type, err_type } => {
            format!("Result__{}_{}", mangle_type(ok_type), mangle_type(err_type))
        }

        Type::Error => "Error".to_string(),
    }
}
