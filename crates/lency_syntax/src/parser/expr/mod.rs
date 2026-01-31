//! Expression Parser
//!
//! 表达式解析：字面量、变量、运算符、函数调用等

use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

mod atom;
mod binary;
pub mod intrinsics;
pub mod literal;
mod postfix;
mod unary;

pub type ParserError = Simple<Token>;

/// 解析表达式 (公共接口)
pub fn expr_parser() -> impl Parser<Token, Expr, Error = ParserError> + Clone {
    recursive(|expr| {
        let atom = atom::parser(expr.clone()).boxed();
        let postfix = postfix::parser(atom, expr.clone()).boxed();
        let unary = unary::parser(postfix).boxed();
        binary::parser(unary).boxed()
    })
}
