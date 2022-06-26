use crate::error;
use crate::span::Span;
use anyhow::{bail, Result};
use std::cmp::Ordering;
use std::{iter, str::Chars};

use super::errors::{LexerError, LexerErrorKind::*};
use super::token::{Token, TokenKind::*};

//==============================================================================
// Type Definitions
//==============================================================================

/// An implementation of Raccoon's tokenizer.
///
/// Check [`lexer.grammar`](#lexer.grammar) for the language's lexer grammar specification.
#[derive(Debug)]
pub struct Lexer<'a> {
    /// The source code to be tokenized broken down into characters.
    pub chars: Chars<'a>,
    /// The current position in the source code.
    pub cursor: u32,
    /// The indentation type of the source code.
    pub indent_kind: IndentKind,
    /// A stcck for maintaining indentation scopes.
    pub indent_stack: Vec<IndentationScope>,
    /// The number of spaces that represents an indentation.
    pub indent_factor: i32,
    /// Token buffer for tokens like consecutive Dedents that makes sense to be lexed together.
    pub token_buffer: Vec<Token>,
}

/// Represents an indentation scope usually introduced by having an indentation-conserving block within possibly nested brackets.
#[derive(Debug)]
pub struct IndentationScope {
    /// The current space count.
    pub space_count: i32,
    /// The number of spaces this indentation scope started with.
    pub start_space_count: i32,
    /// The bracket kind that initiated this indentation.
    pub bracket_kind: BracketKind,
    /// If another un-closed bracket has been encountered in this indentation scope.
    pub within_brackets: bool,
}

#[derive(PartialEq, Debug)]
/// The different kinds of indentation.
pub enum IndentKind {
    Unknown,
    Tab,
    Space,
}

#[derive(PartialEq, Debug)]
/// The different kinds of indentation.
pub enum BracketKind {
    Parens,
    SquareBraces,
    SquigglyBraces,
    Unknown,
}

//==============================================================================
// Implementations
//==============================================================================

impl<'a> Lexer<'a> {
    fn new(code: &'a str) -> Self {
        Self {
            chars: code.chars(),
            cursor: 0,
            indent_kind: IndentKind::Unknown,
            indent_stack: vec![IndentationScope::default()],
            indent_factor: 0,
            token_buffer: Vec::new(),
        }
    }

