// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::fmt::Display;

use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    MixedSpaces(Span),
}

impl std::error::Error for LexerError {}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::MixedSpaces(span) => f.debug_tuple("MixedSpaces").field(span).finish(),
        }
    }
}
