#![forbid(unsafe_code)]

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod passes;
pub mod pass_manager;
pub mod codegen;
pub mod verification;
pub mod diagnostics;
pub mod compiler;

pub use ast::*;
pub use compiler::*;
pub use diagnostics::*;
