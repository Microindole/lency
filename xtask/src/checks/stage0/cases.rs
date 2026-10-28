pub(super) const FAILURES: &[(&str, &str)] = &[
    ("0", "Integer Division by Zero"),
    ("1", "Index Out of Bounds"),
    ("2", "out of bounds"),
    ("3", "Invalid integer conversion"),
    ("4", "invalid free"),
    ("5", "invalid free"),
    ("6", "explicit stage0 failure"),
    ("7", "Cannot pop from empty Vec"),
    ("8", "Index Out of Bounds"),
    ("9", "Integer Remainder by Zero"),
    ("10", "Integer Division Overflow"),
    ("11", "Index Out of Bounds"),
    ("12", "Invalid float conversion"),
    ("13", "Failed to Read File"),
];

pub(super) const REJECTED: &[(&str, &str)] = &[
    (
        "void_variable",
        "void noop() {} int main() { var value = noop() return 0 }",
    ),
    ("null_type", "int main() { var value = null return 0 }"),
    (
        "empty_vec_type",
        "int main() { var values = vec[] return 0 }",
    ),
    (
        "print_void",
        "void noop() {} int main() { print(noop()) return 0 }",
    ),
    (
        "imported",
        "import tests.stage0.modules.invalid\nint main() { return 0 }",
    ),
    ("undefined", "int main() { return missing }"),
    ("condition", "int main() { if 1 { return 1 } return 0 }"),
    (
        "argument",
        "int f(int x) { return x } int main() { return f(true) }",
    ),
    (
        "return",
        "int f() { return true } int main() { return f() }",
    ),
    (
        "missing_return",
        "int f(bool x) { if x { return 1 } } int main() { return f(true) }",
    ),
    (
        "nullable",
        "struct P { int x } int main() { P? p = null return p.x }",
    ),
    (
        "field",
        "struct P { int x int y } int main() { var p = P { x: 1 } return 0 }",
    ),
    ("assignment", "int main() { (1 + 2) = 3 return 0 }"),
    (
        "function_value",
        "int f() { return 1 } int main() { return f + 1 }",
    ),
    ("call", "int main() { return len(\"abc\")(7) }"),
    ("free_scalar", "int main() { free(7) return 0 }"),
    (
        "match",
        "int main() { return match true { case true => 1 } }",
    ),
    ("break", "int main() { break return 0 }"),
    ("continue", "int main() { continue return 0 }"),
    ("vec", "int main() { var xs = vec[1, true] return 0 }"),
];

pub(super) fn rejection_diagnostic(name: &str) -> &'static str {
    match name {
        "undefined" => "undefined variable",
        "void_variable" => "value type",
        "null_type" | "empty_vec_type" => "cannot infer type",
        "print_void" => "printable value",
        "condition" | "argument" | "vec" => "type mismatch",
        "return" | "imported" => "return type mismatch",
        "missing_return" => "must return a value",
        "nullable" => "might be null",
        "field" => "missing field",
        "assignment" => "invalid assignment target",
        "function_value" => "without a call",
        "call" => "not callable",
        "free_scalar" => "heap reference",
        "match" => "not exhaustive",
        "break" | "continue" => "outside loop",
        _ => unreachable!("unknown semantic contract"),
    }
}
