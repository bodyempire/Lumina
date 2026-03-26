use tower_lsp::lsp_types::*;
use lumina_diagnostics::Diagnostic as LuminaDiag;

/// Generate code actions (quick fixes) for known diagnostic codes.
pub fn generate_actions(uri: &Url, diags: &[LuminaDiag]) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    for d in diags {
        let l = d.location.line.saturating_sub(1);
        let c = d.location.col.saturating_sub(1);
        let range = Range {
            start: Position { line: l, character: c },
            end: Position { line: l, character: c + d.location.len.max(1) },
        };

        let diag = tower_lsp::lsp_types::Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(d.code.clone())),
            source: Some("lumina".into()),
            message: d.message.clone(),
            ..Default::default()
        };

        match d.code.as_str() {
            // L001: Unknown entity — suggest creating a stub entity definition
            "L001" => {
                // Extract the entity name from the diagnostic message
                let entity_name = extract_quoted(&d.message).unwrap_or("NewEntity".to_string());
                let insert_pos = Position { line: l.saturating_sub(1), character: 0 };

                let mut changes = std::collections::HashMap::new();
                changes.insert(uri.clone(), vec![TextEdit {
                    range: Range { start: insert_pos, end: insert_pos },
                    new_text: format!(
                        "entity {} {{\n  // TODO: define fields\n}}\n\n",
                        entity_name
                    ),
                }]);

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Create entity '{}'", entity_name),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diag]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }

            // L036: ref target entity doesn't exist — same fix as L001
            "L036" => {
                let entity_name = extract_quoted(&d.message).unwrap_or("TargetEntity".to_string());
                let insert_pos = Position { line: l.saturating_sub(1), character: 0 };

                let mut changes = std::collections::HashMap::new();
                changes.insert(uri.clone(), vec![TextEdit {
                    range: Range { start: insert_pos, end: insert_pos },
                    new_text: format!(
                        "entity {} {{\n  // TODO: define fields\n}}\n\n",
                        entity_name
                    ),
                }]);

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Create referenced entity '{}'", entity_name),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diag]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }

            // L024: prev() not allowed — suggest removing prev() wrapper
            "L024" => {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Remove prev() wrapper".to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diag]),
                    // We can't auto-edit prev() removal without knowing the inner expression,
                    // but we can offer the action and let the user fix it manually
                    edit: None,
                    ..Default::default()
                }));
            }

            // L035: Too many AND clauses — suggest splitting into multiple rules  
            "L035" => {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Split into multiple rules (max 3 AND clauses)".to_string(),
                    kind: Some(CodeActionKind::REFACTOR),
                    diagnostics: Some(vec![diag]),
                    edit: None,
                    ..Default::default()
                }));
            }

            // L038: write target must be external — suggest changing to update
            "L038" => {
                let mut changes = std::collections::HashMap::new();
                changes.insert(uri.clone(), vec![TextEdit {
                    range,
                    new_text: "update".to_string(),
                }]);

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Change 'write' to 'update' (target is not external)".to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diag]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }

            _ => {} // No quick fix for other codes
        }
    }

    actions
}

/// Extract a quoted string like 'Foo' from a diagnostic message.
fn extract_quoted(msg: &str) -> Option<String> {
    let start = msg.find('\'')?;
    let rest = &msg[start + 1..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}
