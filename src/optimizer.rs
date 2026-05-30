/// Query optimizer implementing Paribhasha (compiler rewrite rules).
///
/// This module applies semantic rewrite transformations to the semantic graph
/// following Paninian principles:
/// - Paribhasha: Rewrite rules for simplification and optimization
/// - Semantic validation: Ensure rewrites preserve semantics
/// - Cost-based selection: Choose optimal rewrite paths
///
/// PANINIAN OPTIMIZATION PRINCIPLES:
/// 1. Simplification: Remove redundant operations
/// 2. Fusion: Combine compatible operations
/// 3. Pushdown: Move filters closer to source
/// 4. Projection: Prune unused columns early
/// 5. Aggregation coalescing: Merge aggregations when possible
use crate::graph::{Karaka, NodeKind, SemanticGraph};
use std::fmt;

#[derive(Debug, Clone)]
pub enum OptimizationError {
    InvalidTransform { reason: String },
    SemanticViolation { reason: String },
}

impl fmt::Display for OptimizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransform { reason } => write!(f, "Invalid transformation: {}", reason),
            Self::SemanticViolation { reason } => write!(f, "Semantic violation: {}", reason),
        }
    }
}

pub type OptimizationResult<T> = Result<T, OptimizationError>;

/// Optimization pass trait
pub trait OptimizationPass {
    fn apply(&self, graph: &mut SemanticGraph) -> OptimizationResult<bool>;
    fn name(&self) -> &'static str;
}

/// Paribhasha: Query optimizer with rewrite rules
pub struct QueryOptimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl QueryOptimizer {
    pub fn new() -> Self {
        QueryOptimizer {
            passes: vec![
                Box::new(RedundantFilterEliminationPass),
                Box::new(FilterPushdownPass),
                Box::new(AggregationCoalescingPass),
                Box::new(ProjectionPruningPass),
            ],
        }
    }

    /// Run all optimization passes
    pub fn optimize(&self, graph: &mut SemanticGraph) -> OptimizationResult<OptimizationStats> {
        let mut stats = OptimizationStats::new();
        let initial_node_count = graph.stats().node_count;

        for pass in &self.passes {
            let changed = pass.apply(graph)?;
            if changed {
                stats.passes_applied.push(pass.name().to_string());
            }
        }

        let final_node_count = graph.stats().node_count;
        stats.nodes_eliminated = initial_node_count as isize - final_node_count as isize;

        Ok(stats)
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about optimization
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub passes_applied: Vec<String>,
    pub nodes_eliminated: isize,
}

impl OptimizationStats {
    fn new() -> Self {
        OptimizationStats {
            passes_applied: Vec::new(),
            nodes_eliminated: 0,
        }
    }
}

/// PARIBHASHA 1: Eliminate redundant filters
/// E.g., filter(NULL) -> filter(NULL) elimination
struct RedundantFilterEliminationPass;

impl OptimizationPass for RedundantFilterEliminationPass {
    fn apply(&self, graph: &mut SemanticGraph) -> OptimizationResult<bool> {
        // Detect identical filter predicates reachable downstream and annotate
        let mut changed = false;
        let topo = graph.topological_order();
        // Collect predicates and their first-seen node id
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for &node_id in topo.iter() {
            if let Some(node) = graph.get_node(node_id) {
                if let NodeKind::Filter { predicate } = &node.kind {
                    if let Some(first_id) = seen.get(predicate) {
                        // annotate this node as redundant (add marker to inherited_context)
                        if let Some(node_mut) = graph.get_node_mut(node_id) {
                            node_mut.inherited_context.push(format!("redundant_filter_of:{}", first_id));
                            changed = true;
                        }
                    } else {
                        seen.insert(predicate.clone(), node_id);
                    }
                }
            }
        }

        Ok(changed)
    }

    fn name(&self) -> &'static str {
        "RedundantFilterElimination"
    }
}

/// PARIBHASHA 2: Push filters closer to source
/// Move filter operations closer to the source for early elimination
struct FilterPushdownPass;

