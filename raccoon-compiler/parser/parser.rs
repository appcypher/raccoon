use std::num::NonZeroUsize;

use lru::LruCache;
use raccoon_macros::memoize;

use crate::{ir::Ast, lexer::Token};

/// A recursive descent packrat parser.
///
/// It is designed to have the following properties:
/// - Results of all rule paths, at any given cursor position, are memoized.
/// - A parser function result should not hold values, but references to token elements.
pub struct Parser<'t> {
    /// The tokens to parse.
    tokens: &'t [Token],
    /// The current position in the source code.
    cursor: u32,
    /// An LRU cache of the results of all rule paths.
    cache: LruCache<(u32, String), Option<Ast>>, // TODO(appcypher)
}

impl<'t> Parser<'t> {
    /// Creates a new parser.
    pub fn new(tokens: &'t [Token], cache_size: usize) -> Self {
        Self {
            tokens,
            cursor: 0,
            cache: LruCache::new(NonZeroUsize::new(cache_size).unwrap()),
        }
    }

    #[memoize]
    pub fn parse(&mut self) -> Option<Ast> {
        todo!()
    }
}
