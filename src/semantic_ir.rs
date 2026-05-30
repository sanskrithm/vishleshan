use crate::graph::{NodeKind, SemanticGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Column types for schema contracts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ColumnType {
    String,
    Int,
    Float,
    Bool,
    Timestamp,
}

/// Schema for a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub columns: HashMap<String, ColumnType>,
}

impl Schema {
    pub fn new() -> Self {
        Schema { columns: HashMap::new() }
    }

    pub fn with_columns(cols: Vec<(&str, ColumnType)>) -> Self {
        let mut s = Schema::new();
        for (name, ty) in cols {
            s.columns.insert(name.to_string(), ty);
        }
        s
    }

    pub fn has_column(&self, col: &str) -> bool {
        self.columns.contains_key(col)
    }
}

/// Registry of dataset schema contracts
#[derive(Debug, Default)]
pub struct SchemaRegistry {
    pub datasets: HashMap<String, Schema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        SchemaRegistry { datasets: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, schema: Schema) {
        self.datasets.insert(name.to_string(), schema);
    }

    pub fn get(&self, name: &str) -> Option<&Schema> {
        self.datasets.get(name)
    }
}

/// Diagnostic emitted during validation
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub node_id: usize,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node#{}: {}", self.node_id, self.message)
    }
}

/// High-level Semantic IR wrapper providing schema-aware validation
pub struct SemanticIR {
    pub graph: SemanticGraph,
}

impl SemanticIR {
    pub fn new(graph: SemanticGraph) -> Self {
        SemanticIR { graph }
    }

    /// Validate the graph against the provided schema registry.
    /// Returns Ok(()) when valid, or Vec<Diagnostic> when errors found.
    pub fn validate_against_registry(&self, registry: &SchemaRegistry) -> Result<(), Vec<Diagnostic>> {
        let mut diags: Vec<Diagnostic> = Vec::new();

        // 1. Ensure all Source nodes reference registered datasets
        for node in self.graph.nodes.values() {
            match &node.kind {
                NodeKind::Source { name } => {
                    if registry.get(name).is_none() {
                        diags.push(Diagnostic { node_id: node.id, message: format!("Unknown source dataset '{}'.", name) });
                    }
                }
                NodeKind::Aggregate { op: _, columns } => {
                    // For aggregates, try to find a dataset in the graph's entry's context
                    // Use simple heuristic: check each registered dataset for required columns
                    let mut satisfied = false;
                    for (ds_name, schema) in &registry.datasets {
                        if columns.iter().all(|c| schema.has_column(c)) {
                            satisfied = true;
                            break;
                        }
                    }
                    if !satisfied {
                        diags.push(Diagnostic { node_id: node.id, message: format!("Aggregate on unknown columns: {:?}", columns) });
                    }
                }
                NodeKind::Project { columns } => {
                    // Ensure columns exist in some registered schema
                    let mut missing: Vec<String> = Vec::new();
                    for col in columns {
                        let mut found = false;
                        for schema in registry.datasets.values() {
                            if schema.has_column(col) {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            missing.push(col.clone());
                        }
                    }
                    if !missing.is_empty() {
                        diags.push(Diagnostic { node_id: node.id, message: format!("Project references missing columns: {:?}", missing) });
                    }
                }
                NodeKind::Filter { predicate } => {
                    // Very small heuristic: extract tokens and check whether at least one token matches any column
                    let tokens: HashSet<String> = predicate
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();

                    let mut any = false;
                    for schema in registry.datasets.values() {
                        for tok in &tokens {
                            if schema.has_column(tok) {
                                any = true;
                                break;
                            }
                        }
                        if any { break; }
                    }
                    if !any {
                        diags.push(Diagnostic { node_id: node.id, message: format!("Filter predicate references unknown columns or literals only: {}", predicate) });
                    }
                }
                _ => {}
            }
        }

        if diags.is_empty() { Ok(()) } else { Err(diags) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{SemanticGraph, NodeKind, Karaka};

    #[test]
    fn test_schema_registry_validation_ok() {
        let mut graph = SemanticGraph::new();
        let src = graph.add_node(NodeKind::Source { name: "sales".to_string() }, false);
        let filter = graph.add_node(NodeKind::Filter { predicate: "revenue > 0".to_string() }, false);
        let agg = graph.add_node(NodeKind::Aggregate { op: "sum".to_string(), columns: vec!["revenue".to_string()] }, false);
        let render = graph.add_node(NodeKind::Render, true);

        graph.add_edge(src, filter, Karaka::Source);
        graph.add_edge(filter, agg, Karaka::Object);
        graph.add_edge(agg, render, Karaka::Object);
        graph.set_entry(src);
        graph.set_exit(render);

        let registry = {
            let mut r = SchemaRegistry::new();
            r.register("sales", Schema::with_columns(vec![
                ("customer_id", ColumnType::String),
                ("region", ColumnType::String),
                ("revenue", ColumnType::Float),
            ]));
            r
        };

        let ir = SemanticIR::new(graph);
        let res = ir.validate_against_registry(&registry);
        assert!(res.is_ok());
    }

    #[test]
    fn test_schema_registry_validation_failures() {
        let mut graph = SemanticGraph::new();
        let src = graph.add_node(NodeKind::Source { name: "unknown_ds".to_string() }, false);
        let filter = graph.add_node(NodeKind::Filter { predicate: "mystery > 0".to_string() }, false);
        let render = graph.add_node(NodeKind::Render, true);

        graph.add_edge(src, filter, Karaka::Source);
        graph.add_edge(filter, render, Karaka::Object);
        graph.set_entry(src);
        graph.set_exit(render);

        let registry = SchemaRegistry::new();

        let ir = SemanticIR::new(graph);
        let res = ir.validate_against_registry(&registry);
        assert!(res.is_err());
        let diags = res.err().unwrap();
        assert!(diags.len() >= 1);
    }
}
