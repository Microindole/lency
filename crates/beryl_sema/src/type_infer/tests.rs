//! Tests for type inference module
//!
//! Extracted from inline tests to keep source files clean

use super::TypeInferer;
use crate::error::SemanticError;
use crate::scope::ScopeStack;
use beryl_syntax::ast::{Expr, ExprKind, Literal, Type};

/// Helper: Create a test scope stack
fn create_test_scopes() -> ScopeStack {
    ScopeStack::new() // Already initializes with a global scope
}

/// Helper: Create an expression
fn make_expr(kind: ExprKind) -> Expr {
    Expr { kind, span: 0..1 }
}

#[test]
fn test_infer_array_literal() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // [1, 2, 3]
    let mut elements = vec![
        make_expr(ExprKind::Literal(Literal::Int(1))),
        make_expr(ExprKind::Literal(Literal::Int(2))),
        make_expr(ExprKind::Literal(Literal::Int(3))),
    ];

    let result = inferer.infer_array(&mut elements, &(0..10));
    assert!(result.is_ok());

    let arr_type = result.unwrap();
    match arr_type {
        Type::Array { element_type, size } => {
            assert_eq!(*element_type, Type::Int);
            assert_eq!(size, 3);
        }
        _ => panic!("Expected Array type, got {:?}", arr_type),
    }
}

#[test]
fn test_infer_array_literal_float() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // [1.0, 2.5, 3.15]
    let mut elements = vec![
        make_expr(ExprKind::Literal(Literal::Float(1.0))),
        make_expr(ExprKind::Literal(Literal::Float(2.5))),
        make_expr(ExprKind::Literal(Literal::Float(3.15))),
    ];

    let result = inferer.infer_array(&mut elements, &(0..10));
    assert!(result.is_ok());

    let arr_type = result.unwrap();
    match arr_type {
        Type::Array { element_type, size } => {
            assert_eq!(*element_type, Type::Float);
            assert_eq!(size, 3);
        }
        _ => panic!("Expected Array type"),
    }
}

#[test]
fn test_infer_empty_array_error() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // [] - Empty array should error
    let mut elements = vec![];
    let result = inferer.infer_array(&mut elements, &(0..2));

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SemanticError::CannotInferType { .. }
    ));
}

#[test]
fn test_infer_array_mixed_types_error() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // [1, "hello"] - Mixed types should error
    let mut elements = vec![
        make_expr(ExprKind::Literal(Literal::Int(1))),
        make_expr(ExprKind::Literal(Literal::String("hello".to_string()))),
    ];

    let result = inferer.infer_array(&mut elements, &(0..15));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SemanticError::TypeMismatch { .. }
    ));
}

#[test]
fn test_infer_index_access_int_array() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // Create array expression (using literals)
    let mut array_expr = make_expr(ExprKind::Array(vec![
        make_expr(ExprKind::Literal(Literal::Int(10))),
        make_expr(ExprKind::Literal(Literal::Int(20))),
        make_expr(ExprKind::Literal(Literal::Int(30))),
    ]));

    // Index expression: arr[1]
    let mut index_expr = make_expr(ExprKind::Literal(Literal::Int(1)));

    let result = inferer.infer_index(&mut array_expr, &mut index_expr, &(0..10));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int);
}

#[test]
fn test_infer_index_compile_time_bounds_check_negative() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // arr: [3]int
    let mut array_expr = make_expr(ExprKind::Array(vec![
        make_expr(ExprKind::Literal(Literal::Int(1))),
        make_expr(ExprKind::Literal(Literal::Int(2))),
        make_expr(ExprKind::Literal(Literal::Int(3))),
    ]));

    // arr[-1] - Negative index should error at compile time
    let mut index_expr = make_expr(ExprKind::Literal(Literal::Int(-1)));

    let result = inferer.infer_index(&mut array_expr, &mut index_expr, &(0..10));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SemanticError::ArrayIndexOutOfBounds {
            index: -1,
            size: 3,
            ..
        }
    ));
}

#[test]
fn test_infer_index_compile_time_bounds_check_overflow() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // arr: [3]int
    let mut array_expr = make_expr(ExprKind::Array(vec![
        make_expr(ExprKind::Literal(Literal::Int(1))),
        make_expr(ExprKind::Literal(Literal::Int(2))),
        make_expr(ExprKind::Literal(Literal::Int(3))),
    ]));

    // arr[5] - Out of bounds index should error at compile time
    let mut index_expr = make_expr(ExprKind::Literal(Literal::Int(5)));

    let result = inferer.infer_index(&mut array_expr, &mut index_expr, &(0..10));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SemanticError::ArrayIndexOutOfBounds {
            index: 5,
            size: 3,
            ..
        }
    ));
}

#[test]
fn test_infer_index_non_int_index_error() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // arr: [3]int
    let mut array_expr = make_expr(ExprKind::Array(vec![
        make_expr(ExprKind::Literal(Literal::Int(1))),
        make_expr(ExprKind::Literal(Literal::Int(2))),
        make_expr(ExprKind::Literal(Literal::Int(3))),
    ]));

    // arr["hello"] - String index should error
    let mut index_expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));

    let result = inferer.infer_index(&mut array_expr, &mut index_expr, &(0..10));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SemanticError::TypeMismatch { .. }
    ));
}

#[test]
fn test_infer_array_length_property() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // arr.length
    let mut array_expr = make_expr(ExprKind::Array(vec![
        make_expr(ExprKind::Literal(Literal::Int(1))),
        make_expr(ExprKind::Literal(Literal::Int(2))),
        make_expr(ExprKind::Literal(Literal::Int(3))),
    ]));

    let result = inferer.infer_get(&mut array_expr, "length", &(0..10));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int);
}

#[test]
fn test_infer_array_invalid_property() {
    let mut scopes = create_test_scopes();
    let mut inferer = TypeInferer::new(&mut scopes);

    // arr.foo - Array doesn't have foo property
    let mut array_expr = make_expr(ExprKind::Array(vec![make_expr(ExprKind::Literal(
        Literal::Int(1),
    ))]));

    let result = inferer.infer_get(&mut array_expr, "foo", &(0..10));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SemanticError::UndefinedField { .. }
    ));
}
