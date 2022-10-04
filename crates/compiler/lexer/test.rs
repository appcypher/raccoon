use crate::{
    lexer::{BytesKind, Keyword, StringKind, Token, TokenKind, IntegerKind},
    span::Span,
};

use super::{LexerError, LexerErrorKind::*};

use utils::*;

#[test]
fn can_tokenize_newlines() {
    let result_macos: Vec<_> = get_tokens("\r");
    let result_windows: Vec<_> = get_tokens("\r\n");
    let result_unix: Vec<_> = get_tokens("\n");
    let result_mixed: Vec<_> = get_tokens("\r\r\n\n");

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

#[test]
fn can_skip_comments() {
    let result_eof = get_tokens("# This is a comment");
    let result_newline = get_tokens("\n# This is a comment\n");

    assert_eq!(result_eof, vec![]);

    assert_eq!(
        result_newline,
        vec![
            TokenResult::Ok(Token::new(TokenKind::Newline, Span::new(0, 1))),
            TokenResult::Ok(Token::new(TokenKind::Newline, Span::new(20, 21)))
        ]
    );
}

#[test]
fn can_skip_useless_spaces() {
    let result_eof = get_tokens("\t  \t");
    let result_newline = get_tokens(" \t\t\n");

    assert_eq!(result_eof, vec![]);

    assert_eq!(
        result_newline,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Newline,
            Span::new(3, 4)
        ))]
    );
}

#[test]
fn can_skip_line_continuation() {
    let result_simple = get_tokens("\\\n");
    let result_complex = get_tokens("\r\n\\\r\n");

    // Failures

    let result_missing_consecutive_newline = get_tokens("\r\n\\");

    assert_eq!(result_simple, vec![]);

    assert_eq!(
        result_complex,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Newline,
            Span::new(0, 2)
        ))]
    );

    assert_eq!(
        result_missing_consecutive_newline,
        vec![
            TokenResult::Ok(Token::new(TokenKind::Newline, Span::new(0, 2))),
            TokenResult::Err(LexerError::new(
                InvalidLineContinuationEscapeSequence,
                Span::new(2, 3)
            ))
        ]
    );
}

#[test]
fn can_tokenize_short_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"'hello there!'"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"'hello\tthere\r\nnew\\world!'"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#""hello there!""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#""hello\tthere\r\nnew\\world!""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"'hello there!"#);
    let result_unterminated_single_quote_string_2 = get_tokens("'hello there!\n");

    let result_unterminated_double_quote_string = get_tokens(r#""hello there!"#);
    let result_unterminated_double_quote_string_2 = get_tokens("\"hello there!\n");

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Str),
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(r"hello\tthere\r\nnew\\world!".to_string(), StringKind::Str),
            Span::new(0, 29)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Str),
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(r"hello\tthere\r\nnew\\world!".to_string(), StringKind::Str),
            Span::new(0, 29)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 13)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 13)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 13)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 13)
        ))]
    );
}

#[test]
fn can_tokenize_long_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"'''hello\tthere\r\nnew\\world!'''"#);
    let result_single_quote_string_with_newlines: Vec<_> =
        get_tokens("'''hello there\n\r\nnew world!'''");

    let result_double_quote_string: Vec<_> = get_tokens(r#""""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#""""hello\tthere\r\nnew\\world!""""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"'''hello there!''"#);
    let result_unterminated_single_quote_string_2 = get_tokens(r#"'''hello there!'"#);
    let result_unterminated_double_quote_string = get_tokens(r#""""hello there!"""#);
    let result_unterminated_double_quote_string_2 = get_tokens(r#""""hello there!""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Str),
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(r"hello\tthere\r\nnew\\world!".to_string(), StringKind::Str),
            Span::new(0, 33)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_newlines,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there\n\r\nnew world!".to_string(), StringKind::Str),
            Span::new(0, 30)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Str),
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(r"hello\tthere\r\nnew\\world!".to_string(), StringKind::Str),
            Span::new(0, 33)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 16)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 16)
        ))]
    );
}

