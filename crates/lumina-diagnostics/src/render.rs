use crate::Diagnostic;

pub struct DiagnosticRenderer;

impl DiagnosticRenderer {
    /// Render one diagnostic to a multi-line string.
    pub fn render(d: &Diagnostic) -> String {
        let mut out = String::new();

        // Header: error[L003]: message
        out.push_str(&format!("error[{}]: {}\n", d.code, d.message));

        // Location: --> file.lum:4:3
        out.push_str(&format!(" --> {}:{}:{}\n", 
            d.location.file, d.location.line, d.location.col));

        // Gutter: build padding to align line numbers
        let gutter = d.location.line.to_string();
        let pad = " ".repeat(gutter.len());
        
        out.push_str(&format!("{} |\n", pad));
        out.push_str(&format!("{} | {}\n", gutter, d.source_line));

        // Caret: spaces + carets under the error token
        let spaces = " ".repeat((d.location.col.saturating_sub(1)) as usize);
        let carets = "^".repeat(d.location.len.max(1) as usize);
        
        out.push_str(&format!("{} | {}{}\n", pad, spaces, carets));
        out.push_str(&format!("{} |\n", pad));

        // Optional help line
        if let Some(help) = &d.help {
            out.push_str(&format!(" = help: {}\n", help));
        }

        out
    }

    /// Render multiple diagnostics, separated by blank lines.
    pub fn render_all(diags: &[Diagnostic]) -> String {
        diags.iter()
            .map(Self::render)
            .collect::<Vec<_>>()
            .join("\n")
    }
}
