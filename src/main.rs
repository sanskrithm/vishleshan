/// Pāṇini-RS: A morphology-driven Sanskrit DSL for high-performance data processing.
///
/// ARCHITECTURE OVERVIEW:
/// 
/// The Pāṇini-RS compiler implements a complete compilation pipeline:
///
///   Input DSL (Sanskrit Morphemes)
///        ↓
///   Lexer (token.rs) → Token Stream
///        ↓
///   Parser (parser.rs) → Abstract Syntax Tree (AST)
///        ↓
///   Semantic Analyzer (compiler.rs) → Query Plan
///        ↓
///   Runtime Execution → Polars LazyFrame → Output
///
/// LANGUAGE DESIGN:
///
/// The language is NOT traditional function syntax. Instead, it uses:
///
/// 1. KĀRAKA SUFFIXES (Grammatical Cases as Semantic Relations):
///    - -āt (Apādāna): Source dataset declaration
///    - -ena (Karaṇa): Instrument binding (parameters inherited by all operations)
///
/// 2. SŪTRA SUFFIXES (Control Flow):
///    - -tvā: Lazy operation (pipeline continuation)
///    - -ti: Terminal operation (execute and output)
///
/// 3. DHĀTU ROOTS (Semantic Operations):
///    - chid: Filter
///    - ci: Group-by
///    - yuj: Aggregate/Sum
///    - dṛś: Render/Visualize
///
/// ANUVṚTTI (State Inheritance):
/// Once an instrument is bound with -ena, it's automatically inherited by
/// all subsequent operations without re-declaration.
///
/// EXAMPLE PROGRAM:
///   data-āt sales-ena chid-tvā yuj-tvā dṛś-ti
///
/// Semantic interpretation:
///   1. data-āt        → Load dataset "data" (source)
///   2. sales-ena      → Bind "sales" as inherited instrument
///   3. chid-tvā       → Filter using "sales" (lazy)
///   4. yuj-tvā        → Aggregate "sales" (lazy)
///   5. dṛś-ti         → Render/output (terminal)

mod ast;
mod compiler;
mod lexer;
mod parser;
mod token;

use compiler::CompilerState;
use lexer::Lexer;
use parser::Parser;
use std::io::{self, Write};

/// Display formatted output for a query plan
fn display_query_plan(plan: &compiler::QueryPlan) {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║          COMPILED QUERY PLAN                    ║");
    println!("╚════════════════════════════════════════════════╝\n");

    println!("📦 Inherited Parameters (Anuvṛtti):");
    for param in &plan.context {
        println!("   • {}", param);
    }

    println!("\n🔄 Query Execution Pipeline:");
    for (i, step) in plan.steps.iter().enumerate() {
        println!("   {} → {}", i + 1, step);
    }
    println!();
}

/// Display the parsed AST
fn display_ast(program: &ast::Program) {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║          ABSTRACT SYNTAX TREE                   ║");
    println!("╚════════════════════════════════════════════════╝\n");
    println!("{}\n", program);
}

/// Display the token stream
fn display_tokens(tokens: &[token::Token]) {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║          TOKEN STREAM                           ║");
    println!("╚════════════════════════════════════════════════╝\n");
    for (i, token) in tokens.iter().enumerate() {
        if token != &token::Token::Eof {
            println!("   Token {}: {}", i, token);
        }
    }
    println!();
}

/// Process a single Pāṇini-RS program
fn process_program(input: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Input Program:");
    println!("   {}\n", input);

    // PHASE 1: LEXICAL ANALYSIS
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;

    if verbose {
        display_tokens(&tokens);
    }

    // PHASE 2: SYNTACTIC ANALYSIS (PARSING)
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    if verbose {
        display_ast(&program);
    }

    // PHASE 3: SEMANTIC ANALYSIS & QUERY PLANNING
    let mut compiler = CompilerState::new();
    let query_plan = compiler.compile(&program)?;

    display_query_plan(&query_plan);

    println!("✅ Compilation succeeded!\n");

    Ok(())
}

