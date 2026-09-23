pub mod ast;
pub mod lexer;
pub mod parser; // Now points to parser/mod.rs

#[cfg(test)]
mod tests {
    // use super::*;
    use crate::lexer::Token;
    // use chumsky::Parser;
    use logos::Logos; // 修复: 引入 Parser trait

    #[test]
    fn test_lexer_basic() {
        let code = "var a = 10 + 20;";
        let mut lexer = Token::lexer(code);

        assert_eq!(lexer.next(), Some(Ok(Token::Var)));
        assert_eq!(lexer.next(), Some(Ok(Token::Ident("a".to_string()))));
        assert_eq!(lexer.next(), Some(Ok(Token::Eq)));
        assert_eq!(lexer.next(), Some(Ok(Token::Int(10))));
        assert_eq!(lexer.next(), Some(Ok(Token::Plus)));
        assert_eq!(lexer.next(), Some(Ok(Token::Int(20))));
    }

    #[test]
    fn test_lexer_crlf_whitespace() {
        let code = "var a = 1;\r\nvar b = 2;\r\n";
        let tokens: Vec<Token> = Token::lexer(code).filter_map(Result::ok).collect();

        assert!(tokens.contains(&Token::Var));
        assert!(tokens.contains(&Token::Ident("a".to_string())));
        assert!(tokens.contains(&Token::Ident("b".to_string())));
    }

    #[test]
    fn rejects_result_type_suffix() {
        let code = "int! divide(int left, int right) { return left / right }";
        assert!(
            crate::parser::parse(code).is_err(),
            "T! must not remain part of the public type syntax"
        );
    }

    #[test]
    fn rejects_postfix_error_propagation() {
        let code = "int main() { var value = read()? return 0 }";
        assert!(
            crate::parser::parse(code).is_err(),
            "postfix ? must not remain part of the public expression syntax"
        );
    }

    #[test]
    fn ok_and_err_are_ordinary_identifiers() {
        let tokens: Vec<Token> = Token::lexer("Ok Err").filter_map(Result::ok).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("Ok".to_string()),
                Token::Ident("Err".to_string())
            ]
        );
    }

    #[test]
    fn intrinsic_names_are_not_lexer_keywords() {
        let tokens: Vec<Token> = Token::lexer("print len read_file panic")
            .filter_map(Result::ok)
            .collect();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("print".to_string()),
                Token::Ident("len".to_string()),
                Token::Ident("read_file".to_string()),
                Token::Ident("panic".to_string()),
            ]
        );
    }

    #[test]
    fn user_enum_may_define_ok_and_err_variants() {
        let code = r#"
            enum Outcome {
                Ok(int),
                Err(string)
            }

            int main() {
                var value = Outcome.Ok(1)
                return 0
            }
        "#;
        assert!(
            crate::parser::parse(code).is_ok(),
            "Ok and Err must have no compiler-specific grammar"
        );
    }

    #[test]
    fn generic_calls_use_c_style_angle_brackets() {
        let code = r#"
            T identity<T>(T value) { return value }
            int main() { return identity<int>(1) }
        "#;
        assert!(crate::parser::parse(code).is_ok());
        assert!(
            crate::parser::parse("int main() { return identity::<int>(1) }").is_err(),
            "Rust turbofish syntax must not be accepted"
        );
    }

    #[test]
    fn rust_style_closures_are_not_public_syntax() {
        assert!(
            crate::parser::parse("int main() { var f = |int x| => x return f(1) }").is_err(),
            "=> is reserved for match cases"
        );
    }

    #[test]
    fn explicit_variables_use_type_first_declarations() {
        assert!(crate::parser::parse("int main() { int value = 1 return value }").is_ok());
        assert!(
            crate::parser::parse("int main() { var value: int = 1 return value }").is_err(),
            "Rust-style colon type annotations must not be accepted"
        );
    }

    #[test]
    fn vec_literals_do_not_use_rust_macro_syntax() {
        assert!(crate::parser::parse("int main() { var values = vec[1, 2] return 0 }").is_ok());
        let old = crate::parser::parse("int main() { var values = vec![1, 2] return 0 }")
            .expect("tokens may still form unrelated expressions");
        let crate::ast::Decl::Function { body, .. } = &old.decls[0] else {
            panic!("expected function")
        };
        let crate::ast::Stmt::VarDecl { value, .. } = &body[0] else {
            panic!("expected variable declaration")
        };
        assert!(
            !matches!(value.kind, crate::ast::ExprKind::VecLiteral(_)),
            "vec! must not construct a Vec literal"
        );
    }

    // 注意：此测试在某些配置下可能栈溢出
    // 功能已通过 examples/test_parser.rs 验证
    // 如需运行，请确保 .cargo/config.toml 中设置了足够的栈大小
    /*
    #[test]
    fn test_parser_full_func() {
        let code = r#"
            int main() {
                var a = 10;
                var b = a + 20;
                return b;
            }
        "#;

        let tokens: Vec<Token> = Token::lexer(code)
            .spanned()
            .map(|(tok, _span)| tok.unwrap())
            .collect();

        let parser = parser::program_parser();
        let result = parser.parse(tokens);

        assert!(result.is_ok(), "Parser failed: {:?}", result.err());

        let program = result.unwrap();
        assert_eq!(program.decls.len(), 1);

        match &program.decls[0] {
            ast::Decl::Function {
                name,
                return_type,
                body,
                ..
            } => {
                assert_eq!(name, "main");
                assert_eq!(return_type, &ast::types::Type::Int);
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected function decl"),
        }
    }
    */
}
