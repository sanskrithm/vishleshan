/// Panini-RS: Morphology-driven semantic analytics compiler.
///
/// This is a production-grade compiler for SQL-like analytics expressed
/// through Sanskrit morphological semantics.
///
/// CORE MODULES:
/// 1. dhatu - Extended dhatu system (operation opcodes) with ASCII transliteration
/// 2. graph - Semantic graph representation with Karaka relations
/// 3. lexer - ASCII-only tokenizer (zero-copy FSM)
/// 4. semantic - Semantic analyzer building semantic graphs with Anuvrtti
///
/// COMPILATION PIPELINE:
/// Input (ASCII transliterated Sanskrit)
///   ↓
/// Lexer (tokenization, ASCII only)
///   ↓
/// Semantic Analyzer (builds Karaka dependency graph)
///   ↓
/// Anuvrtti Engine (context inheritance propagation)
///   ↓
/// Polars Runtime (LazyFrame lowering)
///   ↓
/// Output (Arrow-backed execution)
///
/// FUTURE MODULES (v0.3+):
/// - optimizer - Query optimization passes (rewrite rules)
/// - planner - Execution DAG planning
/// - runtime - Polars LazyFrame lowering (detailed)
pub mod dhatu;
pub mod graph;
pub mod lexer;
pub mod semantic;
pub mod optimizer;

// Future modules (placeholders)
// pub mod planner;
// pub mod runtime;

pub use dhatu::*;
pub use graph::*;
pub use lexer::*;
pub use semantic::*;
pub use optimizer::*;
