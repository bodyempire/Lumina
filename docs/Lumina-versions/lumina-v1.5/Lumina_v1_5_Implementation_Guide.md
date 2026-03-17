**LUMINA**

**v1.5 Implementation Guide**

_For Antigravity AI -- exact file paths, complete Rust code, build order,_

_anti-hallucination guards, and verification steps for all 8 v1.5 features._

_"Describe what is true. Lumina figures out what to do."_

Chapters 27-34 | 8 Features | Rust + TypeScript | March 2026

_Designed and authored by Isaac Ishimwe_

**Preface**

**How To Use This Document**

_Rules for Antigravity AI -- read before writing any code_

**CRITICAL Core Philosophy Guard -- Non-Negotiable**

Lumina is a declarative reactive language for IoT, monitoring, and automation.

Every feature must help engineers describe what is TRUE about their system.

Do NOT add general computation features not listed in this document.

Do NOT add sqrt(), log(), random(), string manipulation, or data processing.

If a feature looks like something Python already does well -- it does not belong.

When in doubt: "Does this describe truth about a reactive system?" If not -- stop.

**CRITICAL Implementation Rules -- Non-Negotiable**

RULE 1: v1.4 complete and all 40+ tests passing before starting v1.5.

RULE 2: Every file path in this document is exact. Never invent paths.

RULE 3: cargo test --workspace must pass at the end of every phase.

RULE 4: lumina-lsp is a NEW binary crate. Do NOT add LSP to lumina-cli.

RULE 5: LuminaAdapter trait lives in lumina-runtime. Not a new crate.

RULE 6: prev(), when any/all, aggregate, cooldown are runtime+analyzer changes only.

RULE 7: alert is a new Action variant. Do NOT remove or replace show.

RULE 8: Playground v2 is pure TypeScript. No new Rust crates.

**NOTE v1.4 Workspace Entering v1.5**

crates/lumina-lexer/ -- all v1.4 tokens present

crates/lumina-parser/ -- FnDecl, ImportDecl, RuleParam, InterpolatedString in AST

crates/lumina-analyzer/ -- Vec&lt;Diagnostic&gt;, L001-L023 active

crates/lumina-diagnostics/ -- DiagnosticRenderer, SourceLocation

crates/lumina-runtime/ -- Evaluator, Value::List, built-in list fns, adapter stub

crates/lumina-ffi/ -- C API, Python bindings, Go wrapper

crates/lumina-wasm/ -- WASM target, playground v1

crates/lumina-cli/ -- run/check/repl, ModuleLoader, ReplSession

extensions/lumina-vscode/ -- grammar + snippets (no LSP client yet)

40+ tests passing. cargo test --workspace must stay green throughout all 8 phases.

**Chapter 27**

**Language Server Protocol**

_Implementation -- lumina-lsp binary crate + VS Code TypeScript client_

The language server is a new binary crate: crates/lumina-lsp/. It uses tower-lsp for JSON-RPC. The VS Code extension gains a TypeScript client that launches lumina-lsp as a subprocess.

# **27.1 New Crate Cargo.toml**

| **crates/lumina-lsp/Cargo.toml**                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[package\]<br><br>name = "lumina-lsp"<br><br>version = "0.1.0"<br><br>edition = "2021"<br><br>\[\[bin\]\]<br><br>name = "lumina-lsp"<br><br>\[dependencies\]<br><br>lumina-parser = { path = "../lumina-parser" }<br><br>lumina-analyzer = { path = "../lumina-analyzer" }<br><br>lumina-diagnostics = { path = "../lumina-diagnostics" }<br><br>tower-lsp = "0.20"<br><br>tokio = { version = "1", features = \["full"\] }<br><br>serde_json = "1"<br><br>dashmap = "5" |

| **Add to workspace Cargo.toml**                                                |
| ------------------------------------------------------------------------------ |
| // In members = \[...\] after "crates/lumina-cli":<br><br>"crates/lumina-lsp", |

# **27.2 src/main.rs**

| **crates/lumina-lsp/src/main.rs**                                                                                                                                                                                                                                                                                                                                   |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| mod backend; mod diag; mod hover;<br><br>use backend::LuminaBackend;<br><br>use tower_lsp::{LspService, Server};<br><br>#\[tokio::main\]<br><br>async fn main() {<br><br>let (service, socket) = LspService::new(\|client\| LuminaBackend::new(client));<br><br>Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)<br><br>.serve(service).await;<br><br>} |

# **27.3 src/backend.rs**

| **crates/lumina-lsp/src/backend.rs -- Part 1: struct**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use dashmap::DashMap;<br><br>use tower_lsp::jsonrpc::Result;<br><br>use tower_lsp::lsp_types::\*;<br><br>use tower_lsp::{Client, LanguageServer};<br><br>use lumina_parser::parse;<br><br>use lumina_analyzer::analyze;<br><br>use crate::{diag::to_lsp_diags, hover::hover_at};<br><br>pub struct LuminaBackend {<br><br>client: Client,<br><br>docs: DashMap&lt;Url, (String, Option<lumina_parser::Program&gt;)>,<br><br>}<br><br>impl LuminaBackend {<br><br>pub fn new(client: Client) -> Self {<br><br>Self { client, docs: DashMap::new() }<br><br>}<br><br>async fn refresh(&self, uri: Url, src: String) {<br><br>let prog = parse(&src).ok();<br><br>let diags = prog.as_ref()<br><br>.map(\|p\| analyze(p, &src, uri.path(), true))<br><br>.unwrap_or_default();<br><br>self.docs.insert(uri.clone(), (src, prog));<br><br>self.client.publish_diagnostics(uri, to_lsp_diags(&diags), None).await;<br><br>}<br><br>} |

| **src/backend.rs -- Part 2: LanguageServer trait**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| #\[tower_lsp::async_trait\]<br><br>impl LanguageServer for LuminaBackend {<br><br>async fn initialize(&self, \_: InitializeParams) -> Result&lt;InitializeResult&gt; {<br><br>Ok(InitializeResult { capabilities: ServerCapabilities {<br><br>text_document_sync: Some(TextDocumentSyncCapability::Kind(<br><br>TextDocumentSyncKind::FULL)),<br><br>hover_provider: Some(HoverProviderCapability::Simple(true)),<br><br>definition_provider: Some(OneOf::Left(true)),<br><br>document_symbol_provider: Some(OneOf::Left(true)),<br><br>completion_provider: Some(CompletionOptions {<br><br>trigger_characters: Some(vec!\[".".into(), " ".into()\]),<br><br>..Default::default() }),<br><br>..Default::default() }, ..Default::default() })<br><br>}<br><br>async fn initialized(&self, \_: InitializedParams) {}<br><br>async fn shutdown(&self) -> Result&lt;()&gt; { Ok(()) }<br><br>async fn did_open(&self, p: DidOpenTextDocumentParams) {<br><br>self.refresh(p.text_document.uri, p.text_document.text).await;<br><br>}<br><br>async fn did_change(&self, p: DidChangeTextDocumentParams) {<br><br>let src = p.content_changes.into_iter()<br><br>.last().map(\|c\| c.text).unwrap_or_default();<br><br>self.refresh(p.text_document.uri, src).await;<br><br>}<br><br>async fn hover(&self, p: HoverParams) -> Result&lt;Option<Hover&gt;> {<br><br>let uri = p.text_document_position_params.text_document.uri;<br><br>let pos = p.text_document_position_params.position;<br><br>Ok(self.docs.get(&uri).and_then(\|e\| {<br><br>let (src, prog) = e.value();<br><br>prog.as_ref().and_then(\|p\| hover_at(p, src, pos))<br><br>}))<br><br>}<br><br>} |

