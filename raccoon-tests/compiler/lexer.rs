use anyhow::Result;
use raccoon_compiler::{
    lexer::{Lexer, LexerError, LexerErrorKind::*, Token, TokenKind::*},
    span::Span,
};

// This is a custom result type for testing purpose.
// anyhow::Result does not implement PartialEq so it cannot be used in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenResult {
    Ok(Token),
    Err(LexerError),
}

fn map_token_result(result: Result<Token>) -> TokenResult {
    match result {
        Ok(token) => TokenResult::Ok(token),
        Err(error) => TokenResult::Err(error.downcast().unwrap()),
    }
}

fn get_tokens(code: &str) -> Vec<TokenResult> {
    Lexer::tokenize(code)
        .into_iter()
        .map(map_token_result)
        .collect()
}

#[test]
fn can_tokenize_newlines() {
    let result_macos: Vec<_> = get_tokens("\r");
    let result_windows: Vec<_> = get_tokens("\r\n");
    let result_unix: Vec<_> = get_tokens("\n");
    let result_mixed: Vec<_> = get_tokens("\r\r\n\n");

    assert_eq!(
        result_macos,
        vec![TokenResult::Ok(Token::new(Newline, Span::new(0, 1)))]
    );

    assert_eq!(
        result_windows,
        vec![TokenResult::Ok(Token::new(Newline, Span::new(0, 2)))]
    );

    assert_eq!(
        result_unix,
        vec![TokenResult::Ok(Token::new(Newline, Span::new(0, 1)))]
    );

    assert_eq!(
        result_mixed,
        vec![
            TokenResult::Ok(Token::new(Newline, Span::new(0, 1))),
            TokenResult::Ok(Token::new(Newline, Span::new(1, 3))),
            TokenResult::Ok(Token::new(Newline, Span::new(3, 4)))
        ]
    );
}

#[test]
fn can_skip_comments() {
    let result_eof = get_tokens("# This is a comment");
    let result_newline = get_tokens("# This is a comment\n");

    assert_eq!(result_eof, vec![]);

    assert_eq!(
        result_newline,
        vec![TokenResult::Ok(Token::new(Newline, Span::new(19, 20)))]
    );
}

#[test]
fn can_skip_useless_spaces() {
    let result_eof = get_tokens("\t  \t");
    let result_newline = get_tokens(" \t\t\n");

    assert_eq!(result_eof, vec![]);

    assert_eq!(
        result_newline,
        vec![TokenResult::Ok(Token::new(Newline, Span::new(3, 4)))]
    );
}

#[test]
fn can_skip_line_continuation() {
    let result_simple = get_tokens("\\\n");
    let result_complex = get_tokens("\r\n\\\r\n");
    let result_not_followed_by_newline = get_tokens("\r\n\\");

    assert_eq!(
        result_simple,
        vec![TokenResult::Ok(Token::new(Newline, Span::new(1, 2)))]
    );

    assert_eq!(
        result_complex,
        vec![
            TokenResult::Ok(Token::new(Newline, Span::new(0, 2))),
            TokenResult::Ok(Token::new(Newline, Span::new(3, 5)))
        ]
    );

    assert_eq!(
        result_not_followed_by_newline,
        vec![
            TokenResult::Ok(Token::new(Newline, Span::new(0, 2))),
            TokenResult::Err(LexerError::new(
                InvalidLineContinuationEscapeSequence,
                Span::new(2, 3)
            ))
        ]
    );
}

// TODO(appcypher): Write indentation when more lex rules are implemented.
#[test]
fn can_tokenize_indentations() {}

// TODO(appcypher): When there is more lex rules implemented.
#[test]
fn can_partially_tokenize() {}
