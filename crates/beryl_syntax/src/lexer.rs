use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)] // 关键：加上 Eq 和 Hash
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    // --- 关键字 (Keywords) ---
    #[token("var")]
    Var,
    #[token("const")]
    Const,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    #[token("import")]
    Import,
    #[token("extern")]
    Extern,
    #[token("print")]
    Print,
    #[token("struct")]
    Struct,
    #[token("impl")]
    Impl,
    #[token("vec")]
    Vec,

    // 字面量关键字
    #[token("null")]
    Null,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // --- 基础类型关键字 ---
    #[token("int")]
    TypeInt,
    #[token("float")]
    TypeFloat,
    #[token("bool")]
    TypeBool,
    #[token("string")]
    TypeString,
    #[token("void")]
    TypeVoid,

    // --- 符号 (Symbols) ---
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Leq,
    #[token(">=")]
    Geq,

    #[token("!")]
    Bang,
    #[token("&&")]
    And,
    #[token("||")]
    Or,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("?")]
    Question,
    #[token("?.")]
    QuestionDot,
    #[token("??")]
    QuestionQuestion,
    #[token("=>")]
    Arrow,
    #[regex("_", priority = 3)]
    Underscore,

    #[token("match")]
    Match,

    // --- 复杂数据 (Data) ---
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse().ok())]
    Int(i64),

    // 关键修正：为了 Hash 实现，Float 这里先存 String，AST 阶段再转 f64
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    Float(String),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('\\') => out.push('\\'),
                    Some('"') => out.push('"'),
                    Some('0') => out.push('\0'),
                    Some(o) => out.push(o),
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        out
    })]
    String(String),

    #[regex(r"//[^\n]*", logos::skip)]
    Comment,

    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