/// Interactive REPL for Pāṇini-RS
fn run_repl() -> io::Result<()> {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║  Pāṇini-RS Compiler - Interactive REPL         ║");
    println!("║  A Sanskrit Morphology-Driven DSL              ║");
    println!("╚════════════════════════════════════════════════╝\n");

    println!("Type 'help' for syntax help, 'quit' to exit.\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("pāṇini> ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("\nExiting Pāṇini-RS compiler. 🙏\n");
            break;
        }

        if input == "help" {
            println!(
                r#"
PĀṆINI-RS LANGUAGE REFERENCE

KĀRAKA SUFFIXES (Grammatical Cases):
  -āt   Source (Apādāna) - first token, declares source dataset
  -ena  Instrument (Karaṇa) - declares inherited parameters

SŪTRA SUFFIXES (Control Flow):
  -tvā  Pipeline continuation (lazy operation)
  -ti   Terminal execution (trigger evaluation)

DHĀTU ROOTS (Standard Operations):
  chid  Filter operation
  ci    Group-by operation
  yuj   Aggregate/Sum operation
  dṛś   Render/Print operation

EXAMPLE:
  data-āt sales-ena chid-tvā yuj-tvā dṛś-ti

  Interpretation:
    1. data-āt     → Load "data" dataset
    2. sales-ena   → Bind "sales" as inherited parameter
    3. chid-tvā    → Filter (lazy)
    4. yuj-tvā     → Aggregate (lazy)
    5. dṛś-ti      → Render (terminal)

ANUVṚTTI (State Inheritance):
  Once -ena binds a parameter, it's inherited by all subsequent operations.
"#
            );
            continue;
        }

        if input == "verbose" || input == "v" {
            println!("Verbose mode toggled. Re-enter a program to see detailed output.\n");
            continue;
        }

        match process_program(input, false) {
            Ok(()) => {}
            Err(e) => {
                println!("❌ Compilation error: {}\n", e);
            }
        }
    }

    Ok(())
}

/// Run example programs
fn run_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║  PĀṆINI-RS COMPILER - EXAMPLE PROGRAMS         ║");
    println!("╚════════════════════════════════════════════════╝");

    let examples = vec![
        (
            "Simple Load & Render",
            "data-āt dṛś-ti",
        ),
        (
            "Load with Instrument & Filter",
            "data-āt sales-ena chid-tvā dṛś-ti",
        ),
        (
            "Complete Example: Filter + Aggregate",
            "data-āt sales-ena chid-tvā yuj-tvā dṛś-ti",
        ),
        (
            "Multiple Instruments: Filter & Group",
            "data-āt region-ena sales-ena ci-tvā dṛś-ti",
        ),
        (
            "Complex Pipeline",
            "data-āt region-ena sales-ena chid-tvā ci-tvā yuj-tvā dṛś-ti",
        ),
    ];

    for (name, program) in examples {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📌 Example: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if let Err(e) = process_program(program, true) {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    // Check for command-line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "examples" => {
                if let Err(e) = run_examples() {
                    eprintln!("Fatal error: {}", e);
                    std::process::exit(1);
                }
            }
            "repl" | "-i" => {
                run_repl()?;
            }
            "-h" | "--help" => {
                println!(
                    r#"
PĀṆINI-RS COMPILER

Usage: panini_rs [COMMAND] [ARGS]

COMMANDS:
  examples     Run compiled example programs
  repl, -i     Start interactive REPL
  -h, --help   Show this help message

QUICK START:
  # Interactive REPL
  panini_rs repl

  # Run examples
  panini_rs examples

LANGUAGE:
  Write programs in Sanskrit morphological form:
  
  data-āt sales-ena chid-tvā yuj-tvā dṛś-ti
  
  Where:
    -āt   = source dataset (Apādāna)
    -ena  = inherited parameter (Karaṇa)
    -tvā  = lazy operation (pipeline continuation)
    -ti   = terminal operation (execute)
"#
                );
            }
            program => {
                // Direct program execution
                if let Err(e) = process_program(program, false) {
                    eprintln!("❌ Compilation error: {}\n", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        // Default: run REPL
        run_repl()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline_simple() {
        let program = "data-āt dṛś-ti";
        let result = process_program(program, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_full_pipeline_with_instruments() {
        let program = "data-āt sales-ena chid-tvā dṛś-ti";
        let result = process_program(program, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_full_pipeline_example() {
        let program = "data-āt sales-ena chid-tvā yuj-tvā dṛś-ti";
        let result = process_program(program, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compilation_error_handling() {
        // Missing source
        let program = "sales-ena dṛś-ti";
        let result = process_program(program, false);
        assert!(result.is_err());

        // No terminal operation
        let program = "data-āt sales-ena chid-tvā";
        let result = process_program(program, false);
        assert!(result.is_err());
    }
}
