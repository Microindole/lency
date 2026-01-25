//! Name Mangling for Monomorphization
//!
//! 负责将泛型类型转换为唯一的字符串标识符。
//! 规则：
//! - 基础类型保持不变: int, string
//! - 泛型实例化: Name__Arg1_Arg2 (e.g., Box<int> -> Box__int)
//! - 嵌套泛型递归处理: Vec<Box<int>> -> Vec__Box__int

use lency_syntax::ast::Type;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn mangle_type(ty: &Type) -> String {
    let mangled = mangle_type_internal(ty);

    // macOS ld64 has strict symbol name length limits. Using aggressive 32 char threshold
    // with 16 char prefix ensures final name (~32 chars) leaves ample room for method names.
    if mangled.len() > 32 {
        let mut hasher = DefaultHasher::new();
        mangled.hash(&mut hasher);
        let hash = hasher.finish();

        // Take first 16 chars to keep some readability
        let prefix: String = mangled.chars().take(16).collect();

        format!("{}_{:x}", prefix, hash)
    } else {
        mangled
    }
}

fn mangle_type_internal(ty: &Type) -> String {
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

        // Vec<T> -> Vec__T (Lency 的 Vec 实际上可以视为 Generic("Vec", [T]))
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

        // Function int(int, int) -> Fn__int__int_int
        Type::Function {
            param_types,
            return_type,
        } => {
            let params: Vec<String> = param_types.iter().map(mangle_type).collect();
            format!("Fn__{}_{}", mangle_type(return_type), params.join("_"))
        }

        Type::Error => "Error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mangle_basic() {
        assert_eq!(mangle_type(&Type::Int), "int");
        assert_eq!(mangle_type(&Type::Struct("MyStruct".into())), "MyStruct");
    }

    #[test]
    fn test_mangle_nested_short() {
        let ty = Type::Generic("Box".into(), vec![Type::Int]);
        assert_eq!(mangle_type(&ty), "Box__int");
    }

    #[test]
    fn test_mangle_long_truncation() {
        // Construct a deeply nested type to produce a long string
        let mut ty = Type::Int;
        for _ in 0..20 {
            ty = Type::Vec(Box::new(ty));
        }
        // Original without hash would be Vec__Vec__....__int
        // "Vec__".len() is 5. 20 times is 100 chars. > 48.

        let mangled = mangle_type(&ty);
        // 16 (prefix) + 1 ("_") + 16 (hex) = 33 chars max
        assert!(mangled.len() <= 34);
        assert!(mangled.contains("_"));
    }

    #[test]
    fn test_mangle_deterministic() {
        let mut ty = Type::Int;
        for _ in 0..20 {
            ty = Type::Vec(Box::new(ty));
        }

        let m1 = mangle_type(&ty);
        let m2 = mangle_type(&ty);
        assert_eq!(m1, m2);
    }
}
