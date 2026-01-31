//! Statement Parser
//!
//! 语句解析：变量声明、赋值、return、if、while、block等

use super::expr::expr_parser;
use super::helpers::{ident_parser, type_parser};
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 解析语句 (公共接口)
pub fn stmt_parser() -> impl Parser<Token, Stmt, Error = ParserError> + Clone {
    let expr = expr_parser();
    let type_p = type_parser();
    let ident = ident_parser();

    recursive(|stmt| {
        // Block 逻辑 (返回 Vec<Stmt>)
        let raw_block = stmt
            .clone()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        // 变量声明: var x: int = 1;
        let var_decl = just(Token::Var)
            .ignore_then(ident.clone())
            .then(just(Token::Colon).ignore_then(type_p.clone()).or_not())
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon).or_not())
            .map_with_span(|((name, ty), value), span| Stmt::VarDecl {
                span,
                name,
                ty,
                value,
            });

        // 赋值语句: x = 10;
        // 赋值语句 & 表达式语句
        // 合并处理以避免前缀冲突，并支持复杂的左值赋值 (e.g. this.count = 1)
        let expr_based_stmt = expr
            .clone()
            .then(
                just(Token::Eq)
                    .ignore_then(expr.clone())
                    .then_ignore(just(Token::Semicolon).or_not())
                    .map(Some)
                    .or(just(Token::Semicolon).or_not().to(None)),
            )
            .map_with_span(|(lhs, rhs_opt), span| match rhs_opt {
                Some(rhs) => Stmt::Assignment {
                    span,
                    target: lhs,
                    value: rhs,
                },
                None => Stmt::Expression(lhs),
            });

        // 块语句: { ... }
        let block_stmt = raw_block.clone().map(Stmt::Block);

        // Return
        let ret = just(Token::Return)
            .ignore_then(expr.clone().or_not())
            .then_ignore(just(Token::Semicolon).or_not())
            .map_with_span(|value, span| Stmt::Return { span, value });

        // If
        let if_stmt = just(Token::If)
            .ignore_then(expr.clone())
            .then(raw_block.clone())
            .then(just(Token::Else).ignore_then(raw_block.clone()).or_not())
            .map_with_span(|((condition, then_block), else_block), span| Stmt::If {
                span,
                condition,
                then_block,
                else_block,
            });

        // While
        let while_stmt = just(Token::While)
            .ignore_then(expr.clone())
            .then(raw_block.clone())
            .map_with_span(|(condition, body), span| Stmt::While {
                span,
                condition,
                body,
            });

        // For 循环: 支持 Classic For 和 For-In
        let for_stmt = just(Token::For).ignore_then(
            // 1. Try For-In first: for x in arr { ... }
            ident
                .clone()
                .then_ignore(just(Token::In))
                .then(expr.clone())
                .then(raw_block.clone())
                .map_with_span(|((iterator, iterable), body), span| Stmt::ForIn {
                    span,
                    iterator,
                    iterable,
                    body,
                })
                // 2. Fallback to Classic For: for var i = 0; ...
                .or(just(Token::Var)
                    .ignore_then(ident.clone())
                    .then(just(Token::Colon).ignore_then(type_p.clone()).or_not())
                    .then_ignore(just(Token::Eq))
                    .then(expr.clone())
                    .then_ignore(just(Token::Semicolon))
                    .map_with_span(|((name, ty), value), span| {
                        Some(Box::new(Stmt::VarDecl {
                            span,
                            name,
                            ty,
                            value,
                        }))
                    })
                    .or(just(Token::Semicolon).to(None))
                    .then(
                        // Condition
                        expr.clone().then_ignore(just(Token::Semicolon)).or_not(),
                    )
                    .then(
                        // update
                        ident
                            .clone()
                            .then_ignore(just(Token::Eq))
                            .then(expr.clone())
                            .map_with_span(|(name, value), span| {
                                let target_span = span.clone();
                                Stmt::Assignment {
                                    span,
                                    target: Expr {
                                        kind: ExprKind::Variable(name),
                                        span: target_span,
                                    },
                                    value,
                                }
                            })
                            .map(|s| Some(Box::new(s)))
                            .or_not()
                            .map(|opt| opt.flatten()),
                    )
                    .then(raw_block.clone())
                    .map_with_span(|(((init, condition), update), body), span| Stmt::For {
                        span,
                        init,
                        condition,
                        update,
                        body,
                    })),
        );

        // Break
        let break_stmt = just(Token::Break)
            .then_ignore(just(Token::Semicolon).or_not())
            .map_with_span(|_, span| Stmt::Break { span });

        // Continue
        let continue_stmt = just(Token::Continue)
            .then_ignore(just(Token::Semicolon).or_not())
            .map_with_span(|_, span| Stmt::Continue { span });

        // 表达式语句

        var_decl
            .or(block_stmt)
            .or(ret)
            .or(if_stmt)
            .or(while_stmt)
            .or(for_stmt)
            .or(break_stmt)
            .or(continue_stmt)
            // 必须放在最后，作为兜底
            .or(expr_based_stmt)
            .boxed()
    })
}