# **27.4 src/diag.rs and src/hover.rs**

| **src/diag.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use tower_lsp::lsp_types::\*;<br><br>use lumina_diagnostics::Diagnostic;<br><br>pub fn to_lsp_diags(ds: &\[Diagnostic\]) -> Vec&lt;tower_lsp::lsp_types::Diagnostic&gt; {<br><br>ds.iter().map(\|d\| {<br><br>let l = d.location.line.saturating_sub(1);<br><br>let c = d.location.col.saturating_sub(1);<br><br>tower_lsp::lsp_types::Diagnostic {<br><br>range: Range {<br><br>start: Position { line: l, character: c },<br><br>end: Position { line: l, character: c + d.location.len.max(1) },<br><br>},<br><br>severity: Some(DiagnosticSeverity::ERROR),<br><br>code: Some(NumberOrString::String(d.code.clone())),<br><br>source: Some("lumina".into()),<br><br>message: d.message.clone(),<br><br>related_information: None, tags: None,<br><br>code_description: None, data: None,<br><br>}<br><br>}).collect()<br><br>} |

| **src/hover.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use tower_lsp::lsp_types::\*;<br><br>use lumina_parser::{Program, Statement};<br><br>pub fn hover_at(prog: &Program, \_src: &str, pos: Position) -> Option&lt;Hover&gt; {<br><br>for stmt in &prog.statements {<br><br>if let Statement::Entity(e) = stmt {<br><br>for f in &e.fields {<br><br>let l = f.span.line.saturating_sub(1);<br><br>let c = f.span.col.saturating_sub(1);<br><br>if pos.line == l && pos.character >= c && pos.character <= c + f.span.len {<br><br>let mut lines = vec!\[format!("\*\*{}\*\*: {}", f.name, f.type_display())\];<br><br>if let Some(d) = &f.doc { lines.push(d.clone()); }<br><br>if let Some((lo,hi)) = f.range { lines.push(format!("Range: {} to {}",lo,hi)); }<br><br>return Some(Hover {<br><br>contents: HoverContents::Markup(MarkupContent {<br><br>kind: MarkupKind::Markdown,<br><br>value: lines.join(" \\n"),<br><br>}), range: None,<br><br>});<br><br>}<br><br>}<br><br>}<br><br>}<br><br>None<br><br>} |

# **27.5 VS Code Extension Update**

| **extensions/lumina-vscode/src/extension.ts**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| import \* as path from "path";<br><br>import { workspace, ExtensionContext } from "vscode";<br><br>import {<br><br>LanguageClient, LanguageClientOptions,<br><br>ServerOptions, TransportKind<br><br>} from "vscode-languageclient/node";<br><br>let client: LanguageClient;<br><br>export function activate(context: ExtensionContext) {<br><br>const serverOptions: ServerOptions = {<br><br>command: "lumina-lsp", args: \[\], transport: TransportKind.stdio<br><br>};<br><br>const clientOptions: LanguageClientOptions = {<br><br>documentSelector: \[{ scheme: "file", language: "lumina" }\],<br><br>synchronize: { fileEvents: workspace.createFileSystemWatcher("\*\*/\*.lum") }<br><br>};<br><br>client = new LanguageClient("lumina","Lumina",serverOptions,clientOptions);<br><br>client.start();<br><br>}<br><br>export function deactivate(): Thenable&lt;void&gt; \| undefined {<br><br>return client ? client.stop() : undefined;<br><br>} |

| **package.json additions**                                                                                                                                                                   |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "main": "./out/extension.js",<br><br>"activationEvents": \["onLanguage:lumina"\],<br><br>"dependencies": { "vscode-languageclient": "^9.0.0" },<br><br>"scripts": { "compile": "tsc -p ./" } |

# **27.6 Build Order**

**BUILD Chapter 27 -- exact sequence**

Step 1: Create crates/lumina-lsp/ with Cargo.toml, src/main.rs, src/backend.rs, src/diag.rs, src/hover.rs.

Step 2: Add "crates/lumina-lsp" to workspace Cargo.toml members.

Step 3: cargo build -p lumina-lsp (downloads tower-lsp on first build).

Step 4: cargo test --workspace (all v1.4 tests must still pass).

Step 5: cargo install --path crates/lumina-lsp (puts lumina-lsp on PATH).

Step 6: Add src/extension.ts + update package.json in extensions/lumina-vscode/.

Step 7: cd extensions/lumina-vscode && npm install && npm run compile.

Step 8: vsce package && install .vsix. Open .lum file -- verify squiggles on L001.

**Chapter 28**

**External Entities**

_Implementation -- LuminaAdapter trait + built-in adapters + tick() wiring_

External entities parse and analyze correctly since v1.3. The runtime silently ignores them. This chapter adds LuminaAdapter to lumina-runtime and wires adapter polling into tick(). No parser or analyzer changes needed.

# **28.1 Files To Create**

| **File**                                             | **Purpose**                    |
| ---------------------------------------------------- | ------------------------------ |
| crates/lumina-runtime/src/adapter.rs                 | LuminaAdapter trait definition |
| crates/lumina-runtime/src/adapters/mod.rs            | Adapter module root            |
| crates/lumina-runtime/src/adapters/static_adapter.rs | StaticAdapter -- for testing   |
| crates/lumina-runtime/src/adapters/channel.rs        | ChannelAdapter -- Rust mpsc    |

# **28.2 adapter.rs**

