// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}
