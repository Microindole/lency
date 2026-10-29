use lency_syntax::ast::{Decl, Stmt};

#[test]
fn calls_on_separate_lines_cannot_become_a_function_type_declaration() {
    for separator in ["\n", "\r\n", " // comment\n"] {
        let source = format!("void main() {{ int? value = 7{separator}free(value){separator}free(value){separator}value = null }}");
        let program = lency_syntax::parser::parse(&source).unwrap();
        let Decl::Function { body, .. } = &program.decls[0] else {
            panic!("function expected")
        };
        assert_eq!(body.len(), 4);
        assert!(matches!(body[1], Stmt::Expression(_)));
        assert!(matches!(body[2], Stmt::Expression(_)));
        assert!(matches!(body[3], Stmt::Assignment { .. }));
    }
}

#[test]
fn multiline_arguments_and_same_line_function_types_still_parse() {
    for source in [
        "void main() { print(\n len(\"text\")\n) }",
        "void main() { int(int) callback = other }",
        "int compute(\n int left,\n int right\n) { return left + right }",
    ] {
        lency_syntax::parser::parse(source).expect(source);
    }
}

#[test]
fn bare_return_ends_at_a_newline() {
    let source = "void main() { return\n print(\"unreachable\") }";
    let program = lency_syntax::parser::parse(source).unwrap();
    let Decl::Function { body, .. } = &program.decls[0] else {
        panic!("function expected")
    };
    assert!(matches!(body[0], Stmt::Return { value: None, .. }));
    assert_eq!(body.len(), 2);
}
