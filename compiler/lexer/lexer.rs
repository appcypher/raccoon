use crate::span::Span;
use anyhow::Result;
use std::str::Chars;

use super::errors::LexerError;
use super::token::{Token, TokenKind::*};
use super::utils::is_horizontal_whitespace;

/// An implementation of Raccoon's tokenizer.
///
/// Check [`lexer.grammar`](#lexer.grammar) for the language's lexer grammar specification.
#[derive(Debug)]
pub struct Lexer<'a> {
    pub chars: Chars<'a>,
    pub cursor: u32,
}

/// Holds top-level indentation information as well as indentation information of code in brackets
#[derive(Debug)]
pub struct Indentation {}

/// Represents a block of code introduced by a colon and an indent within brackets.
/// A block can be the body content of a lambda or a match expression.
#[derive(Debug)]
pub struct Block {}

/// The different kinds of indentation.
pub enum IndentKind {
    Unknown,
    Tab,
    Space,
}

impl<'a> Lexer<'a> {
    /// Creates a new `Lexer` iterator.
    pub fn tokenize(code: &'a str) -> impl Iterator<Item = Result<Token>> + 'a {
        let mut lexer = Lexer {
            chars: code.chars(),
            cursor: 0,
        };

        std::iter::from_fn(move || lexer.advance_token())
    }

    // Returns the next character in code and advances the cursor position.
    fn eat_char(&mut self) -> Option<char> {
        match self.chars.next() {
            value @ Some(_) => {
                self.cursor += 1;
                value
            }
            None => None,
        }
    }

    // Returns the next character in code without advancing the cursor position.
    fn peek_char(&mut self, offset: Option<u32>) -> Option<char> {
        let offset = offset.unwrap_or(0);
        // TODO(appcypher): Investigate a better way without cloning.
        // Tried `self.chars.by_ref().peekmore().peek_nth(offset as usize).cloned()` using peekmore lib but it still consumes the iterator.
        self.chars.clone().nth(offset as usize)
    }

    // Returns the next token in the code.
    fn advance_token(&mut self) -> Option<Result<Token>> {
        let start = self.cursor;

        if let Some(char) = self.eat_char() {
            let token = match char {
                '\r' | '\n' => self.handle_newline(char, start),
                _ => Ok(Token::new(Unknown, Span::new(start, start + 1))),
            };

            return Some(token);
        }

        None
    }
}

impl Lexer<'_> {
    /// Handles a newline character.
    fn handle_newline(&mut self, char: char, start: u32) -> Result<Token> {
        // Eat the next char if it is a Windows-native newline.
        if char == '\r' && self.peek_char(None) == Some('\n') {
            self.eat_char();
        }

        let mut space_count = 0;
        let mut prev_space = None;
        let mut mixed_spaces = false;

        // Count the number of spaces and detect mixed space types.
        while is_horizontal_whitespace(self.peek_char(None).unwrap_or_default()) {
            let current_space = self.eat_char();

            // Check if spaces match.
            if space_count > 0 && current_space == prev_space {
                mixed_spaces = true;
            }

            prev_space = current_space;
            space_count += 1;
        }

        let peek_char = self.peek_char(None);

        // Check if the next char is not a newline.
        if peek_char != Some('\r') || peek_char != Some('\n') {
            if mixed_spaces {
                return Err(LexerError::MixedSpaces(Span::new(start, self.cursor)).into());
            }
        }

        Ok(Token::new(Newline, Span::new(start, self.cursor)))
    }
}
