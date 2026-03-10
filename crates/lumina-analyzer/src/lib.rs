pub mod ast {
    pub use lumina_parser::ast::*;
}
pub mod types;
pub mod graph;
pub mod analyzer;

pub use analyzer::{Analyzer, AnalyzerError, AnalyzedProgram};
use lumina_parser::ast::Program;

pub fn analyze(program: Program) -> Result<AnalyzedProgram, Vec<AnalyzerError>> {
    let analyzer = Analyzer::new();
    analyzer.analyze(program)
}
