use raccoon_compiler::Lexer;
use test_strategy::proptest;

use super::strategy::code;

#[proptest]
fn can_tokenize_valid_code(#[strategy(code())] code: String) {
    let results = Lexer::tokenize(&code).collect::<Vec<_>>();

    for result in results {
        assert!(result.is_ok());
    }
}
