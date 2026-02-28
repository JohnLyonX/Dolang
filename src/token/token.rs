// Token types for Dolang lexer.
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Eof,
    Semicolon,
    Plus,
    Minus,      // -
    Mul,        // *
    Div,        // /
    Mod,        // %
    Print,
    Assign,
    VarDecl,    // $ - variable declaration
    ConstDecl,  // $@ - constant declaration

    // Comparison operators
    Eq,         // ==
    Ne,         // !=
    Gt,         // >
    Lt,         // <
    Gte,        // >=
    Lte,        // <=

    // Logical operators
    And,        // &&
    Or,         // ||
    Not,        // !

    // Control flow keywords
    If,         // $if
    Elif,       // $elif
    Else,       // $else
    While,      // $while
    Loop,       // $loop
    For,        // $for
    Break,      // $break
    Continue,   // $continue
    Fn,         // $fn
    Return,     // $#

    // Block delimiters
    LBrace,     // {
    RBrace,     // }
    LParen,     // (
    RParen,     // )
    Comma,      // ,
    Arrow,      // ->

    // Compound assignment operators
    PlusAssign,   // +=
    MinusAssign,  // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=

    // Literals
    Number,
    Char,
    Bool,
    String,
    Ident,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Eof => write!(f, "EOF"),
            Type::Semicolon => write!(f, ";"),
            Type::Plus => write!(f, "+"),
            Type::Minus => write!(f, "-"),
            Type::Mul => write!(f, "*"),
            Type::Div => write!(f, "/"),
            Type::Mod => write!(f, "%"),
            Type::Print => write!(f, "$>>"),
            Type::Assign => write!(f, "="),
            Type::VarDecl => write!(f, "$"),
            Type::ConstDecl => write!(f, "$@"),
            Type::Eq => write!(f, "=="),
            Type::Ne => write!(f, "!="),
            Type::Gt => write!(f, ">"),
            Type::Lt => write!(f, "<"),
            Type::Gte => write!(f, ">="),
            Type::Lte => write!(f, "<="),
            Type::And => write!(f, "&&"),
            Type::Or => write!(f, "||"),
            Type::Not => write!(f, "!"),
            Type::If => write!(f, "$if"),
            Type::Elif => write!(f, "$elif"),
            Type::Else => write!(f, "$else"),
            Type::While => write!(f, "$while"),
            Type::Loop => write!(f, "$loop"),
            Type::For => write!(f, "$for"),
            Type::Break => write!(f, "$break"),
            Type::Continue => write!(f, "$continue"),
            Type::Fn => write!(f, "$fn"),
            Type::Return => write!(f, "$#"),
            Type::LBrace => write!(f, "{{"),
            Type::RBrace => write!(f, "}}"),
            Type::LParen => write!(f, "("),
            Type::RParen => write!(f, ")"),
            Type::Comma => write!(f, ","),
            Type::Arrow => write!(f, "->"),
            Type::PlusAssign => write!(f, "+="),
            Type::MinusAssign => write!(f, "-="),
            Type::MulAssign => write!(f, "*="),
            Type::DivAssign => write!(f, "/="),
            Type::ModAssign => write!(f, "%="),
            Type::Number => write!(f, "NUMBER"),
            Type::Char => write!(f, "CHAR"),
            Type::Bool => write!(f, "BOOL"),
            Type::String => write!(f, "STRING"),
            Type::Ident => write!(f, "IDENT"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: Type,
    pub literal: String,
    #[allow(dead_code)]
    pub pos: usize, // rune index in the source
}

impl Token {
    pub fn new(typ: Type, literal: &str, pos: usize) -> Self {
        Self {
            typ,
            literal: literal.to_string(),
            pos,
        }
    }

    pub fn eof(pos: usize) -> Self {
        Self {
            typ: Type::Eof,
            literal: String::new(),
            pos,
        }
    }
}
