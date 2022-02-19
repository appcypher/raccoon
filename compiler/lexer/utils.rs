// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

pub fn is_horizontal_whitespace(char: char) -> bool {
    match char {
        ' ' | '\t' => true,
        _ => false,
    }
}
