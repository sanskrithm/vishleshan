/// Panini-RS: Morphology-driven semantic analytics compiler.
///
/// This is a production-grade compiler for SQL-like analytics expressed
/// through Sanskrit morphological semantics.
///
/// CORE MODULES:
/// 1. dhatu - Extended dhatu system (operation opcodes) with ASCII transliteration
/// 2. graph - Semantic graph representation with Karaka relations
/// 3. lexer_v2 - ASCII-only tokenizer (zero-copy FSM)
/// 4. semantic - Semantic analyzer building semantic graphs with Anuvrtti
/// 5. optimizer - Query optimization passes (rewrite rules)
/// 6. planner - Execution DAG planning
/// 7. runtime - Polars LazyFrame lowering
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
/// Optimizer (rewrite passes)
///   ↓
/// Execution Planner (DAG generation)
///   ↓
/// Polars Runtime (LazyFrame lowering)
///   ↓
/// Output (Arrow-backed execution)

pub mod dhatu;
pub mod graph;
pub mod lexer_v2;
pub mod semantic;

// Future modules (placeholders)
// pub mod optimizer;
// pub mod planner;
// pub mod runtime;

pub use dhatu::*;
pub use graph::*;
pub use lexer_v2::*;
pub use semantic::*;
