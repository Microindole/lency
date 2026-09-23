use super::super::helpers::{ident_parser, type_parser};
use super::literal;
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

use super::ParserError;

pub fn parser(
    expr: impl Parser<Token, Expr, Error = ParserError> + Clone,
) -> impl Parser<Token, Expr, Error = ParserError> + Clone {
    // 字面量
    let val = literal::literal_parser();
    // 基本原子表达式
    let ident = ident_parser().map_with_span(|name, span| Expr {
        kind: ExprKind::Variable(name),
        span,
    });

    let paren = expr
        .clone()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    let match_expr = just(Token::Match)
        .ignore_then(expr.clone())
        .then(
            just(Token::Case)
                .ignore_then(crate::parser::pattern::pattern_parser())
                .then_ignore(just(Token::Arrow))
                .then(expr.clone())
                .map_with_span(|(pattern, body), span| MatchCase {
                    pattern,
                    body: Box::new(body),
                    span,
                })
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(value, cases), span| Expr {
            kind: ExprKind::Match {
                value: Box::new(value),
                cases,
                default: None, // Default is handled via Wildcard pattern now
            },
            span,
        });

    // Array literal: [1, 2, 3]
    let array_literal = expr
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map_with_span(|elements, span| Expr {
            kind: ExprKind::Array(elements),
            span,
        });

    // Vec 字面量: vec[1, 2, 3]
    let vec_literal = select! { Token::Ident(name) if name == "vec" => () }
        .ignore_then(
            expr.clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LBracket), just(Token::RBracket)),
        )
        .map_with_span(|elements, span| Expr {
            kind: ExprKind::VecLiteral(elements),
            span,
        });

    // Struct literal: Point { x: 10, y: 20 } or Box<int> { value: 10 }
    let struct_literal = type_parser()
        .then(
            ident_parser()
                .then_ignore(just(Token::Colon))
                .then(expr.clone())
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(type_, fields), span| Expr {
            kind: ExprKind::StructLiteral { type_, fields },
            span,
        });

    // Unit literal: ()
    let unit = just(Token::LParen)
        .ignore_then(just(Token::RParen))
        .map_with_span(|_, span| Expr {
            kind: ExprKind::Unit,
            span,
        });

    // let atom = val.or(call).or(ident).or(paren);
    // Integrate match_expr. Should be high precedence.
    match_expr
        .or(vec_literal)
        .or(array_literal)
        .or(struct_literal)
        .or(unit) // Check unit () before paren (expr)
        .or(val)
        .or(ident)
        .or(paren)
}