#[test]
fn can_tokenize_short_raw_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"r'hello there!'"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"r'hello\tthere\r\nnew\\world!'"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#"r"hello there!""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"r"hello\tthere\r\nnew\\world!""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"r'hello there!"#);
    let result_unterminated_single_quote_string_2 = get_tokens("r'hello there!\n");

    let result_unterminated_double_quote_string = get_tokens(r#"r"hello there!"#);
    let result_unterminated_double_quote_string_2 = get_tokens("r\"hello there!\n");

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawStr),
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawStr
            ),
            Span::new(0, 30)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawStr),
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawStr
            ),
            Span::new(0, 30)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );
}

#[test]
fn can_tokenize_long_raw_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"r'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"r'''hello\tthere\r\nnew\\world!'''"#);
    let result_single_quote_string_with_newlines: Vec<_> =
        get_tokens("r'''hello there\n\r\nnew world!'''");

    let result_double_quote_string: Vec<_> = get_tokens(r#"r"""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"r"""hello\tthere\r\nnew\\world!""""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"r'''hello there!''"#);
    let result_unterminated_single_quote_string_2 = get_tokens(r#"r'''hello there!'"#);
    let result_unterminated_double_quote_string = get_tokens(r#"r"""hello there!"""#);
    let result_unterminated_double_quote_string_2 = get_tokens(r#"r"""hello there!""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawStr),
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawStr
            ),
            Span::new(0, 34)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_newlines,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                "hello there\n\r\nnew world!".to_string(),
                StringKind::RawStr
            ),
            Span::new(0, 31)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawStr),
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawStr
            ),
            Span::new(0, 34)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );
}

#[test]
fn can_tokenize_short_format_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"f'hello there!'"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"f'hello\tthere\r\nnew\\world!'"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#"f"hello there!""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"f"hello\tthere\r\nnew\\world!""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"f'hello there!"#);
    let result_unterminated_single_quote_string_2 = get_tokens("f'hello there!\n");

    let result_unterminated_double_quote_string = get_tokens(r#"f"hello there!"#);
    let result_unterminated_double_quote_string_2 = get_tokens("f\"hello there!\n");

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Format),
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::Format
            ),
            Span::new(0, 30)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Format),
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::Format
            ),
            Span::new(0, 30)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );
}

#[test]
fn can_tokenize_long_format_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"f'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"f'''hello\tthere\r\nnew\\world!'''"#);
    let result_single_quote_string_with_newlines: Vec<_> =
        get_tokens("f'''hello there\n\r\nnew world!'''");

    let result_double_quote_string: Vec<_> = get_tokens(r#"f"""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"f"""hello\tthere\r\nnew\\world!""""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"f'''hello there!''"#);
    let result_unterminated_single_quote_string_2 = get_tokens(r#"f'''hello there!'"#);
    let result_unterminated_double_quote_string = get_tokens(r#"f"""hello there!"""#);
    let result_unterminated_double_quote_string_2 = get_tokens(r#"f"""hello there!""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Format),
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::Format
            ),
            Span::new(0, 34)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_newlines,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                "hello there\n\r\nnew world!".to_string(),
                StringKind::Format
            ),
            Span::new(0, 31)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::Format),
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::Format
            ),
            Span::new(0, 34)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );
}

#[test]
fn can_tokenize_short_raw_format_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"rf'hello there!'"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rf'hello\tthere\r\nnew\\world!'"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#"rf"hello there!""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rf"hello\tthere\r\nnew\\world!""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"rf'hello there!"#);
    let result_unterminated_single_quote_string_2 = get_tokens("rf'hello there!\n");

    let result_unterminated_double_quote_string = get_tokens(r#"rf"hello there!"#);
    let result_unterminated_double_quote_string_2 = get_tokens("rf\"hello there!\n");

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawFormat),
            Span::new(0, 16)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 31)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawFormat),
            Span::new(0, 16)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 31)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );
}

