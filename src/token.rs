/// Token definitions for the Pāṇini-RS lexer.
/// 
/// Tokens represent morphological units corresponding to Sanskrit grammatical cases.
/// This module defines the strongly-typed token enumeration that serves as the
/// intermediate representation between lexical analysis and parsing.
///
/// LINGUISTIC MAPPING:
/// - Kāraka suffixes encode semantic relationships (source, instrument, agent)
/// - Sūtra suffixes encode control flow (pipeline continuation, terminal)
/// - Dhātu roots encode semantic operations

use std::fmt;

/// Source/Apādāna case: indicates source data or origin
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CaseMarker {
    /// -āt suffix: source/apādāna (where data comes from)
    Source,
    /// -ena suffix: instrument/karaṇa (inherited parameter state)
    Instrument,
}

impl fmt::Display for CaseMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaseMarker::Source => write!(f, "āt"),
            CaseMarker::Instrument => write!(f, "ena"),
        }
    }
}

/// Sūtra suffixes control execution semantics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionMarker {
    /// -tvā suffix: pipeline continuation (lazy operation)
    Continue,
    /// -ti suffix: terminal execution (trigger evaluation)
    Terminal,
}

impl fmt::Display for ExecutionMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMarker::Continue => write!(f, "tvā"),
            ExecutionMarker::Terminal => write!(f, "ti"),
        }
    }
}

/// Dhātu (verb roots) are semantic opcodes for operations
/// 
/// These correspond to operations in the execution graph.
/// Inspired by Sanskrit dhātupāṭha (verb root tables).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dhatu {
    /// chid: filter operation (apply predicates)
    Chid,
    /// ci: group operation (aggregation grouping)
    Ci,
    /// yuj: sum/aggregate operation (numerical reduction)
    Yuj,
    /// adhyaya: select/project operation (projection of columns)
    Adhyaya,
    /// dṛś/drsh: render/print operation (visualization/execution)
    Drsh,
    /// Custom dhātu with string identifier
    Custom(String),
}

impl fmt::Display for Dhatu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dhatu::Chid => write!(f, "chid"),
            Dhatu::Ci => write!(f, "ci"),
            Dhatu::Yuj => write!(f, "yuj"),
            Dhatu::Adhyaya => write!(f, "adhyaya"),
            Dhatu::Drsh => write!(f, "dṛś"),
            Dhatu::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// A compound morpheme pairing a nominal root with a case marker.
///
/// Represents the pattern: root-suffix (e.g., "data-āt", "sales-ena")
/// This is the fundamental semantic unit binding identifiers to case relations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphemeCompound {
    /// Root identifier (dataset name, parameter name, etc.)
    pub root: String,
    /// Grammatical case marker encoding semantic relationship
    pub case: CaseMarker,
}

impl fmt::Display for MorphemeCompound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.root, self.case)
    }
}

/// A verb form pairing a dhātu root with execution marker.
///
/// Represents the pattern: dhātu-marker (e.g., "chid-tvā", "dṛś-ti")
/// This encodes the semantic operation and whether it's lazy or terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbForm {
    /// Dhātu (operation root)
    pub dhatu: Dhatu,
    /// Execution marker controlling lazy vs terminal evaluation
    pub execution: ExecutionMarker,
}

impl fmt::Display for VerbForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.dhatu, self.execution)
    }
}

/// Strongly-typed token enumeration.
///
/// Each variant represents a distinct lexical unit with semantic significance.
/// The parser consumes these tokens to build the Abstract Syntax Tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Nominal morpheme compound: identifier + case suffix
    /// E.g., TokenKind::Morpheme("data", Source)
    Morpheme(MorphemeCompound),
    
    /// Verbal morpheme: dhātu + execution suffix
    /// E.g., TokenKind::Verb(Chid, Continue)
    Verb(VerbForm),
    
    /// End of program marker
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Morpheme(m) => write!(f, "Morpheme({})", m),
            Token::Verb(v) => write!(f, "Verb({})", v),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_marker_display() {
        assert_eq!(CaseMarker::Source.to_string(), "āt");
        assert_eq!(CaseMarker::Instrument.to_string(), "ena");
    }

    #[test]
    fn test_execution_marker_display() {
        assert_eq!(ExecutionMarker::Continue.to_string(), "tvā");
        assert_eq!(ExecutionMarker::Terminal.to_string(), "ti");
    }

    #[test]
    fn test_dhatu_display() {
        assert_eq!(Dhatu::Chid.to_string(), "chid");
        assert_eq!(Dhatu::Ci.to_string(), "ci");
        assert_eq!(Dhatu::Yuj.to_string(), "yuj");
        assert_eq!(Dhatu::Drsh.to_string(), "dṛś");
    }

    #[test]
    fn test_morpheme_compound() {
        let m = MorphemeCompound {
            root: "data".to_string(),
            case: CaseMarker::Source,
        };
        assert_eq!(m.to_string(), "data-āt");
    }

    #[test]
    fn test_verb_form() {
        let v = VerbForm {
            dhatu: Dhatu::Chid,
            execution: ExecutionMarker::Continue,
        };
        assert_eq!(v.to_string(), "chid-tvā");
    }

    #[test]
    fn test_token_display() {
        let m = Token::Morpheme(MorphemeCompound {
            root: "sales".to_string(),
            case: CaseMarker::Instrument,
        });
        assert_eq!(m.to_string(), "Morpheme(sales-ena)");

        let v = Token::Verb(VerbForm {
            dhatu: Dhatu::Yuj,
            execution: ExecutionMarker::Continue,
        });
        assert_eq!(v.to_string(), "Verb(yuj-tvā)");
    }
}
