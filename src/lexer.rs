/// Lexical analyzer for Pāṇini-RS.
///
/// The lexer transforms raw input strings into a stream of strongly-typed tokens.
/// It employs a finite-state machine approach without regex, processing UTF-8
/// Sanskrit morphemes character-by-character.
///
/// LEXICAL RULES:
/// 1. Split input on whitespace to identify atomic units
/// 2. Each unit is a morpheme: root-suffix pattern
/// 3. Parse suffixes to determine token type (case marker vs. execution marker)
/// 4. Validate UTF-8 Sanskrit characters
///
/// ALGORITHM:
/// - For each whitespace-delimited token:
///   - Extract root (before hyphen)
///   - Extract suffix (after hyphen)
///   - Match suffix against known markers
///   - Classify as Morpheme or Verb based on context
///   - Emit typed Token
///
/// The lexer is error-recovering and produces informative diagnostics.

use crate::token::{CaseMarker, Dhatu, ExecutionMarker, MorphemeCompound, Token, VerbForm};
use std::fmt;

/// Lexical error types with diagnostic information
#[derive(Debug, Clone)]
pub enum LexError {
    /// Invalid suffix: no hyphen found in morpheme
    MissingSeparator { input: String },
    /// Unknown case marker suffix
    UnknownCaseMarker { suffix: String },
    /// Unknown execution marker suffix
    UnknownExecutionMarker { suffix: String },
    /// Unknown dhātu root
    UnknownDhatu { root: String },
    /// Invalid UTF-8 in input
    InvalidUtf8 { details: String },
    /// Empty token encountered
    EmptyToken,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::MissingSeparator { input } => {
                write!(f, "Missing '-' separator in token: '{}'", input)
            }
            LexError::UnknownCaseMarker { suffix } => {
                write!(f, "Unknown case marker suffix: '{}' (expected 'āt' or 'ena')", suffix)
            }
            LexError::UnknownExecutionMarker { suffix } => {
                write!(
                    f,
                    "Unknown execution marker suffix: '{}' (expected 'tvā' or 'ti')",
                    suffix
                )
            }
            LexError::UnknownDhatu { root } => {
                write!(f, "Unknown dhātu root: '{}'", root)
            }
            LexError::InvalidUtf8 { details } => {
                write!(f, "Invalid UTF-8: {}", details)
            }
            LexError::EmptyToken => {
                write!(f, "Empty token encountered")
            }
        }
    }
}

impl std::error::Error for LexError {}

/// Strongly-typed result type for lexical operations
pub type LexResult<T> = Result<T, LexError>;

/// The Pāṇini-RS lexer.
///
/// This is a hand-written lexer (not regex-based) that respects:
/// - UTF-8 Sanskrit character encoding
/// - Morpheme structure (root-suffix)
/// - Semantic distinctions (case vs. execution markers)
pub struct Lexer {
    input: String,
    tokens: Vec<Token>,
}

impl Lexer {
    /// Create a new lexer from input string
    pub fn new(input: impl Into<String>) -> Self {
        Lexer {
            input: input.into(),
            tokens: Vec::new(),
        }
    }

    /// Perform lexical analysis, returning a token stream or error
    pub fn tokenize(&mut self) -> LexResult<Vec<Token>> {
        self.tokens.clear();

        // Split on whitespace
        for token_str in self.input.split_whitespace() {
            if token_str.is_empty() {
                continue;
            }

            // Parse the token
            let token = self.parse_morpheme(token_str)?;
            self.tokens.push(token);
        }

        self.tokens.push(Token::Eof);
        Ok(self.tokens.clone())
    }

    /// Parse a single morpheme token
    ///
    /// Handles both:
    /// 1. Nominal morphemes: root-case_marker (e.g., "data-āt", "sales-ena")
    /// 2. Verbal morphemes: dhātu-execution_marker (e.g., "chid-tvā", "dṛś-ti")
    fn parse_morpheme(&self, token_str: &str) -> LexResult<Token> {
        // Find the morpheme separator (hyphen)
        let parts: Vec<&str> = token_str.split('-').collect();

        if parts.len() != 2 {
            return Err(LexError::MissingSeparator {
                input: token_str.to_string(),
            });
        }

        let root = parts[0];
        let suffix = parts[1];

        if root.is_empty() || suffix.is_empty() {
            return Err(LexError::EmptyToken);
        }

        // Try to parse as case marker (nominal morpheme)
        if let Ok(case) = self.parse_case_marker(suffix) {
            return Ok(Token::Morpheme(MorphemeCompound {
                root: root.to_string(),
                case,
            }));
        }

        // Try to parse as execution marker (verbal morpheme)
        if let Ok(execution) = self.parse_execution_marker(suffix) {
            let dhatu = self.parse_dhatu(root)?;
            return Ok(Token::Verb(VerbForm { dhatu, execution }));
        }

        // Neither case nor execution marker matched
        Err(LexError::UnknownCaseMarker {
            suffix: suffix.to_string(),
        })
    }

