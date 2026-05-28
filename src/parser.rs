/// Syntactic analyzer for Pāṇini-RS.
///
/// The parser consumes a token stream (from the lexer) and produces an Abstract
/// Syntax Tree (AST). It implements a recursive descent parser that respects the
/// morphological structure and case-driven semantics of the language.
///
/// PARSING ALGORITHM:
/// 1. Extract source morpheme (Apādāna case, -āt)
/// 2. Extract instrument morphemes (Karaṇa case, -ena) - implements Anuvṛtti
/// 3. Extract operation sequence (verbal morphemes with dhātu + execution markers)
/// 4. Validate that final operation is terminal (-ti)
///
/// SEMANTIC RULES:
/// - Source must be first morpheme with -āt case
/// - Instruments must appear before operations
/// - Operations may be lazy (-tvā) or terminal (-ti)
/// - Exactly one terminal operation required (the final one)

use crate::ast::{Operation, Program};
use crate::token::{CaseMarker, ExecutionMarker, MorphemeCompound, Token, VerbForm};
use std::fmt;

/// Parse errors with diagnostic context
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Unexpected end of token stream
    UnexpectedEof,
    /// Expected a source morpheme but didn't find one
    NoSourceMorpheme,
    /// Expected a morpheme token but got something else
    ExpectedMorpheme { got: String },
    /// Expected a verb token but got something else
    ExpectedVerb { got: String },
    /// Semantic error: instruments appeared after operations
    InstrumentsAfterOperations,
    /// Semantic error: multiple source declarations
    MultipleSourceDeclations,
    /// Semantic error: no terminal operation
    NoTerminalOperation,
    /// Program validation failed
    ValidationError { reason: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected end of input"),
            ParseError::NoSourceMorpheme => {
                write!(f, "No source morpheme found (missing -āt suffix)")
            }
            ParseError::ExpectedMorpheme { got } => {
                write!(f, "Expected morpheme token, got: {}", got)
            }
            ParseError::ExpectedVerb { got } => {
                write!(f, "Expected verb token, got: {}", got)
            }
            ParseError::InstrumentsAfterOperations => {
                write!(
                    f,
                    "Instruments (-ena) must appear before operations (dhātu)"
                )
            }
            ParseError::MultipleSourceDeclations => {
                write!(f, "Multiple source declarations (-āt) not allowed")
            }
            ParseError::NoTerminalOperation => {
                write!(f, "No terminal operation (missing -ti)")
            }
            ParseError::ValidationError { reason } => {
                write!(f, "Program validation failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Strongly-typed result for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// The Pāṇini-RS parser.
///
/// Uses a recursive descent strategy to build the AST from a token stream.
/// The parser enforces structural and semantic constraints on programs.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser from a token stream
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    /// Parse a complete program from tokens
    ///
    /// PARSING PHASES:
    /// 1. Parse source declaration (Apādāna)
    /// 2. Parse instrument declarations (Karaṇa + Anuvṛtti)
    /// 3. Parse operation sequence (dhātu forms)
    /// 4. Validate program structure
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut program = Program::new();

        // Phase 1: Extract source morpheme (must be first)
        program.source = self.parse_source()?;

        // Phase 2: Extract instrument morphemes
        program.instruments = self.parse_instruments()?;

        // Phase 3: Extract operation sequence
        program.operations = self.parse_operations()?;

        // Phase 4: Validate program
        program
            .validate()
            .map_err(|reason| ParseError::ValidationError { reason })?;

        Ok(program)
    }

    /// Parse the source morpheme (Apādāna case, -āt)
    ///
    /// The source is the first token and must have -āt case marker.
    /// It identifies which dataset the program operates on.
    fn parse_source(&mut self) -> ParseResult<String> {
        let token = self.peek()?;

        match token {
            Token::Morpheme(m) => {
                if m.case != CaseMarker::Source {
                    return Err(ParseError::NoSourceMorpheme);
                }
                let source = m.root.clone();
                self.advance();
                Ok(source)
            }
            Token::Eof => Err(ParseError::NoSourceMorpheme),
            _ => Err(ParseError::ExpectedMorpheme {
                got: token.to_string(),
            }),
        }
    }

    /// Parse instrument morphemes (Karaṇa case, -ena)
    ///
    /// Instruments are parameters bound with -ena and inherited by all operations.
    /// Implements Anuvṛtti (automatic state inheritance).
    /// 
    /// Parsing continues while morphemes have -ena case.
    /// Stops at first verb token (operation).
    fn parse_instruments(&mut self) -> ParseResult<Vec<String>> {
        let mut instruments = Vec::new();

        loop {
            let token = self.peek()?;

            match token {
                Token::Morpheme(m) => {
                    match m.case {
                        CaseMarker::Instrument => {
                            instruments.push(m.root.clone());
                            self.advance();
                        }
                        CaseMarker::Source => {
                            return Err(ParseError::MultipleSourceDeclations);
                        }
                    }
                }
                Token::Verb(_) => {
                    // Operations have begun, stop collecting instruments
                    break;
                }
                Token::Eof => {
                    return Err(ParseError::NoTerminalOperation);
                }
            }
        }

        Ok(instruments)
    }

    /// Parse the operation sequence
    ///
    /// Operations are verb tokens (dhātu + execution marker).
    /// The sequence must end with a terminal operation (-ti).
    fn parse_operations(&mut self) -> ParseResult<Vec<Operation>> {
        let mut operations = Vec::new();

        loop {
            let token = self.peek()?;

            match token {
                Token::Verb(v) => {
                    let op = Operation {
                        dhatu: v.dhatu.clone(),
                        execution: v.execution.clone(),
                    };
                    operations.push(op);
                    self.advance();

                    // Terminal operation ends the program
                    if v.execution == ExecutionMarker::Terminal {
                        break;
                    }
                }
                Token::Morpheme(_) => {
                    return Err(ParseError::InstrumentsAfterOperations);
                }
                Token::Eof => {
                    return Err(ParseError::NoTerminalOperation);
                }
            }
        }

        Ok(operations)
    }

    /// Peek at the current token without consuming it
    fn peek(&self) -> ParseResult<Token> {
        if self.position < self.tokens.len() {
            Ok(self.tokens[self.position].clone())
        } else {
            Err(ParseError::UnexpectedEof)
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{CaseMarker, Dhatu, ExecutionMarker, MorphemeCompound, VerbForm};

    fn make_source_token(name: &str) -> Token {
        Token::Morpheme(MorphemeCompound {
            root: name.to_string(),
            case: CaseMarker::Source,
        })
    }

    fn make_instrument_token(name: &str) -> Token {
        Token::Morpheme(MorphemeCompound {
            root: name.to_string(),
            case: CaseMarker::Instrument,
        })
    }

    fn make_verb_token(dhatu: Dhatu, execution: ExecutionMarker) -> Token {
        Token::Verb(VerbForm { dhatu, execution })
    }

    #[test]
    fn test_parse_simple_program() {
        let tokens = vec![
            make_source_token("data"),
            make_verb_token(Dhatu::Drsh, ExecutionMarker::Terminal),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.source, "data");
        assert_eq!(program.instruments.len(), 0);
        assert_eq!(program.operations.len(), 1);
    }

    #[test]
    fn test_parse_with_instruments() {
        let tokens = vec![
            make_source_token("data"),
            make_instrument_token("sales"),
            make_verb_token(Dhatu::Drsh, ExecutionMarker::Terminal),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.source, "data");
        assert_eq!(program.instruments.len(), 1);
        assert_eq!(program.instruments[0], "sales");
    }

    #[test]
    fn test_parse_multiple_instruments() {
        let tokens = vec![
            make_source_token("data"),
            make_instrument_token("sales"),
            make_instrument_token("region"),
            make_verb_token(Dhatu::Drsh, ExecutionMarker::Terminal),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.instruments.len(), 2);
        assert_eq!(program.instruments[0], "sales");
        assert_eq!(program.instruments[1], "region");
    }

    #[test]
    fn test_parse_multiple_operations() {
        let tokens = vec![
            make_source_token("data"),
            make_instrument_token("sales"),
            make_verb_token(Dhatu::Chid, ExecutionMarker::Continue),
            make_verb_token(Dhatu::Yuj, ExecutionMarker::Continue),
            make_verb_token(Dhatu::Drsh, ExecutionMarker::Terminal),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.operations.len(), 3);
        assert_eq!(program.operations[0].dhatu, Dhatu::Chid);
        assert_eq!(program.operations[1].dhatu, Dhatu::Yuj);
        assert_eq!(program.operations[2].dhatu, Dhatu::Drsh);
    }

    #[test]
    fn test_parse_example_program() {
        // data-āt sales-ena chid-tvā yuj-tvā dṛś-ti
        let tokens = vec![
            make_source_token("data"),
            make_instrument_token("sales"),
            make_verb_token(Dhatu::Chid, ExecutionMarker::Continue),
            make_verb_token(Dhatu::Yuj, ExecutionMarker::Continue),
            make_verb_token(Dhatu::Drsh, ExecutionMarker::Terminal),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.source, "data");
        assert_eq!(program.instruments, vec!["sales"]);
        assert_eq!(program.operations.len(), 3);
    }

    #[test]
    fn test_parse_error_no_source() {
        let tokens = vec![
            make_instrument_token("sales"),
            make_verb_token(Dhatu::Drsh, ExecutionMarker::Terminal),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_no_terminal_operation() {
        let tokens = vec![
            make_source_token("data"),
            make_verb_token(Dhatu::Chid, ExecutionMarker::Continue),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_instruments_after_operations() {
        let tokens = vec![
            make_source_token("data"),
            make_verb_token(Dhatu::Chid, ExecutionMarker::Continue),
            make_instrument_token("sales"),
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }
}
