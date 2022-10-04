use std::fmt::Display;

use crate::span::Span;

//------------------------------------------------------------------------------
// Type Definitions
//------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexerErrorKind {
    MixedSpaces,
    UnknownIndent,
    MixedIndentFactors,
    InconsistentDedent,
    InvalidInBracketDedent,
    UnterminatedString,
    InvalidLineContinuationEscapeSequence,
    MissingDigitPartInFloatFraction,
    MissingDigitPartInFloatExponent,
    MissingDigitPartInBinInteger,
    MissingDigitPartInOctInteger,
    MissingDigitPartInDecInteger,
    MissingDigitPartInHexInteger,
    InvalidDigitInInteger,
    InvalidCharacterAfterUnderscoreInDigitPart,
    InvalidLeadingZeroInDecInteger,
    InvalidCharacterInByteString,
    InvalidCharacter,
    InvalidOperator
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexerError {
    pub(crate) kind: LexerErrorKind,
    pub(crate) span: Span,
}

//------------------------------------------------------------------------------
// Implementations
//------------------------------------------------------------------------------

impl LexerError {
    pub fn new(kind: LexerErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl std::error::Error for LexerError {}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LexerError")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .finish()
    }
}