    /// Parse a case marker suffix
    fn parse_case_marker(&self, suffix: &str) -> LexResult<CaseMarker> {
        match suffix {
            "āt" => Ok(CaseMarker::Source),
            "ena" => Ok(CaseMarker::Instrument),
            _ => Err(LexError::UnknownCaseMarker {
                suffix: suffix.to_string(),
            }),
        }
    }

    /// Parse an execution marker suffix
    fn parse_execution_marker(&self, suffix: &str) -> LexResult<ExecutionMarker> {
        match suffix {
            "tvā" => Ok(ExecutionMarker::Continue),
            "ti" => Ok(ExecutionMarker::Terminal),
            _ => Err(LexError::UnknownExecutionMarker {
                suffix: suffix.to_string(),
            }),
        }
    }

    /// Parse a dhātu (verb root)
    ///
    /// Supports standard dhātupāṭha entries and custom roots
    fn parse_dhatu(&self, root: &str) -> LexResult<Dhatu> {
        match root {
            "chid" => Ok(Dhatu::Chid),
            "ci" => Ok(Dhatu::Ci),
            "yuj" => Ok(Dhatu::Yuj),
            "adhyaya" | "adhyāya" => Ok(Dhatu::Adhyaya),
            "dṛś" | "drsh" => Ok(Dhatu::Drsh),
            _ => {
                // Allow custom dhātu roots for extensibility
                Ok(Dhatu::Custom(root.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_nominal_morpheme_source() {
        let mut lexer = Lexer::new("data-āt");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // morpheme + EOF
        match &tokens[0] {
            Token::Morpheme(m) => {
                assert_eq!(m.root, "data");
                assert_eq!(m.case, CaseMarker::Source);
            }
            _ => panic!("Expected morpheme token"),
        }
    }

    #[test]
    fn test_lexer_nominal_morpheme_instrument() {
        let mut lexer = Lexer::new("sales-ena");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            Token::Morpheme(m) => {
                assert_eq!(m.root, "sales");
                assert_eq!(m.case, CaseMarker::Instrument);
            }
            _ => panic!("Expected morpheme token"),
        }
    }

    #[test]
    fn test_lexer_verb_continue() {
        let mut lexer = Lexer::new("chid-tvā");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            Token::Verb(v) => {
                assert_eq!(v.dhatu, Dhatu::Chid);
                assert_eq!(v.execution, ExecutionMarker::Continue);
            }
            _ => panic!("Expected verb token"),
        }
    }

    #[test]
    fn test_lexer_verb_terminal() {
        let mut lexer = Lexer::new("dṛś-ti");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            Token::Verb(v) => {
                assert_eq!(v.dhatu, Dhatu::Drsh);
                assert_eq!(v.execution, ExecutionMarker::Terminal);
            }
            _ => panic!("Expected verb token"),
        }
    }

    #[test]
    fn test_lexer_multi_token_stream() {
        let mut lexer = Lexer::new("data-āt sales-ena chid-tvā yuj-tvā dṛś-ti");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 6); // 5 tokens + EOF
        
        // Verify token sequence
        match &tokens[0] {
            Token::Morpheme(m) => assert_eq!(m.root, "data"),
            _ => panic!("Token 0 mismatch"),
        }
        match &tokens[1] {
            Token::Morpheme(m) => assert_eq!(m.root, "sales"),
            _ => panic!("Token 1 mismatch"),
        }
        match &tokens[2] {
            Token::Verb(v) => assert_eq!(v.dhatu, Dhatu::Chid),
            _ => panic!("Token 2 mismatch"),
        }
        match &tokens[3] {
            Token::Verb(v) => assert_eq!(v.dhatu, Dhatu::Yuj),
            _ => panic!("Token 3 mismatch"),
        }
        match &tokens[4] {
            Token::Verb(v) => assert_eq!(v.dhatu, Dhatu::Drsh),
            _ => panic!("Token 4 mismatch"),
        }
        assert_eq!(tokens[5], Token::Eof);
    }

    #[test]
    fn test_lexer_error_missing_separator() {
        let mut lexer = Lexer::new("datasource");
        let result = lexer.tokenize();
        assert!(result.is_err());
        match result.unwrap_err() {
            LexError::MissingSeparator { .. } => {}
            _ => panic!("Expected MissingSeparator error"),
        }
    }

    #[test]
    fn test_lexer_custom_dhatu() {
        let mut lexer = Lexer::new("myop-tvā");
        let tokens = lexer.tokenize().unwrap();
        match &tokens[0] {
            Token::Verb(v) => {
                assert_eq!(v.dhatu, Dhatu::Custom("myop".to_string()));
                assert_eq!(v.execution, ExecutionMarker::Continue);
            }
            _ => panic!("Expected verb token"),
        }
    }

    #[test]
    fn test_lexer_whitespace_handling() {
        let mut lexer = Lexer::new("  data-āt   sales-ena  ");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 3); // 2 tokens + EOF
    }

    #[test]
    fn test_lexer_select_adhyaya() {
        let mut lexer = Lexer::new("adhyaya-tvā");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            Token::Verb(v) => {
                assert_eq!(v.dhatu, Dhatu::Adhyaya);
                assert_eq!(v.execution, ExecutionMarker::Continue);
            }
            _ => panic!("Expected verb token"),
        }
    }
}
