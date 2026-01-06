use beryl_syntax::lexer::Token;
use beryl_syntax::parser;
use chumsky::Parser;
use logos::Logos;

fn main() {
    println!("Testing simple parsing...");

    // 测试1：最简单的代码
    let code1 = "int main() { return 0; }";
    println!("\nTest 1: {}", code1);
    let tokens1: Vec<Token> = Token::lexer(code1)
        .spanned()
        .map(|(tok, _span)| tok.unwrap())
        .collect();
    println!("Tokens: {:?}", tokens1);

    println!("Creating parser...");
    let parser1 = parser::program_parser();
    println!("Parser created, parsing...");
    match parser1.parse(tokens1) {
        Ok(prog) => println!("✓ Test 1 passed: {:?}", prog),
        Err(e) => println!("✗ Test 1 failed: {:?}", e),
    }

    // 测试2：简单struct（无泛型）
    let code2 = "struct Point { int x; }";
    println!("\nTest 2: {}", code2);
    let tokens2: Vec<Token> = Token::lexer(code2)
        .spanned()
        .map(|(tok, _span)| tok.unwrap())
        .collect();

    let parser2 = parser::program_parser();
    match parser2.parse(tokens2) {
        Ok(prog) => println!("✓ Test 2 passed: {:?}", prog),
        Err(e) => println!("✗ Test 2 failed: {:?}", e),
    }

    // 测试3：泛型struct
    let code3 = "struct Box<T> { T value; }";
    println!("\nTest 3 (generic): {}", code3);
    let tokens3: Vec<Token> = Token::lexer(code3)
        .spanned()
        .map(|(tok, _span)| tok.unwrap())
        .collect();
    println!("Tokens: {:?}", tokens3);

    println!("Creating parser for generic...");
    let parser3 = parser::program_parser();
    println!("Parser created, parsing generic struct...");
    match parser3.parse(tokens3) {
        Ok(prog) => println!("✓ Test 3 passed: {:?}", prog),
        Err(e) => println!("✗ Test 3 failed: {:?}", e),
    }

    // 测试4：和lib.rs中失败的测试一样
    let code4 = r#"
        int main() {
            var a = 10;
            var b = a + 20;
            return b;
        }
    "#;
    println!("\nTest 4 (like test_parser_full_func): {}", code4.trim());
    let tokens4: Vec<Token> = Token::lexer(code4)
        .spanned()
        .map(|(tok, _span)| tok.unwrap())
        .collect();

    let parser4 = parser::program_parser();
    match parser4.parse(tokens4) {
        Ok(_prog) => println!("✓ Test 4 passed!"),
        Err(e) => println!("✗ Test 4 failed: {:?}", e),
    }

    println!("\nAll tests completed!");
}
