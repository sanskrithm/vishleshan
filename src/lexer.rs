/// ASCII-only lexer for Panini-RS.
///
/// Tokenizes input using hand-written FSM (no regex).
/// All tokens are ASCII transliteration only.
///
/// Example:
///   sales-at revenue-ena chid-tva yuj-tva drsh-ti
///
/// Tokens:
///   Morpheme("sales", "at")
///   Morpheme("revenue", "ena")
///   Verb("chid", "tva")
///   Verb("yuj", "tva")
///   Verb("drsh", "ti")

use crate::dhatu::{Dhatu, KarakaSuffix, SutraSuffix};
use std::fmt;

/// Lexical error types
#[derive(Debug, Clone)]
pub enum LexError {
    MissingSeparator { input: String },
    UnknownSuffix { suffix: String },
    InvalidUtf8 { details: String },
    EmptyToken,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator { input } => {
                write!(f, "Missing '-' separator in: '{}'", input)
            }
            Self::UnknownSuffix { suffix } => {
                write!(f, "Unknown suffix: '{}'", suffix)
            }
            Self::InvalidUtf8 { details } => {
                write!(f, "Invalid UTF-8: {}", details)
            }
            Self::EmptyToken => write!(f, "Empty token"),
        }
    }
}

pub type LexResult<T> = Result<T, LexError>;

/// Token types (ASCII transliteration)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Morpheme: root-karaka
    Morpheme {
        root: String,
        karaka: KarakaSuffix,
    },
    /// Verb form: dhatu-sutra
    Verb {
        dhatu: Dhatu,
        sutra: SutraSuffix,
    },
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Morpheme { root, karaka } => write!(f, "Morpheme({}-{})", root, karaka),
            Self::Verb { dhatu, sutra } => write!(f, "Verb({}-{})", dhatu, sutra),
            Self::Eof => write!(f, "EOF"),
        }
    }
}

/// ASCII lexer using hand-written FSM
pub struct Lexer {
    input: String,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: impl Into<String>) -> Self {
        Lexer {
            input: input.into(),
            tokens: Vec::new(),
        }
    }

    /// Tokenize input string
    pub fn tokenize(&mut self) -> LexResult<Vec<Token>> {
        self.tokens.clear();

        for token_str in self.input.split_whitespace() {
            if token_str.is_empty() {
                continue;
            }
            let token = self.parse_token(token_str)?;
            self.tokens.push(token);
        }

        self.tokens.push(Token::Eof);
        Ok(self.tokens.clone())
    }

    /// Parse a single token
    fn parse_token(&self, token_str: &str) -> LexResult<Token> {
        // Split on '-'
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

        // Try Karaka suffix first (morpheme)
        if let Ok(karaka) = self.parse_karaka(suffix) {
            return Ok(Token::Morpheme {
                root: root.to_string(),
                karaka,
            });
        }

        // Try Sutra + Dhatu (verb)
        if let Ok(sutra) = self.parse_sutra(suffix) {
            if let Some(dhatu) = Dhatu::from_ascii(root) {
                return Ok(Token::Verb { dhatu, sutra });
            }
        }

        Err(LexError::UnknownSuffix {
            suffix: suffix.to_string(),
        })
    }

    /// Parse Karaka suffix
    fn parse_karaka(&self, s: &str) -> LexResult<KarakaSuffix> {
        KarakaSuffix::from_ascii(s).ok_or(LexError::UnknownSuffix {
            suffix: s.to_string(),
        })
    }

    /// Parse Sutra suffix
    fn parse_sutra(&self, s: &str) -> LexResult<SutraSuffix> {
        SutraSuffix::from_ascii(s).ok_or(LexError::UnknownSuffix {
            suffix: s.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_source() {
        let mut lexer = Lexer::new("sales-at");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // token + EOF
        match &tokens[0] {
            Token::Morpheme { root, karaka } => {
                assert_eq!(root, "sales");
                assert_eq!(*karaka, KarakaSuffix::At);
            }
            _ => panic!("Expected morpheme"),
        }
    }

    #[test]
    fn test_lex_verb() {
        let mut lexer = Lexer::new("chid-tva");
        let tokens = lexer.tokenize().unwrap();
        match &tokens[0] {
            Token::Verb { dhatu, sutra } => {
                assert_eq!(*dhatu, Dhatu::Transformation(crate::dhatu::TransformationDhatu::Chid));
                assert_eq!(*sutra, SutraSuffix::Tva);
            }
            _ => panic!("Expected verb"),
        }
    }

    #[test]
    fn test_lex_complete_program() {
        let mut lexer = Lexer::new("sales-at revenue-ena chid-tva yuj-tva drsh-ti");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 6); // 5 tokens + EOF
    }

    #[test]
    fn test_lex_error_missing_separator() {
        let mut lexer = Lexer::new("noseparator");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }
}
