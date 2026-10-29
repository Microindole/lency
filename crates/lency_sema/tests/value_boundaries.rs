use lency_sema::SemanticError;

#[test]
fn unsupported_call_targets_are_not_successful_error_types() {
    for source in [
        "int main() { return (1 + 2)(3) }",
        "int main() { return len(\"text\")(3) }",
        "void main() { print(len(\"text\")(3)) }",
    ] {
        let mut program = lency_syntax::parser::parse(source).expect(source);
        let errors = lency_sema::analyze(&mut program).expect_err(source);
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, SemanticError::NotCallable { .. })),
            "{errors:?}"
        );
    }
}

#[test]
fn declarations_are_not_their_return_values() {
    for source in [
        "int value() { return 7 } int main() { int result = value return result }",
        "string text() { return \"text\" } int main() { free(text) return 0 }",
        "int value() { return 7 } int main() { return value + 1 }",
        "struct Item { int value } int main() { var item = Item return 0 }",
        "enum State { Idle } int main() { var state = State return 0 }",
    ] {
        let mut program = lency_syntax::parser::parse(source).expect(source);
        assert!(lency_sema::analyze(&mut program).is_err(), "{source}");
    }
}

#[test]
fn ordinary_calls_and_enum_members_keep_their_types() {
    let source = "enum State { Idle, Value(int) } int value() { return 7 } int main() { int result = value() var state = State.Value(result) free(state) return result - 7 }";
    let mut program = lency_syntax::parser::parse(source).unwrap();
    lency_sema::analyze(&mut program).unwrap();
}