#[test]
fn can_tokenize_long_raw_format_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"rf'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rf'''hello\tthere\r\nnew\\world!'''"#);
    let result_single_quote_string_with_newlines: Vec<_> =
        get_tokens("rf'''hello there\n\r\nnew world!'''");

    let result_double_quote_string: Vec<_> = get_tokens(r#"rf"""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rf"""hello\tthere\r\nnew\\world!""""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"rf'''hello there!''"#);
    let result_unterminated_single_quote_string_2 = get_tokens(r#"rf'''hello there!'"#);
    let result_unterminated_double_quote_string = get_tokens(r#"rf"""hello there!"""#);
    let result_unterminated_double_quote_string_2 = get_tokens(r#"rf"""hello there!""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawFormat),
            Span::new(0, 20)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 35)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_newlines,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                "hello there\n\r\nnew world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 32)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawFormat),
            Span::new(0, 20)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 35)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );
}

#[test]
fn can_tokenize_short_raw_byte_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"rb'hello there!'"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rb'hello\tthere\r\nnew\\world!'"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#"rb"hello there!""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rb"hello\tthere\r\nnew\\world!""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"rb'hello there!"#);
    let result_unterminated_single_quote_string_2 = get_tokens("rb'hello there!\n");

    let result_unterminated_double_quote_string = get_tokens(r#"rb"hello there!"#);
    let result_unterminated_double_quote_string_2 = get_tokens("rb\"hello there!\n");

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr("hello there!".to_string(), BytesKind::RawBytes),
            Span::new(0, 16)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr(
                r"hello\tthere\r\nnew\\world!".to_string(),
                BytesKind::RawBytes
            ),
            Span::new(0, 31)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr("hello there!".to_string(), BytesKind::RawBytes),
            Span::new(0, 16)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr(
                r"hello\tthere\r\nnew\\world!".to_string(),
                BytesKind::RawBytes
            ),
            Span::new(0, 31)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 15)
        ))]
    );
}

#[test]
fn can_tokenize_long_raw_byte_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"rf'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rf'''hello\tthere\r\nnew\\world!'''"#);
    let result_single_quote_string_with_newlines: Vec<_> =
        get_tokens("rf'''hello there\n\r\nnew world!'''");

    let result_double_quote_string: Vec<_> = get_tokens(r#"rf"""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"rf"""hello\tthere\r\nnew\\world!""""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"rf'''hello there!''"#);
    let result_unterminated_single_quote_string_2 = get_tokens(r#"rf'''hello there!'"#);
    let result_unterminated_double_quote_string = get_tokens(r#"rf"""hello there!"""#);
    let result_unterminated_double_quote_string_2 = get_tokens(r#"rf"""hello there!""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawFormat),
            Span::new(0, 20)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 35)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_newlines,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                "hello there\n\r\nnew world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 32)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str("hello there!".to_string(), StringKind::RawFormat),
            Span::new(0, 20)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Str(
                r"hello\tthere\r\nnew\\world!".to_string(),
                StringKind::RawFormat
            ),
            Span::new(0, 35)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );
}

#[test]
fn can_tokenize_short_byte_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"b'hello there!'"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"b'hello\tthere\r\nnew\\world!'"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#"b"hello there!""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"b"hello\tthere\r\nnew\\world!""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"b'hello there!"#);
    let result_unterminated_single_quote_string_2 = get_tokens("b'hello there!\n");

    let result_unterminated_double_quote_string = get_tokens(r#"b"hello there!"#);
    let result_unterminated_double_quote_string_2 = get_tokens("b\"hello there!\n");

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr("hello there!".to_string(), BytesKind::Bytes),
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr(r"hello\tthere\r\nnew\\world!".to_string(), BytesKind::Bytes),
            Span::new(0, 30)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr("hello there!".to_string(), BytesKind::Bytes),
            Span::new(0, 15)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr(r"hello\tthere\r\nnew\\world!".to_string(), BytesKind::Bytes),
            Span::new(0, 30)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 14)
        ))]
    );
}

