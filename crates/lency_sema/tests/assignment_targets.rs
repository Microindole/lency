use lency_sema::SemanticError;

#[test]
fn values_and_read_only_properties_cannot_be_assignment_targets() {
    for source in [
        "int main() { 1 = 2 return 0 }",
        "int main() { (1 + 2) = 3 return 0 }",
        "int get() { return 1 } int main() { get() = 2 return 0 }",
        "int get() { return 1 } int main() { get = 2 return 0 }",
        "int main() { int values[1] = {7} values.length = 2 return 0 }",
        "struct Item { int value } int main() { Item? item = null item?.value = 2 return 0 }",
        "enum State { Idle, Busy } int main() { State.Idle = State.Busy return 0 }",
    ] {
        let mut program = lency_syntax::parser::parse(source).expect("valid syntax");
        assert!(lency_sema::analyze(&mut program).is_err(), "{source}");
    }
}

#[test]
fn nullable_member_access_needs_a_check_and_rejects_wrong_field_values() {
    for source in [
        "struct Item { int value } int main() { Item? item = null item.value = 2 return 0 }",
        "struct Item { int value } int main() { Item? item = null if item != null { item.value = \"wrong\" } return 0 }",
    ] {
        let mut program = lency_syntax::parser::parse(source).unwrap();
        let errors = lency_sema::analyze(&mut program).unwrap_err();
        assert!(errors.iter().any(|error| matches!(error, SemanticError::PossibleNullAccess { .. } | SemanticError::TypeMismatch { .. })), "{errors:?}");
    }
}
