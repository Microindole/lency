//! Expression Parser
//!
//! 表达式解析：字面量、变量、运算符、函数调用等

use super::helpers::{ident_parser, type_parser};
pub mod literal;
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

#[derive(Clone)]
enum PostfixOp {
    Index(Expr),
    Member(String, Span),
    SafeMember(String, Span),
    Call(Vec<Expr>, Span),
    GenericInstantiation(Vec<Type>, Span),
}

/// 解析表达式 (公共接口)
pub fn expr_parser() -> impl Parser<Token, Expr, Error = ParserError> + Clone {
    recursive(|expr| {
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
                // 解析 match body { ... }
                // 格式: literal => expr (重复)
                // 最后可选: _ => expr
                ident_parser()
                    .or_not()
                    .ignore_then(just(Token::LBrace)) // Hack: ident_parser check is weird here? No, just brace.
                    .ignore_then(
                        // Cases: value => expr
                        literal::literal_value_parser()
                            .then_ignore(just(Token::Arrow))
                            .then(expr.clone())
                            .map_with_span(|(pattern, body), span| MatchCase {
                                pattern: MatchPattern::Literal(pattern),
                                body: Box::new(body),
                                span,
                            })
                            .repeated()
                            .then(
                                // Default: _ => expr
                                just(Token::Underscore)
                                    .ignore_then(just(Token::Arrow))
                                    .ignore_then(expr.clone())
                                    .map(Box::new)
                                    .or_not(),
                            )
                            .delimited_by(
                                just(Token::LBrace).or_not(), // .or_not because we might have consumed it? No.
                                just(Token::RBrace),
                            ),
                    ),
            )
            .map_with_span(|(value, (cases, default)), span| Expr {
                kind: ExprKind::Match {
                    value: Box::new(value),
                    cases,
                    default,
                },
                span,
            });

        // Print expression: print(x)
        let print_expr = just(Token::Print)
            .ignore_then(
                expr.clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .map_with_span(|arg, span| Expr {
                kind: ExprKind::Print(Box::new(arg)),
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

        // Vec 字面量: vec![1, 2, 3]
        let vec_literal = just(Token::Vec)
            .ignore_then(just(Token::Bang))
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

        // Ok 构造器: Ok(value)
        let ok_expr = just(Token::Ok)
            .ignore_then(
                expr.clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .map_with_span(|inner, span| Expr {
                kind: ExprKind::Ok(Box::new(inner)),
                span,
            });

        // Err 构造器: Err(message)
        let err_expr = just(Token::Err)
            .ignore_then(
                expr.clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .map_with_span(|inner, span| Expr {
                kind: ExprKind::Err(Box::new(inner)),
                span,
            });

        // let atom = val.or(call).or(ident).or(paren);
        // Integrate match_expr. Should be high precedence.
        let atom = match_expr
            .or(print_expr)
            .or(vec_literal)
            .or(array_literal)
            .or(ok_expr)
            .or(err_expr)
            .or(struct_literal)
            .or(val)
            .or(ident)
            .or(paren);

        // 后缀操作符: 索引 arr[i] 或 成员 obj.field 或 安全访问 obj?.field
        let postfix = atom.clone()
            .then(
                expr.clone()
                    .delimited_by(just(Token::LBracket), just(Token::RBracket))
                    .map(PostfixOp::Index)
                    .or(just(Token::Dot)
                        .ignore_then(ident_parser().map_with_span(|n, s| (n, s)))
                        .map(|(n, s)| PostfixOp::Member(n, s)))
                    .or(just(Token::QuestionDot)
                        .ignore_then(ident_parser().map_with_span(|n, s| (n, s)))
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
            // .boxed() // If needed for complexity
            ;

        // Unary: -x, !x
        let unary = just(Token::Minus)
            .to(UnaryOp::Neg)
            .or(just(Token::Bang).to(UnaryOp::Not))
            .map_with_span(|op, span| (op, span))
            .repeated()
            .then(postfix)
            .foldr(|(op, span), rhs| {
                let new_span = span.start..rhs.span.end;
                Expr {
                    kind: ExprKind::Unary(op, Box::new(rhs)),
                    span: new_span,
                }
            });

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
            });

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
            });

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
            });

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
            });

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
            });

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
    })
}
