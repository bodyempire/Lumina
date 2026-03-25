use tower_lsp::lsp_types::*;
use lumina_parser::ast::{Program, Statement, Field};

pub fn hover_at(prog: &Program, _src: &str, pos: Position) -> Option<Hover> {
    for stmt in &prog.statements {
        let fields = match stmt {
            Statement::Entity(e) => &e.fields,
            Statement::ExternalEntity(e) => &e.fields,
            _ => continue,
        };

        for f in fields {
            let (name, span, doc, range) = match f {
                Field::Stored(sf) => (
                    &sf.name,
                    &sf.span,
                    &sf.metadata.doc,
                    sf.metadata.range,
                ),
                Field::Derived(df) => (
                    &df.name,
                    &df.span,
                    &df.metadata.doc,
                    df.metadata.range,
                ),
                Field::Ref(rf) => (
                    &rf.name,
                    &rf.span,
                    &None,
                    None,
                ),
            };

            let type_label = match f {
                Field::Stored(sf) => format!("{:?}", sf.ty),
                Field::Derived(_) => "derived".to_string(),
                Field::Ref(rf) => format!("ref {}", rf.target_entity),
            };

            let l = span.line.saturating_sub(1);
            let c = span.col.saturating_sub(1);
            let field_len = (span.end.saturating_sub(span.start)).max(1) as u32;

            if pos.line == l && pos.character >= c && pos.character <= c + field_len {
                let mut lines = vec![format!("**{}**: {}", name, type_label)];
                if let Some(d) = doc { lines.push(d.clone()); }
                if let Some((lo, hi)) = range { lines.push(format!("Range: {} to {}", lo, hi)); }
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: lines.join("\n\n"),
                    }),
                    range: None,
                });
            }
        }
    }
    None
}
