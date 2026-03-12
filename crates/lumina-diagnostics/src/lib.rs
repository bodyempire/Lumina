pub mod location;
pub mod render;

pub use location::{SourceLocation, extract_line};
pub use render::DiagnosticRenderer;

/// A fully-resolved compiler or runtime diagnostic.
/// Every error in v1.4 produces one of these.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String, // "L003", "R006", etc.
    pub message: String, // short human message
    pub location: SourceLocation,
    pub source_line: String, // raw text of the offending line
    pub help: Option<String>, // optional "help: ..." suggestion
}

impl Diagnostic {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        location: SourceLocation,
        source_line: impl Into<String>,
        help: Option<String>,
    ) -> Self {
        Self { 
            code: code.into(), 
            message: message.into(),
            location, 
            source_line: source_line.into(), 
            help 
        }
    }
}
