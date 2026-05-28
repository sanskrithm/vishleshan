/// Abstract Syntax Tree for Pāṇini-RS.
///
/// The AST represents the semantic structure of a Pāṇini-RS program after parsing.
/// It encodes:
/// 1. Data source declarations (Apādāna kāraka)
/// 2. Inherited instrument bindings (Karaṇa kāraka, Anuvṛtti)
/// 3. Operation pipeline (lazy sequence of operations)
/// 4. Terminal execution marker
///
/// KEY DESIGN PRINCIPLE: ANUVṚTTI (STATE INHERITANCE)
/// Once a parameter is bound with the Instrument case (-ena), it is automatically
/// inherited by subsequent operations. The compiler need not re-bind it.
///
/// EXAMPLE PROGRAM AST:
/// ```text
/// Program {
///   source: "data",
///   instruments: ["sales"],
///   operations: [
///     Operation(Chid, Continue),     // Filter using "sales"
///     Operation(Yuj, Continue),      // Aggregate using "sales"
///     Operation(Drsh, Terminal),     // Render output
///   ]
/// }
/// ```

use crate::token::{CaseMarker, Dhatu, ExecutionMarker};
use std::fmt;

/// A single operation in the execution pipeline.
///
/// Each operation applies a dhātu-driven transformation to the data pipeline.
/// The execution marker determines whether it's lazy (Continue) or terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    /// The semantic operation (dhātu)
    pub dhatu: Dhatu,
    /// Whether this is a lazy (tvā) or terminal (ti) operation
    pub execution: ExecutionMarker,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.dhatu, self.execution)
    }
}

/// The Abstract Syntax Tree for a complete Pāṇini-RS program.
///
/// This represents the semantic structure after parsing and captures:
/// - The source dataset (Apādāna: where data originates)
/// - Inherited instruments (Karaṇa: parameters that flow through all operations)
/// - The operation sequence (lazy pipeline + terminal operation)
///
/// ANUVṚTTI IMPLEMENTATION:
/// The `instruments` field holds all declared instrument-case morphemes.
/// These are accessible to all operations without explicit re-declaration.
#[derive(Debug, Clone)]
pub struct Program {
    /// Source dataset identifier (Apādāna kāraka)
    /// Example: "data" from "data-āt"
    pub source: String,

    /// Inherited parameters (Karaṇa kāraka)
    /// These are bound once and available to all operations.
    /// Example: ["sales"] from "sales-ena"
    ///
    /// ANUVṚTTI SEMANTICS:
    /// This list implements the "once declared, inherited by all" principle.
    /// Operations can reference these without explicit binding.
    pub instruments: Vec<String>,

    /// Operation sequence forming the execution pipeline
    /// Example: [Chid(Continue), Yuj(Continue), Drsh(Terminal)]
    pub operations: Vec<Operation>,
}

impl Program {
    /// Create a new empty program
    pub fn new() -> Self {
        Program {
            source: String::new(),
            instruments: Vec::new(),
            operations: Vec::new(),
        }
    }

    /// Verify that the program is well-formed
    ///
    /// Checks:
    /// 1. Source dataset is declared
    /// 2. At least one operation exists
    /// 3. Last operation is terminal
    pub fn validate(&self) -> Result<(), String> {
        if self.source.is_empty() {
            return Err("No source dataset declared (missing -āt morpheme)".to_string());
        }

        if self.operations.is_empty() {
            return Err("No operations declared".to_string());
        }

        // Check that final operation is terminal
        match self.operations.last() {
            Some(op) => {
                if op.execution != ExecutionMarker::Terminal {
                    return Err(
                        "Final operation must be terminal (use -ti suffix, e.g., dṛś-ti)"
                            .to_string(),
                    );
                }
            }
            None => return Err("No operations found".to_string()),
        }

        Ok(())
    }

    /// Get all lazy operations (those with -tvā)
    pub fn lazy_operations(&self) -> Vec<&Operation> {
        self.operations
            .iter()
            .filter(|op| op.execution == ExecutionMarker::Continue)
            .collect()
    }

    /// Get the terminal operation (the one with -ti)
    pub fn terminal_operation(&self) -> Option<&Operation> {
        self.operations
            .iter()
            .find(|op| op.execution == ExecutionMarker::Terminal)
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Program {{")?;
        writeln!(f, "  source: \"{}\"", self.source)?;
        write!(f, "  instruments: [")?;
        for (i, inst) in self.instruments.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "\"{}\"", inst)?;
        }
        writeln!(f, "]")?;
        write!(f, "  operations: [")?;
        for (i, op) in self.operations.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", op)?;
        }
        writeln!(f, "]")?;
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_display() {
        let op = Operation {
            dhatu: Dhatu::Chid,
            execution: ExecutionMarker::Continue,
        };
        assert_eq!(op.to_string(), "chid(tvā)");
    }

    #[test]
    fn test_program_creation() {
        let mut prog = Program::new();
        prog.source = "data".to_string();
        prog.instruments = vec!["sales".to_string()];
        prog.operations = vec![
            Operation {
                dhatu: Dhatu::Chid,
                execution: ExecutionMarker::Continue,
            },
            Operation {
                dhatu: Dhatu::Drsh,
                execution: ExecutionMarker::Terminal,
            },
        ];

        assert_eq!(prog.source, "data");
        assert_eq!(prog.instruments.len(), 1);
        assert_eq!(prog.operations.len(), 2);
    }

    #[test]
    fn test_program_validation_success() {
        let mut prog = Program::new();
        prog.source = "data".to_string();
        prog.operations = vec![Operation {
            dhatu: Dhatu::Drsh,
            execution: ExecutionMarker::Terminal,
        }];

        assert!(prog.validate().is_ok());
    }

    #[test]
    fn test_program_validation_missing_source() {
        let prog = Program::new();
        let result = prog.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("No source dataset declared"));
    }

    #[test]
    fn test_program_validation_no_operations() {
        let mut prog = Program::new();
        prog.source = "data".to_string();
        let result = prog.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No operations declared"));
    }

    #[test]
    fn test_program_validation_non_terminal_final() {
        let mut prog = Program::new();
        prog.source = "data".to_string();
        prog.operations = vec![Operation {
            dhatu: Dhatu::Chid,
            execution: ExecutionMarker::Continue,
        }];

        let result = prog.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Final operation must be terminal"));
    }

    #[test]
    fn test_lazy_operations() {
        let mut prog = Program::new();
        prog.source = "data".to_string();
        prog.operations = vec![
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

        let lazy = prog.lazy_operations();
        assert_eq!(lazy.len(), 2);
    }

    #[test]
    fn test_terminal_operation() {
        let mut prog = Program::new();
        prog.source = "data".to_string();
        prog.operations = vec![
            Operation {
                dhatu: Dhatu::Chid,
                execution: ExecutionMarker::Continue,
            },
            Operation {
                dhatu: Dhatu::Drsh,
                execution: ExecutionMarker::Terminal,
            },
        ];

        let term = prog.terminal_operation();
        assert!(term.is_some());
        assert_eq!(term.unwrap().dhatu, Dhatu::Drsh);
    }

    #[test]
    fn test_program_display() {
        let mut prog = Program::new();
        prog.source = "data".to_string();
        prog.instruments = vec!["sales".to_string()];
        prog.operations = vec![Operation {
            dhatu: Dhatu::Drsh,
            execution: ExecutionMarker::Terminal,
        }];

        let display = prog.to_string();
        assert!(display.contains("source: \"data\""));
        assert!(display.contains("instruments: [\"sales\"]"));
        assert!(display.contains("drsh(ti)"));
    }
}
