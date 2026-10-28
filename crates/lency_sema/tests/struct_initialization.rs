#[test]
fn missing_and_duplicate_fields_are_rejected() {
    for source in [
        "struct Pair { int left int right } int main() { var pair = Pair { left: 1 } return 0 }",
        "struct Pair { int left int right } int main() { var pair = Pair { left: 1, left: 2, right: 3 } return 0 }",
        "struct Pair { int left string left } int main() { return 0 }",
        "enum State { Idle, Idle } int main() { return 0 }",
        "struct Pair { int value } impl Pair { int get() { return 1 } } impl Pair { int get() { return 2 } } int main() { return 0 }",
    ] {
        let mut program = lency_syntax::parser::parse(source).expect("syntax");
        assert!(lency_sema::analyze(&mut program).is_err(), "{source}");
    }
}

#[test]
fn non_exhaustive_scalar_and_refutable_enum_matches_are_rejected() {
    for source in [
        "int main() { return match 1 { case 0 => 7 } }",
        "int main() { return match true { case true => 7 } }",
        "enum Item { Value(int) } int main() { var item = Item.Value(1) return match item { case Item.Value(0) => 7 } }",
    ] {
        let mut program = lency_syntax::parser::parse(source).expect("syntax");
        assert!(lency_sema::analyze(&mut program).is_err(), "{source}");
    }
}

#[test]
fn boolean_payload_matrices_must_cover_every_combination() {
    let prefix = "enum Pair { Bits(bool, bool) } int main() { var pair = Pair.Bits(true, false) return match pair { ";
    let complete = "case Pair.Bits(true, true) => 1, case Pair.Bits(true, false) => 2, case Pair.Bits(false, true) => 3, case Pair.Bits(false, false) => 4";
    let partial = "case Pair.Bits(true, true) => 1, case Pair.Bits(true, false) => 2, case Pair.Bits(false, true) => 3";
    for (arms, valid) in [(complete, true), (partial, false)] {
        let source = format!("{prefix}{arms} }} }}");
        let mut program = lency_syntax::parser::parse(&source).unwrap();
        assert_eq!(lency_sema::analyze(&mut program).is_ok(), valid);
    }
}
