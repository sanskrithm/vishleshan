/// Panini-RS main entry point with REPL, CLI, and examples.
///
/// Usage:
///   panini                    # Start REPL
///   panini example            # Run examples
///   panini "program-code"     # Execute program

mod dhatu;
mod graph;
mod lexer_v2;
mod semantic;

use dhatu::*;
use graph::*;
use lexer_v2::Lexer;
use semantic::SemanticAnalyzer;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "example" {
        run_examples();
    } else if args.len() > 1 {
        // Run program from arguments
        let program = args[1..].join(" ");
        run_program(&program);
    } else {
        // Start REPL
        repl();
    }
}

fn repl() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║ Panini-RS 0.2.0 - Morphology-Driven Analytics Compiler   ║");
    println!("║ ASCII Transliteration + Semantic Graph IR                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Examples:");
    println!("  sales-at revenue-ena chid-tva yuj-tva drsh-ti");
    println!("  data-at region-ena ci-tva madh-tva drsh-ti");
    println!();
    println!("Type 'help' for commands, 'examples' to see programs.");
    println!();

    loop {
        print!("panini> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                match input {
                    "help" => print_help(),
                    "examples" => print_examples(),
                    "exit" | "quit" => break,
                    _ => run_program(input),
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }

    println!("\nGoodbye!");
}

fn run_program(input: &str) {
    println!();

    // Phase 1: Lexical analysis
    println!("=== PHASE 1: LEXICAL ANALYSIS ===");
    let mut lexer = Lexer::new(input);
    let tokens = match lexer.tokenize() {
        Ok(t) => {
            for token in &t {
                if token != &lexer_v2::Token::Eof {
                    println!("  {}", token);
                }
            }
            t
        }
        Err(err) => {
            eprintln!("✗ Lexical error: {}", err);
            return;
        }
    };

    // Phase 2: Semantic analysis
    println!();
    println!("=== PHASE 2: SEMANTIC ANALYSIS ===");
    let mut analyzer = SemanticAnalyzer::new();
    let graph = match analyzer.analyze(&tokens) {
        Ok(g) => {
            println!("✓ Semantic graph built successfully");
            g
        }
        Err(err) => {
            eprintln!("✗ Semantic error: {}", err);
            return;
        }
    };

    // Phase 3: Graph validation
    println!();
    println!("=== PHASE 3: GRAPH VALIDATION ===");
    match graph.validate() {
        Ok(_) => println!("✓ Semantic graph is valid"),
        Err(err) => {
            eprintln!("✗ Graph validation failed: {}", err);
            return;
        }
    }

    // Phase 4: Display graph
    println!();
    println!("=== PHASE 4: SEMANTIC GRAPH ===");
    println!("{}", graph);

    // Phase 5: Topological order (execution plan)
    println!();
    println!("=== PHASE 5: EXECUTION PLAN (Topological Order) ===");
    let topo_order = graph.topological_order();
    for (i, node_id) in topo_order.iter().rev().enumerate() {
        if let Some(node) = graph.get_node(*node_id) {
            println!("  Step {}: {}", i + 1, node);
        }
    }

    println!();
    println!("✓ Program compiled successfully");
    println!();
}

fn run_examples() {
    let examples = vec![
        (
            "Example 1: Simple filter and aggregate",
            "sales-at revenue-ena chid-tva yuj-tva drsh-ti",
        ),
        (
            "Example 2: Group and average",
            "data-at region-ena ci-tva madh-tva drsh-ti",
        ),
        (
            "Example 3: Multi-instrument with projection",
            "sales-at region-ena revenue-ena adhyaya-tva drsh-ti",
        ),
        (
            "Example 4: Count aggregation",
            "orders-at status-ena gan-tva drsh-ti",
        ),
    ];

    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║ PANINI-RS EXAMPLES                                         ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    for (name, program) in examples {
        println!("{}", name);
        println!("Program: {}", program);
        run_program(program);
        println!();
        println!("────────────────────────────────────────────────────────────");
        println!();
    }
}

fn print_help() {
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║ PANINI-RS HELP                                             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("COMMANDS:");
    println!("  help       - Show this help message");
    println!("  examples   - Show example programs");
    println!("  exit/quit  - Exit the REPL");
    println!();
    println!("SYNTAX:");
    println!();
    println!("  Kārakas (semantic roles):");
    println!("    -at     apādāna (source)");
    println!("    -ena    karaṇa (instrument/parameter)");
    println!("    -asya   karma (output target, reserved)");
    println!();
    println!("  Sūtra suffixes (execution control):");
    println!("    -tva    lazy continuation (pipeline)");
    println!("    -ti     terminal execution (collect)");
    println!();
    println!("  Dhātu operations (ASCII only):");
    println!();
    println!("  Aggregation:");
    println!("    yuj     sum");
    println!("    madh    mean/average");
    println!("    gan     count");
    println!("    lagh    minimum");
    println!("    mah     maximum");
    println!();
    println!("  Transformation:");
    println!("    chid    filter");
    println!("    adhyaya project/select");
    println!("    kram    sort");
    println!("    vibhaj  partition");
    println!();
    println!("  Relational:");
    println!("    ci      group-by");
    println!("    mel     join");
    println!();
    println!("  Terminal:");
    println!("    drsh    render/print");
    println!();
    println!("PANINIAN PRINCIPLES:");
    println!("  - Morphology-driven semantics (no positional arguments)");
    println!("  - Anuvrtti: inherited context propagation");
    println!("  - Kāraka: semantic dependency relations");
    println!("  - Semantic graph IR (not just AST)");
    println!("  - ASCII transliteration (UTF-8 free)");
    println!();
}

fn print_examples() {
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║ QUICK EXAMPLES                                             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("EXAMPLE 1: Filter + Sum");
    println!("  sales-at revenue-ena chid-tva yuj-tva drsh-ti");
    println!();
    println!("  Semantics:");
    println!("    1. Load 'sales' dataset (apādāna, -at)");
    println!("    2. Inherit 'revenue' as parameter (karaṇa, -ena)");
    println!("    3. Filter rows (chid)");
    println!("    4. Sum columns (yuj)");
    println!("    5. Render output (drsh-ti, terminal)");
    println!();
    println!("EXAMPLE 2: Group + Average");
    println!("  data-at region-ena ci-tva madh-tva drsh-ti");
    println!();
    println!("  Semantics:");
    println!("    1. Load 'data' dataset");
    println!("    2. Inherit 'region' as grouping key");
    println!("    3. Group by region (ci)");
    println!("    4. Calculate mean (madh)");
    println!("    5. Render output");
    println!();
    println!("EXAMPLE 3: Multiple Parameters (Anuvrtti)");
    println!("  sales-at region-ena revenue-ena ci-tva yuj-tva drsh-ti");
    println!();
    println!("  Semantics:");
    println!("    1. Load 'sales' dataset");
    println!("    2. Inherit 'region' and 'revenue' (both available everywhere)");
    println!("    3. Group by region (ci)");
    println!("    4. Sum revenue (yuj)");
    println!("    5. Render output");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_integration() {
        let input = "sales-at revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_semantic_analysis_integration() {
        let input = "sales-at revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_dhatu_parsing() {
        assert_eq!(Dhatu::from_ascii("chid"), Some(Dhatu::Transformation(TransformationDhatu::Chid)));
        assert_eq!(Dhatu::from_ascii("yuj"), Some(Dhatu::Aggregation(AggregationDhatu::Yuj)));
        assert_eq!(Dhatu::from_ascii("drsh"), Some(Dhatu::Terminal(TerminalDhatu::Drsh)));
    }
}
