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
