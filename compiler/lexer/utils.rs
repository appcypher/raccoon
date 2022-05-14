pub fn is_horizontal_whitespace(char: char) -> bool {
    matches!(char, ' ' | '\t')
}
