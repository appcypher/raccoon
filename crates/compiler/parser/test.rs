use super::*;

#[test]
fn test_parse() {
    let tokens = vec![];
    let mut parser = Parser::new(&tokens, 1000);
    parser.parse();
}
