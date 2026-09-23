use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)] // 关键：加上 Eq 和 Hash
#[logos(skip r"[ \t\r\n\f]+")]
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
    #[token("struct")]
    Struct,
    #[token("impl")]
    Impl,
    #[token("trait")]
    Trait,
    #[token("as")]
    As,

    #[token("enum")]
    Enum,

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
    #[token("case")]
    Case,

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
        match self {
            Token::Var => write!(f, "var"),
            Token::Const => write!(f, "const"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Return => write!(f, "return"),
            Token::Import => write!(f, "import"),
            Token::Extern => write!(f, "extern"),
            Token::Struct => write!(f, "struct"),
            Token::Impl => write!(f, "impl"),
            Token::Trait => write!(f, "trait"),
            Token::As => write!(f, "as"),
            Token::Enum => write!(f, "enum"),
            Token::Null => write!(f, "null"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::TypeInt => write!(f, "int"),
            Token::TypeFloat => write!(f, "float"),
            Token::TypeBool => write!(f, "bool"),
            Token::TypeString => write!(f, "string"),
            Token::TypeVoid => write!(f, "void"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Leq => write!(f, "<="),
            Token::Geq => write!(f, ">="),
            Token::Bang => write!(f, "!"),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"), // Escaped brace
            Token::RBrace => write!(f, "}}"), // Escaped brace
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Question => write!(f, "?"),
            Token::QuestionDot => write!(f, "?."),
            Token::QuestionQuestion => write!(f, "??"),
            Token::Arrow => write!(f, "=>"),
            Token::Underscore => write!(f, "_"),
            Token::Match => write!(f, "match"),
            Token::Case => write!(f, "case"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Int(i) => write!(f, "{}", i),
            Token::Float(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s), // Quote string
            Token::Comment => write!(f, "<comment>"),
            Token::Error => write!(f, "<error>"),
        }
    }
}
