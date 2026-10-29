#[test]
fn free_requires_one_heap_reference() {
    for source in [
        "int main() { free(7) return 0 }",
        "int main() { free(false) return 0 }",
        "int main() { free(2.5) return 0 }",
        "int main() { int data[1] = {1} free(data) return 0 }",
        "int main() { free() return 0 }",
        "int main() { free(null, null) return 0 }",
        "int main() { free(missing) return 0 }",
    ] {
        let mut program = lency_syntax::parser::parse(source).unwrap();
        assert!(lency_sema::analyze(&mut program).is_err(), "{source}");
    }
}

#[test]
fn references_and_nullable_boxes_can_be_released() {
    for source in [
        "struct Item { int value } int main() { var item = Item { value: 7 } free(item) return 0 }",
        "int main() { var items = vec[1, 2] free(items) free(null) return 0 }",
        "int main() { int? value = 7 if value != null { free(value) } return 0 }",
        "int main() { var value = int_to_string(7) free(value) return 0 }",
    ] {
        let mut program = lency_syntax::parser::parse(source).unwrap();
        lency_sema::analyze(&mut program).expect(source);
    }
}
