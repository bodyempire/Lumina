use tower_lsp::lsp_types::*;
use lumina_parser::ast::*;

/// Generate inlay hints for a Lumina program.
///
/// Current hints:
/// - Derived fields: show inferred type based on the expression structure
/// - Stored fields with @range: show the valid range inline
/// - Ref fields: show the target entity type
pub fn get_inlay_hints(prog: &Program) -> Vec<InlayHint> {
    let mut hints = Vec::new();

    for stmt in &prog.statements {
        let fields = match stmt {
            Statement::Entity(e) => &e.fields,
            Statement::ExternalEntity(e) => &e.fields,
            _ => continue,
        };

        for f in fields {
            match f {
                Field::Derived(df) => {
                    let inferred = infer_expr_type(&df.expr);
                    let l = df.span.line.saturating_sub(1);
                    let c = df.span.col.saturating_sub(1) + df.name.len() as u32;
                    hints.push(InlayHint {
                        position: Position { line: l, character: c },
                        label: InlayHintLabel::String(format!(": {}", inferred)),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(InlayHintTooltip::String(
                            format!("Derived field — computed as {}", inferred)
                        )),
                        padding_left: Some(true),
                        padding_right: None,
                        data: None,
                    });
                }
                Field::Stored(sf) => {
                    // Show @range as an inlay hint if present
                    if let Some((lo, hi)) = sf.metadata.range {
                        let l = sf.span.line.saturating_sub(1);
                        let c = sf.span.col.saturating_sub(1) + sf.name.len() as u32 + format!(": {:?}", sf.ty).len() as u32;
                        hints.push(InlayHint {
                            position: Position { line: l, character: c },
                            label: InlayHintLabel::String(format!(" [{} .. {}]", lo, hi)),
                            kind: None,
                            text_edits: None,
                            tooltip: Some(InlayHintTooltip::String(
                                format!("Valid range: {} to {}", lo, hi)
                            )),
                            padding_left: Some(true),
                            padding_right: None,
                            data: None,
                        });
                    }
                }
                Field::Ref(rf) => {
                    let l = rf.span.line.saturating_sub(1);
                    let c = rf.span.col.saturating_sub(1) + rf.name.len() as u32;
                    hints.push(InlayHint {
                        position: Position { line: l, character: c },
                        label: InlayHintLabel::String(format!("→ {}", rf.target_entity)),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(InlayHintTooltip::String(
                            format!("Reference to entity '{}'", rf.target_entity)
                        )),
                        padding_left: Some(true),
                        padding_right: None,
                        data: None,
                    });
                }
            }
        }
    }

    hints
}

/// Simple type inference from expression structure.
fn infer_expr_type(expr: &Expr) -> &'static str {
    match expr {
        Expr::Number(_) => "Number",
        Expr::Text(_) | Expr::InterpolatedString(_) => "Text",
        Expr::Bool(_) => "Boolean",
        Expr::ListLiteral(_) => "List",
        Expr::Binary { op, .. } => {
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => "Number",
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => "Boolean",
                BinOp::And | BinOp::Or => "Boolean",
            }
        }
        Expr::Unary { op, .. } => {
            match op {
                UnOp::Neg => "Number",
                UnOp::Not => "Boolean",
            }
        }
        Expr::If { then_, .. } => infer_expr_type(then_),
        Expr::Call { name, .. } => {
            match name.as_str() {
                "abs" | "round" | "floor" | "ceil" | "min" | "max" | "len" | "avg"
                | "sum" | "count" | "clamp" => "Number",
                "now" => "Timestamp",
                "contains" | "startsWith" | "endsWith" => "Boolean",
                "upper" | "lower" | "trim" | "format" | "join" => "Text",
                _ => "dynamic",
            }
        }
        Expr::FieldAccess { field, .. } => {
            // .age returns Number, other field accesses are dynamic
            if field == "age" { "Number" } else { "dynamic" }
        }
        _ => "dynamic",
    }
}
