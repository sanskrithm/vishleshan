/// Semantic compiler and query planner for Pāṇini-RS.
///
/// CompilerState manages:
/// 1. Anuvṛtti (state inheritance) - once bound, parameters flow through all operations
/// 2. Query planning - transforms the AST into Polars LazyFrame operations
/// 3. Semantic validation - ensures operations are semantically correct
///
/// ANUVṚTTI IMPLEMENTATION:
/// The compiler maintains a context of "inherited" parameters. When an instrument
/// is declared with -ena, it enters the context and is automatically available
/// to all subsequent operations without re-declaration.
///
/// POLARS LOWERING:
/// The compiler lowers the high-level morphological AST into Polars LazyFrame
/// operations. Each dhātu maps to specific Polars methods:
/// - chid (filter) → filter(predicate)
/// - ci (group) → group_by(columns)
/// - yuj (aggregate) → sum/mean/etc.
/// - dṛś (render) → collect() + print
///
/// This is a query planner, similar to LLVM IR generation in traditional compilers.

use crate::ast::{Operation, Program};
use crate::token::{Dhatu, ExecutionMarker};
use std::collections::HashSet;
use std::fmt;

/// Semantic errors during compilation
#[derive(Debug, Clone)]
pub enum CompilerError {
    /// Unknown operation
    UnknownDhatu { dhatu: String },
    /// Operation requires specific data context
    OperationRequiresContext { dhatu: String },
    /// Query planning failed
    QueryPlanError { reason: String },
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::UnknownDhatu { dhatu } => {
                write!(f, "Unknown dhātu operation: {}", dhatu)
            }
            CompilerError::OperationRequiresContext { dhatu } => {
                write!(
                    f,
                    "Operation '{}' requires inherited context (add instruments with -ena)",
                    dhatu
                )
            }
            CompilerError::QueryPlanError { reason } => {
                write!(f, "Query planning failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for CompilerError {}

pub type CompileResult<T> = Result<T, CompilerError>;

/// Represents a single compiled operation step.
/// This is an intermediate form between the AST and Polars execution.
#[derive(Debug, Clone)]
pub enum QueryStep {
    /// Load source data from a dataset
    LoadSource { dataset: String },
    
    /// Filter operation using inherited instruments
    Filter { conditions: Vec<String> },
    
    /// Group-by operation
    GroupBy { columns: Vec<String> },
    
    /// Numerical aggregation (sum, mean, etc.)
    Aggregate { operation: String, columns: Vec<String> },
    
    /// Collect and render (execute pipeline)
    Render,
}

impl fmt::Display for QueryStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryStep::LoadSource { dataset } => write!(f, "Load({})", dataset),
            QueryStep::Filter { conditions } => {
                write!(f, "Filter({})", conditions.join(", "))
            }
            QueryStep::GroupBy { columns } => {
                write!(f, "GroupBy({})", columns.join(", "))
            }
            QueryStep::Aggregate { operation, columns } => {
                write!(f, "{}({})", operation, columns.join(", "))
            }
            QueryStep::Render => write!(f, "Render"),
        }
    }
}

/// The compiled query plan.
/// This is the output of semantic analysis and query planning.
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// Sequence of operations to execute
    pub steps: Vec<QueryStep>,
    /// Inherited parameters available to all operations
    pub context: HashSet<String>,
}

impl fmt::Display for QueryPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "QueryPlan {{")?;
        writeln!(f, "  context: {{")?;
        for param in &self.context {
            writeln!(f, "    {}", param)?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "  steps: [")?;
        for step in &self.steps {
            writeln!(f, "    {}", step)?;
        }
        writeln!(f, "  ]")?;
        write!(f, "}}")
    }
}

/// Semantic compiler for Pāṇini-RS.
///
/// Transforms an AST into a QueryPlan that can be lowered to Polars operations.
pub struct CompilerState {
    /// Inherited parameters (Anuvṛtti)
    context: HashSet<String>,
    /// Generated query steps
    steps: Vec<QueryStep>,
}

impl CompilerState {
    /// Create a new compiler with an empty context
    pub fn new() -> Self {
        CompilerState {
            context: HashSet::new(),
            steps: Vec::new(),
        }
    }

    /// Compile an AST into a QueryPlan
    ///
    /// This implements the core semantic analysis:
    /// 1. Establish source data
    /// 2. Bind instruments to context (Anuvṛtti)
    /// 3. Translate operations to query steps
    /// 4. Generate final query plan
    pub fn compile(&mut self, program: &Program) -> CompileResult<QueryPlan> {
        self.reset();

        // Step 1: Load source
        self.steps.push(QueryStep::LoadSource {
            dataset: program.source.clone(),
        });

        // Step 2: Bind instruments to context (Anuvṛtti)
        for instrument in &program.instruments {
            self.context.insert(instrument.clone());
        }

        // Step 3: Translate operations
        for operation in &program.operations {
            self.compile_operation(operation)?;
        }

        // Build and return the query plan
        let plan = QueryPlan {
            steps: self.steps.clone(),
            context: self.context.clone(),
        };

        Ok(plan)
    }

