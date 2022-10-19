use proptest::strategy::Strategy;
use proptest::collection::vec;

fn identifier() -> impl Strategy<Value = String> {
    r"[A-Za-z_][A-Za-z0-9_]*"
}

fn _tokens() -> impl Strategy<Value = Vec<String>> {
    // TODO(appcypher):
    // - Implement other token kinds. And support randomization of token.
    // - Care must be taken when generating spaces between tokens. Spaces after newlines can result in unwanted indentation.
    vec(identifier(), 0..5000)
}

pub(super) fn code() -> impl Strategy<Value = String> {
    // TODO(appcypher): Fix implementation.
    identifier()
}
