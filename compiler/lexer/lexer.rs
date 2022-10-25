use crate::error;
use crate::span::Span;
use anyhow::{bail, Result};
use std::cmp::Ordering;
use std::{iter, str::Chars};

use super::errors::{LexerError, LexerErrorKind::*};
use super::token::{Token, TokenKind::*};
use super::{BytesKind, Delimiter, IntegerKind, Operator, StringKind, TokenKind};

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
    /// The current indentation level.
    pub indent_level: i32,
    /// The number of spaces that make up an indent or dedent.
    pub indent_size: i32,
    /// Token buffer for tokens like consecutive Dedents that makes sense to be lexed together.
    pub token_buffer: Vec<Token>,
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
            indent_level: 0,
            indent_size: 0,
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
    fn peek_char(&self) -> Option<char> {
        self.chars.clone().next()
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
                    while matches!(self.peek_char(), Some(' ') | Some('\t')) {
                        self.eat_char().unwrap();
                    }

                    continue;
                }
                '\r' | '\n' => {
                    // Lex newlines and indentation.
                    self.tokenize_newline_or_indentation(char, start)
                }
                '#' => {
                    // Skip single line comments.
                    while !matches!(self.peek_char(), Some('\r') | Some('\n') | None) {
                        self.eat_char().unwrap();
                    }

                    continue;
                }
                '\\' => {
                    // Skip line continuation escape sequences.
                    match self.peek_char() {
                        Some('\r') => {
                            self.eat_char();
                            if self.peek_char() == Some('\n') {
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
                    // Tokenize short or long strings.
                    match self.lex_short_or_long_string(char, start, self.peek_string(2) == "''") {
                        Ok(string) => Ok(Token::new(
                            Str(string, StringKind::Str),
                            Span::new(start, self.cursor),
                        )),
                        Err(err) => Err(err),
                    }
                }
                '"' => {
                    // Tokenize short or long strings.
                    match self.lex_short_or_long_string(char, start, self.peek_string(2) == "\"\"")
                    {
                        Ok(string) => Ok(Token::new(
                            Str(string, StringKind::Str),
                            Span::new(start, self.cursor),
                        )),
                        Err(err) => Err(err),
                    }
                }
                '.' => {
                    // Tokenize float or dot operator.
                    match self.peek_char() {
                        Some('0'..='9') => match self.lex_float_fraction(start) {
                            Ok(string) => Ok(if self.peek_string(2) == "im" {
                                self.eat_char();
                                self.eat_char();

                                Token::new(
                                    Imag(format!("0{string}")),
                                    Span::new(start, self.cursor),
                                )
                            } else {
                                Token::new(
                                    Float(format!("0{string}")),
                                    Span::new(start, self.cursor),
                                )
                            }),
                            Err(err) => Err(err),
                        },
                        _ => Ok(Token::new(
                            Delim(Delimiter::Dot),
                            Span::new(start, self.cursor),
                        )),
                    }
                }
                '0' => {
                    // Tokenize integer or float.
                    match self.peek_char() {
                        Some('x' | 'X' | 'b' | 'B' | 'o' | 'O') => {
                            let char = self.eat_char().unwrap();
                            self.tokenize_prefixed_integer(char, start)
                        }
                        Some('_' | '0') => {
                            let char = self.eat_char().unwrap();
                            if char == '_' && !matches!(self.peek_char(), Some('0')) {
                                error(LexerError::new(
                                    InvalidCharacterAfterUnderscoreInDigitPart,
                                    Span::new(start, self.cursor),
                                ))
                            } else {
                                self.tokenize_leading_zero_dec_integer_or_float_or_im(start)
                            }
                        }
                        Some('1'..='9') => error(LexerError::new(
                            InvalidLeadingZeroInDecInteger,
                            Span::new(start, self.cursor),
                        )),
                        Some('.') => {
                            self.eat_char();
                            match self.lex_float_fraction(start) {
                                Ok(string) => Ok(if self.peek_string(2) == "im" {
                                    self.eat_char();
                                    self.eat_char();

                                    Token::new(
                                        Imag(format!("0{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                } else {
                                    Token::new(
                                        Float(format!("0{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                }),
                                Err(err) => Err(err),
                            }
                        }
                        Some('e' | 'E') => {
                            self.eat_char();
                            match self.lex_float_exponent(start) {
                                Ok(string) => Ok(if self.peek_string(2) == "im" {
                                    self.eat_char();
                                    self.eat_char();

                                    Token::new(
                                        Imag(format!("0{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                } else {
                                    Token::new(
                                        Float(format!("0{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                }),
                                Err(err) => Err(err),
                            }
                        }
                        _ => Ok(if self.peek_string(2) == "im" {
                            self.eat_char();
                            self.eat_char();

                            Token::new(Imag(format!("0")), Span::new(start, self.cursor))
                        } else {
                            Token::new(
                                Integer("0".into(), IntegerKind::Dec),
                                Span::new(start, self.cursor),
                            )
                        }),
                    }
                }
                '1'..='9' => {
                    // Tokenize integer or float.
                    match self.peek_char() {
                        Some('_' | '0'..='9') => {
                            let char2 = self.eat_char().unwrap();
                            if char2 == '_' && !matches!(self.peek_char(), Some('0'..='9')) {
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

                                self.tokenize_leading_non_zero_dec_integer_or_float_or_im(
                                    string, start,
                                )
                            }
                        }
                        Some('.') => {
                            self.eat_char();
                            match self.lex_float_fraction(start) {
                                Ok(string) => Ok(if self.peek_string(2) == "im" {
                                    self.eat_char();
                                    self.eat_char();

                                    Token::new(
                                        Imag(format!("{char}{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                } else {
                                    Token::new(
                                        Float(format!("{char}{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                }),
                                Err(err) => Err(err),
                            }
                        }
                        Some('e' | 'E') => {
                            self.eat_char();
                            match self.lex_float_exponent(start) {
                                Ok(string) => Ok(if self.peek_string(2) == "im" {
                                    self.eat_char();
                                    self.eat_char();

                                    Token::new(
                                        Imag(format!("{char}{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                } else {
                                    Token::new(
                                        Float(format!("{char}{string}")),
                                        Span::new(start, self.cursor),
                                    )
                                }),
                                Err(err) => Err(err),
                            }
                        }
                        _ => Ok(if self.peek_string(2) == "im" {
                            self.eat_char();
                            self.eat_char();

                            Token::new(Imag(format!("{char}")), Span::new(start, self.cursor))
                        } else {
                            Token::new(
                                Integer(format!("{char}"), IntegerKind::Dec),
                                Span::new(start, self.cursor),
                            )
                        }),
                    }
                }
                'f' => {
                    // Tokenize format string or identifier or keyword.
                    match self.peek_char() {
                        Some('"') => {
                            let char = self.eat_char().unwrap();
                            match self.lex_short_or_long_string(
                                char,
                                start,
                                self.peek_string(2) == "\"\"",
                            ) {
                                Ok(string) => Ok(Token::new(
                                    Str(string, StringKind::Format),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('\'') => {
                            let char = self.eat_char().unwrap();
                            match self.lex_short_or_long_string(
                                char,
                                start,
                                self.peek_string(2) == "''",
                            ) {
                                Ok(string) => Ok(Token::new(
                                    Str(string, StringKind::Format),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
                            let char = self.eat_char().unwrap();
                            Ok(self.tokenize_identifier_or_keyword(format!("f{char}"), start))
                        }
                        _ => Ok(self.tokenize_identifier_or_keyword(format!("f"), start)),
                    }
                }
                'b' => {
                    // Tokenize byte string or identifier or keyword.
                    match self.peek_char() {
                        Some('"') => {
                            let char = self.eat_char().unwrap();
                            match self.lex_short_or_long_bytes(
                                char,
                                start,
                                self.peek_string(2) == "\"\"",
                            ) {
                                Ok(string) => Ok(Token::new(
                                    ByteStr(string, BytesKind::Bytes),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('\'') => {
                            let char = self.eat_char().unwrap();
                            match self.lex_short_or_long_bytes(
                                char,
                                start,
                                self.peek_string(2) == "''",
                            ) {
                                Ok(string) => Ok(Token::new(
                                    ByteStr(string, BytesKind::Bytes),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
                            let char = self.eat_char().unwrap();
                            Ok(self.tokenize_identifier_or_keyword(format!("b{char}"), start))
                        }
                        _ => Ok(self.tokenize_identifier_or_keyword(format!("b"), start)),
                    }
                }
                'r' => {
                    // Tokenize raw string or raw format string or raw byte string or identifier or keyword.
                    match self.peek_char() {
                        Some('"') => {
                            let char = self.eat_char().unwrap();
                            match self.lex_short_or_long_bytes(
                                char,
                                start,
                                self.peek_string(2) == "\"\"",
                            ) {
                                Ok(string) => Ok(Token::new(
                                    Str(string, StringKind::RawStr),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('\'') => {
                            let char = self.eat_char().unwrap();
                            match self.lex_short_or_long_bytes(
                                char,
                                start,
                                self.peek_string(2) == "''",
                            ) {
                                Ok(string) => Ok(Token::new(
                                    Str(string, StringKind::RawStr),
                                    Span::new(start, self.cursor),
                                )),
                                Err(err) => Err(err),
                            }
                        }
                        Some('b') => {
                            self.eat_char();
                            match self.peek_char() {
                                Some('"') => {
                                    let char = self.eat_char().unwrap();
                                    match self.lex_short_or_long_bytes(
                                        char,
                                        start,
                                        self.peek_string(2) == "\"\"",
                                    ) {
                                        Ok(string) => Ok(Token::new(
                                            ByteStr(string, BytesKind::RawBytes),
                                            Span::new(start, self.cursor),
                                        )),
                                        Err(err) => Err(err),
                                    }
                                }
                                Some('\'') => {
                                    let char = self.eat_char().unwrap();
                                    match self.lex_short_or_long_bytes(
                                        char,
                                        start,
                                        self.peek_string(2) == "''",
                                    ) {
                                        Ok(string) => Ok(Token::new(
                                            ByteStr(string, BytesKind::RawBytes),
                                            Span::new(start, self.cursor),
                                        )),
                                        Err(err) => Err(err),
                                    }
                                }
                                Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
                                    let char = self.eat_char().unwrap();
                                    Ok(self
                                        .tokenize_identifier_or_keyword(format!("rb{char}"), start))
                                }
                                _ => Ok(self.tokenize_identifier_or_keyword(format!("rb"), start)),
                            }
                        }
                        Some('f') => {
                            self.eat_char();
                            match self.peek_char() {
                                Some('"') => {
                                    let char = self.eat_char().unwrap();
                                    match self.lex_short_or_long_bytes(
                                        char,
                                        start,
                                        self.peek_string(2) == "\"\"",
                                    ) {
                                        Ok(string) => Ok(Token::new(
                                            Str(string, StringKind::RawFormat),
                                            Span::new(start, self.cursor),
                                        )),
                                        Err(err) => Err(err),
                                    }
                                }
                                Some('\'') => {
                                    let char = self.eat_char().unwrap();
                                    match self.lex_short_or_long_bytes(
                                        char,
                                        start,
                                        self.peek_string(2) == "''",
                                    ) {
                                        Ok(string) => Ok(Token::new(
                                            Str(string, StringKind::RawFormat),
                                            Span::new(start, self.cursor),
                                        )),
                                        Err(err) => Err(err),
                                    }
                                }
                                Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
                                    let char = self.eat_char().unwrap();
                                    Ok(self
                                        .tokenize_identifier_or_keyword(format!("rf{char}"), start))
                                }
                                _ => Ok(self.tokenize_identifier_or_keyword(format!("rf"), start)),
                            }
                        }
                        Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
                            let char = self.eat_char().unwrap();
                            Ok(self.tokenize_identifier_or_keyword(format!("r{char}"), start))
                        }
                        _ => Ok(self.tokenize_identifier_or_keyword(format!("r"), start)),
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    Ok(self.tokenize_identifier_or_keyword(format!("{char}"), start))
                }
                '/' => Ok(match self.peek_char() {
                    Some('/') => {
                        self.eat_char();
                        match self.peek_char() {
                            Some('=') => {
                                self.eat_char();
                                Token::new(
                                    Delim(Delimiter::IntDivAssign),
                                    Span::new(start, self.cursor),
                                )
                            }
                            _ => Token::new(Op(Operator::IntDiv), Span::new(start, self.cursor)),
                        }
                    }
                    Some('=') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::DivAssign), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::Div), Span::new(start, self.cursor)),
                }),
                '>' => Ok(match self.peek_char() {
                    Some('>') => {
                        self.eat_char();
                        match self.peek_char() {
                            Some('=') => {
                                self.eat_char();
                                Token::new(
                                    Delim(Delimiter::ShiftRAssign),
                                    Span::new(start, self.cursor),
                                )
                            }
                            _ => Token::new(Op(Operator::ShiftR), Span::new(start, self.cursor)),
                        }
                    }
                    Some('=') => {
                        self.eat_char();
                        Token::new(Op(Operator::GreaterEq), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::Greater), Span::new(start, self.cursor)),
                }),
                '<' => Ok(match self.peek_char() {
                    Some('<') => {
                        self.eat_char();
                        match self.peek_char() {
                            Some('=') => {
                                self.eat_char();
                                Token::new(
                                    Delim(Delimiter::ShiftLAssign),
                                    Span::new(start, self.cursor),
                                )
                            }
                            _ => Token::new(Op(Operator::ShiftL), Span::new(start, self.cursor)),
                        }
                    }
                    Some('=') => {
                        self.eat_char();
                        Token::new(Op(Operator::LessEq), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::Less), Span::new(start, self.cursor)),
                }),
                '=' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(Op(Operator::Eq), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Delim(Delimiter::Assign), Span::new(start, self.cursor)),
                }),
                '!' => match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Ok(Token::new(
                            Op(Operator::NotEq),
                            Span::new(start, self.cursor),
                        ))
                    }
                    _ => error(LexerError::new(
                        InvalidOperator,
                        Span::new(start, self.cursor),
                    )),
                },
                '|' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::BitOrAssign), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::BitOr), Span::new(start, self.cursor)),
                }),
                '-' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::MinusAssign), Span::new(start, self.cursor))
                    }
                    Some('>') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::Arrow), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::Minus), Span::new(start, self.cursor)),
                }),
                '+' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::PlusAssign), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::Plus), Span::new(start, self.cursor)),
                }),
                '*' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::MulAssign), Span::new(start, self.cursor))
                    }
                    Some('*') => {
                        self.eat_char();
                        Token::new(Op(Operator::Pow), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::Mul), Span::new(start, self.cursor)),
                }),
                '^' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(
                            Delim(Delimiter::BitXorAssign),
                            Span::new(start, self.cursor),
                        )
                    }
                    _ => Token::new(Op(Operator::BitXor), Span::new(start, self.cursor)),
                }),
                '&' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(
                            Delim(Delimiter::BitAndAssign),
                            Span::new(start, self.cursor),
                        )
                    }
                    _ => Token::new(Op(Operator::BitAnd), Span::new(start, self.cursor)),
                }),
                '%' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::ModAssign), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Op(Operator::Mod), Span::new(start, self.cursor)),
                }),
                '@' => Ok(match self.peek_char() {
                    Some('=') => {
                        self.eat_char();
                        Token::new(Delim(Delimiter::AtAssign), Span::new(start, self.cursor))
                    }
                    _ => Token::new(Delim(Delimiter::At), Span::new(start, self.cursor)),
                }),
                '~' => Ok(Token::new(
                    Op(Operator::BitNot),
                    Span::new(start, self.cursor),
                )),
                '²' => Ok(Token::new(
                    Op(Operator::Square),
                    Span::new(start, self.cursor),
                )),
                '√' => Ok(Token::new(
                    Op(Operator::Sqrt),
                    Span::new(start, self.cursor),
                )),
                '{' => Ok(Token::new(
                    Delim(Delimiter::LBrace),
                    Span::new(start, self.cursor),
                )),
                '}' => Ok(Token::new(
                    Delim(Delimiter::RBrace),
                    Span::new(start, self.cursor),
                )),
                '(' => Ok(Token::new(
                    Delim(Delimiter::LParen),
                    Span::new(start, self.cursor),
                )),
                ')' => Ok(Token::new(
                    Delim(Delimiter::RParen),
                    Span::new(start, self.cursor),
                )),
                '[' => Ok(Token::new(
                    Delim(Delimiter::LBracket),
                    Span::new(start, self.cursor),
                )),
                ']' => Ok(Token::new(
                    Delim(Delimiter::RBracket),
                    Span::new(start, self.cursor),
                )),
                ',' => Ok(Token::new(
                    Delim(Delimiter::Comma),
                    Span::new(start, self.cursor),
                )),
                ':' => Ok(Token::new(
                    Delim(Delimiter::Colon),
                    Span::new(start, self.cursor),
                )),
                ';' => Ok(Token::new(
                    Delim(Delimiter::SemiColon),
                    Span::new(start, self.cursor),
                )),
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
    fn tokenize_newline_or_indentation(&mut self, char: char, start: u32) -> Result<Token> {
        // Eat the next char if it is a Windows-native newline.
        if char == '\r' && self.peek_char() == Some('\n') {
            self.eat_char();
        }

        let mut space_count = 0;
        let mut mixed_spaces = false;

        // Count the number of spaces and detect mixed space types.
        let mut prev_space = None;
        while matches!(self.peek_char(), Some(' ') | Some('\t')) {
            let space = self.eat_char();

            // Check if spaces match.
            if prev_space.is_some() && space != prev_space {
                mixed_spaces = true;
            }

            prev_space = space;
            space_count += 1;
        }

        let prev_space_count = self.indent_level * self.indent_size;
        let indent_diff = space_count - prev_space_count;
        let indent_diff_abs = indent_diff.abs();

        // Check if the next char is not a newline or comment delimiter.
        let peek_char = self.peek_char();
        if space_count > 0 && !matches!(peek_char, Some('\r') | Some('\n') | Some('#')) {
            // Check if spaces aren't mixed.
            if mixed_spaces {
                bail!(LexerError::new(MixedSpaces, Span::new(start, self.cursor)));
            }

            // Check spaces remain consistent between indents.
            let space_kind: IndentKind = prev_space.unwrap().try_into()?;
            if self.indent_kind != IndentKind::Unknown && space_kind != self.indent_kind {
                bail!(LexerError::new(
                    InconsistentIndent,
                    Span::new(start, self.cursor)
                ));
            }

            match (space_count - prev_space_count).cmp(&0) {
                Ordering::Greater => {
                    // An indentation.
                    // Check if it is the first indentation.
                    if self.indent_size == 0 {
                        self.indent_size = indent_diff;
                        self.indent_kind = space_kind;
                    } else if self.indent_size != indent_diff {
                        bail!(LexerError::new(
                            MixedIndentSizes,
                            Span::new(start, self.cursor)
                        ));
                    }

                    self.indent_level = space_count / self.indent_size;

                    return Ok(Token::new(Indent, Span::new(start, self.cursor)));
                }
                Ordering::Less => {
                    // A dedentation.
                    if indent_diff % self.indent_size != 0 {
                        bail!(LexerError::new(
                            InconsistentDedent,
                            Span::new(start, self.cursor)
                        ))
                    }

                    // Add dedents in token buffer except the last.
                    for _ in 1..(indent_diff_abs / self.indent_size) {
                        self.token_buffer
                            .push(Token::new(Dedent, Span::new(start, self.cursor)));
                    }

                    self.indent_level = space_count / self.indent_size;

                    return Ok(Token::new(Dedent, Span::new(start, self.cursor)));
                }
                Ordering::Equal => (),
            };
        } else if peek_char == None {
            // If the code ends with a newline, calculate dedents.
            match indent_diff.cmp(&0) {
                Ordering::Less => {
                    // Add dedents in token buffer except the last.
                    for _ in 1..(indent_diff_abs / self.indent_size) {
                        self.token_buffer
                            .push(Token::new(Dedent, Span::new(start, self.cursor)));
                    }

                    self.indent_level = space_count / self.indent_size;

                    return Ok(Token::new(Dedent, Span::new(start, self.cursor)));
                }
                _ => (),
            }
        }

        Ok(Token::new(Newline, Span::new(start, self.cursor)).into())
    }

    /// Tokenizes integers that start with `0b | 0o | 0x`.
    fn tokenize_prefixed_integer(&mut self, char: char, start: u32) -> Result<Token> {
        match char {
            'b' | 'B' => match self.peek_char() {
                Some('_' | '0' | '1') => {
                    let char = self.eat_char().unwrap();
                    let binary = self.lex_prefixed_digits(char, start, IntBase::Bin)?;
                    Ok(Token::new(
                        TokenKind::Integer(binary, IntegerKind::Bin),
                        Span::new(start, self.cursor),
                    ))
                }
                _ => bail!(LexerError::new(
                    MissingDigitPartInBinInteger,
                    Span::new(start, self.cursor)
                )),
            },
            'o' | 'O' => match self.peek_char() {
                Some('_' | '0'..='7') => {
                    let char = self.eat_char().unwrap();
                    let octal = self.lex_prefixed_digits(char, start, IntBase::Oct)?;
                    Ok(Token::new(
                        TokenKind::Integer(octal, IntegerKind::Oct),
                        Span::new(start, self.cursor),
                    ))
                }
                _ => bail!(LexerError::new(
                    MissingDigitPartInOctInteger,
                    Span::new(start, self.cursor)
                )),
            },
            'x' | 'X' => match self.peek_char() {
                Some('_' | '0'..='9' | 'a'..='f' | 'A'..='F') => {
                    let char = self.eat_char().unwrap();
                    let hex = self.lex_prefixed_digits(char, start, IntBase::Hex)?;
                    Ok(Token::new(
                        TokenKind::Integer(hex, IntegerKind::Hex),
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
    fn tokenize_leading_zero_dec_integer_or_float_or_im(&mut self, start: u32) -> Result<Token> {
        loop {
            match self.peek_char() {
                Some('_') => {
                    self.eat_char();

                    if !matches!(self.peek_char(), Some('0')) {
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
                        Ok(string) => Ok(if self.peek_string(2) == "im" {
                            self.eat_char();
                            self.eat_char();

                            Token::new(Imag(format!("0{string}")), Span::new(start, self.cursor))
                        } else {
                            Token::new(Float(format!("0{string}")), Span::new(start, self.cursor))
                        }),
                        Err(err) => Err(err),
                    };
                }
                Some('e') => {
                    self.eat_char();
                    break match self.lex_float_exponent(start) {
                        Ok(string) => Ok(if self.peek_string(2) == "im" {
                            self.eat_char();
                            self.eat_char();

                            Token::new(Imag(format!("0{string}")), Span::new(start, self.cursor))
                        } else {
                            Token::new(Float(format!("0{string}")), Span::new(start, self.cursor))
                        }),
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
                    break Ok(if self.peek_string(2) == "im" {
                        self.eat_char();
                        self.eat_char();

                        Token::new(Imag("0".into()), Span::new(start, self.cursor))
                    } else {
                        Token::new(
                            Integer("0".into(), IntegerKind::Dec),
                            Span::new(start, self.cursor),
                        )
                    });
                }
            }
        }
    }

    /// Tokenizes decimal integers or floats that start with `1-9`.
    fn tokenize_leading_non_zero_dec_integer_or_float_or_im(
        &mut self,
        mut initial_string: String,
        start: u32,
    ) -> Result<Token> {
        loop {
            match self.peek_char() {
                Some('_') => {
                    self.eat_char();

                    if !matches!(self.peek_char(), Some('0'..='9')) {
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
                        Ok(string) => Ok(if self.peek_string(2) == "im" {
                            self.eat_char();
                            self.eat_char();

                            Token::new(
                                Imag(format!("{initial_string}{string}")),
                                Span::new(start, self.cursor),
                            )
                        } else {
                            Token::new(
                                Float(format!("{initial_string}{string}")),
                                Span::new(start, self.cursor),
                            )
                        }),
                        Err(err) => Err(err),
                    };
                }
                Some('e' | 'E') => {
                    self.eat_char();
                    break match self.lex_float_exponent(start) {
                        Ok(string) => Ok(if self.peek_string(2) == "im" {
                            self.eat_char();
                            self.eat_char();

                            Token::new(
                                Imag(format!("{initial_string}{string}")),
                                Span::new(start, self.cursor),
                            )
                        } else {
                            Token::new(
                                Float(format!("{initial_string}{string}")),
                                Span::new(start, self.cursor),
                            )
                        }),
                        Err(err) => Err(err),
                    };
                }
                _ => {
                    break Ok(if self.peek_string(2) == "im" {
                        self.eat_char();
                        self.eat_char();

                        Token::new(Imag(initial_string), Span::new(start, self.cursor))
                    } else {
                        Token::new(
                            Integer(initial_string, IntegerKind::Dec),
                            Span::new(start, self.cursor),
                        )
                    });
                }
            }
        }
    }

    /// Tokenizes identifiers or keywords.
    fn tokenize_identifier_or_keyword(&mut self, mut initial_string: String, start: u32) -> Token {
        loop {
            match self.peek_char() {
                Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
                    initial_string.push(self.eat_char().unwrap());
                    continue;
                }
                _ => {
                    use crate::lexer::Keyword;
                    let kind = Keyword::try_from(&*initial_string)
                        .map(|keyword| TokenKind::Keyword(keyword))
                        .unwrap_or(TokenKind::Identifier(initial_string));

                    break Token::new(kind, Span::new(start, self.cursor));
                }
            }
        }
    }
}

/// Helper lexer functions.
impl Lexer<'_> {
    /// Lexes `"." digit_part exponent?`
    fn lex_float_fraction(&mut self, start: u32) -> Result<String> {
        let mut fraction = format!(".");

        // Lex compulsory `digit_part = digit ("_"? digit)*`
        match self.peek_char() {
            Some('0'..='9') => {
                let char = self.eat_char().unwrap();
                fraction.push_str(&self.lex_dec_digits(char, start)?)
            }
            _ => bail!(LexerError::new(
                MissingDigitPartInFloatFraction,
                Span::new(start, self.cursor)
            )),
        }

        // Lex optional `exponent = "e" ("+" | "-")? digit_part`
        if matches!(self.peek_char(), Some('e')) {
            self.eat_char();
            fraction.push_str(&self.lex_float_exponent(start)?);
        }

        Ok(fraction)
    }

    /// Lexes a short or long string.
    fn lex_short_or_long_string(&mut self, char: char, start: u32, long: bool) -> Result<String> {
        // Skip long string delimiter
        if long {
            self.eat_char().unwrap();
            self.eat_char().unwrap();
        }

        let mut string = String::new();
        loop {
            match self.peek_char() {
                Some(peek_char) => {
                    if peek_char == char {
                        // Handle delimiter.
                        self.eat_char();

                        // Check for long string delimiter.
                        if long && self.peek_string(2) != format!("{char}{char}") {
                            // Consume any incomplete delimiter to make the cursor position right.
                            if self.peek_char() == Some(char) {
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
                    } else if !long && matches!(peek_char, '\n' | '\r') {
                        // Handle newline in short string.
                        bail!(LexerError::new(
                            UnterminatedString,
                            Span::new(start, self.cursor)
                        ));
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

        Ok(string)
    }

    /// Lexes a short or long bytes.
    fn lex_short_or_long_bytes(&mut self, char: char, start: u32, long: bool) -> Result<String> {
        // Skip long string delimiter
        if long {
            self.eat_char().unwrap();
            self.eat_char().unwrap();
        }

        let mut bytes = String::new();
        loop {
            match self.peek_char() {
                Some(peek_char) => {
                    if peek_char == char {
                        // Handle delimiter.
                        self.eat_char();

                        // Check for long bytes delimiter.
                        if long && self.peek_string(2) != format!("{char}{char}") {
                            // Consume any incomplete delimiter to make the cursor position right.
                            if self.peek_char() == Some(char) {
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
                    } else if !long && matches!(peek_char, '\n' | '\r') {
                        // Handle newline in short bytes.
                        bail!(LexerError::new(
                            UnterminatedString,
                            Span::new(start, self.cursor)
                        ));
                    } else if peek_char.is_ascii() {
                        bytes.push(self.eat_char().unwrap());
                    } else {
                        bail!(LexerError::new(
                            InvalidCharacterInByteString,
                            Span::new(start, self.cursor)
                        ));
                    }
                }
                None => bail!(LexerError::new(
                    UnterminatedString,
                    Span::new(start, self.cursor)
                )),
            }
        }

        Ok(bytes)
    }

    /// Lexes `"e" ("+" | "-")? digit_part`
    fn lex_float_exponent(&mut self, start: u32) -> Result<String> {
        let mut exponent = format!("e");

        // Lex ("+" | "-")?
        if matches!(self.peek_char(), Some('-' | '+')) {
            exponent.push(self.eat_char().unwrap());
        } else {
            exponent.push('+');
        }

        // Lex compulsory digit_part = digit ("_"? digit)*
        match self.peek_char() {
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
            match self.peek_char() {
                Some('_') => {
                    self.eat_char();

                    if !matches!(self.peek_char(), Some('0'..='9')) {
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
            match self.peek_char() {
                Some('_') => {
                    self.eat_char();

                    let peek_char = self.peek_char();
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
