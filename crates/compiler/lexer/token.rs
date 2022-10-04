use std::convert::TryFrom;

use anyhow::{bail, Result};

use crate::span::Span;

//------------------------------------------------------------------------------
// Type Definitions
//------------------------------------------------------------------------------

/// Represents a valid Raccoon token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// The valid kinds of token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Newline,
    Indent,
    Dedent,
    Identifier(String),
    Float(String),
    Integer(String, IntegerKind),
    Imag(String),   // TODO: Implement this
    Str(String, StringKind),
    ByteStr(String, BytesKind),
    Op(Operator),
    Delim(Delimiter),
    Keyword(Keyword),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerKind {
    Bin,
    Oct,
    Dec,
    Hex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringKind {
    Str,
    RawStr,
    Format,
    RawFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytesKind {
    Bytes,
    RawBytes,
}

/// The valid keywords.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    And,
    As,
    Assert,
    Async,
    Await,
    Break,
    Class,
    Const,
    Continue,
    Def,
    Del,
    Elif,
    Else,
    Enum,
    Except,
    False,
    Finally,
    For,
    From,
    Global,
    If,
    Import,
    In,
    Interface,
    Is,
    Lambda,
    Let,
    Macro,
    Match,
    Mut,
    Nonlocal,
    Not,
    Or,
    Pass,
    Ptr,
    Raise,
    Ref,
    Return,
    True,
    Try,
    Typealias,
    Val,
    Var,
    Where,
    While,
    With,
    Yield,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Plus,
    Minus,
    Mul,
    Div,
    IntDiv,
    Mod,
    ShiftL,
    ShiftR,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Lesser,
    Greater,
    LesserEq,
    GreaterEq,
    Eq,
    NotEq,
    Pow,
    Square,
    Root,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delimiter {
    OpenParen,
    CloseParen,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenCurlyBracket,
    CloseCurlyBracket,
    Comma,
    Colon,
    Dot,
    Semicolon,
    At,
    Assign,
    Arrow,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    IntDivAssign,
    ModAssign,
    AtAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShiftRAssign,
    ShiftLAssign,
}

//------------------------------------------------------------------------------
// Implementations
//------------------------------------------------------------------------------

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Keyword {
    pub fn is_valid(s: &str) -> bool {
        Keyword::try_from(s).is_ok()
    }
}

impl TryFrom<&str> for Keyword {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        use Keyword::*;
        Ok(match s {
            "and" => And,
            "as" => As,
            "assert" => Assert,
            "async" => Async,
            "await" => Await,
            "break" => Break,
            "class" => Class,
            "const" => Const,
            "continue" => Continue,
            "def" => Def,
            "del" => Del,
            "elif" => Elif,
            "else" => Else,
            "enum" => Enum,
            "except" => Except,
            "false" => False,
            "finally" => Finally,
            "for" => For,
            "from" => From,
            "global" => Global,
            "if" => If,
            "import" => Import,
            "in" => In,
            "interface" => Interface,
            "is" => Is,
            "lambda" => Lambda,
            "let" => Let,
            "macro" => Macro,
            "match" => Match,
            "mut" => Mut,
            "nonlocal" => Nonlocal,
            "not" => Not,
            "or" => Or,
            "pass" => Pass,
            "ptr" => Ptr,
            "raise" => Raise,
            "ref" => Ref,
            "return" => Return,
            "true" => True,
            "try" => Try,
            "typealias" => Typealias,
            "val" => Val,
            "var" => Var,
            "where" => Where,
            "while" => While,
            "with" => With,
            "yield" => Yield,
            _ => bail!("invalid keyword: {}", s),
        })
    }
}

impl TryFrom<&str> for Operator {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        use Operator::*;
        Ok(match s {
            "+" => Plus,
            "-" => Minus,
            "*" => Mul,
            "/" => Div,
            "//" => IntDiv,
            "%" => Mod,
            "<<" => ShiftL,
            ">>" => ShiftR,
            "&" => BitAnd,
            "|" => BitOr,
            "^" => BitXor,
            "~" => BitNot,
            "<" => Lesser,
            ">" => Greater,
            "<=" => LesserEq,
            ">=" => GreaterEq,
            "==" => Eq,
            "!=" => NotEq,
            "**" => Pow,
            "²" => Square,
            "√" => Root,
            _ => bail!("invalid operator: {}", s),
        })
    }
}

impl TryFrom<&str> for Delimiter {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        use Delimiter::*;
        Ok(match s {
            "(" => OpenParen,
            ")" => CloseParen,
            "[" => OpenSquareBracket,
            "]" => CloseSquareBracket,
            "{" => OpenCurlyBracket,
            "}" => CloseCurlyBracket,
            "," => Comma,
            ":" => Colon,
            "." => Dot,
            ";" => Semicolon,
            "@" => At,
            "=" => Assign,
            "->" => Arrow,
            "+=" => PlusAssign,
            "-=" => MinusAssign,
            "*=" => MulAssign,
            "/=" => DivAssign,
            "//=" => IntDivAssign,
            "%=" => ModAssign,
            "@=" => AtAssign,
            "&=" => BitAndAssign,
            "|=" => BitOrAssign,
            "^=" => BitXorAssign,
            ">>=" => ShiftRAssign,
            "<<=" => ShiftLAssign,
            _ => bail!("invalid delimiter: {}", s),
        })
    }
}
