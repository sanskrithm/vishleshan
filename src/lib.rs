/// Pāṇini-RS Compiler Library
///
/// This library provides the complete compilation infrastructure for the
/// Pāṇini-RS morphology-driven Sanskrit DSL.
///
/// # Modules
///
/// - `token`: Token definitions and lexical units
/// - `lexer`: Lexical analyzer (tokenizer)
/// - `parser`: Syntactic analyzer (parser)
/// - `ast`: Abstract Syntax Tree definitions
/// - `compiler`: Semantic analyzer and query planner

pub mod ast;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod token;

// Re-export main types for convenience
pub use ast::{Operation, Program};
pub use compiler::{CompilerState, CompilerError, QueryPlan, QueryStep};
pub use lexer::{Lexer, LexError};
pub use parser::{Parser, ParseError};
pub use token::{CaseMarker, Dhatu, ExecutionMarker, MorphemeCompound, Token, VerbForm};