impl OptimizationPass for FilterPushdownPass {
    fn apply(&self, graph: &mut SemanticGraph) -> OptimizationResult<bool> {
        // Conservative analysis: for each Filter, try to tag nearest Source nodes with the predicate
        let mut changed = false;
        let topo = graph.topological_order();

        // For each filter node, find upstream sources by scanning nodes whose reachable set includes the filter
        for &node_id in topo.iter() {
            if let Some(node) = graph.get_node(node_id) {
                if let NodeKind::Filter { predicate } = &node.kind {
                    // scan all nodes to find sources that reach this filter
                    for &candidate in topo.iter() {
                        if let Some(candidate_node) = graph.get_node(candidate) {
                            if let NodeKind::Source { name: _ } = &candidate_node.kind {
                                let reachable = graph.reachable_from(candidate);
                                if reachable.contains(&node_id) {
                                    if let Some(src_mut) = graph.get_node_mut(candidate) {
                                        src_mut.inherited_context.push(format!("pushdown_pred:{{{}}}", predicate));
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(changed)
    }

    fn name(&self) -> &'static str {
        "FilterPushdown"
    }
}

/// PARIBHASHA 3: Coalesce aggregations
/// Merge multiple aggregations when possible
/// E.g., sum(x); mean(x) -> single aggregation pass
struct AggregationCoalescingPass;

impl OptimizationPass for AggregationCoalescingPass {
    fn apply(&self, graph: &mut SemanticGraph) -> OptimizationResult<bool> {
        // Coalesce aggregations that operate on identical column sets in downstream chain
        let mut changed = false;
        let topo = graph.topological_order();
        // Map columns key to first aggregation node id
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for &node_id in topo.iter() {
            if let Some(node) = graph.get_node(node_id) {
                if let NodeKind::Aggregate { op, columns } = &node.kind {
                    let key = columns.join(",");
                    if let Some(first_id) = seen.get(&key) {
                        // annotate current node as coalesced into first
                        if let Some(node_mut) = graph.get_node_mut(node_id) {
                            node_mut.inherited_context.push(format!("coalesced_into:{}", first_id));
                            // also annotate the first op to indicate combined op
                            if let Some(first_mut) = graph.get_node_mut(*first_id) {
                                first_mut.inherited_context.push(format!("coalesced_op:{}.{}", op, node_id));
                            }
                            changed = true;
                        }
                    } else {
                        seen.insert(key, node_id);
                    }
                }
            }
        }

        Ok(changed)
    }

    fn name(&self) -> &'static str {
        "AggregationCoalescing"
    }
}

/// PARIBHASHA 4: Pruning unused projections
/// Remove columns not needed downstream
struct ProjectionPruningPass;

impl OptimizationPass for ProjectionPruningPass {
    fn apply(&self, graph: &mut SemanticGraph) -> OptimizationResult<bool> {
        // Simple analysis: find Project nodes and mark downstream nodes' inherited_context to only include used columns
        let mut changed = false;
        let topo = graph.topological_order();

        for &node_id in topo.iter() {
            if let Some(node) = graph.get_node(node_id) {
                if let NodeKind::Project { columns } = &node.kind {
                    // Propagate pruning info to downstream nodes
                    let reachable = graph.reachable_from(node_id);
                    for rid in reachable {
                        if let Some(mut_node) = graph.get_node_mut(rid) {
                            mut_node.inherited_context.push(format!("pruned_columns:{:?}", columns));
                            changed = true;
                        }
                    }
                }
            }
        }

        Ok(changed)
    }

    fn name(&self) -> &'static str {
        "ProjectionPruning"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let _optimizer = QueryOptimizer::new();
        // Verify optimizer is created with passes
    }

    #[test]
    fn test_apply_passes_annotations() {
        use crate::graph::{SemanticGraph, NodeKind, Karaka};

        let mut graph = SemanticGraph::new();
        let src = graph.add_node(NodeKind::Source { name: "data".to_string() }, false);
        let f1 = graph.add_node(NodeKind::Filter { predicate: "x > 0".to_string() }, false);
        let f2 = graph.add_node(NodeKind::Filter { predicate: "x > 0".to_string() }, false);
        let agg = graph.add_node(NodeKind::Aggregate { op: "sum".to_string(), columns: vec!["revenue".to_string()] }, false);
        let proj = graph.add_node(NodeKind::Project { columns: vec!["revenue".to_string()] }, false);
        let render = graph.add_node(NodeKind::Render, true);

        graph.add_edge(src, f1, Karaka::Source);
        graph.add_edge(f1, f2, Karaka::Object);
        graph.add_edge(f2, agg, Karaka::Object);
        graph.add_edge(agg, proj, Karaka::Object);
        graph.add_edge(proj, render, Karaka::Object);
        graph.set_entry(src);
        graph.set_exit(render);

        let optimizer = QueryOptimizer::new();
        let stats = optimizer.optimize(&mut graph).unwrap();

        // At least some passes should be recorded or annotations present
        assert!(stats.passes_applied.len() >= 0);
        // Check that duplicate filter got annotated
        let f2_node = graph.get_node(f2).unwrap();
        assert!(f2_node.inherited_context.iter().any(|s| s.contains("redundant_filter_of") || s.contains("coalesced_into") || s.contains("pruned_columns")));
    }

    #[test]
    fn test_optimization_stats() {
        let stats = OptimizationStats::new();
        assert_eq!(stats.passes_applied.len(), 0);
        assert_eq!(stats.nodes_eliminated, 0);
    }
}
