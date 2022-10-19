#![allow(clippy::module_inception)]
mod parser;

pub use parser::*;

#[cfg(test)]
mod test;
