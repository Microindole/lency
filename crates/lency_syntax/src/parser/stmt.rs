//! Statement Parser
//!
//! 语句解析：变量声明、赋值、return、if、while、block等

use super::expr::expr_parser;
use super::helpers::{ident_parser, initializer_parser, typed_name_parser};
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 解析语句 (公共接口)
pub fn stmt_parser() -> impl Parser<Token, Stmt, Error = ParserError> + Clone {
    let expr = expr_parser();
    let typed_name = typed_name_parser();
    let ident = ident_parser();

    recursive(|stmt| {
        // Block 逻辑 (返回 Vec<Stmt>)
        let raw_block = stmt
            .clone()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        // 推导变量声明: var x = 1
        let inferred_var_decl = just(Token::Var)
            .ignore_then(ident.clone())
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon).or_not())
            .map_with_span(|(name, value), span| Stmt::VarDecl {
                span,
                name,
                ty: None,
                value,
            });

        // 显式变量声明: int x = 1
        let typed_var_decl = typed_name
            .clone()
            .then_ignore(just(Token::Eq))
            .then(initializer_parser(expr.clone()))
            .then_ignore(just(Token::Semicolon).or_not())
            .map_with_span(|((ty, name), value), span| Stmt::VarDecl {
                span,
                name,
                ty: Some(ty),
                value,
            });

        let var_decl = inferred_var_decl.or(typed_var_decl);

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

        let assignment = ident
            .clone()
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .map_with_span(|(name, value), span| Stmt::Assignment {
                target: Expr {
                    kind: ExprKind::Variable(name),
                    span: span.clone(),
                },
                span,
                value,
            });
        let inferred_init = just(Token::Var)
            .ignore_then(ident.clone())
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .map_with_span(|(name, value), span| Stmt::VarDecl {
                span,
                name,
                ty: None,
                value,
            });
        let typed_init = typed_name
            .clone()
            .then_ignore(just(Token::Eq))
            .then(initializer_parser(expr.clone()))
            .map_with_span(|((ty, name), value), span| Stmt::VarDecl {
                span,
                name,
                ty: Some(ty),
                value,
            });
        // 空段仍以分隔符标识；完整三段可省略分号。
        let classic_head = inferred_init
            .or(typed_init)
            .or(assignment.clone())
            .map(|init| Some(Box::new(init)))
            .then_ignore(just(Token::Semicolon).or_not())
            .or(just(Token::Semicolon).to(None))
            .then(
                expr.clone()
                    .map(Some)
                    .then_ignore(just(Token::Semicolon).or_not())
                    .or(just(Token::Semicolon).to(None)),
            )
            .then(assignment.map(Box::new).or_not())
            .boxed();
        let classic = classic_head
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .or(classic_head)
            .then(raw_block.clone())
            .map_with_span(|(((init, condition), update), body), span| Stmt::For {
                span,
                init,
                condition,
                update,
                body,
            });
        let for_in_head = ident
            .clone()
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .boxed();
        let for_in = for_in_head
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .or(for_in_head)
            .then(raw_block.clone())
            .map_with_span(|((iterator, iterable), body), span| Stmt::ForIn {
                span,
                iterator,
                iterable,
                body,
            });
        let for_stmt = just(Token::For).ignore_then(for_in.or(classic));

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
