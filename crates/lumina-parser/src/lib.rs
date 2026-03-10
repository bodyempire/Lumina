pub mod ast;
pub mod error;
pub mod parser;

use lumina_lexer::{tokenize, LexError};
use crate::ast::Program;
use crate::error::ParseError;
use crate::parser::Parser;

#[derive(Debug)]
pub enum LuminaError {
    Lex(LexError),
    Parse(ParseError),
}

impl std::fmt::Display for LuminaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LuminaError::Lex(e) => write!(f, "{}", e),
            LuminaError::Parse(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for LuminaError {}

impl From<LexError> for LuminaError {
    fn from(e: LexError) -> Self { LuminaError::Lex(e) }
}

impl From<ParseError> for LuminaError {
    fn from(e: ParseError) -> Self { LuminaError::Parse(e) }
}

pub fn parse(source: &str) -> Result<Program, LuminaError> {
    let tokens = tokenize(source)?;
    let parser = Parser::new(tokens);
    let program = parser.parse()?;
    Ok(program)
}