    /// Creates a new `Lexer` iterator.
    pub fn tokenize(code: &'a str) -> impl Iterator<Item = Result<Token>> + 'a {
        let mut lexer = Lexer::new(code);
        iter::from_fn(move || lexer.next_token())
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
        self.chars.clone().nth(offset.unwrap_or(0) as usize)
    }

    // Returns the next token in the code.
    fn next_token(&mut self) -> Option<Result<Token>> {
        // Check tokens in the token buffer first.
        if let Some(token) = self.token_buffer.pop() {
            return Some(Ok(token));
        }

        // Some there are skips until a token is found which is why a loop is needed.
        while let Some(char) = self.eat_char() {
            // The cursor index before the current character.
            let start = self.cursor - 1;
            let token = match char {
                ' ' | '\t' => {
                    // Skip horizontal spaces.
                    while matches!(self.peek_char(None), Some(' ') | Some('\t')) {
                        self.eat_char().unwrap();
                    }

                    continue;
                }
                '\r' | '\n' => {
                    // Lex newlines and indentation.
                    match self.handle_newline(char, start) {
                        Ok(Some(token)) => Ok(token),
                        Ok(None) => {
                            // This means skip newlines if the code is in a bracket pair.
                            continue;
                        }
                        Err(err) => Err(err),
                    }
                }
                '#' => {
                    // Skip single line comments.
                    while !matches!(self.peek_char(None), Some('\r') | Some('\n') | None) {
                        self.eat_char().unwrap();
                    }

                    continue;
                }
                '\\' => {
                    // Skip line continuation escape sequences.
                    match self.peek_char(None) {
                        Some('\r') | Some('\n') => {
                            // A newline is expected to follow a line continuation escape sequence.
                            continue;
                        }
                        _ => error(LexerError::new(
                            InvalidLineContinuationEscapeSequence,
                            Span::new(start, self.cursor),
                        )),
                    }
                }
                '\'' => {
                    todo!()
                }
                _ => Ok(Token::new(Unknown, Span::new(start, start + 1))),
            };

            return Some(token);
        }

        None
    }
}

impl Lexer<'_> {
    /// Handles a newline character which can possibly lead to lexing indents and dedents.
    fn handle_newline(&mut self, char: char, start: u32) -> Result<Option<Token>> {
        // Eat the next char if it is a Windows-native newline.
        if char == '\r' && self.peek_char(None) == Some('\n') {
            self.eat_char();
        }

        let mut space_count = 0;
        let mut prev_space = None;
        let mut mixed_spaces = false;

        // Count the number of spaces and detect mixed space types.
        while matches!(self.peek_char(None), Some(' ') | Some('\t')) {
            let current_space = self.eat_char();

            // Check if spaces match.
            if space_count > 0 && current_space != prev_space {
                mixed_spaces = true;
            }

            prev_space = current_space;
            space_count += 1;
        }

        let peek_char = self.peek_char(None);
        let indent_scope = self.indent_stack.last_mut().unwrap();

        // Check if the next char is not a newline.
        if !matches!(peek_char, Some('\r') | Some('\n')) && prev_space.is_some() {
            // Check if spaces aren't mixed.
            if mixed_spaces {
                bail!(LexerError::new(MixedSpaces, Span::new(start, self.cursor)));
            }

            // Check spaces remain consistent between indents.
            let space_kind: IndentKind = prev_space.unwrap().try_into()?;
            if space_kind != self.indent_kind {
                bail!(LexerError::new(MixedSpaces, Span::new(start, self.cursor)));
            }

            // Focus on indentation that is not in brackets.
            if !indent_scope.within_brackets {
                let indent_diff = space_count - indent_scope.space_count;
                let token = match (space_count - indent_scope.space_count).cmp(&0) {
                    Ordering::Greater => {
                        // An indentation.
                        // Check if it is the first indentation.
                        if self.indent_factor == 0 {
                            self.indent_factor = indent_diff;
                            self.indent_kind = space_kind;
                        } else if self.indent_factor != indent_diff {
                            bail!(LexerError::new(
                                MixedIndentFactors,
                                Span::new(start, self.cursor)
                            ));
                        }

                        Token::new(Indent, Span::new(start, self.cursor))
                    }
                    Ordering::Less => {
                        // A dedentation.
                        let indent_diff_abs = indent_diff.abs();
                        if indent_diff_abs % self.indent_factor != 0 {
                            bail!(LexerError::new(
                                InconsistentDedent,
                                Span::new(start, self.cursor)
                            ))
                        }

                        // Add dedents in token buffer except the last.
                        for _ in 1..(indent_diff_abs / self.indent_factor) {
                            self.token_buffer
                                .push(Token::new(Dedent, Span::new(start, self.cursor)));
                        }

                        Token::new(Dedent, Span::new(start, self.cursor))
                    }
                    Ordering::Equal => Token::new(Newline, Span::new(start, self.cursor)),
                };

                // Update the indentation space count.
                indent_scope.space_count = space_count;
                return Ok(Some(token));
            }
        }

        // Skip indentations and newlines when inside brackets.
        if !indent_scope.within_brackets {
            return Ok(Some(Token::new(Newline, Span::new(start, self.cursor))));
        }

        Ok(None)
    }
}

impl From<char> for IndentKind {
    fn from(value: char) -> Self {
        match value {
            ' ' => IndentKind::Space,
            '\t' => IndentKind::Tab,
            _ => IndentKind::Unknown,
        }
    }
}

impl Default for IndentationScope {
    fn default() -> Self {
        IndentationScope {
            space_count: 0,
            within_brackets: false,
            start_space_count: 0,
            bracket_kind: BracketKind::Unknown,
        }
    }
}
