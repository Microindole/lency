use super::super::helpers::{ident_parser, type_parser};
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

use super::ParserError;

#[derive(Clone)]
enum PostfixOp {
    Index(Expr),
    Member(String, Span),
    SafeMember(String, Span),
    Call(Vec<Expr>, Span),
    GenericInstantiation(Vec<Type>, Span),
}

pub fn parser(
    atom: impl Parser<Token, Expr, Error = ParserError> + Clone,
    expr: impl Parser<Token, Expr, Error = ParserError> + Clone,
) -> impl Parser<Token, Expr, Error = ParserError> + Clone {
    atom.then(
        expr.clone()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(PostfixOp::Index)
            .or(just(Token::Dot)
                .ignore_then(
                    ident_parser()
                        .or(just(Token::Len).to("len".to_string()))
                        .map_with_span(|n, s| (n, s)),
                )
                .map(|(n, s)| PostfixOp::Member(n, s)))
            .or(just(Token::QuestionDot)
                .ignore_then(
                    ident_parser()
                        .or(just(Token::Len).to("len".to_string()))
                        .map_with_span(|n, s| (n, s)),
                )
                .map(|(n, s)| PostfixOp::SafeMember(n, s)))
            .or(just(Token::Colon)
                .then(just(Token::Colon))
                .ignore_then(just(Token::Lt))
                .ignore_then(
                    type_parser()
                        .separated_by(just(Token::Comma))
                        .allow_trailing(),
                )
                .then_ignore(just(Token::Gt))
                .map_with_span(PostfixOp::GenericInstantiation))
            .or(expr
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map_with_span(PostfixOp::Call))
            .repeated(),
    )
    .foldl(|lhs, op| match op {
        PostfixOp::Index(index) => {
            let span = lhs.span.start..index.span.end;
            Expr {
                kind: ExprKind::Index {
                    array: Box::new(lhs),
                    index: Box::new(index),
                },
                span,
            }
        }
        PostfixOp::SafeMember(name, name_span) => {
            let span = lhs.span.start..name_span.end;
            Expr {
                kind: ExprKind::SafeGet {
                    object: Box::new(lhs),
                    name,
                },
                span,
            }
        }
        PostfixOp::Member(name, name_span) => {
            let span = lhs.span.start..name_span.end;
            Expr {
                kind: ExprKind::Get {
                    object: Box::new(lhs),
                    name,
                },
                span,
            }
        }
        PostfixOp::Call(args, call_span) => {
            let span = lhs.span.start..call_span.end;
            Expr {
                kind: ExprKind::Call {
                    callee: Box::new(lhs),
                    args,
                },
                span,
            }
        }
        PostfixOp::GenericInstantiation(args, args_span) => {
            let span = lhs.span.start..args_span.end;
            Expr {
                kind: ExprKind::GenericInstantiation {
                    base: Box::new(lhs),
                    args,
                },
                span,
            }
        }
    })
}