#[test]
fn can_tokenize_long_byte_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"b'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"b'''hello\tthere\r\nnew\\world!'''"#);
    let result_single_quote_string_with_newlines: Vec<_> =
        get_tokens("b'''hello there\n\r\nnew world!'''");

    let result_double_quote_string: Vec<_> = get_tokens(r#"b"""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"b"""hello\tthere\r\nnew\\world!""""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"b'''hello there!''"#);
    let result_unterminated_single_quote_string_2 = get_tokens(r#"b'''hello there!'"#);
    let result_unterminated_double_quote_string = get_tokens(r#"b"""hello there!"""#);
    let result_unterminated_double_quote_string_2 = get_tokens(r#"b"""hello there!""#);

    assert_eq!(
        result_single_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr("hello there!".to_string(), BytesKind::Bytes),
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr(r"hello\tthere\r\nnew\\world!".to_string(), BytesKind::Bytes),
            Span::new(0, 34)
        ))]
    );

    assert_eq!(
        result_single_quote_string_with_newlines,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr("hello there\n\r\nnew world!".to_string(), BytesKind::Bytes),
            Span::new(0, 31)
        ))]
    );

    assert_eq!(
        result_double_quote_string,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr("hello there!".to_string(), BytesKind::Bytes),
            Span::new(0, 19)
        ))]
    );

    assert_eq!(
        result_double_quote_string_with_escape_seq,
        vec![TokenResult::Ok(Token::new(
            TokenKind::ByteStr(r"hello\tthere\r\nnew\\world!".to_string(), BytesKind::Bytes),
            Span::new(0, 34)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_single_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 18)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string_2,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );
}

#[test]
fn can_tokenize_floats() {
    let result_leading_zero = get_tokens("0.1_23");
    let result_leading_zero_with_exponent = get_tokens("0.1_23e4_5");
    let result_leading_zero_with_negative_exponent = get_tokens("0.1_23e-4_5");
    let result_leading_zero_with_positive_exponent = get_tokens("0.1_23e+4_5");
    let result_leading_zero_without_fraction = get_tokens("0e+4_5");

    let result_leading_dot = get_tokens(".1_23");
    let result_leading_dot_with_exponent = get_tokens(".1_23e4_5");
    let result_leading_dot_with_negative_exponent = get_tokens(".1_23e-4_5");
    let result_leading_dot_with_positive_exponent = get_tokens(".1_23e+4_5");

    let result_leading_non_zero_digit = get_tokens("12_3.4_56e+7_8");
    let result_leading_non_zero_digit_no_fraction_with_positive_exponent = get_tokens("1_23e+4_5");
    let result_leading_non_zero_digit_no_fraction_with_negative_exponent = get_tokens("12_3e-4_5");

    // Failures

    let result_leading_zero_incomplete_exponent = get_tokens("0.1_23e");
    let result_leading_zero_incomplete_exponent_with_sign = get_tokens("0.1_23e+");
    let result_leading_zero_incomplete_fraction = get_tokens("0.e-1");
    let result_leading_zero_incomplete_fraction_with_underscore = get_tokens("0._e-1");
    let result_leading_zero_incomplete_exponent_without_fraction = get_tokens("0e");
    let result_leading_zero_incomplete_exponent_with_sign_without_fraction = get_tokens("0e-");

    let result_leading_non_zero_incomplete_exponent = get_tokens("1_2.3_45e");
    let result_leading_non_zero_incomplete_exponent_with_sign = get_tokens("1_2.3_45e-");
    let result_leading_non_zero_incomplete_fraction = get_tokens("1_2.e7_8");
    let result_leading_non_zero_incomplete_fraction_with_underscore = get_tokens("1_2._e7_8");
    let result_leading_non_zero_incomplete_exponent_without_fraction = get_tokens("1_23e");
    let result_leading_non_zero_incomplete_exponent_with_sign_without_fraction =
        get_tokens("1_23e+");

    assert_eq!(
        result_leading_zero,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123".to_string()),
            Span::new(0, 6)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123e+45".to_string()),
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_negative_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123e-45".to_string()),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_positive_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123e+45".to_string()),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_leading_zero_without_fraction,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0e+45".to_string()),
            Span::new(0, 6)
        ))]
    );

    assert_eq!(
        result_leading_dot,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123".to_string()),
            Span::new(0, 5)
        ))]
    );

    assert_eq!(
        result_leading_dot_with_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123e+45".to_string()),
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_dot_with_negative_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123e-45".to_string()),
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_leading_dot_with_positive_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("0.123e+45".to_string()),
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_digit,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("123.456e+78".to_string()),
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_digit_no_fraction_with_positive_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("123e+45".to_string()),
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_digit_no_fraction_with_negative_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Float("123e-45".to_string()),
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_zero_incomplete_exponent,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 7)
        ))]
    );

    assert_eq!(
        result_leading_zero_incomplete_exponent_with_sign,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 8)
        ))]
    );

    assert_eq!(
        result_leading_zero_incomplete_fraction,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatFraction,
            Span::new(0, 2)
        ))]
    );

    assert_eq!(
        result_leading_zero_incomplete_fraction_with_underscore,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatFraction,
            Span::new(0, 2)
        ))]
    );

    assert_eq!(
        result_leading_zero_incomplete_exponent_without_fraction,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 2)
        ))]
    );

    assert_eq!(
        result_leading_zero_incomplete_exponent_with_sign_without_fraction,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 3)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_incomplete_exponent,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_incomplete_exponent_with_sign,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_incomplete_fraction,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatFraction,
            Span::new(0, 4)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_incomplete_fraction_with_underscore,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatFraction,
            Span::new(0, 4)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_incomplete_exponent_without_fraction,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 5)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_incomplete_exponent_with_sign_without_fraction,
        vec![TokenResult::Err(LexerError::new(
            MissingDigitPartInFloatExponent,
            Span::new(0, 6)
        ))]
    );
}

