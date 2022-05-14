use anyhow::Result;
use raccoon_compiler::{
    lexer::{Lexer, LexerError, Token, TokenKind},
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

#[test]
fn tokenize_newlines() {
    let result_macos: Vec<_> = {
        let tokens = Lexer::tokenize("\r");
        tokens.into_iter().map(map_token_result).collect()
    };

    let result_windows: Vec<_> = {
        let tokens = Lexer::tokenize("\r\n");
        tokens.into_iter().map(map_token_result).collect()
    };

    let result_unix: Vec<_> = {
        let tokens = Lexer::tokenize("\n");
        tokens.into_iter().map(map_token_result).collect()
    };

    let result_mixed: Vec<_> = {
        let tokens = Lexer::tokenize("\r\r\n\n");
        tokens.into_iter().map(map_token_result).collect()
    };

    assert_eq!(
        result_macos,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Newline,
            Span::new(0, 1)
        ))]
    );

    assert_eq!(
        result_windows,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Newline,
            Span::new(0, 2)
        ))]
    );

    assert_eq!(
        result_unix,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Newline,
            Span::new(0, 1)
        ))]
    );

    assert_eq!(
        result_mixed,
        vec![
            TokenResult::Ok(Token::new(TokenKind::Newline, Span::new(0, 1))),
            TokenResult::Ok(Token::new(TokenKind::Newline, Span::new(1, 3))),
            TokenResult::Ok(Token::new(TokenKind::Newline, Span::new(3, 4)))
        ]
    );
}
