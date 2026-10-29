use lency_syntax::{
    parser::parse,
    source_map::{shift_program, SourceMap},
};

#[test]
fn imported_positions_keep_file_line_and_unicode_column() {
    let mut sources = SourceMap::main("main.lcy", "int main() { return 0 }");
    let library = "// 中文\nint work() {\n    return 7\n}";
    let base = sources.add("library.lcy", library);
    let location = sources.location(base + library.find("return").unwrap());
    assert_eq!(
        (location.file.as_str(), location.line, location.column),
        ("library.lcy", 3, 5)
    );
    let location = sources.location(base + "// 中".len());
    assert_eq!((location.line, location.column), (1, 5));
    let mut program = parse(library).unwrap();
    shift_program(&mut program, base);
    assert!(lency_syntax::source_map::decl_span(&program.decls[0]).start >= base);
}
