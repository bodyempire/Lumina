use tower_lsp::lsp_types::*;
use lumina_diagnostics::Diagnostic;

pub fn to_lsp_diags(ds: &[Diagnostic]) -> Vec<tower_lsp::lsp_types::Diagnostic> {
    ds.iter().map(|d| {
        let l = d.location.line.saturating_sub(1);
        let c = d.location.col.saturating_sub(1);
        tower_lsp::lsp_types::Diagnostic {
            range: Range {
                start: Position { line: l, character: c },
                end: Position { line: l, character: c + d.location.len.max(1) },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(d.code.clone())),
            source: Some("lumina".into()),
            message: d.message.clone(),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        }
    }).collect()
}
