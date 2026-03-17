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
                match analyze(p.clone(), &src, uri.path(), true) {
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
