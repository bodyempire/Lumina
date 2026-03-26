use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use lumina_parser::parse;
use lumina_analyzer::analyze;
use crate::{diag::to_lsp_diags, hover::hover_at};
use crate::semantic;
use crate::refs;
use crate::action;
use crate::inlay;

pub struct LuminaBackend {
    client: Client,
    docs: DashMap<Url, (String, Option<lumina_parser::ast::Program>, Vec<lumina_diagnostics::Diagnostic>)>,
}

impl LuminaBackend {
    pub fn new(client: Client) -> Self {
        Self { client, docs: DashMap::new() }
    }

    async fn refresh(&self, uri: Url, src: String) {
        let prog = parse(&src).ok();
        let diags = match &prog {
            Some(p) => {
                let mut merged_prog = p.clone();
                
                // Simple recursive import resolution for the LSP context
                fn load_imports(prog: &lumina_parser::ast::Program, dir: &std::path::Path, out: &mut Vec<lumina_parser::ast::Statement>, visited: &mut std::collections::HashSet<std::path::PathBuf>) {
                    for import in prog.imports() {
                        let dep_path = dir.join(&import.path);
                        if let Ok(dep_canonical) = dep_path.canonicalize() {
                            if visited.insert(dep_canonical.clone()) {
                                if let Ok(dep_src) = std::fs::read_to_string(&dep_canonical) {
                                    if let Ok(dep_prog) = parse(&dep_src) {
                                        // Recursively load transitive imports
                                        load_imports(&dep_prog, dep_canonical.parent().unwrap(), out, visited);
                                        // Collect statements
                                        for stmt in dep_prog.statements {
                                            if !matches!(stmt, lumina_parser::ast::Statement::Import(_)) {
                                                out.push(stmt);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                let mut visited = std::collections::HashSet::new();
                let mut extra_statements = Vec::new();
                if let Ok(path) = uri.to_file_path() {
                    if let Ok(canonical) = path.canonicalize() {
                        visited.insert(canonical.clone());
                        load_imports(p, canonical.parent().unwrap(), &mut extra_statements, &mut visited);
                    }
                }
                
                // Prepend imported statements so they are available
                let mut final_statements = extra_statements;
                for stmt in p.statements.clone() {
                    if !matches!(stmt, lumina_parser::ast::Statement::Import(_)) {
                        final_statements.push(stmt);
                    }
                }
                merged_prog.statements = final_statements;

                match analyze(merged_prog, &src, uri.path(), true) {
                    Ok(_analyzed) => vec![],
                    Err(diagnostics) => diagnostics,
                }
            }
            None => vec![],
        };
        self.docs.insert(uri.clone(), (src, prog, diags.clone()));
        self.client.publish_diagnostics(uri, to_lsp_diags(&diags), None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LuminaBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), " ".into()]),
                    ..Default::default()
                }),
                rename_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                    SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: semantic::LEGEND_TYPES.to_vec(),
                            token_modifiers: vec![],
                        },
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        ..Default::default()
                    }
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> { Ok(()) }

    async fn did_open(&self, p: DidOpenTextDocumentParams) {
        self.refresh(p.text_document.uri, p.text_document.text).await;
    }

    async fn did_change(&self, p: DidChangeTextDocumentParams) {
        let src = p.content_changes.into_iter()
            .last().map(|c| c.text).unwrap_or_default();
        self.refresh(p.text_document.uri, src).await;
    }

    async fn hover(&self, p: HoverParams) -> Result<Option<Hover>> {
        let uri = p.text_document_position_params.text_document.uri;
        let pos = p.text_document_position_params.position;
        Ok(self.docs.get(&uri).and_then(|e| {
            let (src, prog, _) = e.value();
            prog.as_ref().and_then(|p| hover_at(p, src, pos))
        }))
    }

    async fn references(&self, p: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = p.text_document_position.text_document.uri;
        let pos = p.text_document_position.position;
        if let Some(doc) = self.docs.get(&uri) {
            let (src, prog, _) = doc.value();
            if let Some(prog) = prog {
                if let Some(symbol) = refs::symbol_at_position(prog, src, pos) {
                    let locations = refs::find_references_in_program(prog, &symbol, &uri);
                    return Ok(Some(locations));
                }
            }
        }
        Ok(Some(vec![]))
    }

    async fn rename(&self, p: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = p.text_document_position.text_document.uri;
        let pos = p.text_document_position.position;
        let new_name = p.new_name;
        if let Some(doc) = self.docs.get(&uri) {
            let (src, prog, _) = doc.value();
            if let Some(prog) = prog {
                if let Some(old_name) = refs::symbol_at_position(prog, src, pos) {
                    let edits = refs::build_rename_edits(prog, src, &uri, &old_name, &new_name);
                    if !edits.is_empty() {
                        let mut changes = std::collections::HashMap::new();
                        changes.insert(uri.clone(), edits);
                        return Ok(Some(WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        }));
                    }
                }
            }
        }
        Ok(None)
    }

    async fn code_action(&self, p: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = p.text_document.uri;
        if let Some(doc) = self.docs.get(&uri) {
            let (_, _, diags) = doc.value();
            let actions = action::generate_actions(&uri, diags);
            return Ok(Some(actions));
        }
        Ok(None)
    }

    async fn inlay_hint(&self, p: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = p.text_document.uri;
        if let Some(doc) = self.docs.get(&uri) {
            if let Some(prog) = &doc.value().1 {
                return Ok(Some(inlay::get_inlay_hints(prog)));
            }
        }
        Ok(None)
    }

    async fn semantic_tokens_full(&self, p: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let uri = p.text_document.uri;
        if let Some(doc) = self.docs.get(&uri) {
            let (src, prog, _) = doc.value();
            if let Some(prog) = prog {
                let tokens = semantic::get_semantic_tokens(prog, src);
                return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: tokens,
                })));
            }
        }
        Ok(None)
    }
}
