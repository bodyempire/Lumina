use lumina_lexer::token::Span;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span:    Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self { message: message.into(), span }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Parse error at line {}: {}", self.span.line, self.message)
    }
}

impl std::error::Error for ParseError {}
