//! 消除换行后的普通调用与 C 风格函数类型声明之间的前缀歧义。

use crate::ast::Span;
use crate::lexer::Token;

pub fn statement_boundaries(source: &str, tokens: Vec<(Token, Span)>) -> Vec<(Token, Span)> {
    let mut output: Vec<(Token, Span)> = Vec::with_capacity(tokens.len());
    let mut parentheses = 0usize;
    let mut brackets = 0usize;
    let mut braces = 0usize;
    for (token, span) in tokens {
        if parentheses == 0 && brackets == 0 && braces > 0 && token != Token::Semicolon {
            if let Some((Token::Return, previous)) = output.last() {
                if source[previous.end..span.start].contains('\n') {
                    output.push((Token::Semicolon, previous.end..previous.end));
                }
            }
        }
        if parentheses == 0 && brackets == 0 && braces > 0 && matches!(token, Token::Ident(_)) {
            if let Some((Token::RParen, previous)) = output.last() {
                if source[previous.end..span.start].contains('\n') {
                    output.push((Token::Semicolon, previous.end..previous.end));
                }
            }
        }
        match token {
            Token::LParen => parentheses += 1,
            Token::RParen => parentheses = parentheses.saturating_sub(1),
            Token::LBracket => brackets += 1,
            Token::RBracket => brackets = brackets.saturating_sub(1),
            Token::LBrace => braces += 1,
            Token::RBrace => braces = braces.saturating_sub(1),
            _ => {}
        }
        output.push((token, span));
    }
    output
}
