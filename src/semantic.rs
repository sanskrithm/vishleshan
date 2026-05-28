/// Semantic analyzer building semantic graph with Anuvrtti.
///
/// This module transforms tokens into a semantic graph.
/// It implements:
/// 1. Karaka role assignment (semantic dependency relations)
/// 2. Anuvrtti propagation (inherited context)
/// 3. Graph construction for optimization
///
/// PANINIAN PRINCIPLES:
/// - Anuvrtti: Once a parameter is bound (with -ena), it is inherited by all
///   subsequent operations
/// - Adhikara: Governing scope propagation
/// - Vipratisedha: Conflict resolution (first-match wins for overloading)

use crate::dhatu::{AggregationDhatu, Dhatu, KarakaSuffix, RelationalDhatu, SutraSuffix, TransformationDhatu};
use crate::graph::{Karaka, NodeKind, SemanticGraph};
use crate::lexer_v2::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub enum SemanticError {
    NoSourceFound,
    NoTerminalFound,
    InvalidSequence { reason: String },
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSourceFound => write!(f, "No source morpheme (missing -at)"),
            Self::NoTerminalFound => write!(f, "No terminal operation (missing -ti)"),
            Self::InvalidSequence { reason } => write!(f, "Invalid sequence: {}", reason),
        }
    }
}

pub type SemanticResult<T> = Result<T, SemanticError>;

