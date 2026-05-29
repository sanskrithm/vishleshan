/// Semantic graph representation for Panini-RS programs.
///
/// This module implements the core semantic graph that represents:
/// 1. Dependency relations between operations (Karaka theory)
/// 2. Inherited context propagation (Anuvrtti)
/// 3. Operation rewrite chains (Sutra system)
/// 4. Execution semantics encoded in edges and nodes
///
/// ARCHITECTURE:
/// - Nodes represent semantic operations or data sources
/// - Edges represent semantic dependencies (Karakas)
/// - Graph is acyclic (DAG) for lazy evaluation
/// - Supports optimizer rewrite passes

use indexmap::IndexMap;
use std::fmt;

/// Karaka represents semantic roles in the graph.
/// Inspired by Panini's Karaka theory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Karaka {
    /// Apādāna (source): Where does data originate?
    Source,
    /// Karaṇa (instrument): What data/context is used?
    Instrument,
    /// Karmaphal (object): What is being operated on?
    Object,
    /// Adhikarana (location): In what scope does this operate?
    Scope,
}

impl fmt::Display for Karaka {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Karaka::Source => write!(f, "source"),
            Karaka::Instrument => write!(f, "instrument"),
            Karaka::Object => write!(f, "object"),
            Karaka::Scope => write!(f, "scope"),
        }
    }
}

/// Semantic node types in the computation graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Load source data
    Source { name: String },
    /// Filter operation
    Filter { predicate: String },
    /// Group operation
    GroupBy { columns: Vec<String> },
    /// Aggregation (sum, mean, min, max, count, etc.)
    Aggregate { op: String, columns: Vec<String> },
    /// Join operation
    Join { kind: String, on: Vec<String> },
    /// Sort operation
    Sort { columns: Vec<String>, desc: bool },
    /// Project/select columns
    Project { columns: Vec<String> },
    /// Terminal render operation
    Render,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::Source { name } => write!(f, "Source({})", name),
            NodeKind::Filter { predicate } => write!(f, "Filter({})", predicate),
            NodeKind::GroupBy { columns } => write!(f, "GroupBy({:?})", columns),
            NodeKind::Aggregate { op, columns } => write!(f, "{}({:?})", op, columns),
            NodeKind::Join { kind, on } => write!(f, "Join({}, {:?})", kind, on),
            NodeKind::Sort { columns, desc } => {
                write!(f, "Sort({:?}, desc={})", columns, desc)
            }
            NodeKind::Project { columns } => write!(f, "Project({:?})", columns),
            NodeKind::Render => write!(f, "Render"),
        }
    }
}

/// Edge in the semantic graph representing Karaka relations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Semantic role this dependency plays
    pub karaka: Karaka,
    /// Optional metadata about the dependency
    pub metadata: Option<String>,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.karaka, 
               self.metadata.as_ref().unwrap_or(&"_".to_string()))
    }
}

/// Semantic graph node with metadata.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub kind: NodeKind,
    /// Inherited context available at this node (Anuvrtti)
    pub inherited_context: Vec<String>,
    /// Whether this is a lazy (Continue) or terminal (Terminal) operation
    pub is_terminal: bool,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node#{}: {} [context: {:?}, terminal: {}]", 
               self.id, self.kind, self.inherited_context, self.is_terminal)
    }
}

/// The complete semantic graph for a Panini-RS program.
/// This is the main IR between parsing and optimization.
#[derive(Debug, Clone)]
pub struct SemanticGraph {
    /// Nodes in the computation DAG
    nodes: IndexMap<usize, Node>,
    /// Adjacency: node_id -> (target_node_id, edge)
    edges: IndexMap<usize, Vec<(usize, Edge)>>,
    /// Counter for generating unique node IDs
    next_id: usize,
    /// Entry point (source node)
    entry_id: Option<usize>,
    /// Exit point (terminal render node)
    exit_id: Option<usize>,
}

impl SemanticGraph {
    /// Create a new empty semantic graph
    pub fn new() -> Self {
        SemanticGraph {
            nodes: IndexMap::new(),
            edges: IndexMap::new(),
            next_id: 0,
            entry_id: None,
            exit_id: None,
        }
    }

    /// Add a node to the graph, returning its ID
    pub fn add_node(&mut self, kind: NodeKind, is_terminal: bool) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = Node {
            id,
            kind,
            inherited_context: Vec::new(),
            is_terminal,
        };

        self.nodes.insert(id, node);
        self.edges.insert(id, Vec::new());

