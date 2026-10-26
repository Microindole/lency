#[test]
fn unchecked_scalar_use_and_incompatible_nullable_equality_are_rejected() {
    for source in [
        "int main() { int? value = null return value + 1 }",
        "int main() { int? value = null int result = value return result }",
        "int take(int value) { return value } int main() { int? value = null return take(value) }",
        "int main() { bool? left = true string? right = \"text\" if left == right { return 1 } return 0 }",
        "int main() { int? value = 7 if value != null { int? value = null return value + 1 } return 0 }",
        "int main() { int? value = 7 if value != null { { int? value = null return value + 1 } } return 0 }",
    ] {
        let mut program = lency_syntax::parser::parse(source).expect("syntax");
        assert!(lency_sema::analyze(&mut program).is_err(), "{source}");
    }
}