    /// Compile a single operation
    fn compile_operation(&mut self, operation: &Operation) -> CompileResult<()> {
        match &operation.dhatu {
            Dhatu::Chid => {
                // Filter operation: uses inherited instruments as filter conditions
                let conditions: Vec<String> = self
                    .context
                    .iter()
                    .map(|s| format!("{} IS NOT NULL", s))
                    .collect();

                self.steps.push(QueryStep::Filter { conditions });
            }

            Dhatu::Ci => {
                // Group-by operation: uses instruments as grouping keys
                let columns: Vec<String> = self.context.iter().cloned().collect();

                if columns.is_empty() {
                    return Err(CompilerError::OperationRequiresContext {
                        dhatu: "ci".to_string(),
                    });
                }

                self.steps.push(QueryStep::GroupBy { columns });
            }

            Dhatu::Yuj => {
                // Aggregation: sum operation using context
                let columns: Vec<String> = self.context.iter().cloned().collect();

                if columns.is_empty() {
                    return Err(CompilerError::OperationRequiresContext {
                        dhatu: "yuj".to_string(),
                    });
                }

                self.steps.push(QueryStep::Aggregate {
                    operation: "sum".to_string(),
                    columns,
                });
            }

            Dhatu::Drsh => {
                // Render: always terminal, executes the pipeline
                self.steps.push(QueryStep::Render);
            }

            Dhatu::Custom(name) => {
                return Err(CompilerError::UnknownDhatu {
                    dhatu: name.clone(),
                });
            }
        }

        Ok(())
    }

    /// Reset the compiler state
    fn reset(&mut self) {
        self.context.clear();
        self.steps.clear();
    }

    /// Get the current context (inherited parameters)
    pub fn context(&self) -> &HashSet<String> {
        &self.context
    }

    /// Get the current query steps
    pub fn steps(&self) -> &[QueryStep] {
        &self.steps
    }
}

impl Default for CompilerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{CaseMarker, MorphemeCompound};

    fn make_program(source: &str, instruments: Vec<&str>) -> Program {
        let mut prog = Program::new();
        prog.source = source.to_string();
        prog.instruments = instruments.iter().map(|s| s.to_string()).collect();
        prog.operations = vec![Operation {
            dhatu: Dhatu::Drsh,
            execution: ExecutionMarker::Terminal,
        }];
        prog
    }

    #[test]
    fn test_compiler_basic() {
        let program = make_program("data", vec![]);
        let mut compiler = CompilerState::new();
        let plan = compiler.compile(&program).unwrap();

        assert_eq!(plan.steps.len(), 2); // LoadSource + Render
    }

    #[test]
    fn test_compiler_with_instruments() {
        let program = make_program("data", vec!["sales", "region"]);
        let mut compiler = CompilerState::new();
        let plan = compiler.compile(&program).unwrap();

        assert_eq!(plan.context.len(), 2);
        assert!(plan.context.contains("sales"));
        assert!(plan.context.contains("region"));
    }

    #[test]
    fn test_compiler_filter_operation() {
        let mut program = make_program("data", vec!["sales"]);
        program.operations = vec![
            Operation {
                dhatu: Dhatu::Chid,
                execution: ExecutionMarker::Continue,
            },
            Operation {
                dhatu: Dhatu::Drsh,
                execution: ExecutionMarker::Terminal,
            },
        ];

        let mut compiler = CompilerState::new();
        let plan = compiler.compile(&program).unwrap();

        // Should have: LoadSource, Filter, Render
        assert_eq!(plan.steps.len(), 3);
        match &plan.steps[1] {
            QueryStep::Filter { .. } => {} // Good
            _ => panic!("Expected Filter step"),
        }
    }

    #[test]
    fn test_compiler_group_aggregation() {
        let mut program = make_program("data", vec!["region"]);
        program.operations = vec![
            Operation {
                dhatu: Dhatu::Ci,
                execution: ExecutionMarker::Continue,
            },
            Operation {
                dhatu: Dhatu::Yuj,
                execution: ExecutionMarker::Continue,
            },
            Operation {
                dhatu: Dhatu::Drsh,
                execution: ExecutionMarker::Terminal,
            },
        ];

        let mut compiler = CompilerState::new();
        let plan = compiler.compile(&program).unwrap();

        // LoadSource + GroupBy + Aggregate + Render
        assert_eq!(plan.steps.len(), 4);
    }

    #[test]
    fn test_compiler_context_inheritance() {
        let program = make_program("data", vec!["col1", "col2", "col3"]);
        let mut compiler = CompilerState::new();
        let plan = compiler.compile(&program).unwrap();

        // All instruments should be in context
        assert_eq!(plan.context.len(), 3);
        assert!(plan.context.contains("col1"));
        assert!(plan.context.contains("col2"));
        assert!(plan.context.contains("col3"));
    }

    #[test]
    fn test_compiler_example_program() {
        // Simulate: data-āt sales-ena chid-tvā yuj-tvā dṛś-ti
        let mut program = make_program("data", vec!["sales"]);
        program.operations = vec![
            Operation {
                dhatu: Dhatu::Chid,
                execution: ExecutionMarker::Continue,
            },
            Operation {
                dhatu: Dhatu::Yuj,
                execution: ExecutionMarker::Continue,
            },
            Operation {
                dhatu: Dhatu::Drsh,
                execution: ExecutionMarker::Terminal,
            },
        ];

        let mut compiler = CompilerState::new();
        let plan = compiler.compile(&program).unwrap();

        // Verify structure
        assert_eq!(plan.context.len(), 1);
        assert_eq!(plan.steps.len(), 4); // Load, Filter, Aggregate, Render
    }

    #[test]
    fn test_query_step_display() {
        let step = QueryStep::LoadSource {
            dataset: "sales".to_string(),
        };
        assert_eq!(step.to_string(), "Load(sales)");

        let step = QueryStep::Filter {
            conditions: vec!["a > 0".to_string()],
        };
        assert_eq!(step.to_string(), "Filter(a > 0)");
    }

    #[test]
    fn test_query_plan_display() {
        let plan = QueryPlan {
            steps: vec![
                QueryStep::LoadSource {
                    dataset: "data".to_string(),
                },
                QueryStep::Render,
            ],
            context: HashSet::new(),
        };
        let display = plan.to_string();
        assert!(display.contains("QueryPlan"));
        assert!(display.contains("Load(data)"));
    }
}
