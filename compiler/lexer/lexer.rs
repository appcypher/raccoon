// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

/// An implementation of Raccoon's tokenizer.
///
/// Check [`lexer.grammar`](#lexer.grammar) for the language's lexer grammar specification.
///
/// ### NOTE
/// May turn [lex] function into an iterator in the future to support generating tokens on demand.
///
/// As noted¹ by some, certain lexer errors may be caused by invalid syntax, but the lexer error
/// shows first because it comes before the parser.
///
/// In addition to lex function becoming a generator, may also change Parser.code to an generator to be gotten on demand.
///
/// This has the benefit of not keeping everything in memory in case lexer / parser fails early
///
/// 1. https://medium.com/@gvanrossum_83706/building-a-peg-parser-d4869b5958fb#2a80
pub struct Lexer {}

/// Represents a valid Raccoon token.
pub struct Token {}

/// Holds top-level indentation information as well as indentation information of code in brackets
pub struct Indentation {}

/// Represents a block of code introduced by a colon and an indent within brackets.
/// A block can be the body content of a lambda or a match expression.
pub struct Block {}

/// The valid kinds of token.
pub enum TokenKind {
    Identifier,
    Newline,
    DecFloat,
    DecFloatImag,
    Indent,
    Dedent,
    PrefixedString,
    ByteString,
    String,
    DecInteger,
    DecIntegerImag,
    BinInteger,
    OctInteger,
    HexInteger,
    Operator,
    Delimiter,
    Keyword,
}

/// The different kinds of indentation.
pub enum IndentKind {
    Unknown,
    Tab,
    Space,
}


impl Lexer {
    /// Creates a new `Lexer`.
    pub fn new(_code: &str) -> Self {
        Self {}
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