        id
    }

    /// Add a directed edge between two nodes
    pub fn add_edge(&mut self, from: usize, to: usize, karaka: Karaka) {
        if let Some(edges) = self.edges.get_mut(&from) {
            edges.push((
                to,
                Edge {
                    karaka,
                    metadata: None,
                },
            ));
        }
    }

    /// Set entry point (source node)
    pub fn set_entry(&mut self, node_id: usize) {
        self.entry_id = Some(node_id);
    }

    /// Set exit point (terminal render node)
    pub fn set_exit(&mut self, node_id: usize) {
        self.exit_id = Some(node_id);
    }

    /// Get a node by ID
    pub fn get_node(&self, id: usize) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Get outgoing edges from a node
    pub fn get_edges(&self, node_id: usize) -> Option<&[(usize, Edge)]> {
        self.edges.get(&node_id).map(|v| v.as_slice())
    }

    /// Propagate inherited context through the graph (Anuvrtti)
    /// This implements semantic inheritance - once a parameter is bound,
    /// it flows through all subsequent operations
    pub fn propagate_context(&mut self, initial_context: Vec<String>) {
        if let Some(entry_id) = self.entry_id {
            self.propagate_from(entry_id, initial_context);
        }
    }

    /// Recursively propagate context from a starting node
    fn propagate_from(&mut self, node_id: usize, context: Vec<String>) {
        // Get outgoing edges
        let next_edges: Vec<(usize, Karaka)> = if let Some(edges) = self.edges.get(&node_id) {
            edges.iter().map(|(to, edge)| (*to, edge.karaka)).collect()
        } else {
            Vec::new()
        };

        // Set context for this node
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.inherited_context = context.clone();
        }

        // Propagate to all successors
        for (target_id, _karaka) in next_edges {
            self.propagate_from(target_id, context.clone());
        }
    }

    /// Collect all nodes in topological order (for execution planning)
    pub fn topological_order(&self) -> Vec<usize> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        if let Some(entry) = self.entry_id {
            self.topo_visit(entry, &mut visited, &mut order);
        }
        
        order
    }

    /// Recursive topological sort helper
    fn topo_visit(&self, node_id: usize, visited: &mut std::collections::HashSet<usize>, order: &mut Vec<usize>) {
        if visited.contains(&node_id) {
            return;
        }
        visited.insert(node_id);

        if let Some(edges) = self.edges.get(&node_id) {
            for (target, _) in edges {
                self.topo_visit(*target, visited, order);
            }
        }

        order.push(node_id);
    }

    /// Validate the graph
    pub fn validate(&self) -> Result<(), String> {
        if self.entry_id.is_none() {
            return Err("No entry node (source)".to_string());
        }
        if self.exit_id.is_none() {
            return Err("No exit node (render)".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for SemanticGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SemanticGraph {{")?;
        writeln!(f, "  nodes:")?;
        for node in self.nodes.values() {
            writeln!(f, "    {}", node)?;
        }
        writeln!(f, "  edges:")?;
        for (from, edges) in &self.edges {
            for (to, edge) in edges {
                writeln!(f, "    {} -> {} [{}]", from, to, edge)?;
            }
        }
        writeln!(f, "  entry: {:?}, exit: {:?}", self.entry_id, self.exit_id)?;
        write!(f, "}}")
    }
}

impl Default for SemanticGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_graph_creation() {
        let mut graph = SemanticGraph::new();
        let source_id = graph.add_node(NodeKind::Source { name: "sales".to_string() }, false);
        let render_id = graph.add_node(NodeKind::Render, true);
        
        graph.set_entry(source_id);
        graph.set_exit(render_id);
        
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_semantic_graph_edges() {
        let mut graph = SemanticGraph::new();
        let source = graph.add_node(NodeKind::Source { name: "data".to_string() }, false);
        let filter = graph.add_node(NodeKind::Filter { predicate: "revenue > 0".to_string() }, false);
        
        graph.add_edge(source, filter, Karaka::Source);
        
        let edges = graph.get_edges(source).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].0, filter);
        assert_eq!(edges[0].1.karaka, Karaka::Source);
    }

    #[test]
    fn test_anuvrtti_context_propagation() {
        let mut graph = SemanticGraph::new();
        let source = graph.add_node(NodeKind::Source { name: "data".to_string() }, false);
        let filter = graph.add_node(NodeKind::Filter { predicate: "x > 0".to_string() }, false);
        let agg = graph.add_node(NodeKind::Aggregate { 
            op: "sum".to_string(), 
            columns: vec!["revenue".to_string()] 
        }, false);
        
        graph.add_edge(source, filter, Karaka::Source);
        graph.add_edge(filter, agg, Karaka::Object);
        graph.set_entry(source);
        
        let initial_context = vec!["revenue".to_string(), "region".to_string()];
        graph.propagate_context(initial_context);
        
        // All nodes should have inherited context
        assert_eq!(graph.get_node(source).unwrap().inherited_context.len(), 2);
        assert_eq!(graph.get_node(filter).unwrap().inherited_context.len(), 2);
        assert_eq!(graph.get_node(agg).unwrap().inherited_context.len(), 2);
    }

    #[test]
    fn test_topological_order() {
        let mut graph = SemanticGraph::new();
        let source = graph.add_node(NodeKind::Source { name: "data".to_string() }, false);
        let filter = graph.add_node(NodeKind::Filter { predicate: "x > 0".to_string() }, false);
        let render = graph.add_node(NodeKind::Render, true);
        
        graph.add_edge(source, filter, Karaka::Source);
        graph.add_edge(filter, render, Karaka::Object);
        graph.set_entry(source);
        
        let order = graph.topological_order();
        assert_eq!(order, vec![render, filter, source]);
    }
}
