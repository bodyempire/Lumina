pub mod ast {
    pub use lumina_parser::ast::*;
}
pub mod types;
pub mod graph;
pub mod analyzer;

pub use analyzer::{Analyzer, AnalyzerError, AnalyzedProgram};
use lumina_parser::ast::Program;
use lumina_diagnostics::{Diagnostic, SourceLocation, extract_line};

pub fn analyze(program: Program, source: &str, filename: &str, allow_imports: bool) -> Result<AnalyzedProgram, Vec<Diagnostic>> {
    let mut analyzer = Analyzer::new();
    analyzer.allow_imports = allow_imports;
    match analyzer.analyze(program) {
        Ok(analyzed) => Ok(analyzed),
        Err(raw_errors) => {
            let diags = raw_errors.into_iter().map(|e| {
                Diagnostic::new(
                    e.code.to_string(),
                    e.message.to_string(),
                    SourceLocation::from_span(e.span.line, e.span.col, e.span.end.saturating_sub(e.span.start).max(1), filename),
                    extract_line(source, e.span.line),
                    help_for_code(&e.code),
                )
            }).collect();
            Err(diags)
        }
    }
}

fn help_for_code(code: &str) -> Option<String> {
    match code {
        "L001" => Some("rename one of the entity declarations".into()),
        "L002" => Some("check spelling or add the entity declaration".into()),
        "L003" => Some("break the cycle by making one field stored (field: Type)".into()),
        "L004" => Some("verify the field type and the literal type match".into()),
        "L005" => Some("check field spelling or add the field to the entity".into()),
        "L006" => Some("@range only applies to Number fields; ensure min < max".into()),
        "L007" => Some("check entity name in the when clause".into()),
        "L008" => Some("add a let binding for the instance before using it".into()),
        "L009" => Some("instance names must be globally unique".into()),
        "L010" => Some("@affects only applies to stored fields".into()),
        "R004" => Some("check the list is non-empty and the index is within range".into()),
        _ => None,
    }
}
