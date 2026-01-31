use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

use super::ParserError;

pub fn parser<'a>(
    unary: impl Parser<Token, Expr, Error = ParserError> + Clone + 'a,
) -> impl Parser<Token, Expr, Error = ParserError> + Clone + 'a {
    // Product: *, /, %
    let product = unary
        .clone()
        .then(
            just(Token::Star)
                .to(BinaryOp::Mul)
                .or(just(Token::Slash).to(BinaryOp::Div))
                .or(just(Token::Percent).to(BinaryOp::Mod))
                .then(unary)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let span = lhs.span.start..rhs.span.end;
            Expr {
                kind: ExprKind::Binary(Box::new(lhs), op, Box::new(rhs)),
                span,
            }
        })
        .boxed();

    // Sum: +, -
    let sum = product
        .clone()
        .then(
            just(Token::Plus)
                .to(BinaryOp::Add)
                .or(just(Token::Minus).to(BinaryOp::Sub))
                .then(product)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let span = lhs.span.start..rhs.span.end;
            Expr {
                kind: ExprKind::Binary(Box::new(lhs), op, Box::new(rhs)),
                span,
            }
        })
        .boxed();

    // Comparison: <, >, <=, >=, ==, !=
    let comparison = sum
        .clone()
        .then(
            just(Token::EqEq)
                .to(BinaryOp::Eq)
                .or(just(Token::NotEq).to(BinaryOp::Neq))
                .or(just(Token::Leq).to(BinaryOp::Leq))
                .or(just(Token::Geq).to(BinaryOp::Geq))
                .or(just(Token::Lt).to(BinaryOp::Lt))
                .or(just(Token::Gt).to(BinaryOp::Gt))
                .then(sum)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let span = lhs.span.start..rhs.span.end;
            Expr {
                kind: ExprKind::Binary(Box::new(lhs), op, Box::new(rhs)),
                span,
            }
        })
        .boxed();

    // Logical And: &&
    let logical_and = comparison
        .clone()
        .then(
            just(Token::And)
                .to(BinaryOp::And)
                .then(comparison)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let span = lhs.span.start..rhs.span.end;
            Expr {
                kind: ExprKind::Binary(Box::new(lhs), op, Box::new(rhs)),
                span,
            }
        })
        .boxed();

    // Logical Or: ||
    let logical_or = logical_and
        .clone()
        .then(
            just(Token::Or)
                .to(BinaryOp::Or)
                .then(logical_and)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let span = lhs.span.start..rhs.span.end;
            Expr {
                kind: ExprKind::Binary(Box::new(lhs), op, Box::new(rhs)),
                span,
            }
        })
        .boxed();

    // Elvis: ?? (Right associative)
    logical_or
        .clone()
        .then_ignore(just(Token::QuestionQuestion))
        .repeated()
        .then(logical_or)
        .foldr(|lhs, rhs| {
            let span = lhs.span.start..rhs.span.end;
            Expr {
                kind: ExprKind::Binary(Box::new(lhs), BinaryOp::Elvis, Box::new(rhs)),
                span,
            }
        })
        .boxed()
}