#[test]
fn can_tokenize_prefixed_integers() {
    let result_bin = get_tokens("0b1010_1010");
    let result_bin_leading_underscore = get_tokens("0b_1010_1010");

    let result_oct = get_tokens("0o123_456_7");
    let result_oct_leading_underscore = get_tokens("0o_123_456_7");

    let result_hex = get_tokens("0x1234_5678_9abc_def0");
    let result_hex_leading_underscore = get_tokens("0x_1234_5678_9abc_def0");
    let result_hex_uppercase = get_tokens("0X1234_5678_9ABC_DEF0");

    let result_bin_invalid_digits = get_tokens("0b1234");
    let result_bin_multiple_underscores = get_tokens("0b1010__1010");

    let result_oct_invalid_digits = get_tokens("0o1238");
    let result_oct_multiple_underscores = get_tokens("0o123_456__7");

    let result_hex_multiple_underscores = get_tokens("0x1234_5678__9abc_def0");

    assert_eq!(
        result_bin,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("10101010".to_string(), IntegerKind::Bin),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_bin_leading_underscore,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("10101010".to_string(), IntegerKind::Bin),
            Span::new(0, 12)
        ))]
    );

    assert_eq!(
        result_oct,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("1234567".to_string(), IntegerKind::Oct),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_oct_leading_underscore,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("1234567".to_string(), IntegerKind::Oct),
            Span::new(0, 12)
        ))]
    );

    assert_eq!(
        result_hex,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("123456789abcdef0".to_string(), IntegerKind::Hex),
            Span::new(0, 21)
        ))]
    );

    assert_eq!(
        result_hex_leading_underscore,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("123456789abcdef0".to_string(), IntegerKind::Hex),
            Span::new(0, 22)
        ))]
    );

    assert_eq!(
        result_hex_uppercase,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("123456789ABCDEF0".to_string(), IntegerKind::Hex),
            Span::new(0, 21)
        ))]
    );

    assert_eq!(
        result_bin_invalid_digits,
        vec![TokenResult::Err(LexerError::new(
            InvalidDigitInInteger,
            Span::new(0, 3)
        ))]
    );

    assert_eq!(
        result_bin_multiple_underscores,
        vec![TokenResult::Err(LexerError::new(
            InvalidCharacterAfterUnderscoreInDigitPart,
            Span::new(0, 7)
        ))]
    );

    assert_eq!(
        result_oct_invalid_digits,
        vec![TokenResult::Err(LexerError::new(
            InvalidDigitInInteger,
            Span::new(0, 5)
        ))]
    );

    assert_eq!(
        result_oct_multiple_underscores,
        vec![TokenResult::Err(LexerError::new(
            InvalidCharacterAfterUnderscoreInDigitPart,
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_hex_multiple_underscores,
        vec![TokenResult::Err(LexerError::new(
            InvalidCharacterAfterUnderscoreInDigitPart,
            Span::new(0, 12)
        ))]
    );
}

#[test]
fn can_tokenize_dec_integers() {
    let result_leading_zero = get_tokens("0_000_000");

    let result_leading_non_zero = get_tokens("1_234_567_890");

    let result_leading_zero_with_trailing_underscore = get_tokens("0_a");
    let result_leading_zero_with_non_zero_digit = get_tokens("01");

    // Failures

    let result_leading_non_zero_with_trailing_underscore = get_tokens("1_");
    let result_leading_non_zero_with_trailing_underscore_2 = get_tokens("1_234_h");

    assert_eq!(
        result_leading_zero,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("0".to_string(), IntegerKind::Dec),
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_non_zero,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Integer("1234567890".to_string(), IntegerKind::Dec),
            Span::new(0, 13)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_trailing_underscore,
        vec![TokenResult::Err(LexerError::new(
            InvalidCharacterAfterUnderscoreInDigitPart,
            Span::new(0, 2)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_non_zero_digit,
        vec![TokenResult::Err(LexerError::new(
            InvalidLeadingZeroInDecInteger,
            Span::new(0, 1)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_with_trailing_underscore,
        vec![TokenResult::Err(LexerError::new(
            InvalidCharacterAfterUnderscoreInDigitPart,
            Span::new(0, 2)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_with_trailing_underscore_2,
        vec![TokenResult::Err(LexerError::new(
            InvalidCharacterAfterUnderscoreInDigitPart,
            Span::new(0, 6)
        ))]
    );
}

#[test]
fn can_tokenize_identifiers() {
    let result_no_string_prefix_conflict =
        get_tokens("f fa f1 f_ fa_1 b b1 b_ b_1 r r1 r_ r_1 rf rf1 rf_ rf_1 rb rb1 rb_ rb_1");
    let result_leading_underscore = get_tokens("_ _1 _a _a1 _a_ _a_1");
    let result_with_numbers = get_tokens("a1234567890 a_1234567890 a12_345__67890");
    let result_valid_characters =
        get_tokens("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_");

    assert_eq!(
        result_no_string_prefix_conflict,
        vec![
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("f".to_string()),
                Span::new(0, 1)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("fa".to_string()),
                Span::new(2, 4)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("f1".to_string()),
                Span::new(5, 7)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("f_".to_string()),
                Span::new(8, 10)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("fa_1".to_string()),
                Span::new(11, 15)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("b".to_string()),
                Span::new(16, 17)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("b1".to_string()),
                Span::new(18, 20)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("b_".to_string()),
                Span::new(21, 23)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("b_1".to_string()),
                Span::new(24, 27)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("r".to_string()),
                Span::new(28, 29)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("r1".to_string()),
                Span::new(30, 32)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("r_".to_string()),
                Span::new(33, 35)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("r_1".to_string()),
                Span::new(36, 39)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rf".to_string()),
                Span::new(40, 42)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rf1".to_string()),
                Span::new(43, 46)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rf_".to_string()),
                Span::new(47, 50)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rf_1".to_string()),
                Span::new(51, 55)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rb".to_string()),
                Span::new(56, 58)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rb1".to_string()),
                Span::new(59, 62)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rb_".to_string()),
                Span::new(63, 66)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("rb_1".to_string()),
                Span::new(67, 71)
            )),
        ]
    );

    assert_eq!(
        result_leading_underscore,
        vec![
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("_".to_string()),
                Span::new(0, 1)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("_1".to_string()),
                Span::new(2, 4)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("_a".to_string()),
                Span::new(5, 7)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("_a1".to_string()),
                Span::new(8, 11)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("_a_".to_string()),
                Span::new(12, 15)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("_a_1".to_string()),
                Span::new(16, 20)
            )),
        ]
    );

    assert_eq!(
        result_with_numbers,
        vec![
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("a1234567890".to_string()),
                Span::new(0, 11)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("a_1234567890".to_string()),
                Span::new(12, 24)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Identifier("a12_345__67890".to_string()),
                Span::new(25, 39)
            )),
        ]
    );

    assert_eq!(
        result_valid_characters,
        vec![TokenResult::Ok(Token::new(
            TokenKind::Identifier(
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_".to_string()
            ),
            Span::new(0, 63)
        )),]
    );
}

#[test]
fn can_tokenize_keywords() {
    let result_valid_keywords = get_tokens("and as assert async await break class const continue def del elif else enum except false finally for from global if import in interface is lambda let macro match mut nonlocal not or pass ptr raise ref return true try typealias val var where while with yield");

    assert_eq!(
        result_valid_keywords,
        vec![
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::And),
                Span::new(0, 3)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::As),
                Span::new(4, 6)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Assert),
                Span::new(7, 13)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Async),
                Span::new(14, 19)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Await),
                Span::new(20, 25)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Break),
                Span::new(26, 31)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Class),
                Span::new(32, 37)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Const),
                Span::new(38, 43)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Continue),
                Span::new(44, 52)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Def),
                Span::new(53, 56)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Del),
                Span::new(57, 60)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Elif),
                Span::new(61, 65)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Else),
                Span::new(66, 70)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Enum),
                Span::new(71, 75)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Except),
                Span::new(76, 82)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::False),
                Span::new(83, 88)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Finally),
                Span::new(89, 96)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::For),
                Span::new(97, 100)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::From),
                Span::new(101, 105)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Global),
                Span::new(106, 112)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::If),
                Span::new(113, 115)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Import),
                Span::new(116, 122)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::In),
                Span::new(123, 125)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Interface),
                Span::new(126, 135)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Is),
                Span::new(136, 138)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Lambda),
                Span::new(139, 145)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Let),
                Span::new(146, 149)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Macro),
                Span::new(150, 155)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Match),
                Span::new(156, 161)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Mut),
                Span::new(162, 165)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Nonlocal),
                Span::new(166, 174)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Not),
                Span::new(175, 178)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Or),
                Span::new(179, 181)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Pass),
                Span::new(182, 186)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Ptr),
                Span::new(187, 190)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Raise),
                Span::new(191, 196)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Ref),
                Span::new(197, 200)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Return),
                Span::new(201, 207)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::True),
                Span::new(208, 212)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Try),
                Span::new(213, 216)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Typealias),
                Span::new(217, 226)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Val),
                Span::new(227, 230)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Var),
                Span::new(231, 234)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Where),
                Span::new(235, 240)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::While),
                Span::new(241, 246)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::With),
                Span::new(247, 251)
            )),
            TokenResult::Ok(Token::new(
                TokenKind::Keyword(Keyword::Yield),
                Span::new(252, 257)
            )),
        ]
    );
}

// // TODO(appcypher): Write indentation when more lex rules are implemented.
// #[test]
// fn can_tokenize_indentations() {
//     todo!()
// }

// // TODO(appcypher): When there is more lex rules implemented.
// #[test]
// fn can_partially_tokenize() {
//     todo!()
// }

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
        Lexer::tokenize(code).map(map_token_result).collect()
    }

    fn map_token_result(result: Result<Token>) -> TokenResult {
        match result {
            Ok(token) => TokenResult::Ok(token),
            Err(error) => TokenResult::Err(error.downcast().unwrap()),
        }
    }
}
