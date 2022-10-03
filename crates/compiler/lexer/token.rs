use crate::span::Span;

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
    Delimiter,
    Identifier(String),
    DecFloat(String),
    DecFloatImag(String),
    PrefixedStr(String),
    ByteStr(String),
    Str(String),
    DecInteger(String),
    DecIntegerImag(String),
    BinInteger(String),
    OctInteger(String),
    HexInteger(String),
    Operator(String),
    Keyword(String),
    Comment(String),
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
