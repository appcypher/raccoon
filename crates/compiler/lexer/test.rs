use crate::{
    lexer::{Token, TokenKind},
    span::Span,
};

use super::{LexerError, LexerErrorKind::*};

use utils::*;

#[test]
fn can_skip_newlines() {
    let result_macos: Vec<_> = get_tokens("\r");
    let result_windows: Vec<_> = get_tokens("\r\n");
    let result_unix: Vec<_> = get_tokens("\n");
    let result_mixed: Vec<_> = get_tokens("\r\r\n\n");

    assert_eq!(result_macos, vec![]);

    assert_eq!(result_windows, vec![]);

    assert_eq!(result_unix, vec![]);

    assert_eq!(result_mixed, vec![]);
}

#[test]
fn can_skip_comments() {
    let result_eof = get_tokens("# This is a comment");
    let result_newline = get_tokens("# This is a comment\n");

    assert_eq!(result_eof, vec![]);

    assert_eq!(result_newline, vec![]);
}

#[test]
fn can_skip_useless_spaces() {
    let result_eof = get_tokens("\t  \t");
    let result_newline = get_tokens(" \t\t\n");

    assert_eq!(result_eof, vec![]);

    assert_eq!(result_newline, vec![]);
}

#[test]
fn can_skip_line_continuation() {
    let result_simple = get_tokens("\\\n");
    let result_complex = get_tokens("\r\n\\\r\n");
    let result_not_followed_by_newline = get_tokens("\r\n\\");

    assert_eq!(result_simple, vec![]);

    assert_eq!(result_complex, vec![]);

    assert_eq!(
        result_not_followed_by_newline,
        vec![TokenResult::Err(LexerError::new(
            InvalidLineContinuationEscapeSequence,
            Span::new(2, 3)
        ))]
    );
}

// // TODO(appcypher): Write indentation when more lex rules are implemented.
// #[test]
// fn can_tokenize_indentations() {
//     assert!(false)
// }

// // TODO(appcypher): When there is more lex rules implemented.
// #[test]
// fn can_partially_tokenize() {
//     assert!(false)
// }

#[test]
fn can_tokenize_short_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"'hello there!'"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"'hello\tthere\r\nnew\\world!'"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#""hello there!""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#""hello\tthere\r\nnew\\world!""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string()),
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello\tthere\r\nnew\\world!".to_string()),
            Span::new(0, 29)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string()),
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello\tthere\r\nnew\\world!".to_string()),
            Span::new(0, 29)
        ))]
    );
}

#[test]
fn can_tokenize_long_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"'''hello\tthere\r\nnew\\world!'''"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#""""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#""""hello\tthere\r\nnew\\world!""""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string()),
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello\tthere\r\nnew\\world!".to_string()),
            Span::new(0, 33)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string()),
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello\tthere\r\nnew\\world!".to_string()),
            Span::new(0, 33)
        ))]
    );
}

mod utils {
    use anyhow::Result;

    use crate::lexer::{Lexer, LexerError, Token};

    // This is a custom result type for testing purpose. `anyhow::Result` does not implement PartialEq so it cannot be used in tests.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) enum TokenResult {
        Ok(Token),
        Err(LexerError),
    }

    pub(super) fn get_tokens(code: &str) -> Vec<TokenResult> {
        Lexer::tokenize(code)
            .into_iter()
            .map(map_token_result)
            .collect()
    }

    fn map_token_result(result: Result<Token>) -> TokenResult {
        match result {
            Ok(token) => TokenResult::Ok(token),
            Err(error) => TokenResult::Err(error.downcast().unwrap()),
        }
    }
}
