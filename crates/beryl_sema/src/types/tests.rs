//! Tests for types module
//!
//! Extracted from inline tests to keep source files clean

use super::info::TypeInfo;
use beryl_syntax::ast::Type;

#[test]
fn test_is_numeric() {
    assert!(Type::Int.is_numeric());
    assert!(Type::Float.is_numeric());
    assert!(!Type::Bool.is_numeric());
    assert!(!Type::String.is_numeric());
    assert!(!Type::Void.is_numeric());
}

#[test]
fn test_is_nullable() {
    let nullable_int = Type::Nullable(Box::new(Type::Int));
    let nullable_string = Type::Nullable(Box::new(Type::String));

    assert!(nullable_int.is_nullable());
    assert!(nullable_string.is_nullable());
    assert!(!Type::Int.is_nullable());
    assert!(!Type::String.is_nullable());
}

#[test]
fn test_is_primitive() {
    assert!(Type::Int.is_primitive());
    assert!(Type::Float.is_primitive());
    assert!(Type::Bool.is_primitive());
    assert!(Type::String.is_primitive());
    assert!(Type::Void.is_primitive());

    assert!(!Type::Struct("User".to_string()).is_primitive());
    assert!(!Type::Nullable(Box::new(Type::Int)).is_primitive());
}

#[test]
fn test_inner_type() {
    let nullable_string = Type::Nullable(Box::new(Type::String));
    assert_eq!(nullable_string.inner_type(), Some(&Type::String));

    assert_eq!(Type::Int.inner_type(), None);
    assert_eq!(Type::String.inner_type(), None);
}

#[test]
fn test_display_name() {
    assert_eq!(Type::Int.display_name(), "int");
    assert_eq!(Type::String.display_name(), "string");
    assert_eq!(Type::Nullable(Box::new(Type::Int)).display_name(), "int?");
}

#[test]
fn test_is_array() {
    let int_array = Type::Array {
        element_type: Box::new(Type::Int),
        size: 5,
    };
    let float_array = Type::Array {
        element_type: Box::new(Type::Float),
        size: 10,
    };

    assert!(int_array.is_array());
    assert!(float_array.is_array());
    assert!(!Type::Int.is_array());
    assert!(!Type::String.is_array());
    assert!(!Type::Struct("User".to_string()).is_array());
}

#[test]
fn test_array_display_name() {
    let arr = Type::Array {
        element_type: Box::new(Type::Int),
        size: 5,
    };
    assert_eq!(arr.display_name(), "[5]int");
}

#[test]
fn test_array_not_primitive() {
    let arr = Type::Array {
        element_type: Box::new(Type::Int),
        size: 3,
    };
    assert!(!arr.is_primitive());
}

#[test]
fn test_array_not_nullable() {
    let arr = Type::Array {
        element_type: Box::new(Type::Int),
        size: 3,
    };
    assert!(!arr.is_nullable());
}

// Registry tests
use super::registry::TypeRegistry;

#[test]
fn test_create_registry() {
    let registry = TypeRegistry::new();
    // Current version just ensures it can be created
    assert!(registry.lookup("anything").is_none());
}

#[test]
fn test_register_alias() {
    let mut registry = TypeRegistry::new();
    registry.register_alias("integer".to_string(), Type::Int);

    assert_eq!(registry.lookup("integer"), Some(&Type::Int));
    assert_eq!(registry.lookup("unknown"), None);
}
