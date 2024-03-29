mod errors;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod span;
pub mod ir;
pub mod codegen;

pub use lexer::Lexer;
// pub use parser::Parser;

pub use errors::*;