/// Semantic analyzer state machine
pub struct SemanticAnalyzer {
    /// Current accumulated inherited context (Anuvrtti)
    context: Vec<String>,
    /// Source dataset name
    source: Option<String>,
    /// Semantic graph being built
    graph: SemanticGraph,
    /// Current node being processed
    current_node: Option<usize>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            context: Vec::new(),
            source: None,
            graph: SemanticGraph::new(),
            current_node: None,
        }
    }

    /// Main analysis entry point: tokens -> semantic graph
    pub fn analyze(&mut self, tokens: &[Token]) -> SemanticResult<SemanticGraph> {
        // Phase 1: Extract source morpheme (Apādāna, -at)
        let mut pos = 0;
        if pos >= tokens.len() {
            return Err(SemanticError::NoSourceFound);
        }

        match &tokens[pos] {
            Token::Morpheme { root, karaka } if *karaka == KarakaSuffix::At => {
                self.source = Some(root.clone());
                let source_node = self.graph.add_node(
                    NodeKind::Source { name: root.clone() },
                    false,
                );
                self.graph.set_entry(source_node);
                self.current_node = Some(source_node);
                pos += 1;
            }
            _ => return Err(SemanticError::NoSourceFound),
        }

        // Phase 2: Extract instruments (Karaṇa, -ena)
        // These are accumulated into context (Anuvrtti)
        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Morpheme { root, karaka } if *karaka == KarakaSuffix::Ena => {
                    self.context.push(root.clone());
                    pos += 1;
                }
                Token::Verb { .. } => break, // Operations start
                _ => break,
            }
        }

        // Phase 3: Extract operations (dhatu forms)
        let mut last_was_terminal = false;
        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Verb { dhatu, sutra } => {
                    last_was_terminal = *sutra == SutraSuffix::Ti;
                    self.process_dhatu(*dhatu, &self.context.clone())?;
                    pos += 1;
                }
                Token::Eof => break,
                _ => break,
            }
        }

        if !last_was_terminal {
            return Err(SemanticError::NoTerminalFound);
        }

        // Set render as exit point
        if let Some(node) = self.current_node {
            self.graph.set_exit(node);
        }

        // Propagate inherited context through the graph
        self.graph.propagate_context(self.context.clone());

        Ok(self.graph.clone())
    }

    /// Process a dhatu operation
    fn process_dhatu(&mut self, dhatu: Dhatu, context: &[String]) -> SemanticResult<()> {
        match dhatu {
            Dhatu::Transformation(TransformationDhatu::Chid) => {
                // Filter operation
                let conditions = context
                    .iter()
                    .map(|c| format!("{} IS NOT NULL", c))
                    .collect::<Vec<_>>()
                    .join(" AND ");

                let node = self.graph.add_node(
                    NodeKind::Filter { predicate: conditions },
                    false,
                );

                if let Some(current) = self.current_node {
                    self.graph.add_edge(current, node, Karaka::Object);
                }
                self.current_node = Some(node);
            }

            Dhatu::Transformation(TransformationDhatu::Adhyaya) => {
                // Project/select operation
                let node = self.graph.add_node(
                    NodeKind::Project { columns: context.to_vec() },
                    false,
                );

                if let Some(current) = self.current_node {
                    self.graph.add_edge(current, node, Karaka::Object);
                }
                self.current_node = Some(node);
            }

            Dhatu::Relational(RelationalDhatu::Ci) => {
                // Group operation
                let node = self.graph.add_node(
                    NodeKind::GroupBy { columns: context.to_vec() },
                    false,
                );

                if let Some(current) = self.current_node {
                    self.graph.add_edge(current, node, Karaka::Instrument);
                }
                self.current_node = Some(node);
            }

            Dhatu::Aggregation(AggregationDhatu::Yuj) => {
                // Sum aggregation
                let node = self.graph.add_node(
                    NodeKind::Aggregate {
                        op: "sum".to_string(),
                        columns: context.to_vec(),
                    },
                    false,
                );

                if let Some(current) = self.current_node {
                    self.graph.add_edge(current, node, Karaka::Object);
                }
                self.current_node = Some(node);
            }

            Dhatu::Aggregation(AggregationDhatu::Madh) => {
                // Mean aggregation
                let node = self.graph.add_node(
                    NodeKind::Aggregate {
                        op: "mean".to_string(),
                        columns: context.to_vec(),
                    },
                    false,
                );

                if let Some(current) = self.current_node {
                    self.graph.add_edge(current, node, Karaka::Object);
                }
                self.current_node = Some(node);
            }

            Dhatu::Aggregation(AggregationDhatu::Gan) => {
                // Count aggregation
                let node = self.graph.add_node(
                    NodeKind::Aggregate {
                        op: "count".to_string(),
                        columns: vec![],
                    },
                    false,
                );

                if let Some(current) = self.current_node {
                    self.graph.add_edge(current, node, Karaka::Object);
                }
                self.current_node = Some(node);
            }

            Dhatu::Terminal(_) => {
                // Render (terminal)
                let node = self.graph.add_node(NodeKind::Render, true);

                if let Some(current) = self.current_node {
                    self.graph.add_edge(current, node, Karaka::Object);
                }
                self.current_node = Some(node);
            }

            _ => {
                return Err(SemanticError::InvalidSequence {
                    reason: format!("Unsupported dhatu: {:?}", dhatu),
                });
            }
        }

        Ok(())
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dhatu::{TransformationDhatu, TerminalDhatu};
    use crate::lexer_v2::Lexer;

    fn tokenize(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize().unwrap()
    }

    #[test]
    fn test_semantic_analysis_simple() {
        let tokens = tokenize("sales-at revenue-ena chid-tva yuj-tva drsh-ti");
        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_semantic_analysis_no_source() {
        let tokens = tokenize("revenue-ena chid-tva drsh-ti");
        let mut analyzer = SemanticAnalyzer::new();
        assert!(analyzer.analyze(&tokens).is_err());
    }

    #[test]
    fn test_semantic_analysis_no_terminal() {
        let tokens = tokenize("sales-at revenue-ena chid-tva");
        let mut analyzer = SemanticAnalyzer::new();
        assert!(analyzer.analyze(&tokens).is_err());
    }

    #[test]
    fn test_anuvrtti_context_propagation() {
        let tokens = tokenize("sales-at region-ena revenue-ena chid-tva yuj-tva drsh-ti");
        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();
        assert_eq!(analyzer.context.len(), 2);
    }
}
