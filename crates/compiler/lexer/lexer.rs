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

/// An implementation of Raccoon's lexer.
///
/// Check [`lexer.grammar`](#lexer.grammar) for the language's lexer grammar specification.
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

/// Integer base kinds.
#[derive(PartialEq, Debug)]
pub enum IntBase {
    Dec,
    Bin,
    Oct,
    Hex,
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
        let mut prev_is_error = false;

        iter::from_fn(move || {
            if prev_is_error {
                return None;
            }

            match lexer.next_token() {
                Some(Ok(token)) => Some(Ok(token)),
                Some(Err(err)) => {
                    prev_is_error = true;
                    Some(Err(err))
                }
                None => None,
            }
        })
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
    // TODO(appcypher): Check usage of peek_char if `offset` argument is necessary.
    fn peek_char(&self, offset: usize) -> Option<char> {
        self.chars.clone().nth(offset)
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

        // Some tokenizing require skips until a token is found which is why a loop is needed.
        while let Some(char) = self.eat_char() {
            // The cursor index before the current character.
            let start = self.cursor - 1;
            let result = match char {
                ' ' | '\t' => {
                    // Skip horizontal spaces.
                    while matches!(self.peek_char(0), Some(' ') | Some('\t')) {
                        self.eat_char().unwrap();
                    }

                    continue;
                }
                '\r' | '\n' => {
                    // Lex newlines and indentation.
                    match self.tokenize_newline_or_indentation(char, start) {
                        Ok(Some(token)) => Ok(token),
                        Ok(None) => continue,
                        Err(err) => Err(err),
                    }
                }
                '#' => {
                    // Skip single line comments.
                    while !matches!(self.peek_char(0), Some('\r') | Some('\n') | None) {
                        self.eat_char().unwrap();
                    }

                    continue;
                }
                '\\' => {
                    // Skip line continuation escape sequences.
                    match self.peek_char(0) {
                        Some('\r') => {
                            self.eat_char();
                            if self.peek_char(0) == Some('\n') {
                                self.eat_char();
                            }
                            continue;
                        }
                        Some('\n') => {
                            self.eat_char();
                            continue;
                        }
                        _ => error(LexerError::new(
                            InvalidLineContinuationEscapeSequence,
                            Span::new(start, self.cursor),
                        )),
                    }
                }
                '\'' => {
                    // Tonkenize short or long strings.
                    self.tokenize_short_or_long_string(char, start, self.peek_string(2) == "''")
                }
                '"' => {
                    // Tokenize short or long strings.
                    self.tokenize_short_or_long_string(char, start, self.peek_string(2) == "\"\"")
                }
                '.' => {
                    // Lex float or dot operator.
                    match self.peek_char(0) {
                        Some('0'..='9') => match self.lex_float_fraction(start) {
                            Ok(string) => Ok(Token::new(
                                DecFloat(format!("0{string}")),
                                Span::new(start, self.cursor),
                            )),
                            Err(err) => Err(err),
                        },
                        _ => Ok(Token::new(
                            Operator(".".into()),
                            Span::new(start, self.cursor),
                        )),
                    }
                }
                '0' => {
                    // Lex integer or float.
                    match self.peek_char(0) {
                        Some('x' | 'X' | 'b' | 'B' | 'o' | 'O') => {
                            let char = self.eat_char().unwrap();
                            self.tokenize_prefixed_integer(char, start)
                        }
                        Some('_' | '0') => {
                            let char = self.eat_char().unwrap();
                            if char == '_' && !matches!(self.peek_char(0), Some('0')) {
                                error(LexerError::new(
                                    InvalidCharacterAfterUnderscoreInDigitPart,
                                    Span::new(start, self.cursor),
                                ))
                            } else {
                                self.tokenize_leading_zero_dec_integer_or_float(start)
                            }
                        }
                        Some('1'..='9') => error(LexerError::new(
                            InvalidLeadingZeroInDecInteger,
                            Span::new(start, self.cursor),
                        )),
                        Some('.') => {
                            self.eat_char();
                            match self.lex_float_fraction(start) {
                                Ok(string) => Ok(Token::new(
                                    DecFloat(format!("0{string}")),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('e' | 'E') => {
                            self.eat_char();
                            match self.lex_float_exponent(start) {
                                Ok(string) => Ok(Token::new(
                                    DecFloat(format!("0{string}")),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        _ => Ok(Token::new(
                            DecInteger("0".into()),
                            Span::new(start, self.cursor),
                        )),
                    }
                }
                '1'..='9' => {
                    // Lex integer or float.
                    match self.peek_char(0) {
                        Some('_' | '0'..='9') => {
                            let char2 = self.eat_char().unwrap();
                            if char2 == '_' && !matches!(self.peek_char(0), Some('0'..='9')) {
                                error(LexerError::new(
                                    InvalidCharacterAfterUnderscoreInDigitPart,
                                    Span::new(start, self.cursor),
                                ))
                            } else {
                                let string = if char2 == '_' {
                                    format!("{char}")
                                } else {
                                    format!("{char}{char2}")
                                };

                                self.tokenize_leading_non_zero_dec_integer_or_float(&string, start)
                            }
                        }
                        Some('.') => {
                            self.eat_char();
                            match self.lex_float_fraction(start) {
                                Ok(string) => Ok(Token::new(
                                    DecFloat(format!("{char}{string}")),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('e' | 'E') => {
                            self.eat_char();
                            match self.lex_float_exponent(start) {
                                Ok(string) => Ok(Token::new(
                                    DecFloat(format!("{char}{string}")),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        _ => Ok(Token::new(
                            DecInteger(format!("{char}")),
                            Span::new(start, self.cursor),
                        )),
                    }
                }
                _ => error(LexerError::new(
                    InvalidCharacter,
                    Span::new(start, self.cursor),
                )),
            };

            return Some(result);
        }

        None
    }
}

/// Tokenizer functions.
impl Lexer<'_> {
    /// Tokenizes a newline character which can possibly lead to lexing indents and dedents.
    fn tokenize_newline_or_indentation(&mut self, char: char, start: u32) -> Result<Option<Token>> {
        // Eat the next char if it is a Windows-native newline.
        if char == '\r' && self.peek_char(0) == Some('\n') {
            self.eat_char();
        }

        let mut space_count = 0;
        let mut prev_space = None;
        let mut mixed_spaces = false;

        // Count the number of spaces and detect mixed space types.
        while matches!(self.peek_char(0), Some(' ') | Some('\t')) {
            let current_space = self.eat_char();

            // Check if spaces match.
            if space_count > 0 && current_space != prev_space {
                mixed_spaces = true;
            }

            prev_space = current_space;
            space_count += 1;
        }

        let peek_char = self.peek_char(0);
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

        Ok(Token::new(Newline, Span::new(start, self.cursor)).into())
    }

    /// Tokenizes a short or long string.
    fn tokenize_short_or_long_string(
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
            match self.peek_char(0) {
                Some(peek_char) => {
                    if peek_char == char {
                        // Handle delimiter.
                        self.eat_char();

                        // Check for long string delimiter.
                        if long_string && self.peek_string(2) != format!("{char}{char}") {
                            // Consume any incomplete delimiter to make the cursor position right.
                            if self.peek_char(0) == Some(char) {
                                self.eat_char();
                            }

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

                        match self.peek_char(0) {
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

    /// Tokenizes integers that start with `0b | 0o | 0x`.
    fn tokenize_prefixed_integer(&mut self, char: char, start: u32) -> Result<Token> {
        match char {
            'b' | 'B' => match self.peek_char(0) {
                Some('_' | '0' | '1') => {
                    let char = self.eat_char().unwrap();
                    let binary = self.lex_prefixed_digits(char, start, IntBase::Bin)?;
                    Ok(Token::new(
                        TokenKind::BinInteger(binary),
                        Span::new(start, self.cursor),
                    ))
                }
                _ => bail!(LexerError::new(
                    MissingDigitPartInBinInteger,
                    Span::new(start, self.cursor)
                )),
            },
            'o' | 'O' => match self.peek_char(0) {
                Some('_' | '0'..='7') => {
                    let char = self.eat_char().unwrap();
                    let octal = self.lex_prefixed_digits(char, start, IntBase::Oct)?;
                    Ok(Token::new(
                        TokenKind::OctInteger(octal),
                        Span::new(start, self.cursor),
                    ))
                }
                _ => bail!(LexerError::new(
                    MissingDigitPartInOctInteger,
                    Span::new(start, self.cursor)
                )),
            },
            'x' | 'X' => match self.peek_char(0) {
                Some('_' | '0'..='9' | 'a'..='f' | 'A'..='F') => {
                    let char = self.eat_char().unwrap();
                    let hex = self.lex_prefixed_digits(char, start, IntBase::Hex)?;
                    Ok(Token::new(
                        TokenKind::HexInteger(hex),
                        Span::new(start, self.cursor),
                    ))
                }
                _ => bail!(LexerError::new(
                    MissingDigitPartInHexInteger,
                    Span::new(start, self.cursor)
                )),
            },
            _ => unreachable!(),
        }
    }

    /// Tokenizes decimal integers or floats that start with `0`.
    fn tokenize_leading_zero_dec_integer_or_float(&mut self, start: u32) -> Result<Token> {
        loop {
            match self.peek_char(0) {
                Some('_') => {
                    self.eat_char();

                    if !matches!(self.peek_char(0), Some('0')) {
                        break error(LexerError::new(
                            InvalidCharacterAfterUnderscoreInDigitPart,
                            Span::new(start, self.cursor),
                        ));
                    }

                    continue;
                }
                Some('0') => {
                    self.eat_char();
                    continue;
                }
                Some('.') => {
                    self.eat_char();
                    break match self.lex_float_fraction(start) {
                        Ok(string) => Ok(Token::new(
                            DecFloat(format!("0{string}")),
                            Span::new(start, self.cursor),
                        )),
                        Err(err) => Err(err),
                    };
                }
                Some('e' | 'E') => {
                    self.eat_char();
                    break match self.lex_float_exponent(start) {
                        Ok(string) => Ok(Token::new(
                            DecFloat(format!("0{string}")),
                            Span::new(start, self.cursor),
                        )),
                        Err(err) => Err(err),
                    };
                }
                Some('1'..='9') => {
                    break error(LexerError::new(
                        InvalidLeadingZeroInDecInteger,
                        Span::new(start, self.cursor),
                    ))
                }
                _ => {
                    break Ok(Token::new(
                        DecInteger("0".into()),
                        Span::new(start, self.cursor),
                    ))
                }
            }
        }
    }

    /// Tokenizes decimal integers or floats that start with `1-9`.
    fn tokenize_leading_non_zero_dec_integer_or_float(
        &mut self,
        string: &str,
        start: u32,
    ) -> Result<Token> {
        let mut initial_string = string.to_owned();

        loop {
            match self.peek_char(0) {
                Some('_') => {
                    self.eat_char();

                    if !matches!(self.peek_char(0), Some('0'..='9')) {
                        break error(LexerError::new(
                            InvalidCharacterAfterUnderscoreInDigitPart,
                            Span::new(start, self.cursor),
                        ));
                    }

                    continue;
                }
                Some('0'..='9') => {
                    initial_string.push(self.eat_char().unwrap());
                    continue;
                }
                Some('.') => {
                    self.eat_char();
                    break match self.lex_float_fraction(start) {
                        Ok(string) => Ok(Token::new(
                            DecFloat(format!("{initial_string}{string}")),
                            Span::new(start, self.cursor),
                        )),
                        Err(err) => Err(err),
                    };
                }
                Some('e' | 'E') => {
                    self.eat_char();
                    break match self.lex_float_exponent(start) {
                        Ok(string) => Ok(Token::new(
                            DecFloat(format!("{initial_string}{string}")),
                            Span::new(start, self.cursor),
                        )),
                        Err(err) => Err(err),
                    };
                }
                _ => {
                    break Ok(Token::new(
                        DecInteger(initial_string),
                        Span::new(start, self.cursor),
                    ))
                }
            }
        }
    }
}

/// Helper lexer functions.
impl Lexer<'_> {
    /// Tokenizes `"." digit_part exponent?`
    fn lex_float_fraction(&mut self, start: u32) -> Result<String> {
        let mut fraction = format!(".");

        // Lex compulsory `digit_part = digit ("_"? digit)*`
        match self.peek_char(0) {
            Some('0'..='9') => {
                let char = self.eat_char().unwrap();
                fraction.push_str(&self.lex_dec_digits(char, start)?)
            }
            _ => bail!(LexerError::new(
                MissingDigitPartInFloatFraction,
                Span::new(start, self.cursor)
            )),
        }

        // Lex optional `exponent = ("e" | "E") ("+" | "-")? digit_part`
        if matches!(self.peek_char(0), Some('e' | 'E')) {
            self.eat_char();
            fraction.push_str(&self.lex_float_exponent(start)?);
        }

        Ok(fraction)
    }

    /// Lexes `("e" | "E") ("+" | "-")? digit_part`
    fn lex_float_exponent(&mut self, start: u32) -> Result<String> {
        let mut exponent = format!("e");

        // Lex ("+" | "-")?
        if matches!(self.peek_char(0), Some('-' | '+')) {
            exponent.push(self.eat_char().unwrap());
        } else {
            exponent.push('+');
        }

        // Lex compulsory digit_part = digit ("_"? digit)*
        match self.peek_char(0) {
            Some('0'..='9') => {
                let char = self.eat_char().unwrap();
                exponent.push_str(&self.lex_dec_digits(char, start)?)
            }
            _ => bail!(LexerError::new(
                MissingDigitPartInFloatExponent,
                Span::new(start, self.cursor)
            )),
        }

        Ok(exponent)
    }

    /// Lexes dec digits
    fn lex_dec_digits(&mut self, char: char, start: u32) -> Result<String> {
        let mut digits = if char != '_' {
            format!("{char}")
        } else {
            String::new()
        };

        loop {
            match self.peek_char(0) {
                Some('_') => {
                    self.eat_char();

                    if !matches!(self.peek_char(0), Some('0'..='9')) {
                        bail!(LexerError::new(
                            InvalidCharacterAfterUnderscoreInDigitPart,
                            Span::new(start, self.cursor),
                        ));
                    }
                }
                Some('0'..='9') => {
                    digits.push(self.eat_char().unwrap());
                }
                _ => break,
            }
        }

        Ok(digits)
    }

    /// Lexes oct, hex, bin digits.
    fn lex_prefixed_digits(&mut self, char: char, start: u32, base: IntBase) -> Result<String> {
        let mut digits = if char != '_' {
            format!("{char}")
        } else {
            String::new()
        };

        loop {
            match self.peek_char(0) {
                Some('_') => {
                    self.eat_char();

                    let peek_char = self.peek_char(0);
                    if !(base == IntBase::Bin && matches!(peek_char, Some('0'..='1'))
                        || base == IntBase::Oct && matches!(peek_char, Some('0'..='7'))
                        || base == IntBase::Hex
                            && matches!(peek_char, Some('0'..='9' | 'a'..='f' | 'A'..='F')))
                    {
                        bail!(LexerError::new(
                            InvalidCharacterAfterUnderscoreInDigitPart,
                            Span::new(start, self.cursor),
                        ));
                    }
                }
                Some('0'..='1') => {
                    digits.push(self.eat_char().unwrap());
                }
                Some('2'..='7') => {
                    if base == IntBase::Bin {
                        bail!(LexerError::new(
                            InvalidDigitInInteger,
                            Span::new(start, self.cursor)
                        ));
                    }
                    digits.push(self.eat_char().unwrap());
                }
                Some('8'..='9' | 'a'..='f' | 'A'..='F') => {
                    if matches!(base, IntBase::Bin | IntBase::Oct) {
                        bail!(LexerError::new(
                            InvalidDigitInInteger,
                            Span::new(start, self.cursor)
                        ));
                    }
                    digits.push(self.eat_char().unwrap());
                }
                _ => break,
            }
        }

        Ok(digits)
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
