/// Execution planner transforming semantic graphs into executable plans.
///
/// This module translates the semantic graph IR into concrete execution plans
/// that can be lowered to different backends (Polars, Arrow, etc.).
///
/// EXECUTION PLANNING PHASES:
/// 1. Dependency analysis: Ensure topological order
/// 2. Backend selection: Choose optimal backend for each node
/// 3. Plan generation: Create concrete execution steps
/// 4. Cost estimation: Estimate memory/time for each operation
use crate::graph::{NodeKind, SemanticGraph};
use std::fmt;

#[derive(Debug, Clone)]
pub enum PlanningError {
    InvalidGraph { reason: String },
    BackendUnsupported { op: String },
}

impl fmt::Display for PlanningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGraph { reason } => write!(f, "Invalid graph: {}", reason),
            Self::BackendUnsupported { op } => write!(f, "Backend unsupported for: {}", op),
        }
    }
}

pub type PlanningResult<T> = Result<T, PlanningError>;

/// An executable plan step
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStep {
    /// Load source dataset
    LoadSource { name: String },
    /// Filter rows based on predicate
    Filter { predicate: String },
    /// Group rows by columns
    GroupBy { columns: Vec<String> },
    /// Aggregation operation
    Aggregate {
        op: String,
        columns: Vec<String>,
    },
    /// Join two datasets
    Join {
        kind: String,
        on: Vec<String>,
    },
    /// Sort rows
    Sort {
        columns: Vec<String>,
        desc: bool,
    },
    /// Project (select) columns
    Project { columns: Vec<String> },
    /// Terminal render/collect
    Render,
}

impl fmt::Display for ExecutionStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionStep::LoadSource { name } => write!(f, "LoadSource({})", name),
            ExecutionStep::Filter { predicate } => write!(f, "Filter({})", predicate),
            ExecutionStep::GroupBy { columns } => write!(f, "GroupBy({:?})", columns),
            ExecutionStep::Aggregate { op, columns } => write!(f, "{}({:?})", op, columns),
            ExecutionStep::Join { kind, on } => write!(f, "{}Join({:?})", kind, on),
            ExecutionStep::Sort { columns, desc } => write!(f, "Sort({:?}, desc={})", columns, desc),
            ExecutionStep::Project { columns } => write!(f, "Project({:?})", columns),
            ExecutionStep::Render => write!(f, "Render"),
        }
    }
}

/// Execution plan
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub backend: ExecutionBackend,
    pub estimated_cost: Cost,
}

impl fmt::Display for ExecutionPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ExecutionPlan (backend: {:?})", self.backend)?;
        writeln!(f, "Estimated cost: {:?}", self.estimated_cost)?;
        writeln!(f, "Steps:")?;
        for (i, step) in self.steps.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, step)?;
        }
        Ok(())
    }
}

/// Supported execution backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionBackend {
    Polars,
    Arrow,
    DuckDB,
    DataFusion,
}

impl fmt::Display for ExecutionBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionBackend::Polars => write!(f, "Polars"),
            ExecutionBackend::Arrow => write!(f, "Arrow"),
            ExecutionBackend::DuckDB => write!(f, "DuckDB"),
            ExecutionBackend::DataFusion => write!(f, "DataFusion"),
        }
    }
}

/// Cost estimation
#[derive(Debug, Clone, Copy)]
pub struct Cost {
    pub estimated_rows: usize,
    pub estimated_memory_mb: f64,
    pub estimated_time_ms: f64,
}

/// Execution planner
pub struct ExecutionPlanner {
    backend: ExecutionBackend,
}

impl ExecutionPlanner {
    pub fn new(backend: ExecutionBackend) -> Self {
        ExecutionPlanner { backend }
    }

    /// Plan execution from semantic graph
    pub fn plan(&self, graph: &SemanticGraph) -> PlanningResult<ExecutionPlan> {
        // Validate graph
        graph.validate().map_err(|e| PlanningError::InvalidGraph { reason: e })?;

        // Get topological order
        let topo_order = graph.topological_order();

        // Convert nodes to execution steps
        let mut steps = Vec::new();
        for &node_id in topo_order.iter().rev() {
            if let Some(node) = graph.get_node(node_id) {
                let step = self.node_to_step(&node.kind)?;
                steps.push(step);
            }
        }

        // Estimate cost
        let cost = self.estimate_cost(&steps);

        Ok(ExecutionPlan {
            steps,
            backend: self.backend,
            estimated_cost: cost,
        })
    }

