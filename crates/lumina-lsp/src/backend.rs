use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use lumina_parser::parse;
use lumina_analyzer::analyze;
use crate::{diag::to_lsp_diags, hover::hover_at};

pub struct LuminaBackend {
    client: Client,
    docs: DashMap<Url, (String, Option<lumina_parser::ast::Program>)>,
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
        self.docs.insert(uri.clone(), (src, prog));
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
            let (src, prog) = e.value();
            prog.as_ref().and_then(|p| hover_at(p, src, pos))
        }))
    }
}
