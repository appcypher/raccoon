use crate::error;
use crate::span::Span;
use anyhow::{bail, Result};
use std::cmp::Ordering;
use std::{iter, str::Chars};

use super::errors::{LexerError, LexerErrorKind::*};
use super::token::{Token, TokenKind::*};
use super::TokenKind;

//------------------------------------------------------------------------------
// Type Definitions
//------------------------------------------------------------------------------

/// An implementation of Raccoon's tokenizer.
///
/// Check [`.grammar`](#.grammar) for the language's lexer grammar specification.
#[derive(Debug)]
pub struct Lexer<'a> {
    /// The source code to be tokenized, broken down into characters.
    pub chars: Chars<'a>,
    /// The current position in the source code.
    pub cursor: u32,
    /// The indentation type of the source code.
    pub indent_kind: IndentKind,
    /// A stack for maintaining indentation/bracket scopes.
    pub scope_stack: Vec<Scope>,
    /// The number of spaces that represents an indentation.
    pub indent_factor: i32,
    /// Token buffer for tokens like consecutive Dedents that makes sense to be lexed together.
    pub token_buffer: Vec<Token>,
}

/// Represents a scope that can be introduced by an indentation-preserving block or a indentation-ignoring bracket.
#[derive(Debug)]
pub enum Scope {
    /// An indentation scope.
    Indent {
        /// The indentation count this scope started at.
        start_space_count: i32,
        /// The current space count.
        space_count: i32,
    },
    /// A bracket scope that ignores indentation and dedentations.
    /// Dedents that are lesser or equal to the scope start count cause an error.
    Bracket {
        /// The indentation count this scope started at.
        start_space_count: i32,
        /// The bracket kind that initiated this scope.
        kind: BracketKind,
    },
    /// This is the top-level scope and it is neither indented or in brackets
    Initial,
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
}

//------------------------------------------------------------------------------
// Implementations
//------------------------------------------------------------------------------

impl<'a> Lexer<'a> {
    fn new(code: &'a str) -> Self {
        Self {
            chars: code.chars(),
            cursor: 0,
            indent_kind: IndentKind::Unknown,
            scope_stack: vec![Scope::Initial],
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
    fn peek_char(&self, offset: Option<usize>) -> Option<char> {
        self.chars.clone().nth(offset.unwrap_or(0))
    }

    // Returns subsequent `length` characters in code without advancing the cursor position.
    fn peek_string(&self, length: usize) -> String {
        self.chars.clone().take(length).collect::<String>()
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
                    match self.handle_newline_or_indentation(char, start) {
                        Ok(Some(token)) => Ok(token),
                        Ok(None) => continue,
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
                    // Lex short or long strings.
                    self.handle_short_or_long_string(char, start, self.peek_string(2) == "''")
                }
                '"' => {
                    // Lex short or long strings.
                    self.handle_short_or_long_string(char, start, self.peek_string(2) == "\"\"")
                }
                '.' => {
                    // Lex float or dot operator.
                    match self.peek_char(None) {
                        Some('0'..='9') => {
                            let char = self.eat_char().unwrap();
                            self.handle_float_fraction_part(char, start)
                        }
                        _ => Ok(Token::new(
                            Operator(".".into()),
                            Span::new(start, self.cursor),
                        )),
                    }
                }
                '0' => {
                    todo!()
                }
                '1'..='9' => {
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
    fn handle_newline_or_indentation(&mut self, char: char, start: u32) -> Result<Option<Token>> {
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
        let scope = self.scope_stack.last_mut().unwrap();

        // Check if the next char is not a newline or comment delimiter.
        if !matches!(peek_char, Some('\r') | Some('\n') | Some('#')) && prev_space.is_some() {
            // Check if spaces aren't mixed.
            if mixed_spaces {
                bail!(LexerError::new(MixedSpaces, Span::new(start, self.cursor)));
            }

            // Check spaces remain consistent between indents.
            let space_kind: IndentKind = prev_space.unwrap().try_into()?;
            if space_kind != self.indent_kind {
                bail!(LexerError::new(MixedSpaces, Span::new(start, self.cursor)));
            }

            match scope {
                Scope::Indent {
                    space_count: scope_space_count,
                    ..
                } => {
                    let indent_diff = space_count - *scope_space_count;
                    let token = match (space_count - *scope_space_count).cmp(&0) {
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

                            Some(Token::new(Indent, Span::new(start, self.cursor)))
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

                            Some(Token::new(Dedent, Span::new(start, self.cursor)))
                        }
                        Ordering::Equal => None,
                    };

                    // Update the indentation space count.
                    *scope_space_count = space_count;

                    return Ok(token);
                }
                Scope::Bracket {
                    start_space_count, ..
                } => {
                    // Check if there is no dedent that is equal or lesser than the scope itself.
                    if space_count <= *start_space_count {
                        bail!(LexerError::new(
                            InvalidInBracketDedent,
                            Span::new(start, self.cursor)
                        ));
                    }
                }
                _ => (),
            };
        }

        Ok(None)
    }

    /// Handles a short or long string.
    fn handle_short_or_long_string(
        &mut self,
        char: char,
        start: u32,
        long_string: bool,
    ) -> Result<Token> {
        // Skip long string delimiter
        if long_string {
            self.eat_char().unwrap();
            self.eat_char().unwrap();
        }

        let mut string = String::new();
        loop {
            match self.peek_char(None) {
                Some(peek_char) => {
                    if peek_char == char {
                        // Handle delimiter.
                        self.eat_char();

                        // Check for long string delimiter.
                        if long_string && self.peek_string(2) != format!("{char}{char}") {
                            bail!(LexerError::new(
                                UnterminatedString,
                                Span::new(start, self.cursor)
                            ));
                        } else {
                            self.eat_char();
                            self.eat_char();
                        };

                        break;
                    } else if peek_char == '\\' {
                        // Handle escape sequences.
                        self.eat_char();

                        match self.peek_char(None) {
                            Some('r') => {
                                self.eat_char();
                                string.push('\r');
                            }
                            Some('t') => {
                                self.eat_char();
                                string.push('\t');
                            }
                            Some('n') => {
                                self.eat_char();
                                string.push('\n');
                            }
                            Some('\\') => {
                                self.eat_char();
                                string.push('\\');
                            }
                            _ => bail!(LexerError::new(
                                InvalidStringEscapeSequence,
                                Span::new(start, self.cursor)
                            )),
                        }
                    } else {
                        string.push(self.eat_char().unwrap());
                    }
                }
                None => bail!(LexerError::new(
                    UnterminatedString,
                    Span::new(start, self.cursor)
                )),
            }
        }

        Ok(Token::new(
            TokenKind::Str(string),
            Span::new(start, self.cursor),
        ))
    }

    /// Handles float fraction part
    fn handle_float_fraction_part(
        &mut self,
        char: char,
        start: u32,
    ) -> Result<Token, anyhow::Error> {
        let mut float = format!("0.{char}");

        while matches!(self.peek_char(None), Some('0'..='9')) {
            float.push(self.eat_char().unwrap());
        }

        // What about the exponent?
        if matches!(self.peek_char(None), Some('e' | 'E')) {
            float.push(self.eat_char().unwrap());

            if matches!(self.peek_char(None), Some('-' | '+')) {
                float.push(self.eat_char().unwrap());
            }

            while matches!(self.peek_char(None), Some('0'..='9')) {
                float.push(self.eat_char().unwrap());
            }
        }

        Ok(Token::new(DecFloat(float), Span::new(start, self.cursor)))
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
