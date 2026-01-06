use crate::ast::{Expr, ExprKind, Literal};
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 解析字面量值 (Primitive)
#[allow(clippy::result_large_err)]
pub fn literal_value_parser() -> impl Parser<Token, Literal, Error = ParserError> + Clone {
    select! {
        Token::Int(x) => Literal::Int(x),
        Token::Float(s) => Literal::Float(s.parse().unwrap_or(0.0)),
        Token::String(s) => Literal::String(s),
        Token::True => Literal::Bool(true),
        Token::False => Literal::Bool(false),
        Token::Null => Literal::Null,
    }
}

/// 解析字面量表达式
pub fn literal_parser() -> impl Parser<Token, Expr, Error = ParserError> + Clone {
    literal_value_parser().map_with_span(|lit, span| Expr {
        kind: ExprKind::Literal(lit),
        span,
    })
}