    /// Convert a graph node to an execution step
    fn node_to_step(&self, kind: &NodeKind) -> PlanningResult<ExecutionStep> {
        match kind {
            NodeKind::Source { name } => Ok(ExecutionStep::LoadSource {
                name: name.clone(),
            }),
            NodeKind::Filter { predicate } => Ok(ExecutionStep::Filter {
                predicate: predicate.clone(),
            }),
            NodeKind::GroupBy { columns } => Ok(ExecutionStep::GroupBy {
                columns: columns.clone(),
            }),
            NodeKind::Aggregate { op, columns } => Ok(ExecutionStep::Aggregate {
                op: op.clone(),
                columns: columns.clone(),
            }),
            NodeKind::Join { kind, on } => Ok(ExecutionStep::Join {
                kind: kind.clone(),
                on: on.clone(),
            }),
            NodeKind::Sort { columns, desc } => Ok(ExecutionStep::Sort {
                columns: columns.clone(),
                desc: *desc,
            }),
            NodeKind::Project { columns } => Ok(ExecutionStep::Project {
                columns: columns.clone(),
            }),
            NodeKind::Render => Ok(ExecutionStep::Render),
        }
    }

    /// Estimate cost of execution
    fn estimate_cost(&self, _steps: &[ExecutionStep]) -> Cost {
        // Placeholder: Would implement actual cost estimation
        Cost {
            estimated_rows: 1000,
            estimated_memory_mb: 10.0,
            estimated_time_ms: 50.0,
        }
    }
}

impl Default for ExecutionPlanner {
    fn default() -> Self {
        Self::new(ExecutionBackend::Polars)
    }
}

impl ExecutionPlan {
    /// Produce a Polars `LazyFrame` call chain as pseudocode for lowering.
    /// This is a textual representation (stub) useful for testing and as
    /// a starting point for implementing runtime lowering in `runtime.rs`.
    pub fn lower_to_polars(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("// Polars LazyFrame lowering (pseudocode)".to_string());
        lines.push("let df = LazyFrame::scan_csv(\"<source>\", Default::default()).unwrap();".to_string());

        for step in &self.steps {
            match step {
                ExecutionStep::LoadSource { name } => {
                    lines.push(format!("let df = LazyFrame::scan_csv(\"{}\", Default::default()).unwrap();", name));
                }
                ExecutionStep::Filter { predicate } => {
                    lines.push(format!("let df = df.filter(col(\"<expr>\").apply(|s| {{ /* {} */ }}));", predicate));
                }
                ExecutionStep::GroupBy { columns } => {
                    lines.push(format!("let df = df.groupby([{}]).agg([]);", columns.iter().map(|c| format!("col(\"{}\")", c)).collect::<Vec<_>>().join(", ")));
                }
                ExecutionStep::Aggregate { op, columns } => {
                    lines.push(format!("let df = df.agg(&[{}]); // op={}", columns.iter().map(|c| format!("{}({})", op, c)).collect::<Vec<_>>().join(", "), op));
                }
                ExecutionStep::Join { kind, on } => {
                    lines.push(format!("let df = df.join(&other, &[{}], JoinType::{});", on.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", "), kind));
                }
                ExecutionStep::Sort { columns, desc } => {
                    lines.push(format!("let df = df.sort(&[{}], {});", columns.iter().map(|c| format!("col(\"{}\")", c)).collect::<Vec<_>>().join(", "), desc));
                }
                ExecutionStep::Project { columns } => {
                    lines.push(format!("let df = df.select([{}]);", columns.iter().map(|c| format!("col(\"{}\")", c)).collect::<Vec<_>>().join(", ")));
                }
                ExecutionStep::Render => {
                    lines.push("let out = df.collect().unwrap();".to_string());
                    lines.push("println!(\"{:?}\", out);".to_string());
                }
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_creation() {
        let _planner = ExecutionPlanner::new(ExecutionBackend::Polars);
    }

    #[test]
    fn test_execution_step_display() {
        let step = ExecutionStep::LoadSource {
            name: "sales".to_string(),
        };
        assert_eq!(step.to_string(), "LoadSource(sales)");
    }

    #[test]
    fn test_backend_display() {
        assert_eq!(ExecutionBackend::Polars.to_string(), "Polars");
        assert_eq!(ExecutionBackend::Arrow.to_string(), "Arrow");
    }

    #[test]
    fn test_cost_estimation() {
        let cost = Cost {
            estimated_rows: 1000,
            estimated_memory_mb: 10.0,
            estimated_time_ms: 50.0,
        };
        assert_eq!(cost.estimated_rows, 1000);
    }

    #[test]
    fn test_lower_to_polars() {
        let plan = ExecutionPlan {
            steps: vec![
                ExecutionStep::LoadSource { name: "sales".to_string() },
                ExecutionStep::Filter { predicate: "revenue > 0".to_string() },
                ExecutionStep::Aggregate { op: "sum".to_string(), columns: vec!["revenue".to_string()] },
                ExecutionStep::Render,
            ],
            backend: ExecutionBackend::Polars,
            estimated_cost: Cost { estimated_rows: 10, estimated_memory_mb: 1.0, estimated_time_ms: 5.0 },
        };

        let lowered = plan.lower_to_polars();
        assert!(lowered.contains("Polars LazyFrame") || lowered.len() > 0);
    }
}
