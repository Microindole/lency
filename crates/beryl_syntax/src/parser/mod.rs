//! Parser Module
//!
//! 模块化的 Parser 实现，遵循开闭原则

pub mod decl;
pub mod expr;
pub mod helpers;
pub mod pattern;
pub mod stmt;

use crate::ast::Program;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 主入口：解析整个程序
pub fn program_parser() -> impl Parser<Token, Program, Error = ParserError> {
    decl::decl_parser()
        .repeated()
        .map(|decls| Program { decls })
        .then_ignore(end())
}
