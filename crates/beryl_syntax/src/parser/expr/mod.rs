//! Expression Parser
//!
//! 表达式解析：字面量、变量、运算符、函数调用等

use super::helpers::ident_parser;
pub mod literal;
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

#[derive(Clone)]
enum PostfixOp {
    Index(Expr),
    Member(String, Span),
    Call(Vec<Expr>, Span),
}

/// 解析表达式 (公共接口)
pub fn expr_parser() -> impl Parser<Token, Expr, Error = ParserError> + Clone {
    recursive(|expr| {
        // 字面量
        // 字面量 (Literal Parser)
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

        // Struct literal: Point { x: 10, y: 20 }
        let struct_literal = ident_parser()
            .then(
                ident_parser()
                    .then_ignore(just(Token::Colon))
                    .then(expr.clone())
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(type_name, fields), span| Expr {
                kind: ExprKind::StructLiteral { type_name, fields },
                span,
            });

        // let atom = val.or(call).or(ident).or(paren);
        // Integrate match_expr. Should be high precedence.
        let atom = match_expr
            .or(print_expr)
            .or(vec_literal)
            .or(array_literal)
            .or(struct_literal)
            .or(val)
            .or(ident)
            .or(paren);

        // 后缀操作符: 索引 arr[i]
        // 后缀操作符: 索引 arr[i] 或 成员 obj.field
        let postfix = atom
            .clone()
            .then(
                expr.clone()
                    .delimited_by(just(Token::LBracket), just(Token::RBracket))
                    .map(PostfixOp::Index)
                    .or(just(Token::Dot)
                        .ignore_then(ident_parser().map_with_span(|n, s| (n, s)))
                        .map(|(n, s)| PostfixOp::Member(n, s)))
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
            });

        // 一元运算符 (!, -)
        let unary = just(Token::Bang)
            .to(UnaryOp::Not)
            .or(just(Token::Minus).to(UnaryOp::Neg))
            .repeated()
            .then(postfix.clone())
            .foldr(|op, rhs| {
                let span = rhs.span.clone();
                Expr {
                    kind: ExprKind::Unary(op, Box::new(rhs)),
                    span,
                }
            })
            .or(postfix);

        // 乘除模
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

        // 加减
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

        // 比较运算符 (<, >, <=, >=, ==, !=)
        let comparison = sum
            .clone()
            .then(
                just(Token::Lt)
                    .to(BinaryOp::Lt)
                    .or(just(Token::Gt).to(BinaryOp::Gt))
                    .or(just(Token::Leq).to(BinaryOp::Leq))
                    .or(just(Token::Geq).to(BinaryOp::Geq))
                    .or(just(Token::EqEq).to(BinaryOp::Eq))
                    .or(just(Token::NotEq).to(BinaryOp::Neq))
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

        // 逻辑与 (&&)
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

        // 逻辑或 (||)
        logical_and
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
    })
}
