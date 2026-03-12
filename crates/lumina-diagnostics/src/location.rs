#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32, // 1-indexed
    pub col: u32,  // 1-indexed
    pub len: u32,  // highlight width in chars (minimum 1)
}

impl SourceLocation {
    pub fn new(file: impl Into<String>, line: u32, col: u32, len: u32) -> Self {
        Self { file: file.into(), line, col, len: len.max(1) }
    }

    /// Build from a Span (which carries line + col from the lexer).
    /// span.line and span.col are already 1-indexed in the v1.3 lexer.
    pub fn from_span(line: u32, col: u32, len: u32, file: impl Into<String>) -> Self {
        Self::new(file, line, col, len)
    }
}

/// Extract the Nth line (1-indexed) from source text.
/// Returns empty string if line number is out of range.
pub fn extract_line(source: &str, line_num: u32) -> String {
    source
        .lines()
        .nth((line_num.saturating_sub(1)) as usize)
        .unwrap_or("")
        .to_string()
}
