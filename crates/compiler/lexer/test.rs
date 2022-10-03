use crate::{
    lexer::{Token, TokenKind},
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
    let result_unterminated_double_quote_string = get_tokens(r#""hello there!"#);

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

    assert_eq!(
        result_unterminated_single_quote_string,
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
}

#[test]
fn can_tokenize_long_strings() {
    let result_single_quote_string: Vec<_> = get_tokens(r#"'''hello there!'''"#);
    let result_single_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#"'''hello\tthere\r\nnew\\world!'''"#);

    let result_double_quote_string: Vec<_> = get_tokens(r#""""hello there!""""#);
    let result_double_quote_string_with_escape_seq: Vec<_> =
        get_tokens(r#""""hello\tthere\r\nnew\\world!""""#);

    // Failures

    let result_unterminated_single_quote_string = get_tokens(r#"'''hello there!''"#);
    let result_unterminated_double_quote_string = get_tokens(r#""""hello there!"""#);

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

    assert_eq!(
        result_unterminated_single_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );

    assert_eq!(
        result_unterminated_double_quote_string,
        vec![TokenResult::Err(LexerError::new(
            UnterminatedString,
            Span::new(0, 17)
        ))]
    );
}

#[test]
fn can_tokenize_floats() {
    let result_leading_zero = get_tokens("0.1_23");
    let result_leading_zero_with_exponent = get_tokens("0.1_23E4_5");
    let result_leading_zero_with_negative_exponent = get_tokens("0.1_23e-4_5");
    let result_leading_zero_with_positive_exponent = get_tokens("0.1_23e+4_5");
    let result_leading_zero_without_fraction = get_tokens("0e+4_5");

    let result_leading_dot = get_tokens(".1_23");
    let result_leading_dot_with_exponent = get_tokens(".1_23E4_5");
    let result_leading_dot_with_negative_exponent = get_tokens(".1_23e-4_5");
    let result_leading_dot_with_positive_exponent = get_tokens(".1_23e+4_5");

    let result_leading_non_zero_digit = get_tokens("12_3.4_56E+7_8");
    let result_leading_non_zero_digit_no_fraction_with_positive_exponent = get_tokens("1_23E+4_5");
    let result_leading_non_zero_digit_no_fraction_with_negative_exponent = get_tokens("12_3e-4_5");

    // Failures

    let result_leading_zero_incomplete_exponent = get_tokens("0.1_23e");
    let result_leading_zero_incomplete_exponent_with_sign = get_tokens("0.1_23e+");
    let result_leading_zero_incomplete_fraction = get_tokens("0.e-1");
    let result_leading_zero_incomplete_fraction_with_underscore = get_tokens("0._E-1");
    let result_leading_zero_incomplete_exponent_without_fraction = get_tokens("0E");
    let result_leading_zero_incomplete_exponent_with_sign_without_fraction = get_tokens("0e-");

    let result_leading_non_zero_incomplete_exponent = get_tokens("1_2.3_45e");
    let result_leading_non_zero_incomplete_exponent_with_sign = get_tokens("1_2.3_45e-");
    let result_leading_non_zero_incomplete_fraction = get_tokens("1_2.e7_8");
    let result_leading_non_zero_incomplete_fraction_with_underscore = get_tokens("1_2._e7_8");
    let result_leading_non_zero_incomplete_exponent_without_fraction = get_tokens("1_23e");
    let result_leading_non_zero_incomplete_exponent_with_sign_without_fraction = get_tokens("1_23e+");

    assert_eq!(
        result_leading_zero,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123".to_string()),
            Span::new(0, 6)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123e+45".to_string()),
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_negative_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123e-45".to_string()),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_leading_zero_with_positive_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123e+45".to_string()),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_leading_zero_without_fraction,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0e+45".to_string()),
            Span::new(0, 6)
        ))]
    );

    assert_eq!(
        result_leading_dot,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123".to_string()),
            Span::new(0, 5)
        ))]
    );

    assert_eq!(
        result_leading_dot_with_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123e+45".to_string()),
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_dot_with_negative_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123e-45".to_string()),
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_leading_dot_with_positive_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("0.123e+45".to_string()),
            Span::new(0, 10)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_digit,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("123.456e+78".to_string()),
            Span::new(0, 14)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_digit_no_fraction_with_positive_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("123e+45".to_string()),
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_non_zero_digit_no_fraction_with_negative_exponent,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecFloat("123e-45".to_string()),
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
            TokenKind::BinInteger("10101010".to_string()),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_bin_leading_underscore,
        vec![TokenResult::Ok(Token::new(
            TokenKind::BinInteger("10101010".to_string()),
            Span::new(0, 12)
        ))]
    );

    assert_eq!(
        result_oct,
        vec![TokenResult::Ok(Token::new(
            TokenKind::OctInteger("1234567".to_string()),
            Span::new(0, 11)
        ))]
    );

    assert_eq!(
        result_oct_leading_underscore,
        vec![TokenResult::Ok(Token::new(
            TokenKind::OctInteger("1234567".to_string()),
            Span::new(0, 12)
        ))]
    );

    assert_eq!(
        result_hex,
        vec![TokenResult::Ok(Token::new(
            TokenKind::HexInteger("123456789abcdef0".to_string()),
            Span::new(0, 21)
        ))]
    );

    assert_eq!(
        result_hex_leading_underscore,
        vec![TokenResult::Ok(Token::new(
            TokenKind::HexInteger("123456789abcdef0".to_string()),
            Span::new(0, 22)
        ))]
    );

    assert_eq!(
        result_hex_uppercase,
        vec![TokenResult::Ok(Token::new(
            TokenKind::HexInteger("123456789ABCDEF0".to_string()),
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
            TokenKind::DecInteger("0".to_string()),
            Span::new(0, 9)
        ))]
    );

    assert_eq!(
        result_leading_non_zero,
        vec![TokenResult::Ok(Token::new(
            TokenKind::DecInteger("1234567890".to_string()),
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