| **crates/lumina-runtime/src/adapter.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use crate::value::Value;<br><br>pub trait LuminaAdapter: Send + Sync {<br><br>/// The external entity name this adapter serves.<br><br>/// Must match: external entity &lt;Name&gt; { ... }<br><br>fn entity_name(&self) -> &str;<br><br>/// Called on every tick(). Return Some((field, value)) if a new value is ready.<br><br>fn poll(&mut self) -> Option&lt;(String, Value)&gt;;<br><br>/// Called when a rule action writes to an external entity field.<br><br>fn on_write(&mut self, field: &str, value: &Value);<br><br>} |

# **28.3 StaticAdapter and ChannelAdapter**

| **adapters/static_adapter.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| use crate::{LuminaAdapter, Value};<br><br>use std::collections::VecDeque;<br><br>pub struct StaticAdapter {<br><br>entity: String,<br><br>queue: VecDeque&lt;(String, Value)&gt;,<br><br>}<br><br>impl StaticAdapter {<br><br>pub fn new(entity: impl Into&lt;String&gt;) -> Self {<br><br>Self { entity: entity.into(), queue: VecDeque::new() }<br><br>}<br><br>pub fn push(&mut self, field: impl Into&lt;String&gt;, value: Value) {<br><br>self.queue.push_back((field.into(), value));<br><br>}<br><br>}<br><br>impl LuminaAdapter for StaticAdapter {<br><br>fn entity_name(&self) -> &str { &self.entity }<br><br>fn poll(&mut self) -> Option&lt;(String, Value)&gt; { self.queue.pop_front() }<br><br>fn on_write(&mut self, \_: &str, \_: &Value) {}<br><br>} |

| **adapters/channel.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use crate::{LuminaAdapter, Value};<br><br>use std::sync::mpsc::{Receiver, Sender};<br><br>pub struct ChannelAdapter {<br><br>entity: String,<br><br>rx: Receiver&lt;(String, Value)&gt;,<br><br>tx: Option&lt;Sender<(String, Value)&gt;>,<br><br>}<br><br>impl ChannelAdapter {<br><br>pub fn new(entity: impl Into&lt;String&gt;, rx: Receiver&lt;(String,Value)&gt;, tx: Option&lt;Sender<(String,Value)&gt;>) -> Self {<br><br>Self { entity: entity.into(), rx, tx }<br><br>}<br><br>}<br><br>impl LuminaAdapter for ChannelAdapter {<br><br>fn entity*name(&self) -> &str { &self.entity }<br><br>fn poll(&mut self) -> Option&lt;(String, Value)&gt; { self.rx.try_recv().ok() }<br><br>fn on_write(&mut self, f: &str, v: &Value) {<br><br>if let Some(tx) = &self.tx { let* = tx.send((f.to_string(), v.clone())); }<br><br>}<br><br>} |

# **28.4 Evaluator Changes**

| **crates/lumina-runtime/src/engine.rs -- add adapter support**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // 1. Add to Evaluator struct:<br><br>adapters: Vec&lt;Box<dyn LuminaAdapter&gt;>,<br><br>// 2. Add to new*empty() and from_source():<br><br>adapters: Vec::new(),<br><br>// 3. New public method:<br><br>pub fn register_adapter(&mut self, a: Box&lt;dyn LuminaAdapter&gt;) {<br><br>self.adapters.push(a);<br><br>}<br><br>// 4. Update tick() -- poll adapters FIRST, before timer processing:<br><br>pub fn tick(&mut self) -> Vec&lt;Event&gt; {<br><br>let updates: Vec&lt;(String, String, Value)&gt; = self.adapters<br><br>.iter_mut()<br><br>.flat_map(\|a\| {<br><br>let name = a.entity_name().to_string();<br><br>std::iter::from_fn(move \| {<br><br>a.poll().map(\|(f, v)\| (name.clone(), f, v))<br><br>}).collect::&lt;Vec<\_&gt;>()<br><br>}).collect();<br><br>for (entity, field, value) in updates {<br><br>if let Some(inst) = self.store.find_instance_of(&entity) {<br><br>let *= self.apply_update(&inst, &field, value);<br><br>}<br><br>}<br><br>self.process_timers() // existing v1.4 timer logic unchanged<br><br>}<br><br>// 5. In apply_update() -- call on_write for external entity fields:<br><br>for a in &mut self.adapters {<br><br>if a.entity_name() == entity_name {<br><br>a.on_write(&field, &value);<br><br>}<br><br>} |

# **28.5 Build Order**

**BUILD Chapter 28 -- exact sequence**

Step 1: Create adapter.rs with LuminaAdapter trait.

Step 2: Create adapters/mod.rs, adapters/static_adapter.rs, adapters/channel.rs.

Step 3: Add "pub mod adapter; pub mod adapters;" to lumina-runtime/src/lib.rs.

Step 4: Add adapters: Vec field + register_adapter() to Evaluator.

Step 5: Update tick() to poll adapters before timer processing.

Step 6: Add on_write() call in apply_update() for external entity fields.

Step 7: cargo test --workspace.

Step 8: Test: external entity + StaticAdapter, push value, verify rule fires.

Step 9: Verify entities without a registered adapter are silently ignored.

**Chapter 29**

**prev()**

_Implementation -- snapshot-based previous field value access_

prev() reuses the existing pre-update snapshot that the runtime already takes for rollback. A new prev_snapshot field is added to EvalContext so derived field evaluation can read the pre-update values during recomputation.

# **29.1 Lexer**

| **crates/lumina-lexer/src/token.rs**                                                                                                            |
| ----------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add:<br><br>// #\[token("prev")\]<br><br>// Prev,<br><br>pub enum Token {<br><br>// ... existing ...<br><br>Prev, // "prev" keyword<br><br>} |

# **29.2 Parser -- Expr::Prev**

| **crates/lumina-parser/src/ast.rs**                                                                   |
| ----------------------------------------------------------------------------------------------------- |
| pub enum Expr {<br><br>// ... existing ...<br><br>Prev { field: String, span: Span }, // NEW<br><br>} |

| **parse_primary() -- add prev() case**                                                                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Token::Prev => {<br><br>let span = self.current_span();<br><br>self.advance(); // consume "prev"<br><br>self.expect(Token::LParen)?;<br><br>let field = self.expect_ident("field name")?;<br><br>self.expect(Token::RParen)?;<br><br>Ok(Expr::Prev { field, span })<br><br>} |

# **29.3 Analyzer -- L024 and L025**

| **crates/lumina-analyzer/src/analyzer.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Expr::Prev { field, span } => {<br><br>// L025: prev() cannot be nested<br><br>if self.in_prev_context {<br><br>self.error("L025", "prev() cannot be nested", \*span);<br><br>return;<br><br>}<br><br>// L024: only stored fields have previous values<br><br>if let Some(field_decl) = self.current_entity_fields.get(field.as_str()) {<br><br>if field_decl.is_derived {<br><br>self.error("L024",<br><br>&format!("prev() cannot be applied to derived field {}", field),<br><br>\*span);<br><br>}<br><br>} else {<br><br>self.error("L005", &format!("unknown field: {}", field), \*span);<br><br>}<br><br>} |

# **29.4 Runtime -- Reading from Snapshot**

| **crates/lumina-runtime/src/engine.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // EvalContext already exists -- add prev_snapshot field:<br><br>pub struct EvalContext {<br><br>pub instance: String,<br><br>pub entity: String,<br><br>pub prev_snapshot: Option&lt;Arc<EntityStore&gt;>, // ADD<br><br>}<br><br>// In apply_update(), create snapshot BEFORE the write:<br><br>let snap = Arc::new(self.store.clone());<br><br>let ctx = EvalContext {<br><br>instance: instance.clone(),<br><br>entity: entity_name.clone(),<br><br>prev_snapshot: Some(snap),<br><br>};<br><br>// Pass ctx with prev_snapshot into recompute_derived()<br><br>// In eval_expr() -- add Expr::Prev case:<br><br>Expr::Prev { field, .. } => {<br><br>match &ctx.prev_snapshot {<br><br>Some(snap) => snap.get_field(&ctx.instance, field)<br><br>.ok_or(RuntimeError::R005 {<br><br>instance: ctx.instance.clone(), field: field.clone() }),<br><br>None => self.store.get_field(&ctx.instance, field)<br><br>.ok_or(RuntimeError::R005 {<br><br>instance: ctx.instance.clone(), field: field.clone() }),<br><br>}<br><br>} |

# **29.5 Build Order**

**BUILD Chapter 29 -- exact sequence**

Step 1: Add Token::Prev to lexer. cargo build -p lumina-lexer.

Step 2: Add Expr::Prev to AST. Add parse case in parse_primary().

Step 3: cargo build -p lumina-parser.

Step 4: Add in_prev_context flag + L024/L025 checks to analyzer.

Step 5: cargo build -p lumina-analyzer.

Step 6: Add prev_snapshot to EvalContext. Pass snapshot from apply_update().

Step 7: Add Expr::Prev case to eval_expr().

Step 8: cargo test --workspace.

Step 9: Test: batteryDrop := prev(battery) - battery. Update battery, verify drop correct.

Step 10: Test L024: prev(derivedField) must report error.

**Chapter 30**

**when any / when all**

_Implementation -- fleet-level trigger evaluation with FleetState counter map_

when any and when all are new Trigger variants. The runtime maintains a FleetState map tracking true/false counts per entity per field, enabling O(1) any/all evaluation on every update without scanning all instances.

# **30.1 Lexer -- Two New Tokens**

| **crates/lumina-lexer/src/token.rs**                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add:<br><br>// #\[token("any")\] Any,<br><br>// #\[token("all")\] All,<br><br>pub enum Token {<br><br>// ... existing ...<br><br>Any,<br><br>All,<br><br>} |

# **30.2 Parser -- New Trigger Variants**

| **crates/lumina-parser/src/ast.rs -- extend Trigger**                                                                                                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pub enum Trigger {<br><br>WhenBecomes { entity: String, field: String, value: Expr, duration: Option&lt;Duration&gt; },<br><br>AnyBecomes { entity: String, field: String, value: Expr, duration: Option&lt;Duration&gt; }, // NEW<br><br>AllBecomes { entity: String, field: String, value: Expr, duration: Option&lt;Duration&gt; }, // NEW<br><br>Every { interval: Duration },<br><br>} |

| **parse_trigger() -- add any/all cases**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // After consuming Token::When:<br><br>Token::Any => {<br><br>self.advance();<br><br>let entity = self.expect_ident("entity name")?;<br><br>self.expect(Token::Dot)?;<br><br>let field = self.expect_ident("field name")?;<br><br>self.expect(Token::Becomes)?;<br><br>let value = self.parse_expr()?;<br><br>let duration = self.parse_optional_for_duration()?;<br><br>Ok(Trigger::AnyBecomes { entity, field, value, duration })<br><br>}<br><br>Token::All => {<br><br>self.advance();<br><br>// same pattern as Any<br><br>Ok(Trigger::AllBecomes { entity, field, value, duration })<br><br>} |

# **30.3 Runtime -- FleetState**

| **crates/lumina-runtime/src/fleet.rs -- new file**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use std::collections::HashMap;<br><br>// Tracks (entity, field) -> (true_count, total_count)<br><br>pub struct FleetState {<br><br>counts: HashMap&lt;(String, String), (usize, usize)&gt;,<br><br>}<br><br>impl FleetState {<br><br>pub fn new() -> Self { Self { counts: Default::default() } }<br><br>pub fn update(&mut self, entity: &str, field: &str, new_val: bool, total: usize) {<br><br>let e = self.counts<br><br>.entry((entity.to_string(), field.to_string()))<br><br>.or_insert((0, total));<br><br>e.1 = total;<br><br>if new_val { e.0 = e.0.saturating_add(1); }<br><br>else { e.0 = e.0.saturating_sub(1); }<br><br>}<br><br>pub fn any_true(&self, entity: &str, field: &str) -> bool {<br><br>self.counts.get(&(entity.to_string(), field.to_string()))<br><br>.map(\|(t, \_)\| \*t > 0).unwrap_or(false)<br><br>}<br><br>pub fn all_true(&self, entity: &str, field: &str) -> bool {<br><br>self.counts.get(&(entity.to_string(), field.to_string()))<br><br>.map(\|(t, total)\| \*t == \*total && \*total > 0).unwrap_or(false)<br><br>}<br><br>} |

| **Evaluator -- wire FleetState into propagation**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Evaluator struct:<br><br>fleet_state: FleetState,<br><br>// After every Boolean field write in propagate():<br><br>if let Value::Boolean(b) = &new_value {<br><br>let total = self.store.count_instances_of(&entity_name);<br><br>self.fleet_state.update(&entity_name, &field_name, \*b, total);<br><br>}<br><br>// In check_rule_trigger() for AnyBecomes:<br><br>Trigger::AnyBecomes { entity, field, value, .. } => {<br><br>let target = eval_bool_literal(value); // true or false<br><br>let now = if target { self.fleet_state.any_true(entity, field) }<br><br>else { !self.fleet_state.all_true(entity, field) };<br><br>// Fire on rising edge: was false, now true<br><br>now && !self.prev_fleet_state.any_state(entity, field, target)<br><br>}<br><br>// AllBecomes: same pattern with fleet_state.all_true() |

# **30.4 Build Order**

**BUILD Chapter 30 -- exact sequence**

Step 1: Add Token::Any, Token::All to lexer. cargo build -p lumina-lexer.

Step 2: Add Trigger::AnyBecomes, Trigger::AllBecomes to AST.

Step 3: Add parse cases for any/all in parse_trigger().

Step 4: cargo build -p lumina-parser.

Step 5: Add L026/L027 checks to analyzer trigger validation.

Step 6: Create crates/lumina-runtime/src/fleet.rs with FleetState.

Step 7: Add fleet_state to Evaluator. Wire Boolean field writes into fleet_state.update().

Step 8: Implement AnyBecomes/AllBecomes in check_rule_trigger() with edge detection.

Step 9: cargo test --workspace.

Step 10: Test: 3 instances, set all isLowBattery=true one by one, verify "when all" fires only after last.

**Chapter 31**

**alert + on clear**

_Implementation -- AlertAction AST node, AlertEvent delivery, on_clear firing_

alert is a new Action variant producing a structured AlertEvent delivered via a registered callback. on clear is an optional block on RuleDecl firing when the trigger condition returns to its prior state. show is unchanged.

# **31.1 New Tokens**

| **crates/lumina-lexer/src/token.rs**                                                                                                                                                                                                                                                                                                                                                                                                                           |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add these tokens:<br><br>// #\[token("alert")\] Alert,<br><br>// #\[token("severity")\] Severity,<br><br>// #\[token("payload")\] Payload,<br><br>// #\[token("clear")\] Clear,<br><br>// NOTE: Check whether Token::On already exists (used in "sync on").<br><br>// If it does -- DO NOT add a duplicate On token.<br><br>pub enum Token {<br><br>Alert,<br><br>Severity,<br><br>Payload,<br><br>Clear,<br><br>// On -- already exists from v1.3<br><br>} |

# **31.2 AST -- AlertAction and on_clear**

| **crates/lumina-parser/src/ast.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Action enum:<br><br>pub enum Action {<br><br>Show(Expr),<br><br>Update { instance: String, field: String, value: Expr },<br><br>Create { entity: String, name: String, fields: Vec&lt;(String, Expr)&gt; },<br><br>Delete { instance: String },<br><br>Alert(AlertAction), // NEW<br><br>}<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct AlertAction {<br><br>pub severity: Expr,<br><br>pub message: Expr,<br><br>pub source: Option&lt;Expr&gt;,<br><br>pub code: Option&lt;Expr&gt;,<br><br>pub payload: Vec&lt;(String, Expr)&gt;,<br><br>pub span: Span,<br><br>}<br><br>// Add to RuleDecl:<br><br>pub struct RuleDecl {<br><br>pub name: String,<br><br>pub params: Vec&lt;RuleParam&gt;,<br><br>pub trigger: Trigger,<br><br>pub cooldown: Option&lt;Duration&gt;,<br><br>pub body: Vec&lt;Action&gt;,<br><br>pub on_clear: Option&lt;Vec<Action&gt;>, // NEW<br><br>pub span: Span,<br><br>} |

# **31.3 Runtime -- AlertEvent and Handler**

| **crates/lumina-runtime/src/alert.rs -- new file**                                                                                                                                                                                                                                                                                                                              |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use std::collections::HashMap;<br><br>use crate::Value;<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct AlertEvent {<br><br>pub severity: String,<br><br>pub message: String,<br><br>pub source: Option&lt;String&gt;,<br><br>pub code: Option&lt;String&gt;,<br><br>pub payload: HashMap&lt;String, Value&gt;,<br><br>pub timestamp: u64,<br><br>pub rule: String,<br><br>} |

| **Evaluator -- alert_handler + rule_active + on_clear**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Evaluator struct:<br><br>alert_handler: Option&lt;Box<dyn Fn(AlertEvent) + Send + Sync&gt;>,<br><br>rule_active: HashMap&lt;String, bool&gt;, // rule_name -> trigger currently active?<br><br>// New public method:<br><br>pub fn on_alert(&mut self, h: impl Fn(AlertEvent) + Send + Sync + 'static) {<br><br>self.alert_handler = Some(Box::new(h));<br><br>}<br><br>// In exec_action() for Action::Alert:<br><br>Action::Alert(a) => {<br><br>let severity = self.eval_expr(&a.severity, ctx)?.as_text()?;<br><br>let message = self.eval_expr(&a.message, ctx)?.as_text()?;<br><br>let source = a.source.as_ref()<br><br>.and_then(\|e\| self.eval_expr(e, ctx).ok())<br><br>.and_then(\|v\| v.into_text());<br><br>let event = AlertEvent {<br><br>severity, message, source, code: None,<br><br>payload: Default::default(),<br><br>timestamp: unix_now(),<br><br>rule: ctx.rule_name.clone(),<br><br>};<br><br>if let Some(h) = &self.alert_handler { h(event); }<br><br>Ok(())<br><br>}<br><br>// After rule fires: mark as active<br><br>// After propagation: check if active rules condition is now false -> fire on_clear |

# **31.4 Build Order**

**BUILD Chapter 31 -- exact sequence**

Step 1: Add Token::Alert, Severity, Payload, Clear (check On already exists).

Step 2: Add AlertAction to AST. Add on_clear field to RuleDecl.

Step 3: Add parse_alert_action(). Update parse_rule() to parse optional "on clear { ... }".

Step 4: cargo build -p lumina-parser.

Step 5: Add L028/L029/L030 to analyzer.

Step 6: Create alert.rs with AlertEvent.

Step 7: Add alert_handler + rule_active to Evaluator. Implement Action::Alert.

Step 8: Implement on_clear firing after propagation loop.

Step 9: cargo test --workspace.

Step 10: Register on_alert handler, fire a rule, verify structured AlertEvent received.

Step 11: Test on clear: raise condition, then recover -- verify resolved event fires.

**Chapter 32**

**aggregate**

_Implementation -- AggregateDecl top-level statement + AggregateStore_

aggregate is a new top-level statement. The parser produces an AggregateDecl. The runtime maintains an AggregateStore that recomputes all aggregate values after every apply_update(). Rules can trigger on aggregate field transitions exactly like entity fields.

# **32.1 New Tokens**

| **crates/lumina-lexer/src/token.rs**                                                                                                                    |
| ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add:<br><br>// #\[token("aggregate")\] Aggregate,<br><br>// #\[token("over")\] Over,<br><br>pub enum Token {<br><br>Aggregate,<br><br>Over,<br><br>} |

# **32.2 Parser -- AggregateDecl**

| **crates/lumina-parser/src/ast.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Statement:<br><br>pub enum Statement {<br><br>// ... existing ...<br><br>Aggregate(AggregateDecl),<br><br>}<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct AggregateDecl {<br><br>pub name: String,<br><br>pub over: String,<br><br>pub fields: Vec&lt;AggregateField&gt;,<br><br>pub span: Span,<br><br>}<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct AggregateField { pub name: String, pub expr: AggregateExpr, pub span: Span }<br><br>#\[derive(Debug, Clone)\]<br><br>pub enum AggregateExpr {<br><br>Avg(String), Min(String), Max(String), Sum(String),<br><br>Count(Option&lt;String&gt;), Any(String), All(String),<br><br>} |

| **parse_aggregate() -- complete**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| fn parse*aggregate(&mut self) -> Result&lt;Statement, ParseError&gt; {<br><br>let span = self.current_span();<br><br>self.expect(Token::Aggregate)?;<br><br>let name = self.expect_ident("aggregate name")?;<br><br>self.expect(Token::Over)?;<br><br>let over = self.expect_ident("entity name")?;<br><br>self.expect(Token::LBrace)?;<br><br>let mut fields = Vec::new();<br><br>while !self.check(Token::RBrace) {<br><br>let fs = self.current_span();<br><br>let fn* = self.expect*ident("field name")?;<br><br>self.expect(Token::DeriveAssign)?;<br><br>let agg_fn = self.expect_ident("aggregate function")?;<br><br>self.expect(Token::LParen)?;<br><br>let arg = if self.check(Token::RParen) { None }<br><br>else { Some(self.expect_ident("field name")?) };<br><br>self.expect(Token::RParen)?;<br><br>let expr = match agg_fn.as_str() {<br><br>"avg" => AggregateExpr::Avg(arg.unwrap_or_default()),<br><br>"min" => AggregateExpr::Min(arg.unwrap_or_default()),<br><br>"max" => AggregateExpr::Max(arg.unwrap_or_default()),<br><br>"sum" => AggregateExpr::Sum(arg.unwrap_or_default()),<br><br>"count" => AggregateExpr::Count(arg),<br><br>"any" => AggregateExpr::Any(arg.unwrap_or_default()),<br><br>"all" => AggregateExpr::All(arg.unwrap_or_default()),<br><br>other => return Err(self.error(&format!("unknown aggregate fn: {}", other))),<br><br>};<br><br>fields.push(AggregateField { name: fn*, expr, span: fs });<br><br>}<br><br>self.expect(Token::RBrace)?;<br><br>Ok(Statement::Aggregate(AggregateDecl { name, over, fields, span }))<br><br>} |

# **32.3 Runtime -- AggregateStore**

| **crates/lumina-runtime/src/aggregate.rs -- new file**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use crate::{Value, EntityStore};<br><br>use lumina_parser::{AggregateDecl, AggregateExpr};<br><br>use std::collections::HashMap;<br><br>pub struct AggregateStore {<br><br>decls: Vec&lt;AggregateDecl&gt;,<br><br>values: HashMap&lt;String, HashMap<String, Value&gt;>,<br><br>}<br><br>impl AggregateStore {<br><br>pub fn new() -> Self { Self { decls: Vec::new(), values: HashMap::new() } }<br><br>pub fn register(&mut self, d: AggregateDecl) { self.decls.push(d); }<br><br>pub fn get(&self, agg: &str, field: &str) -> Option&lt;&Value&gt; {<br><br>self.values.get(agg)?.get(field)<br><br>}<br><br>/// Call after every apply_update().<br><br>pub fn recompute(&mut self, store: &EntityStore) {<br><br>for d in &self.decls {<br><br>let insts = store.instances_of(&d.over);<br><br>let mut vals = HashMap::new();<br><br>for f in &d.fields {<br><br>vals.insert(f.name.clone(), compute(&f.expr, &insts, store));<br><br>}<br><br>self.values.insert(d.name.clone(), vals);<br><br>}<br><br>}<br><br>}<br><br>fn nums(insts: &\[String\], field: &str, store: &EntityStore) -> Vec&lt;f64&gt; {<br><br>insts.iter().filter_map(\|i\|<br><br>store.get_field(i, field).and_then(\|v\| v.as_number().ok())<br><br>).collect()<br><br>}<br><br>fn compute(expr: &AggregateExpr, insts: &\[String\], store: &EntityStore) -> Value {<br><br>match expr {<br><br>AggregateExpr::Avg(f) => {<br><br>let ns = nums(insts, f, store);<br><br>if ns.is_empty() { return Value::Number(0.0); }<br><br>Value::Number(ns.iter().sum::&lt;f64&gt;() / ns.len() as f64)<br><br>}<br><br>AggregateExpr::Min(f) => Value::Number(nums(insts,f,store).into_iter().fold(f64::INFINITY, f64::min)),<br><br>AggregateExpr::Max(f) => Value::Number(nums(insts,f,store).into_iter().fold(f64::NEG_INFINITY, f64::max)),<br><br>AggregateExpr::Sum(f) => Value::Number(nums(insts,f,store).iter().sum()),<br><br>AggregateExpr::Count(None) => Value::Number(insts.len() as f64),<br><br>AggregateExpr::Count(Some(f)) => Value::Number(insts.iter().filter(\|i\|<br><br>store.get_field(i,f).and_then(\|v\|v.as_bool().ok()).unwrap_or(false)<br><br>).count() as f64),<br><br>AggregateExpr::Any(f) => Value::Boolean(insts.iter().any(\|i\|<br><br>store.get_field(i,f).and_then(\|v\|v.as_bool().ok()).unwrap_or(false))),<br><br>AggregateExpr::All(f) => Value::Boolean(!insts.is_empty() && insts.iter().all(\|i\|<br><br>store.get_field(i,f).and_then(\|v\|v.as_bool().ok()).unwrap_or(false))),<br><br>}<br><br>} |

| **Evaluator -- register and recompute aggregates**                                                                                                                                                                                                                                                                                                                                                                                                              |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Evaluator struct:<br><br>agg_store: AggregateStore,<br><br>// In exec_statement() for Statement::Aggregate:<br><br>Statement::Aggregate(d) => { self.agg_store.register(d.clone()); }<br><br>// At the END of every apply_update():<br><br>self.agg_store.recompute(&self.store);<br><br>// In rule trigger evaluation -- when checking aggregate.field transitions:<br><br>// Look up aggregate values from self.agg_store.get(agg_name, field_name) |

# **32.4 Build Order**

**BUILD Chapter 32 -- exact sequence**

Step 1: Add Token::Aggregate, Token::Over to lexer. cargo build -p lumina-lexer.

Step 2: Add AggregateDecl, AggregateField, AggregateExpr to AST.

Step 3: Add parse_aggregate(). Wire into parse_statement().

Step 4: cargo build -p lumina-parser.

Step 5: Add L031/L032/L033 checks to analyzer.

Step 6: Create aggregate.rs with AggregateStore + compute().

Step 7: Add agg_store to Evaluator. Register decls in exec_statement().

Step 8: Call agg_store.recompute() at end of every apply_update().

Step 9: Wire aggregate field access into rule trigger evaluation.

Step 10: cargo test --workspace.

Step 11: Test: 3 moto instances, update batteries, verify avgBattery recomputes correctly.

**Chapter 33**

**Rule Cooldown**

_Implementation -- per-rule per-instance silence tracking_

cooldown is a new optional clause on RuleDecl. After firing, the rule records a timestamp. On each subsequent trigger evaluation, if elapsed time is less than the cooldown duration, the rule body is suppressed. on clear always fires regardless of cooldown.

# **33.1 Lexer**

| **crates/lumina-lexer/src/token.rs**                                                                                                                                                    |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add:<br><br>// #\[token("cooldown")\] Cooldown,<br><br>// Duration literals (30s, 5m, 1h) already parsed for "every"/"for" -- reuse.<br><br>pub enum Token { Cooldown, /\* ... \*/ } |

# **33.2 Parser**

| **RuleDecl -- add cooldown field**                                                                                                                                                                                                                                                           |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pub struct RuleDecl {<br><br>pub name: String,<br><br>pub params: Vec&lt;RuleParam&gt;,<br><br>pub trigger: Trigger,<br><br>pub cooldown: Option&lt;Duration&gt;, // NEW<br><br>pub body: Vec&lt;Action&gt;,<br><br>pub on_clear: Option&lt;Vec<Action&gt;>,<br><br>pub span: Span,<br><br>} |

| **parse_rule() -- parse optional cooldown clause**                                                                                                                                                                                                                  |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // After parsing the trigger, before the rule body:<br><br>let cooldown = if self.check(Token::Cooldown) {<br><br>self.advance(); // consume "cooldown"<br><br>Some(self.parse_duration()?) // reuse existing duration parser<br><br>} else {<br><br>None<br><br>}; |

# **33.3 Runtime -- Cooldown Tracking**

| **crates/lumina-runtime/src/engine.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Evaluator struct:<br><br>// (rule_name, instance_name) -> last fired Instant<br><br>cooldown_map: HashMap&lt;(String, String), std::time::Instant&gt;,<br><br>// Add two helper methods:<br><br>fn should_fire(&self, rule: &RuleDecl, instance: &str) -> bool {<br><br>let cd = match rule.cooldown { Some(d) => d, None => return true };<br><br>let key = (rule.name.clone(), instance.to_string());<br><br>match self.cooldown_map.get(&key) {<br><br>None => true,<br><br>Some(last) => last.elapsed() >= cd.to_std_duration(),<br><br>}<br><br>}<br><br>fn record_firing(&mut self, rule: &RuleDecl, instance: &str) {<br><br>if rule.cooldown.is_some() {<br><br>self.cooldown_map.insert(<br><br>(rule.name.clone(), instance.to_string()),<br><br>std::time::Instant::now(),<br><br>);<br><br>}<br><br>}<br><br>// In fire_rule():<br><br>fn fire_rule(&mut self, rule: &RuleDecl, instance: &str) -> Result&lt;(), RuntimeError&gt; {<br><br>if !self.should_fire(rule, instance) {<br><br>return Ok(()); // suppressed by cooldown<br><br>}<br><br>self.exec_rule_body(&rule.body, instance)?;<br><br>self.record_firing(rule, instance);<br><br>Ok(())<br><br>}<br><br>// on clear is called separately and bypasses should_fire() check.<br><br>// Never gate on_clear body with cooldown. |

# **33.4 Build Order**

**BUILD Chapter 33 -- exact sequence**

Step 1: Add Token::Cooldown to lexer. cargo build -p lumina-lexer.

Step 2: Add cooldown: Option&lt;Duration&gt; to RuleDecl.

Step 3: Add cooldown parsing in parse_rule(). cargo build -p lumina-parser.

Step 4: Add L034 check (duration <= 0) to analyzer. cargo build -p lumina-analyzer.

Step 5: Add cooldown_map to Evaluator.

Step 6: Add should_fire() and record_firing() methods.

Step 7: Gate fire_rule() main body with should_fire(). Never gate on_clear.

Step 8: cargo test --workspace.

Step 9: Test: rule with cooldown 5s. Fire trigger twice in 3s. Verify fires only once.

Step 10: Test: on clear fires during cooldown window.

**Chapter 34**

**Playground v2**

_Implementation -- TypeScript/React upgrade, no new Rust crates required_

Playground v2 is a pure frontend upgrade. The WASM binary from lumina-wasm is unchanged. All new features are TypeScript/React components in playground/src/.

# **34.1 New npm Dependency**

| **playground/package.json**                                                                                                                                |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "dependencies": {<br><br>"react": "^18.0.0",<br><br>"react-dom": "^18.0.0",<br><br>"@monaco-editor/react": "^4.0.0",<br><br>"lz-string": "^1.5.0"<br><br>} |

# **34.2 StatePanel.tsx**

| **playground/src/StatePanel.tsx**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| import React, { useEffect, useState } from "react";<br><br>import type { LuminaWasm } from "./wasm";<br><br>interface Field { name: string; value: unknown; isDerived: boolean; }<br><br>interface Card { name: string; fields: Field\[\]; hasAlert: boolean; }<br><br>export function StatePanel({ rt }: { rt: LuminaWasm \| null }) {<br><br>const \[cards, setCards\] = useState&lt;Card\[\]&gt;(\[\]);<br><br>useEffect(() => {<br><br>if (!rt) return;<br><br>const id = setInterval(() => {<br><br>const state = rt.exportState();<br><br>if (!state?.instances) return;<br><br>setCards(Object.entries(state.instances).map((\[name, inst\]: any) => ({<br><br>name,<br><br>hasAlert: inst.active_alert ?? false,<br><br>fields: Object.entries(inst.fields \| {}).map((\[f, v\]) => ({<br><br>name: f, value: v,<br><br>isDerived: inst.derived_fields?.includes(f) ?? false<br><br>}))<br><br>})));<br><br>}, 200);<br><br>return () => clearInterval(id);<br><br>}, \[rt\]);<br><br>return (<br><br>&lt;div className="state-panel"&gt;<br><br>{cards.map(c => (<br><br>&lt;div key={c.name} className={"card" + (c.hasAlert ? " alert" : "")}&gt;<br><br>&lt;h3&gt;{c.name}{c.hasAlert && &lt;span className="badge"&gt;ALERT&lt;/span&gt;}&lt;/h3&gt;<br><br>{c.fields.map(f => (<br><br>&lt;div key={f.name} className={f.isDerived ? "derived" : "stored"}&gt;<br><br>&lt;span&gt;{f.name}&lt;/span&gt;&lt;span&gt;{String(f.value)}&lt;/span&gt;<br><br>&lt;/div&gt;<br><br>))}<br><br>&lt;/div&gt;<br><br>))}<br><br>&lt;/div&gt;<br><br>);<br><br>} |

# **34.3 AlertTimeline.tsx**

| **playground/src/AlertTimeline.tsx**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| import React from "react";<br><br>interface Entry { severity: string; source?: string; message: string; rule: string; ts: number; }<br><br>const COLORS: Record&lt;string,string&gt; = {<br><br>critical:"#FEE2E2", warning:"#FEF9C3", info:"#DBEAFE", resolved:"#DCFCE7"<br><br>};<br><br>export function AlertTimeline({ events }: { events: Entry\[\] }) {<br><br>return (<br><br>&lt;div className="timeline"&gt;<br><br>&lt;h4&gt;Alert Timeline&lt;/h4&gt;<br><br>{events.length === 0 && &lt;p&gt;No alerts fired&lt;/p&gt;}<br><br>{\[...events\].reverse().map((e, i) => (<br><br>&lt;div key={i} style={{ background: COLORS\[e.severity\] \| "#F5F5F5" }}&gt;<br><br>&lt;span&gt;{e.severity.toUpperCase()}&lt;/span&gt;<br><br>&lt;span&gt;{e.source \| e.rule}&lt;/span&gt;<br><br>&lt;span&gt;{e.message}&lt;/span&gt;<br><br>&lt;/div&gt;<br><br>))}<br><br>&lt;/div&gt;<br><br>);<br><br>} |

# **34.4 VirtualClock.tsx**

| **playground/src/VirtualClock.tsx**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| import React, { useEffect, useRef, useState } from "react";<br><br>import type { LuminaWasm } from "./wasm";<br><br>export function VirtualClock({<br><br>rt, onAlerts<br><br>}: { rt: LuminaWasm \| null; onAlerts: (a: any\[\]) => void }) {<br><br>const \[speed, setSpeed\] = useState(1);<br><br>const \[running, setRunning\] = useState(false);<br><br>const ref = useRef&lt;number \| null&gt;(null);<br><br>useEffect(() => {<br><br>if (!running \| !rt) return;<br><br>ref.current = setInterval(() => {<br><br>const evts = rt.tick();<br><br>if (evts?.length) onAlerts(evts);<br><br>}, 100 / speed) as unknown as number;<br><br>return () => clearInterval(ref.current!);<br><br>}, \[running, speed, rt\]);<br><br>return (<br><br>&lt;div className="clock"&gt;<br><br>&lt;button onClick={() =&gt; setRunning(r => !r)}><br><br>{running ? "Pause" : "Run"}<br><br>&lt;/button&gt;<br><br>{\[1, 10, 100\].map(s => (<br><br><button key={s}<br><br>className={speed === s ? "active" : ""}<br><br>onClick={() => setSpeed(s)}>{s}x&lt;/button&gt;<br><br>))}<br><br>&lt;/div&gt;<br><br>);<br><br>} |

# **34.5 ShareButton.tsx**

| **playground/src/ShareButton.tsx**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| import React from "react";<br><br>import LZString from "lz-string";<br><br>export function ShareButton({ source }: { source: string }) {<br><br>const share = () => {<br><br>const enc = LZString.compressToEncodedURIComponent(source);<br><br>const url = \`\${location.origin}/play#v=2&src=\${enc}\`;<br><br>navigator.clipboard.writeText(url);<br><br>alert("Link copied!");<br><br>};<br><br>return &lt;button onClick={share}&gt;Share&lt;/button&gt;;<br><br>}<br><br>export function loadFromURL(): string \| null {<br><br>const m = location.hash.match(/\[#&\]src=(\[^&\]\*)/);<br><br>return m ? LZString.decompressFromEncodedURIComponent(m\[1\]) : null;<br><br>} |

# **34.6 Build Order**

**BUILD Chapter 34 -- exact sequence**

Step 1: wasm-pack build crates/lumina-wasm --target web (WASM is unchanged).

Step 2: cd playground && npm install (gets lz-string).

Step 3: Create StatePanel.tsx, AlertTimeline.tsx, VirtualClock.tsx, ShareButton.tsx.

Step 4: Update App.tsx to include all new components.

Step 5: npm run build -- must compile with 0 TypeScript errors.

Step 6: npm run dev. Load a fleet program.

Step 7: Drag battery slider down -- verify state panel and aggregates update live.

Step 8: Set clock to 100x -- verify cooldown and timer rules behave correctly.

Step 9: Click Share -- paste URL in new tab, verify same program loads.

**Appendix**

**Complete v1.5 Build Sequence**

_8 phases -- implement in this exact order_

**BUILD Phase 1 -- Chapter 27: Language Server (lumina-lsp crate)**

1\. Create crates/lumina-lsp/ with all 5 files.

2\. Add to workspace. cargo build -p lumina-lsp.

3\. Update VS Code extension. cargo install lumina-lsp.

4\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 2 -- Chapter 28: External Entities (adapter wiring)**

1\. Create adapter.rs and adapters/ directory.

2\. Add to Evaluator. Update tick() and apply_update().

3\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 3 -- Chapter 29: prev()**

1\. Token::Prev. Expr::Prev. Parse case.

2\. L024/L025 in analyzer. prev_snapshot in EvalContext.

3\. Expr::Prev eval from snapshot.

4\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 4 -- Chapter 30: when any / when all**

1\. Token::Any, Token::All. Trigger variants.

2\. Create fleet.rs. Wire Boolean writes into FleetState.

3\. Implement trigger evaluation with edge detection.

4\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 5 -- Chapter 31: alert + on clear**

1\. New tokens. AlertAction in AST. on_clear on RuleDecl.

2\. Create alert.rs. Add handler + active tracking to Evaluator.

3\. Implement Action::Alert and on_clear firing.

4\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 6 -- Chapter 32: aggregate**

1\. New tokens. AggregateDecl in AST.

2\. Create aggregate.rs. Wire recompute() after every apply_update().

3\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 7 -- Chapter 33: cooldown**

1\. Token::Cooldown. cooldown on RuleDecl.

2\. cooldown_map on Evaluator. should_fire() + record_firing().

3\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 8 -- Chapter 34: Playground v2 (frontend only)**

1\. wasm-pack build (no changes).

2\. npm install lz-string. Create 4 components. Update App.tsx.

3\. npm run build \[0 TypeScript errors\].

4\. cargo test --workspace \[STILL GREEN -- no Rust changes\].

**DONE v1.5 Definition of Done**

cargo test --workspace -- all tests pass, zero regressions.

lumina-lsp on PATH -- VS Code shows live squiggles on .lum files.

External entity + StaticAdapter -- push value, verify rule fires.

prev(battery) -- update battery, verify batteryDrop computes correctly.

when all Moto.isOnline becomes false -- fires only when LAST instance goes offline.

alert handler -- structured AlertEvent received with correct severity and message.

on clear -- recover condition, verify resolved event fires.

aggregate FleetStatus.avgBattery -- updates on every battery change.

cooldown 5s -- fires once, body suppressed on second trigger within 5 seconds.

Playground v2 -- live state, clock drives timers, share URL loads correctly.