#![no_main]
use libfuzzer_sys::fuzz_target;
use raccoon_compiler::Lexer;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = Lexer::tokenize(s).collect::<Vec<_>>();
    }
});
