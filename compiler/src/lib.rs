pub mod codegen;
mod errors;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod span;

// pub use parser::Parser;
pub use errors::*;
pub use lexer::Lexer;
