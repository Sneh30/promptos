#![forbid(unsafe_code)]

pub mod ast;
pub mod codegen;
pub mod compiler;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod pass_manager;
pub mod passes;
pub mod semantic;
pub mod verification;

pub use ast::*;
pub use compiler::*;
pub use diagnostics::*;
