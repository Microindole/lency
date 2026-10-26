use lency_syntax::{
    ast::{Decl, Stmt, Type},
    parser::parse,
};

#[test]
fn for_parentheses_and_separators_are_independent() {
    for (open, close) in [("", ""), ("(", ")")] {
        for (first, second) in [(";", ";"), ("", ""), (";", ""), ("", ";")] {
            for init in ["var i = 0", "int i = 0", "i = 0"] {
                let source = format!("int main() {{ var i = 9 for {open}{init}{first} i < 3{second} i = i + 1{close} {{ }} return 0 }}");
                let program = parse(&source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
                let Decl::Function { body, .. } = &program.decls[0] else {
                    panic!("function expected")
                };
                assert!(matches!(body[1], Stmt::For { .. }));
            }
        }
    }
    assert!(parse("int main() { for (;;) { break } return 0 }").is_ok());
    assert!(parse("int main() { for (int i = 0; i < 3; i = i + 1 { } return 0 }").is_err());
}

#[test]
fn grouped_imports_expand_the_shared_prefix() {
    let program = parse("import std.{io, fs, str} int main() { return 0 }").unwrap();
    let Decl::Import { items, .. } = &program.decls[0] else {
        panic!("import expected")
    };
    assert_eq!(
        items.iter().map(|i| i.path.join(".")).collect::<Vec<_>>(),
        ["std.io", "std.fs", "std.str"]
    );
    assert!(parse("import std.{}").is_err());
    assert!(parse("import std.{io,, fs}").is_err());
    assert!(parse("import project.tools.{lexer, parser as syntax}").is_ok());
}

#[test]
fn c_array_declarators_preserve_dimension_order() {
    let program =
        parse("int main() { int matrix[2][3] = {{1, 2, 3}, {4, 5, 6}} return 0 }").unwrap();
    let Decl::Function { body, .. } = &program.decls[0] else {
        panic!("function expected")
    };
    let Stmt::VarDecl { ty, .. } = &body[0] else {
        panic!("variable expected")
    };
    assert_eq!(
        ty,
        &Some(Type::Array {
            size: 2,
            element_type: Box::new(Type::Array {
                size: 3,
                element_type: Box::new(Type::Int)
            })
        })
    );
    for size in ["-1", "0", "4294967296"] {
        assert!(parse(&format!(
            "int main() {{ int data[{size}] = {{1}} return 0 }}"
        ))
        .is_err());
    }
}

#[test]
fn lower_case_vec_is_a_type_not_an_unresolved_generic() {
    let program = parse("int main() { vec<int> values = vec[1, 2] return 0 }").unwrap();
    let Decl::Function { body, .. } = &program.decls[0] else {
        panic!("function expected")
    };
    let Stmt::VarDecl { ty, .. } = &body[0] else {
        panic!("variable expected")
    };
    assert_eq!(ty, &Some(Type::Vec(Box::new(Type::Int))));
}
