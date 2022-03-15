pub fn is_horizontal_whitespace(char: char) -> bool {
    match char {
        ' ' | '\t' => true,
        _ => false,
    }
}
