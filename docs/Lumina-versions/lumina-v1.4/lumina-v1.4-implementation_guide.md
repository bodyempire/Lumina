**◈**

**LUMINA**

**v1.4 Implementation Guide**

_For Antigravity AI - Exact build instructions, file paths, function signatures,_

_anti-hallucination guards, and verification steps for every v1.4 feature._

_"Describe what is true. Lumina figures out what to do."_

Chapters 19-26 · 8 Features · Rust + Go + JSON · March 2026

_Designed and authored by Isaac Ishimwe_

**Preface**

**How To Use This Document**

_Instructions for Antigravity AI - read this page before building anything_

This document is a step-by-step implementation guide for Lumina v1.4. Each chapter covers one feature. Every chapter follows the same structure:

**▶ Chapter structure**

1\. WHAT EXISTS - exact state of the v1.3 codebase this feature builds on top of.

2\. FILES TO CREATE - every new file with its exact path from the workspace root.

3\. FILES TO MODIFY - every existing file that must be changed, with before/after.

4\. EXACT CODE - complete, copy-paste ready Rust/Go/JSON with no placeholders.

5\. BUILD ORDER - the precise sequence to implement changes to avoid compile errors.

6\. ANTI-HALLUCINATION RULES - what NOT to invent, assume, or add.

7\. VERIFICATION - exact cargo test commands and expected output to confirm success.

**🚫 CRITICAL: Rules for Antigravity**

RULE 1 - Never invent file paths. Every path in this document is exact. Do not add, remove, or rename files.

RULE 2 - Never add features not in this document. v1.4 adds exactly what is listed. Nothing else.

RULE 3 - Never remove existing tests. cargo test --workspace must pass at every step.

RULE 4 - Follow the BUILD ORDER section for each chapter. Compiling out of order will cause errors.

RULE 5 - v1.3 had NO fn, NO import, NO lists, NO string interpolation. Do NOT reference these in v1.3 code.

RULE 6 - Do NOT add async/await to lumina-runtime. The runtime is synchronous. Tokio is only for adapters.

RULE 7 - All code in this document compiles. If something looks wrong, re-read it. Do not "fix" it.

**📌 v1.3 Workspace - What Already Exists**

lumina/crates/lumina-lexer/ - tokenizer (logos 0.14)

lumina/crates/lumina-parser/ - AST + recursive descent + Pratt parser

lumina/crates/lumina-analyzer/ - type checker + dependency graph (Kahn's)

lumina/crates/lumina-runtime/ - evaluator, reactive engine, snapshot/rollback

lumina/crates/lumina-ffi/ - C API (.so/.dylib/.dll), lumina.h, lumina_py.py

lumina/crates/lumina-wasm/ - wasm-bindgen WebAssembly target

lumina/crates/lumina-cli/ - CLI binary (run/check/repl commands)

lumina/tests/ - spec/fleet.lum, spec/errors.lum, oracle/

40 tests passing across 6 crates. cargo test --workspace must stay green.

**Chapter 19**

**Enhanced Error Messages**

_Implementation guide - lumina-diagnostics crate from scratch_

v1.3 errors are raw strings with a code and a line number. v1.4 adds the lumina-diagnostics crate. Every error across the workspace gains a source location, a highlighted source line, a caret, and a help suggestion. This chapter gives the exact files, structs, and function bodies to build it.

# **19.1 What Exists in v1.3 (Do Not Remove)**

The current error types are defined in:

| **File** | **Type used for errors** |
| --- | --- |
| crates/lumina-analyzer/src/lib.rs | pub struct AnalyzerError { code: String, message: String, span: Span } |
| crates/lumina-runtime/src/lib.rs | pub enum RuntimeError { R001{..}, R002, R003{..}, ... } |
| crates/lumina-lexer/src/lib.rs | pub struct LexError { message: String, line: u32 } |
| crates/lumina-parser/src/lib.rs | pub struct ParseError { message: String, span: Span } |

# **19.2 Files To Create**

| **File path (from workspace root)** | **Purpose** |
| --- | --- |
| crates/lumina-diagnostics/Cargo.toml | Crate manifest - no Lumina deps |
| crates/lumina-diagnostics/src/lib.rs | SourceLocation, Diagnostic, DiagnosticRenderer |
| crates/lumina-diagnostics/src/location.rs | extract_line() and SourceLocation::from_span() |
| crates/lumina-diagnostics/src/render.rs | DiagnosticRenderer::render() and render_all() |

**⚠️ Do NOT create these files**

Do NOT create crates/lumina-diagnostics/src/main.rs - this is a library crate, not a binary.

Do NOT create separate error type files - all structs live in lib.rs.

Do NOT add lumina-parser or lumina-runtime as dependencies of lumina-diagnostics.

# **19.3 Cargo.toml for lumina-diagnostics**

| **crates/lumina-diagnostics/Cargo.toml - exact content** |
| --- |
| \[package\]<br><br>name = "lumina-diagnostics"<br><br>version = "0.1.0"<br><br>edition = "2021"<br><br>\[lib\]<br><br>name = "lumina_diagnostics"<br><br>\# NO dependencies on other lumina crates.<br><br>\# lumina-diagnostics is a leaf crate - everything depends on it.<br><br>\[dependencies\]<br><br>\# (empty - only std) |

Then add lumina-diagnostics to the workspace Cargo.toml members list:

| **lumina/Cargo.toml - add to members array** |
| --- |
| \[workspace\]<br><br>members = \[<br><br>"crates/lumina-lexer",<br><br>"crates/lumina-parser",<br><br>"crates/lumina-analyzer",<br><br>"crates/lumina-diagnostics", # ADD THIS LINE<br><br>"crates/lumina-runtime",<br><br>"crates/lumina-ffi",<br><br>"crates/lumina-wasm",<br><br>"crates/lumina-cli",<br><br>\] |

# **19.4 crates/lumina-diagnostics/src/lib.rs - Complete File**

| **lib.rs** |
| --- |
| pub mod location;<br><br>pub mod render;<br><br>pub use location::{SourceLocation, extract_line};<br><br>pub use render::DiagnosticRenderer;<br><br>/// A fully-resolved compiler or runtime diagnostic.<br><br>/// Every error in v1.4 produces one of these.<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct Diagnostic {<br><br>pub code: String, // "L003", "R006", etc.<br><br>pub message: String, // short human message<br><br>pub location: SourceLocation,<br><br>pub source_line: String, // raw text of the offending line<br><br>pub help: Option&lt;String&gt;, // optional "help: ..." suggestion<br><br>}<br><br>impl Diagnostic {<br><br>pub fn new(<br><br>code: impl Into&lt;String&gt;,<br><br>message: impl Into&lt;String&gt;,<br><br>location: SourceLocation,<br><br>source_line: impl Into&lt;String&gt;,<br><br>help: Option&lt;String&gt;,<br><br>) -> Self {<br><br>Self { code: code.into(), message: message.into(),<br><br>location, source_line: source_line.into(), help }<br><br>}<br><br>} |

# **19.5 crates/lumina-diagnostics/src/location.rs - Complete File**

| **location.rs** |
| --- |
| #\[derive(Debug, Clone)\]<br><br>pub struct SourceLocation {<br><br>pub file: String,<br><br>pub line: u32, // 1-indexed<br><br>pub col: u32, // 1-indexed<br><br>pub len: u32, // highlight width in chars (minimum 1)<br><br>}<br><br>impl SourceLocation {<br><br>pub fn new(file: impl Into&lt;String&gt;, line: u32, col: u32, len: u32) -> Self {<br><br>Self { file: file.into(), line, col, len: len.max(1) }<br><br>}<br><br>/// Build from a Span (which carries line + col from the lexer).<br><br>/// span.line and span.col are already 1-indexed in the v1.3 lexer.<br><br>pub fn from_span(line: u32, col: u32, len: u32, file: impl Into&lt;String&gt;) -> Self {<br><br>Self::new(file, line, col, len)<br><br>}<br><br>}<br><br>/// Extract the Nth line (1-indexed) from source text.<br><br>/// Returns empty string if line number is out of range.<br><br>pub fn extract_line(source: &str, line_num: u32) -> String {<br><br>source<br><br>.lines()<br><br>.nth((line_num.saturating_sub(1)) as usize)<br><br>.unwrap_or("")<br><br>.to_string()<br><br>} |

# **19.6 crates/lumina-diagnostics/src/render.rs - Complete File**

| **render.rs** |
| --- |
| use crate::Diagnostic;<br><br>pub struct DiagnosticRenderer;<br><br>impl DiagnosticRenderer {<br><br>/// Render one diagnostic to a multi-line string.<br><br>pub fn render(d: &Diagnostic) -> String {<br><br>let mut out = String::new();<br><br>// Header: error\[L003\]: message<br><br>out.push_str(&format!("error\[{}\]: {}\\n", d.code, d.message));<br><br>// Location: --> file.lum:4:3<br><br>out.push_str(&format!(" --> {}:{}:{}\\n",<br><br>d.location.file, d.location.line, d.location.col));<br><br>// Gutter: build padding to align line numbers<br><br>let gutter = d.location.line.to_string();<br><br>let pad = " ".repeat(gutter.len());<br><br>out.push_str(&format!("{} \|\\n", pad));<br><br>out.push_str(&format!("{} \| {}\\n", gutter, d.source_line));<br><br>// Caret: spaces + carets under the error token<br><br>let spaces = " ".repeat((d.location.col.saturating_sub(1)) as usize);<br><br>let carets = "^".repeat(d.location.len.max(1) as usize);<br><br>out.push_str(&format!("{} \| {}{}\\n", pad, spaces, carets));<br><br>out.push_str(&format!("{} \|\\n", pad));<br><br>// Optional help line<br><br>if let Some(help) = &d.help {<br><br>out.push_str(&format!(" = help: {}\\n", help));<br><br>}<br><br>out<br><br>}<br><br>/// Render multiple diagnostics, separated by blank lines.<br><br>pub fn render_all(diags: &\[Diagnostic\]) -> String {<br><br>diags.iter()<br><br>.map(Self::render)<br><br>.collect::&lt;Vec<\_&gt;>()<br><br>.join("\\n")<br><br>}<br><br>} |

# **19.7 Files To Modify - lumina-analyzer**

Add lumina-diagnostics as a dependency, then update analyze() to return Vec&lt;Diagnostic&gt; instead of Vec&lt;AnalyzerError&gt;. Keep AnalyzerError internally - just convert at the boundary.

| **crates/lumina-analyzer/Cargo.toml - add dependency** |
| --- |
| \[dependencies\]<br><br>lumina-parser = { path = "../lumina-parser" }<br><br>lumina-diagnostics = { path = "../lumina-diagnostics" } # ADD THIS |

| **crates/lumina-analyzer/src/lib.rs - updated analyze() signature** |
| --- |
| use lumina_diagnostics::{Diagnostic, DiagnosticRenderer, SourceLocation, extract_line};<br><br>// OLD signature (v1.3) - DO NOT DELETE, keep internally<br><br>// pub fn analyze(program: &Program) -> Vec&lt;AnalyzerError&gt;<br><br>// NEW public signature (v1.4)<br><br>pub fn analyze(program: &Program, source: &str, filename: &str) -> Vec&lt;Diagnostic&gt; {<br><br>let raw_errors = analyze_internal(program); // existing logic unchanged<br><br>raw_errors.into_iter().map(\|e\| {<br><br>Diagnostic::new(<br><br>&e.code,<br><br>&e.message,<br><br>SourceLocation::from_span(e.span.line, e.span.col,<br><br>e.span.end - e.span.start, filename),<br><br>extract_line(source, e.span.line),<br><br>help_for_code(&e.code),<br><br>)<br><br>}).collect()<br><br>}<br><br>// Help text lookup - one entry per L-code<br><br>fn help_for_code(code: &str) -> Option&lt;String&gt; {<br><br>match code {<br><br>"L001" => Some("rename one of the entity declarations".into()),<br><br>"L002" => Some("check spelling or add the entity declaration".into()),<br><br>"L003" => Some("break the cycle by making one field stored (field: Type)".into()),<br><br>"L004" => Some("verify the field type and the literal type match".into()),<br><br>"L005" => Some("check field spelling or add the field to the entity".into()),<br><br>"L006" => Some("@range only applies to Number fields; ensure min < max".into()),<br><br>"L007" => Some("check entity name in the when clause".into()),<br><br>"L008" => Some("add a let binding for the instance before using it".into()),<br><br>"L009" => Some("instance names must be globally unique".into()),<br><br>"L010" => Some("@affects only applies to stored fields".into()),<br><br>_ => None,<br><br>}<br><br>} |

# **19.8 Build Order**

**▶ Exact sequence - do not reorder**

Step 1: Create crates/lumina-diagnostics/ directory and its Cargo.toml.

Step 2: Create src/lib.rs, src/location.rs, src/render.rs with the code above.

Step 3: Add lumina-diagnostics to workspace Cargo.toml members.

Step 4: Run: cargo build -p lumina-diagnostics (must compile with 0 errors).

Step 5: Add lumina-diagnostics to lumina-analyzer/Cargo.toml dependencies.

Step 6: Update analyze() in lumina-analyzer/src/lib.rs as shown above.

Step 7: Update all callers of analyze() in lumina-cli/src/main.rs to pass source + filename.

Step 8: Run: cargo test --workspace (all 40 existing tests must still pass).

# **19.9 Verification**

| **Test commands and expected output** |
| --- |
| \# Must compile<br><br>cargo build -p lumina-diagnostics<br><br>\# Must still pass all 40 tests<br><br>cargo test --workspace<br><br>\# Manual test - run the errors spec file<br><br>cargo run --bin lumina -- check tests/spec/errors.lum<br><br>\# Expected output now includes source line + caret:<br><br>\# error\[L003\]: derived field cycle detected<br><br>\# --> tests/spec/errors.lum:4:3<br><br>\# 4 \| ...<br><br>\# = help: break the cycle by making one field stored |

**Chapter 20**

**REPL v2**

_Implementation guide - persistent state, multi-line detection, inspector commands_

The v1.3 REPL rebuilds the entire Evaluator from scratch on every line of input. v1.4 fixes this. The REPL maintains one ReplSession struct for the lifetime of the session, accumulates source by brace depth, and supports inspector commands (:state :schema :load :save :clear :help :quit).

# **20.1 What Exists in v1.3 (lumina-cli/src/repl.rs or main.rs)**

The v1.3 REPL is a simple loop that reads a line, prepends all previous lines, rebuilds the full Evaluator from scratch, and prints the result. It is either in lumina-cli/src/main.rs or a small repl.rs file. Find it and replace it entirely.

**⚠️ Do NOT keep the v1.3 REPL loop**

The v1.3 approach of rebuilding Evaluator::new() on every input is the bug being fixed.

Delete or completely replace the old loop - do not try to patch it.

The new ReplSession owns a single Evaluator that lives for the whole session.

# **20.2 Files To Create**

| **File path** | **Purpose** |
| --- | --- |
| crates/lumina-cli/src/repl.rs | ReplSession struct, feed(), run_command() |
| crates/lumina-cli/src/commands.rs | Inspector command handlers (:state :schema etc) |

# **20.3 crates/lumina-cli/src/repl.rs - Complete File**

| **repl.rs - Part 1: types** |
| --- |
| use lumina_parser::parse;<br><br>use lumina_analyzer::analyze;<br><br>use lumina_runtime::Evaluator;<br><br>use lumina_diagnostics::DiagnosticRenderer;<br><br>pub struct ReplSession {<br><br>pub evaluator: Evaluator,<br><br>source_accum: String,<br><br>pub brace_depth: i32,<br><br>history: Vec&lt;String&gt;,<br><br>/// Accumulated source across ALL inputs - used by :save<br><br>full_history: String,<br><br>}<br><br>pub enum ReplResult {<br><br>NeedMore, // multi-line construct - show "..." prompt<br><br>Ok(String), // success - optional output string to print<br><br>Error(String), // error - rendered diagnostic string<br><br>} |

| **repl.rs - Part 2: impl ReplSession** |
| --- |
| impl ReplSession {<br><br>pub fn new() -> Self {<br><br>Self {<br><br>evaluator: Evaluator::new_empty(), // see note below<br><br>source_accum: String::new(),<br><br>brace_depth: 0,<br><br>history: Vec::new(),<br><br>full_history: String::new(),<br><br>}<br><br>}<br><br>/// Feed one line of input. Returns what the REPL loop should do.<br><br>pub fn feed(&mut self, line: &str) -> ReplResult {<br><br>// Track brace depth for multi-line detection<br><br>for ch in line.chars() {<br><br>match ch {<br><br>'{' => self.brace_depth += 1,<br><br>'}' => self.brace_depth -= 1,<br><br>_=> {}<br><br>}<br><br>}<br><br>self.source_accum.push_str(line);<br><br>self.source_accum.push('\\n');<br><br>// Multi-line construct still open<br><br>if self.brace_depth > 0 { return ReplResult::NeedMore; }<br><br>// Complete construct - drain accumulator and execute<br><br>let source = std::mem::take(&mut self.source_accum);<br><br>self.history.push(source.clone());<br><br>self.full_history.push_str(&source);<br><br>self.exec_source(&source)<br><br>}<br><br>fn exec_source(&mut self, source: &str) -> ReplResult {<br><br>let program = match parse(source) {<br><br>Ok(p) => p,<br><br>Err(e) => return ReplResult::Error(format!("parse error: {}", e.message)),<br><br>};<br><br>let diags = analyze(&program, source, "&lt;repl&gt;");<br><br>if !diags.is_empty() {<br><br>return ReplResult::Error(DiagnosticRenderer::render_all(&diags));<br><br>}<br><br>let mut output = Vec::new();<br><br>for stmt in &program.statements {<br><br>match self.evaluator.exec_statement(stmt) {<br><br>Ok(()) => {}<br><br>Err(e) => return ReplResult::Error(format!("{:?}", e)),<br><br>}<br><br>}<br><br>// Collect any show output from WASM-style buffer (if enabled)<br><br>let captured = self.evaluator.drain_output();<br><br>output.extend(captured);<br><br>ReplResult::Ok(output.join("\\n"))<br><br>}<br><br>/// Reset to a fresh session.<br><br>pub fn clear(&mut self) {<br><br>\*self = Self::new();<br><br>}<br><br>} |

# **20.4 crates/lumina-cli/src/commands.rs - Inspector Commands**

| **commands.rs - complete file** |
| --- |
| use super::repl::ReplSession;<br><br>use std::fs;<br><br>pub fn run_command(session: &mut ReplSession, cmd: &str) -> String {<br><br>let parts: Vec&lt;&str&gt; = cmd.splitn(2, ' ').collect();<br><br>match parts\[0\] {<br><br>":state" => state_cmd(session),<br><br>":schema" => schema_cmd(session),<br><br>":clear" => clear_cmd(session),<br><br>":help" => help_cmd(),<br><br>":load" => {<br><br>let path = parts.get(1).unwrap_or(&"").trim();<br><br>load_cmd(session, path)<br><br>}<br><br>":save" => {<br><br>let path = parts.get(1).unwrap_or(&"").trim();<br><br>save_cmd(session, path)<br><br>}<br><br>":quit" \| ":q" => std::process::exit(0),<br><br>other => format!("Unknown command: {}. Type :help for commands.", other),<br><br>}<br><br>}<br><br>fn state_cmd(s: &mut ReplSession) -> String {<br><br>let state = s.evaluator.export_state();<br><br>serde_json::to_string_pretty(&state).unwrap_or_else(\|\_\| "{}".into())<br><br>}<br><br>fn schema_cmd(s: &mut ReplSession) -> String {<br><br>// Print entity names and field types from the evaluator's entity registry<br><br>s.evaluator.describe_schema()<br><br>}<br><br>fn clear_cmd(s: &mut ReplSession) -> String {<br><br>s.clear();<br><br>"Session cleared.".into()<br><br>}<br><br>fn load_cmd(s: &mut ReplSession, path: &str) -> String {<br><br>if path.is_empty() { return "Usage: :load &lt;file.lum&gt;".into(); }<br><br>match fs::read_to_string(path) {<br><br>Err(e) => format!("Cannot read {}: {}", path, e),<br><br>Ok(src) => match s.feed(&src) {<br><br>super::repl::ReplResult::Ok(\_) => format!("Loaded {}", path),<br><br>super::repl::ReplResult::Error(e) => e,<br><br>super::repl::ReplResult::NeedMore => "Incomplete construct in file.".into(),<br><br>}<br><br>}<br><br>}<br><br>fn save_cmd(s: &ReplSession, path: &str) -> String {<br><br>if path.is_empty() { return "Usage: :save &lt;file.lum&gt;".into(); }<br><br>match fs::write(path, &s.full_history) {<br><br>Ok(()) => format!("Saved session to {}", path),<br><br>Err(e) => format!("Cannot write {}: {}", path, e),<br><br>}<br><br>}<br><br>fn help_cmd() -> String {<br><br>":state - print current state as JSON\\n\\<br><br>:schema - list declared entities and fields\\n\\<br><br>:load &lt;file&gt; - execute a .lum file into this session\\n\\<br><br>:save &lt;file&gt; - save session source to file\\n\\<br><br>:clear - reset the session\\n\\<br><br>:help - show this list\\n\\<br><br>:quit - exit the REPL".into()<br><br>} |

# **20.5 Evaluator::new_empty() - Add to lumina-runtime**

The REPL needs to create an Evaluator with no pre-loaded program. Add this constructor to lumina-runtime/src/engine.rs if it does not already exist:

| **lumina-runtime/src/engine.rs - add new_empty()** |
| --- |
| impl Evaluator {<br><br>/// Creates an empty evaluator with no entities, rules, or instances.<br><br>/// Used by the REPL - statements are added one at a time via exec_statement().<br><br>pub fn new_empty() -> Self {<br><br>Self {<br><br>store: EntityStore::new(),<br><br>rules: Vec::new(),<br><br>functions: HashMap::new(), // v1.4: fn declarations<br><br>entity_defs: HashMap::new(), // entity schema registry<br><br>timers: TimerHeap::new(),<br><br>depth: 0,<br><br>output: Vec::new(),<br><br>}<br><br>}<br><br>/// Describe all declared entities as a human-readable string.<br><br>/// Used by :schema REPL command.<br><br>pub fn describe_schema(&self) -> String {<br><br>if self.entity_defs.is_empty() {<br><br>return "(no entities declared)".into();<br><br>}<br><br>self.entity_defs.iter()<br><br>.map(\|(name, fields)\| format!("entity {} {{ {} }}", name, fields.join(", ")))<br><br>.collect::&lt;Vec<\_&gt;>()<br><br>.join("\\n")<br><br>}<br><br>} |

# **20.6 Updated Main Loop in lumina-cli/src/main.rs**

| **Updated repl command handler** |
| --- |
| // In the match arm for "repl" command:<br><br>"repl" => {<br><br>use crate::repl::{ReplSession, ReplResult};<br><br>use crate::commands::run_command;<br><br>use std::io::{self, BufRead, Write};<br><br>let mut session = ReplSession::new();<br><br>let stdin = io::stdin();<br><br>loop {<br><br>// Show prompt based on brace depth<br><br>let prompt = if session.brace_depth > 0 { "... " } else { ">>> " };<br><br>print!("{}", prompt);<br><br>io::stdout().flush().ok();<br><br>let mut line = String::new();<br><br>if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 { break; }<br><br>let line = line.trim_end_matches('\\n').trim_end_matches('\\r');<br><br>// Inspector commands start with ":"<br><br>if line.starts_with(':') {<br><br>println!("{}", run_command(&mut session, line));<br><br>continue;<br><br>}<br><br>// Skip blank lines<br><br>if line.trim().is_empty() { continue; }<br><br>match session.feed(line) {<br><br>ReplResult::NeedMore => {} // show "..." next iteration<br><br>ReplResult::Ok(out) => { if !out.is_empty() { println!("{}", out); } }<br><br>ReplResult::Error(err) => { eprintln!("{}", err); }<br><br>}<br><br>}<br><br>} |

# **20.7 Build Order**

**▶ Exact sequence**

Step 1: Add new_empty() and describe_schema() to lumina-runtime/src/engine.rs.

Step 2: Run: cargo build -p lumina-runtime (must compile).

Step 3: Create crates/lumina-cli/src/repl.rs with the full ReplSession code.

Step 4: Create crates/lumina-cli/src/commands.rs with inspector commands.

Step 5: Add "mod repl;" and "mod commands;" to lumina-cli/src/main.rs.

Step 6: Replace the old "repl" match arm in main.rs with the new loop above.

Step 7: Run: cargo test --workspace (all 40 tests must still pass).

Step 8: Manual test: cargo run --bin lumina -- repl

Step 9: Type: entity Moto { battery: Number } then: let m = Moto { battery: 80 }

Step 10: Type :state - should print JSON with moto instance.

**Chapter 21**

**VS Code Extension**

_Implementation guide - TextMate grammar, snippets, language configuration_

The extension lives outside the Rust workspace. It is a standalone npm package in extensions/lumina-vscode/. It does NOT require a language server. v1.4 scope is: syntax highlighting, snippets, bracket matching, and comment toggling. Nothing else.

# **21.1 Directory Structure - Create All These Files**

| **Full directory layout (create exactly this)** |
| --- |
| extensions/<br><br>lumina-vscode/<br><br>package.json<br><br>language-configuration.json<br><br>syntaxes/<br><br>lumina.tmLanguage.json<br><br>snippets/<br><br>lumina.json<br><br>README.md<br><br>\# NOTE: Do NOT create src/ or extension.js - no activation code needed.<br><br>\# Syntax highlighting works purely from the grammar file, no JS needed. |

# **21.2 package.json - Complete File**

| **extensions/lumina-vscode/package.json** |
| --- |
| {<br><br>"name": "lumina-language",<br><br>"displayName": "Lumina",<br><br>"description": "Lumina (.lum) language support - syntax highlighting and snippets",<br><br>"version": "0.1.0",<br><br>"publisher": "isaac-ishimwe",<br><br>"engines": { "vscode": "^1.75.0" },<br><br>"categories": \["Programming Languages"\],<br><br>"contributes": {<br><br>"languages": \[{<br><br>"id": "lumina",<br><br>"aliases": \["Lumina", "lum"\],<br><br>"extensions": \[".lum"\],<br><br>"configuration": "./language-configuration.json"<br><br>}\],<br><br>"grammars": \[{<br><br>"language": "lumina",<br><br>"scopeName": "source.lumina",<br><br>"path": "./syntaxes/lumina.tmLanguage.json"<br><br>}\],<br><br>"snippets": \[{<br><br>"language": "lumina",<br><br>"path": "./snippets/lumina.json"<br><br>}\]<br><br>}<br><br>} |

# **21.3 language-configuration.json - Complete File**

| **extensions/lumina-vscode/language-configuration.json** |
| --- |
| {<br><br>"comments": {<br><br>"lineComment": "--"<br><br>},<br><br>"brackets": \[<br><br>\["{", "}"\],<br><br>\["\[", "\]"\],<br><br>\["(", ")"\]<br><br>\],<br><br>"autoClosingPairs": \[<br><br>{ "open": "{", "close": "}" },<br><br>{ "open": "\[", "close": "\]" },<br><br>{ "open": "(", "close": ")" },<br><br>{ "open": "\\"", "close": "\\"" }<br><br>\],<br><br>"surroundingPairs": \[<br><br>\["{", "}"\], \["\[", "\]"\], \["(", ")"\], \["\\"", "\\""\]<br><br>\]<br><br>} |

# **21.4 lumina.tmLanguage.json - Complete Grammar File**

| **extensions/lumina-vscode/syntaxes/lumina.tmLanguage.json** |
| --- |
| {<br><br>"\$schema": "<https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json>",<br><br>"name": "Lumina",<br><br>"scopeName": "source.lumina",<br><br>"patterns": \[<br><br>{ "include": "#comments" },<br><br>{ "include": "#strings" },<br><br>{ "include": "#numbers" },<br><br>{ "include": "#booleans" },<br><br>{ "include": "#keywords" },<br><br>{ "include": "#types" },<br><br>{ "include": "#operators" },<br><br>{ "include": "#metadata" }<br><br>\],<br><br>"repository": {<br><br>"comments": {<br><br>"name": "comment.line.double-dash.lumina",<br><br>"match": "--.\*\$"<br><br>},<br><br>"strings": {<br><br>"name": "string.quoted.double.lumina",<br><br>"begin": "\\"",<br><br>"end": "\\"",<br><br>"patterns": \[{<br><br>"name": "meta.interpolation.lumina",<br><br>"begin": "\\\\{",<br><br>"end": "\\\\}",<br><br>"patterns": \[{ "include": "\$self" }\]<br><br>}\]<br><br>},<br><br>"numbers": {<br><br>"name": "constant.numeric.lumina",<br><br>"match": "\\\\b\[0-9\]+(\\\\.\[0-9\]+)?\\\\b"<br><br>},<br><br>"booleans": {<br><br>"name": "constant.language.lumina",<br><br>"match": "\\\\b(true\|false)\\\\b"<br><br>},<br><br>"keywords": {<br><br>"name": "keyword.control.lumina",<br><br>"match": "\\\\b(entity\|rule\|when\|becomes\|for\|every\|let\|show\|update\|to\|create\|delete\|if\|then\|else\|and\|or\|not\|is\|external\|sync\|on\|fn\|import)\\\\b"<br><br>},<br><br>"types": {<br><br>"name": "storage.type.lumina",<br><br>"match": "\\\\b(Number\|Text\|Boolean)\\\\b"<br><br>},<br><br>"operators": {<br><br>"name": "keyword.operator.lumina",<br><br>"match": ":=\|->\|==\|!=\|>=\|&lt;=\|&gt;\|<\|\\\\+\|-\|\\\\\*\|/\|%\|\\\\.\|:"<br><br>},<br><br>"metadata": {<br><br>"name": "storage.modifier.lumina",<br><br>"match": "@(range\|doc\|affects)"<br><br>}<br><br>}<br><br>} |

# **21.5 lumina.json Snippets - Complete File**

| **extensions/lumina-vscode/snippets/lumina.json** |
| --- |
| {<br><br>"Entity declaration": {<br><br>"prefix": "entity",<br><br>"body": \[<br><br>"entity \${1:Name} {",<br><br>"\\t\${2:field}: \${3:Number}",<br><br>"\\t\${4:derived} := \${5:expr}",<br><br>"}"<br><br>\],<br><br>"description": "Declare a Lumina entity"<br><br>},<br><br>"Rule when becomes": {<br><br>"prefix": "rwhen",<br><br>"body": \[<br><br>"rule \${1:Name} when \${2:Entity}.\${3:field} becomes \${4:true} {",<br><br>"\\t\${5:show \\"fired\\"}",<br><br>"}"<br><br>\],<br><br>"description": "Rule triggered by state transition"<br><br>},<br><br>"Rule when becomes for": {<br><br>"prefix": "rfor",<br><br>"body": \[<br><br>"rule \${1:Name} when \${2:Entity}.\${3:field} becomes \${4:true} for \${5:5}s {",<br><br>"\\t\${6:show \\"sustained\\"}",<br><br>"}"<br><br>\],<br><br>"description": "Rule triggered after sustained condition"<br><br>},<br><br>"Rule every": {<br><br>"prefix": "revery",<br><br>"body": \[<br><br>"rule \${1:Name} every \${2:30}s {",<br><br>"\\t\${3:show \\"tick\\"}",<br><br>"}"<br><br>\],<br><br>"description": "Rule triggered on a fixed interval"<br><br>},<br><br>"Let binding": {<br><br>"prefix": "let",<br><br>"body": \["let \${1:name} = \${2:Entity} { \${3:field}: \${4:value} }"\],<br><br>"description": "Create an entity instance"<br><br>},<br><br>"Function declaration": {<br><br>"prefix": "fn",<br><br>"body": \[<br><br>"fn \${1:name}(\${2:param}: \${3:Number}) -> \${4:Number} {",<br><br>"\\t\${5:expr}",<br><br>"}"<br><br>\],<br><br>"description": "Pure function declaration (v1.4)"<br><br>}<br><br>} |

# **21.6 Build and Install Commands**

| **Package and install the extension** |
| --- |
| \# One-time: install vsce (VS Code extension packager)<br><br>npm install -g @vscode/vsce<br><br>\# Package the extension into a .vsix file<br><br>cd extensions/lumina-vscode<br><br>vsce package<br><br>\# Output: lumina-language-0.1.0.vsix<br><br>\# Install in VS Code<br><br>code --install-extension lumina-language-0.1.0.vsix<br><br>\# Alternative: drag the .vsix into VS Code Extensions panel<br><br>\# Verify: open any .lum file - keywords should be colored,<br><br>\# typing "entity" + Tab should expand the snippet. |

**⚠️ Do NOT add a language server in v1.4**

No src/extension.js, no activationEvents, no vscode.languages.registerHoverProvider.

No inline error squiggles - that requires LSP which is v1.5 scope.

The extension is purely declarative: grammar + snippets + language-configuration.

The package.json "main" field must be absent or point to nothing.

**Chapter 22**

**Pure Functions - fn keyword**

_Implementation guide - exact AST nodes, analysis rules, evaluation logic_

The fn keyword adds reusable, side-effect-free named expressions. Functions are top-level statements. They take typed parameters and return a single expression value. They cannot access entity instances or trigger any side effects. This chapter gives every code change needed across lexer, parser, analyzer, and runtime.

# **22.1 Lexer Change - Add fn Token**

One token to add. Open crates/lumina-lexer/src/token.rs:

| **crates/lumina-lexer/src/token.rs - add Fn variant** |
| --- |
| pub enum Token {<br><br>// ... all existing tokens unchanged ...<br><br>// v1.4 additions:<br><br>Fn, // "fn" keyword<br><br>Arrow, // "->" (MAY already exist for rules - check first)<br><br>}<br><br>// In the logos derive on Token, add the pattern:<br><br>// #\[token("fn")\]<br><br>// Fn,<br><br>// Arrow "->" may already be tokenized - do NOT add a duplicate.<br><br>// grep for "Arrow" in token.rs before adding. |

# **22.2 Parser Changes - FnDecl AST Node**

| **crates/lumina-parser/src/ast.rs - add FnDecl** |
| --- |
| // Add to the Statement enum:<br><br>pub enum Statement {<br><br>Entity(EntityDecl),<br><br>ExternalEntity(ExternalEntityDecl),<br><br>Let(LetStmt),<br><br>Rule(RuleDecl),<br><br>Action(Action),<br><br>Fn(FnDecl), // NEW - pure function declaration<br><br>}<br><br>// New structs:<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct FnDecl {<br><br>pub name: String,<br><br>pub params: Vec&lt;FnParam&gt;,<br><br>pub returns: LuminaType,<br><br>pub body: Expr,<br><br>pub span: Span,<br><br>}<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct FnParam {<br><br>pub name: String,<br><br>pub type_: LuminaType,<br><br>pub span: Span,<br><br>} |

| **crates/lumina-parser/src/ast.rs - add Call to Expr** |
| --- |
| // Add to the Expr enum:<br><br>pub enum Expr {<br><br>// ... all existing variants ...<br><br>Call { // NEW - function call: clamp(x, 0, 100)<br><br>name: String,<br><br>args: Vec&lt;Expr&gt;,<br><br>span: Span,<br><br>},<br><br>} |

# **22.3 Parser - parse_fn_decl() Function**

| **crates/lumina-parser/src/parser.rs - add parse_fn_decl()** |
| --- |
| // In the top-level parse_statement() match:<br><br>Token::Fn => self.parse_fn_decl(),<br><br>// Implementation:<br><br>fn parse_fn_decl(&mut self) -> Result&lt;Statement, ParseError&gt; {<br><br>let start = self.current_span();<br><br>self.expect(Token::Fn)?;<br><br>let name = self.expect_ident("function name")?;<br><br>self.expect(Token::LParen)?;<br><br>let mut params = Vec::new();<br><br>while !self.check(Token::RParen) {<br><br>let p_span = self.current_span();<br><br>let p_name = self.expect_ident("parameter name")?;<br><br>self.expect(Token::Colon)?;<br><br>let p_type = self.parse_type()?;<br><br>params.push(FnParam { name: p_name, type_: p_type, span: p_span });<br><br>if !self.check(Token::RParen) { self.expect(Token::Comma)?; }<br><br>}<br><br>self.expect(Token::RParen)?;<br><br>self.expect(Token::Arrow)?; // "->"<br><br>let returns = self.parse_type()?;<br><br>self.expect(Token::LBrace)?;<br><br>let body = self.parse_expr()?;<br><br>self.expect(Token::RBrace)?;<br><br>Ok(Statement::Fn(FnDecl {<br><br>name, params, returns, body,<br><br>span: self.span_from(start),<br><br>}))<br><br>} |

| **Parser - parse_call() inside parse_primary()** |
| --- |
| // In parse_primary(), after matching an Ident:<br><br>Token::Ident(name) => {<br><br>let span = self.current_span();<br><br>self.advance();<br><br>// If followed by "(", it's a function call<br><br>if self.check(Token::LParen) {<br><br>self.advance(); // consume "("<br><br>let mut args = Vec::new();<br><br>while !self.check(Token::RParen) {<br><br>args.push(self.parse_expr()?);<br><br>if !self.check(Token::RParen) { self.expect(Token::Comma)?; }<br><br>}<br><br>self.expect(Token::RParen)?;<br><br>return Ok(Expr::Call { name: name.clone(), args, span });<br><br>}<br><br>// Otherwise it's a plain identifier reference<br><br>Ok(Expr::Ident(name.clone(), span))<br><br>} |

# **22.4 Analyzer - New Error Codes L011-L015**

| **Code** | **Meaning** | **When triggered** |
| --- | --- | --- |
| L011 | Duplicate fn name | Two fn declarations share the same name |
| L012 | Unknown fn called | Call to fn that was not declared |
| L013 | Argument count mismatch | Wrong number of args passed to fn call |
| L014 | Return type mismatch | Body expr type != declared return type |
| L015 | fn body accesses entity | fn body references an entity instance/field |

| **crates/lumina-analyzer/src/analyzer.rs - fn analysis** |
| --- |
| // In the Analyzer struct, add:<br><br>fn_defs: HashMap&lt;String, FnDecl&gt;,<br><br>// In analyze_statement():<br><br>Statement::Fn(decl) => {<br><br>if self.fn_defs.contains_key(&decl.name) {<br><br>self.error("L011", &format!("duplicate fn name: {}", decl.name), decl.span);<br><br>return;<br><br>}<br><br>// Check body expr - pass param names as local scope<br><br>let local_scope: HashSet&lt;String&gt; = decl.params.iter()<br><br>.map(\|p\| p.name.clone()).collect();<br><br>self.check_fn_body(&decl.body, &local_scope, decl.span);<br><br>// Infer body type and compare with declared return type<br><br>if let Ok(body_type) = self.infer_type_local(&decl.body, &local_scope) {<br><br>if body_type != decl.returns {<br><br>self.error("L014", "return type mismatch", decl.span);<br><br>}<br><br>}<br><br>self.fn_defs.insert(decl.name.clone(), decl.clone());<br><br>}<br><br>// check_fn_body - ensure no entity access<br><br>fn check_fn_body(&mut self, expr: &Expr, locals: &HashSet&lt;String&gt;, span: Span) {<br><br>match expr {<br><br>Expr::FieldAccess { object, .. } => {<br><br>// If object is not in local params, it's an entity access - L015<br><br>if !locals.contains(object) {<br><br>self.error("L015",<br><br>"fn body cannot access entity fields", span);<br><br>}<br><br>}<br><br>// Recurse into sub-expressions<br><br>_=> expr.children().for_each(\|c\| self.check_fn_body(c, locals, span)),<br><br>}<br><br>}<br><br>// In analyze_expr() - validate Call expressions:<br><br>Expr::Call { name, args, span } => {<br><br>let decl = match self.fn_defs.get(name) {<br><br>Some(d) => d.clone(),<br><br>None => { self.error("L012", &format!("unknown fn: {}", name), \*span); return; }<br><br>};<br><br>if args.len() != decl.params.len() {<br><br>self.error("L013", &format!("fn {} expects {} args, got {}",<br><br>name, decl.params.len(), args.len()), \*span);<br><br>}<br><br>} |

# **22.5 Runtime - Evaluating fn Calls**

| **crates/lumina-runtime/src/engine.rs - fn storage and call evaluation** |
| --- |
| // Add to Evaluator struct:<br><br>pub functions: HashMap&lt;String, FnDecl&gt;,<br><br>// Register fn at exec_statement time:<br><br>Statement::Fn(decl) => {<br><br>self.functions.insert(decl.name.clone(), decl.clone());<br><br>Ok(())<br><br>}<br><br>// In eval_expr() - handle Call:<br><br>Expr::Call { name, args, .. } => {<br><br>let decl = self.functions.get(name)<br><br>.ok_or_else(\| RuntimeError::R002)? // should not happen post-analysis<br><br>.clone();<br><br>// Evaluate each argument in the CALLER's context<br><br>let arg_vals: Vec&lt;Value&gt; = args.iter()<br><br>.map(\|a\| self.eval_expr(a, ctx))<br><br>.collect::&lt;Result<\_, \_&gt;>()?;<br><br>// Bind params to evaluated args<br><br>let mut local: HashMap&lt;String, Value&gt; = HashMap::new();<br><br>for (param, val) in decl.params.iter().zip(arg_vals) {<br><br>local.insert(param.name.clone(), val);<br><br>}<br><br>// Evaluate body with local scope ONLY (no entity store access)<br><br>self.eval_expr_local(&decl.body, &local)<br><br>}<br><br>// eval_expr_local - evaluates expr using only a local var map<br><br>// It does NOT call self.eval_expr() to prevent entity access<br><br>fn eval_expr_local(&self, expr: &Expr, locals: &HashMap&lt;String, Value&gt;)<br><br>\-> Result&lt;Value, RuntimeError&gt;<br><br>{<br><br>match expr {<br><br>Expr::Ident(name, \_) => locals.get(name)<br><br>.cloned()<br><br>.ok_or(RuntimeError::R005 { instance: name.clone(), field: name.clone() }),<br><br>Expr::NumberLit(n) => Ok(Value::Number(\*n)),<br><br>Expr::StringLit(s) => Ok(Value::Text(s.clone())),<br><br>Expr::BoolLit(b) => Ok(Value::Boolean(\*b)),<br><br>Expr::BinaryOp { op, left, right } => {<br><br>let l = self.eval_expr_local(left, locals)?;<br><br>let r = self.eval_expr_local(right, locals)?;<br><br>self.apply_binop(op, l, r)<br><br>}<br><br>Expr::IfThenElse { cond, then, else_} => {<br><br>let c = self.eval_expr_local(cond, locals)?;<br><br>if c.as_bool().unwrap_or(false) {<br><br>self.eval_expr_local(then, locals)<br><br>} else {<br><br>self.eval_expr_local(else_, locals)<br><br>}<br><br>}<br><br>_ => Err(RuntimeError::R002), // unsupported expr in fn body<br><br>}<br><br>} |

# **22.6 Build Order**

**▶ Exact sequence**

Step 1: Add Token::Fn to lumina-lexer/src/token.rs + logos pattern.

Step 2: cargo build -p lumina-lexer

Step 3: Add FnDecl, FnParam, Expr::Call to lumina-parser/src/ast.rs.

Step 4: Add parse_fn_decl() and call detection to lumina-parser/src/parser.rs.

Step 5: cargo build -p lumina-parser

Step 6: Add L011-L015 checks to lumina-analyzer/src/analyzer.rs.

Step 7: cargo build -p lumina-analyzer

Step 8: Add functions field to Evaluator, eval_expr_local(), Call case in eval_expr().

Step 9: cargo test --workspace (all 40 tests must pass).

Step 10: Write a .lum file with fn clamp(v: Number, lo: Number, hi: Number) -> Number { ... }

Step 11: lumina run the file and verify the fn is callable from a derived field.

**Chapter 23**

**Modules - import keyword**

_Implementation guide - file resolution, circular import detection, merged AST_

import lets a .lum file pull in entities, fn declarations, and let bindings from another .lum file. The feature is implemented entirely in lumina-cli - the parser just records the import statement, and a new ModuleLoader in the CLI resolves and merges all files before handing a flat Program to the Evaluator.

# **23.1 Lexer - Add Import Token**

| **crates/lumina-lexer/src/token.rs** |
| --- |
| // Add one token:<br><br>// #\[token("import")\]<br><br>// Import,<br><br>pub enum Token {<br><br>// ... existing tokens ...<br><br>Import, // "import" keyword - v1.4<br><br>} |

# **23.2 Parser - ImportDecl AST Node**

| **crates/lumina-parser/src/ast.rs** |
| --- |
| // Add to Statement enum:<br><br>pub enum Statement {<br><br>// ... existing ...<br><br>Import(ImportDecl), // NEW<br><br>}<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct ImportDecl {<br><br>pub path: String, // e.g. "shared/moto.lum"<br><br>pub span: Span,<br><br>}<br><br>// Add helper to Program:<br><br>impl Program {<br><br>pub fn imports(&self) -> impl Iterator&lt;Item = &ImportDecl&gt; {<br><br>self.statements.iter().filter_map(\|s\| {<br><br>if let Statement::Import(i) = s { Some(i) } else { None }<br><br>})<br><br>}<br><br>} |

| **crates/lumina-parser/src/parser.rs - parse_import()** |
| --- |
| // In parse_statement():<br><br>Token::Import => self.parse_import(),<br><br>fn parse_import(&mut self) -> Result&lt;Statement, ParseError&gt; {<br><br>let span = self.current_span();<br><br>self.expect(Token::Import)?;<br><br>// Expect a string literal as the path<br><br>let path = match self.current_token() {<br><br>Token::StringLit(s) => { let p = s.clone(); self.advance(); p }<br><br>_ => return Err(self.error("expected string path after import")),<br><br>};<br><br>Ok(Statement::Import(ImportDecl { path, span }))<br><br>} |

# **23.3 ModuleLoader - New File in lumina-cli**

Create crates/lumina-cli/src/loader.rs. This is the core of the module system - it does all the file I/O, cycle detection, and AST merging.

| **crates/lumina-cli/src/loader.rs - complete file** |
| --- |
| use std::collections::{HashMap, HashSet};<br><br>use std::path::{Path, PathBuf};<br><br>use std::fs;<br><br>use lumina_parser::{parse, Program, Statement};<br><br>use lumina_diagnostics::Diagnostic;<br><br>pub struct ModuleLoader {<br><br>/// Fully loaded and parsed programs, keyed by canonical path<br><br>loaded: HashMap&lt;PathBuf, Program&gt;,<br><br>/// Load order (topological - dependencies first)<br><br>order: Vec&lt;PathBuf&gt;,<br><br>/// Currently being loaded - used for cycle detection<br><br>in_stack: HashSet&lt;PathBuf&gt;,<br><br>}<br><br>impl ModuleLoader {<br><br>/// Entry point: load an entry .lum file and all its transitive imports.<br><br>/// Returns a single merged Program ready for analysis and execution.<br><br>pub fn load(entry: &Path) -> Result&lt;Program, Vec<Diagnostic&gt;> {<br><br>let mut loader = Self {<br><br>loaded: HashMap::new(),<br><br>order: Vec::new(),<br><br>in_stack: HashSet::new(),<br><br>};<br><br>let canonical = entry.canonicalize().map_err(\|e\| {<br><br>vec!\[file_not_found(entry, &e.to_string())\]<br><br>})?;<br><br>loader.load_recursive(&canonical)?;<br><br>Ok(loader.merge())<br><br>}<br><br>fn load_recursive(&mut self, path: &PathBuf) -> Result&lt;(), Vec<Diagnostic&gt;> {<br><br>// Already loaded - skip (DAG, not tree)<br><br>if self.loaded.contains_key(path) { return Ok(()); }<br><br>// Cycle detection<br><br>if self.in_stack.contains(path) {<br><br>return Err(vec!\[circular_import(path)\]);<br><br>}<br><br>self.in_stack.insert(path.clone());<br><br>// Read and parse<br><br>let source = fs::read_to_string(path).map_err(\|e\| {<br><br>vec!\[file_not_found(path, &e.to_string())\]<br><br>})?;<br><br>let program = parse(&source).map_err(\|e\| {<br><br>vec!\[parse_to_diagnostic(e, path)\]<br><br>})?;<br><br>// Recurse into imports before adding this file<br><br>let dir = path.parent().unwrap_or(Path::new("."));<br><br>for import in program.imports() {<br><br>let dep_path = dir.join(&import.path);<br><br>let dep_canonical = dep_path.canonicalize().map_err(\|e\| {<br><br>vec!\[file_not_found(&dep_path, &e.to_string())\]<br><br>})?;<br><br>self.load_recursive(&dep_canonical)?;<br><br>}<br><br>self.in_stack.remove(path);<br><br>self.loaded.insert(path.clone(), program);<br><br>self.order.push(path.clone());<br><br>Ok(())<br><br>}<br><br>/// Merge all programs in topological order into one flat Program.<br><br>/// Import statements are stripped from the merged output.<br><br>fn merge(&self) -> Program {<br><br>let mut stmts = Vec::new();<br><br>for path in &self.order {<br><br>if let Some(prog) = self.loaded.get(path) {<br><br>for stmt in &prog.statements {<br><br>// Skip import statements - already resolved<br><br>if !matches!(stmt, Statement::Import(\_)) {<br><br>stmts.push(stmt.clone());<br><br>}<br><br>}<br><br>}<br><br>}<br><br>Program { statements: stmts }<br><br>}<br><br>}<br><br>// Error constructors<br><br>fn file_not_found(path: &Path, reason: &str) -> Diagnostic {<br><br>use lumina_diagnostics::{Diagnostic, SourceLocation};<br><br>Diagnostic::new("L017", format!("file not found: {} - {}", path.display(), reason),<br><br>SourceLocation::new("&lt;import&gt;", 0, 0, 0), "", None)<br><br>}<br><br>fn circular_import(path: &Path) -> Diagnostic {<br><br>use lumina_diagnostics::{Diagnostic, SourceLocation};<br><br>Diagnostic::new("L016", format!("circular import: {}", path.display()),<br><br>SourceLocation::new("&lt;import&gt;", 0, 0, 0), "", None)<br><br>}<br><br>fn parse_to_diagnostic(e: lumina_parser::ParseError, path: &Path) -> Diagnostic {<br><br>use lumina_diagnostics::{Diagnostic, SourceLocation};<br><br>Diagnostic::new("P001", e.message,<br><br>SourceLocation::new(path.to_string_lossy(), e.span.line, e.span.col, 1),<br><br>"", None)<br><br>} |

# **23.4 Update lumina-cli/src/main.rs to Use ModuleLoader**

| **main.rs - replace direct parse() call with ModuleLoader::load()** |
| --- |
| // Add module declaration:<br><br>mod loader;<br><br>// In the "run" and "check" command handlers, replace:<br><br>// let source = fs::read_to_string(&file)?;<br><br>// let program = parse(&source)?;<br><br>// With:<br><br>use crate::loader::ModuleLoader;<br><br>let program = match ModuleLoader::load(Path::new(&file)) {<br><br>Ok(p) => p,<br><br>Err(diags) => {<br><br>eprintln!("{}", DiagnosticRenderer::render_all(&diags));<br><br>std::process::exit(1);<br><br>}<br><br>};<br><br>// The rest of the pipeline (analyze -> execute) is unchanged. |

# **23.5 Analyzer - Add L016/L017/L018**

The analyzer does not need to handle import resolution - ModuleLoader does that. But the analyzer must reject import statements in WASM mode (L018). Add a flag to the analyzer:

| **crates/lumina-analyzer/src/lib.rs** |
| --- |
| // Add to analyze() parameters:<br><br>pub fn analyze(<br><br>program: &Program,<br><br>source: &str,<br><br>filename: &str,<br><br>allow_imports: bool, // false for WASM, true for CLI<br><br>) -> Vec&lt;Diagnostic&gt;<br><br>// In analyze_statement():<br><br>Statement::Import(decl) => {<br><br>if !self.allow_imports {<br><br>self.error("L018",<br><br>"import is not supported in single-file (WASM) mode",<br><br>decl.span);<br><br>}<br><br>// If allow_imports == true, the ModuleLoader already resolved this.<br><br>// By the time analyze() sees the merged Program, imports are stripped.<br><br>} |

# **23.6 Build Order**

**▶ Exact sequence**

Step 1: Add Token::Import to lexer. cargo build -p lumina-lexer.

Step 2: Add ImportDecl to parser AST. Add parse_import(). cargo build -p lumina-parser.

Step 3: Create crates/lumina-cli/src/loader.rs with ModuleLoader.

Step 4: Add "mod loader;" to lumina-cli/src/main.rs.

Step 5: Update "run" and "check" command handlers to use ModuleLoader::load().

Step 6: Add allow_imports param to analyze() in lumina-analyzer. Update all callers.

Step 7: cargo test --workspace - all 40 tests must still pass.

Step 8: Create two .lum files where one imports the other and test lumina run.

**Chapter 24**

**String Interpolation**

_Implementation guide - {expr} inside Text literals_

String interpolation allows embedding expressions directly inside double-quoted strings using {expr} syntax. Simple strings with no { } continue to produce StringLit tokens unchanged. Only strings containing { } use the new InterpolatedString AST node. Escaped {{ and }} produce literal braces.

# **24.1 Lexer - Mode-Switching for Interpolation**

The lexer must switch into "interpolation mode" when it encounters { inside a string. The cleanest approach for logos is to implement a custom string tokenizer that produces a Vec of string segment tokens.

| **Strategy: post-process StringLit tokens in the tokenizer** |
| --- |
| // Rather than adding complex lexer modes to logos,<br><br>// handle interpolation in a post-processing step:<br><br>// 1. The logos lexer produces StringLit(String) as before.<br><br>// 2. After tokenization, a pass checks each StringLit for { }.<br><br>// 3. If found, it splits the StringLit into InterpolatedString tokens.<br><br>// New token variants - add to Token enum:<br><br>pub enum Token {<br><br>// ... existing ...<br><br>// StringLit(String) already exists - keep it for simple strings<br><br>// New variants for interpolated strings:<br><br>InterpStringStart, // opening " of interpolated string<br><br>InterpPart(String), // literal text segment<br><br>InterpExprStart, // {<br><br>InterpExprEnd, // }<br><br>InterpStringEnd, // closing "<br><br>} |

| **Post-processing pass - split StringLit with interpolation** |
| --- |
| // In crates/lumina-lexer/src/lib.rs, after logos tokenization:<br><br>pub fn tokenize(source: &str, filename: &str)<br><br>\-> Result&lt;Vec<SpannedToken&gt;, LexError><br><br>{<br><br>let raw = lex_raw(source)?; // existing logos call<br><br>Ok(expand_interpolations(raw))<br><br>}<br><br>fn expand_interpolations(tokens: Vec&lt;SpannedToken&gt;) -> Vec&lt;SpannedToken&gt; {<br><br>let mut out = Vec::new();<br><br>for tok in tokens {<br><br>if let Token::StringLit(ref s) = tok.token {<br><br>if s.contains('{') {<br><br>// Split into interpolation token sequence<br><br>out.extend(split_interpolated(s, tok.span));<br><br>continue;<br><br>}<br><br>}<br><br>out.push(tok);<br><br>}<br><br>out<br><br>}<br><br>fn split_interpolated(s: &str, base_span: Span) -> Vec&lt;SpannedToken&gt; {<br><br>let mut result = Vec::new();<br><br>result.push(spanned(Token::InterpStringStart, base_span));<br><br>let mut chars = s.chars().peekable();<br><br>let mut literal = String::new();<br><br>while let Some(ch) = chars.next() {<br><br>match ch {<br><br>'{' if chars.peek() == Some(&'{') => {<br><br>chars.next(); literal.push('{'); // {{ -> {<br><br>}<br><br>'}' if chars.peek() == Some(&'}') => {<br><br>chars.next(); literal.push('}'); // }} -> }<br><br>}<br><br>'{' => {<br><br>if !literal.is_empty() {<br><br>result.push(spanned(Token::InterpPart(literal.clone()), base_span));<br><br>literal.clear();<br><br>}<br><br>result.push(spanned(Token::InterpExprStart, base_span));<br><br>// Collect until matching }<br><br>let mut expr_src = String::new();<br><br>let mut depth = 1;<br><br>for ch2 in chars.by_ref() {<br><br>if ch2 == '{' { depth += 1; }<br><br>if ch2 == '}' { depth -= 1; if depth == 0 { break; } }<br><br>expr_src.push(ch2);<br><br>}<br><br>// Re-tokenize the expression inside {}<br><br>if let Ok(inner) = lex_raw(&expr_src) {<br><br>result.extend(inner);<br><br>}<br><br>result.push(spanned(Token::InterpExprEnd, base_span));<br><br>}<br><br>c => literal.push(c),<br><br>}<br><br>}<br><br>if !literal.is_empty() {<br><br>result.push(spanned(Token::InterpPart(literal), base_span));<br><br>}<br><br>result.push(spanned(Token::InterpStringEnd, base_span));<br><br>result<br><br>} |

# **24.2 Parser - InterpolatedString AST Node**

| **crates/lumina-parser/src/ast.rs** |
| --- |
| // Add to Expr enum:<br><br>pub enum Expr {<br><br>// ... existing ...<br><br>InterpolatedString(Vec&lt;StringSegment&gt;), // NEW<br><br>}<br><br>#\[derive(Debug, Clone)\]<br><br>pub enum StringSegment {<br><br>Literal(String), // plain text portion<br><br>Expr(Box&lt;Expr&gt;), // {expr} portion<br><br>} |

| **crates/lumina-parser/src/parser.rs - parse interpolated string** |
| --- |
| // In parse_primary(), handle InterpStringStart:<br><br>Token::InterpStringStart => {<br><br>self.advance();<br><br>let mut segments = Vec::new();<br><br>loop {<br><br>match self.current_token() {<br><br>Token::InterpPart(s) => {<br><br>segments.push(StringSegment::Literal(s.clone()));<br><br>self.advance();<br><br>}<br><br>Token::InterpExprStart => {<br><br>self.advance(); // consume {<br><br>let expr = self.parse_expr()?;<br><br>self.expect(Token::InterpExprEnd)?; // consume }<br><br>segments.push(StringSegment::Expr(Box::new(expr)));<br><br>}<br><br>Token::InterpStringEnd => {<br><br>self.advance(); // consume closing "<br><br>break;<br><br>}<br><br>_ => return Err(self.error("unexpected token in interpolated string")),<br><br>}<br><br>}<br><br>Ok(Expr::InterpolatedString(segments))<br><br>} |

# **24.3 Analyzer - Type Check Interpolated Strings**

| **crates/lumina-analyzer/src/analyzer.rs** |
| --- |
| // In infer_type():<br><br>Expr::InterpolatedString(segments) => {<br><br>// Every segment expr must be Number, Text, or Boolean (all stringifiable)<br><br>for seg in segments {<br><br>if let StringSegment::Expr(e) = seg {<br><br>match self.infer_type(e) {<br><br>Ok(LuminaType::Number) \| Ok(LuminaType::Text) \| Ok(LuminaType::Boolean) => {}<br><br>Ok(other) => self.error("L004",<br><br>&format!("interpolated expr must be Number/Text/Boolean, got {:?}", other),<br><br>e.span()),<br><br>Err(\_) => {} // sub-error already reported<br><br>}<br><br>}<br><br>}<br><br>Ok(LuminaType::Text) // interpolated string always produces Text<br><br>} |

# **24.4 Runtime - Evaluate InterpolatedString**

| **crates/lumina-runtime/src/engine.rs - eval_expr()** |
| --- |
| Expr::InterpolatedString(segments) => {<br><br>let mut result = String::new();<br><br>for seg in segments {<br><br>match seg {<br><br>StringSegment::Literal(s) => result.push_str(s),<br><br>StringSegment::Expr(e) => {<br><br>let val = self.eval_expr(e, ctx)?;<br><br>match val {<br><br>Value::Number(n) => {<br><br>// Format cleanly: 80.0 -> "80", 3.14 -> "3.14"<br><br>if n.fract() == 0.0 {<br><br>result.push_str(&format!("{}", n as i64));<br><br>} else {<br><br>result.push_str(&format!("{}", n));<br><br>}<br><br>}<br><br>Value::Text(s) => result.push_str(&s),<br><br>Value::Boolean(b) => result.push_str(if b { "true" } else { "false" }),<br><br>_=> result.push_str("?"),<br><br>}<br><br>}<br><br>}<br><br>}<br><br>Ok(Value::Text(result))<br><br>} |

**📌 📌 Number formatting rule**

80.0 must render as "80" (not "80.0") inside interpolated strings.

Check n.fract() == 0.0 and cast to i64 for whole numbers.

This matches user expectations: "battery: {80.0}%" -> "battery: 80%".

# **24.5 Build Order**

**▶ Exact sequence**

Step 1: Add InterpStringStart/InterpPart/InterpExprStart/InterpExprEnd/InterpStringEnd to Token enum.

Step 2: Add split_interpolated() and expand_interpolations() to lumina-lexer.

Step 3: cargo build -p lumina-lexer.

Step 4: Add InterpolatedString and StringSegment to lumina-parser/src/ast.rs.

Step 5: Add InterpStringStart case to parse_primary() in the parser.

Step 6: cargo build -p lumina-parser.

Step 7: Add InterpolatedString type inference to lumina-analyzer.

Step 8: Add InterpolatedString eval to lumina-runtime.

Step 9: cargo test --workspace.

Step 10: Test: show "battery: {moto1.battery}%" and verify output.

**Chapter 25**

**List Types**

_Implementation guide - Number\[\], Text\[\], Boolean\[\], built-in list functions_

List types add ordered collections to Lumina. A stored field can be Number\[\], Text\[\], or Boolean\[\]. Lists are value types - append() returns a new list, it does not mutate. R004 (previously reserved) becomes active for out-of-bounds access. Eight built-in list functions are added as special-cased Call expressions.

# **25.1 Lexer - List Type Syntax**

| **No new tokens needed - \[\] uses existing LBracket/RBracket** |
| --- |
| // Token::LBracket "\[" and Token::RBracket "\]" already exist.<br><br>// "Number\[\]" is parsed as: Token::Number, Token::LBracket, Token::RBracket.<br><br>// No new tokens required.<br><br>// HOWEVER: list literal \[1, 2, 3\] also uses LBracket/RBracket.<br><br>// The parser distinguishes based on position:<br><br>// In a type context: Number\[\] -> LuminaType::List(Number)<br><br>// In an expression context: \[1, 2\] -> Expr::ListLiteral |

| **crates/lumina-parser/src/ast.rs - LuminaType and Expr additions** |
| --- |
| // Add to LuminaType enum:<br><br>pub enum LuminaType {<br><br>Number,<br><br>Text,<br><br>Boolean,<br><br>List(Box&lt;LuminaType&gt;), // NEW - Number\[\], Text\[\], Boolean\[\]<br><br>}<br><br>// Add to Expr enum:<br><br>pub enum Expr {<br><br>// ... existing ...<br><br>ListLiteral(Vec&lt;Expr&gt;), // NEW - \[1, 2, 3\] or \["a", "b"\]<br><br>Index { // NEW - list\[0\]<br><br>list: Box&lt;Expr&gt;,<br><br>index: Box&lt;Expr&gt;,<br><br>span: Span,<br><br>},<br><br>}<br><br>// Add to Value enum in lumina-runtime:<br><br>pub enum Value {<br><br>Number(f64),<br><br>Text(String),<br><br>Boolean(bool),<br><br>List(Vec&lt;Value&gt;), // NEW<br><br>Null,<br><br>} |

# **25.2 Parser - Type Parsing and List Literals**

| **parse_type() - handle List types** |
| --- |
| fn parse_type(&mut self) -> Result&lt;LuminaType, ParseError&gt; {<br><br>let base = match self.current_token() {<br><br>Token::Number => { self.advance(); LuminaType::Number }<br><br>Token::Text => { self.advance(); LuminaType::Text }<br><br>Token::Boolean => { self.advance(); LuminaType::Boolean }<br><br>_ => return Err(self.error("expected type (Number, Text, Boolean)")),<br><br>};<br><br>// Check for \[\] suffix - makes it a list type<br><br>if self.check(Token::LBracket) {<br><br>self.advance(); // \[<br><br>self.expect(Token::RBracket)?; // \]<br><br>return Ok(LuminaType::List(Box::new(base)));<br><br>}<br><br>Ok(base)<br><br>} |

| **parse_primary() - handle list literals and index access** |
| --- |
| // List literal: \[expr, expr, ...\]<br><br>Token::LBracket => {<br><br>let span = self.current_span();<br><br>self.advance();<br><br>let mut elems = Vec::new();<br><br>while !self.check(Token::RBracket) {<br><br>elems.push(self.parse_expr()?);<br><br>if !self.check(Token::RBracket) { self.expect(Token::Comma)?; }<br><br>}<br><br>self.expect(Token::RBracket)?;<br><br>Ok(Expr::ListLiteral(elems))<br><br>}<br><br>// Index access postfix: expr\[expr\]<br><br>// In parse_postfix() after parsing the primary:<br><br>if self.check(Token::LBracket) {<br><br>let span = self.current_span();<br><br>self.advance();<br><br>let index = self.parse_expr()?;<br><br>self.expect(Token::RBracket)?;<br><br>expr = Expr::Index { list: Box::new(expr), index: Box::new(index), span };<br><br>} |

# **25.3 Built-in List Functions - Analysis and Runtime**

The 8 built-in list functions are handled as special cases in the analyzer and runtime, keyed on the function name. They are NOT fn declarations - they are built-in.

| **Function** | **Input type** | **Return type** |
| --- | --- | --- |
| len(list) | T\[\] | Number |
| min(list) | Number\[\] | Number - R004 if empty |
| max(list) | Number\[\] | Number - R004 if empty |
| sum(list) | Number\[\] | Number |
| append(list, val) | T\[\], T | T\[\] - returns new list |
| head(list) | T\[\] | T - R004 if empty |
| tail(list) | T\[\] | T\[\] - R004 if empty |
| at(list, i) | T\[\], Number | T - R004 if out of bounds |

| **Runtime - built-in list fn evaluation** |
| --- |
| // In eval_expr(), inside the Call handler, check for built-in names first:<br><br>Expr::Call { name, args, span } => {<br><br>// Handle built-in list functions before checking user fn_defs<br><br>match name.as_str() {<br><br>"len" => {<br><br>let list = self.eval_to_list(&args\[0\], ctx, \*span)?;<br><br>return Ok(Value::Number(list.len() as f64));<br><br>}<br><br>"min" => {<br><br>let list = self.eval_to_num_list(&args\[0\], ctx, \*span)?;<br><br>if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }<br><br>return Ok(Value::Number(list.iter().cloned().fold(f64::INFINITY, f64::min)));<br><br>}<br><br>"max" => {<br><br>let list = self.eval_to_num_list(&args\[0\], ctx, \*span)?;<br><br>if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }<br><br>return Ok(Value::Number(list.iter().cloned().fold(f64::NEG_INFINITY, f64::max)));<br><br>}<br><br>"sum" => {<br><br>let list = self.eval_to_num_list(&args\[0\], ctx, \*span)?;<br><br>return Ok(Value::Number(list.iter().sum()));<br><br>}<br><br>"append" => {<br><br>let mut list = self.eval_to_list(&args\[0\], ctx, \*span)?;<br><br>let val = self.eval_expr(&args\[1\], ctx)?;<br><br>list.push(val);<br><br>return Ok(Value::List(list));<br><br>}<br><br>"head" => {<br><br>let list = self.eval_to_list(&args\[0\], ctx, \*span)?;<br><br>if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }<br><br>return Ok(list\[0\].clone());<br><br>}<br><br>"tail" => {<br><br>let list = self.eval_to_list(&args\[0\], ctx, \*span)?;<br><br>if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }<br><br>return Ok(Value::List(list\[1..\].to_vec()));<br><br>}<br><br>"at" => {<br><br>let list = self.eval_to_list(&args\[0\], ctx, \*span)?;<br><br>let idx = self.eval_expr(&args\[1\], ctx)?.as_number()? as usize;<br><br>if idx >= list.len() {<br><br>return Err(RuntimeError::R004 { index: idx, len: list.len() });<br><br>}<br><br>return Ok(list\[idx\].clone());<br><br>}<br><br>// Fall through to user-defined fn_defs lookup<br><br>_ => {}<br><br>}<br><br>// ... user fn lookup continues here ...<br><br>} |

# **25.4 R004 - Now Active**

| **RuntimeError::R004 in lumina-runtime/src/lib.rs** |
| --- |
| // R004 was previously:<br><br>// R004 { index: usize, len: usize }, // list bounds (reserved)<br><br>// In v1.4, it is now fully active. The enum entry stays the same.<br><br>// Add the error message in the Display impl or Diagnostic conversion:<br><br>RuntimeError::R004 { index, len } => {<br><br>format!("list index {} out of bounds for list of length {}", index, len)<br><br>}<br><br>// Also add help text in help_for_code() in the analyzer:<br><br>"R004" => Some("check the list is non-empty and the index is within range".into()), |

# **25.5 Build Order**

**▶ Exact sequence**

Step 1: Add LuminaType::List and Expr::ListLiteral/Index to parser AST.

Step 2: Update parse_type() to handle \[\] suffix.

Step 3: Add list literal and index parsing to parse_primary() and parse_postfix().

Step 4: cargo build -p lumina-parser.

Step 5: Add type inference for List, ListLiteral, Index to lumina-analyzer.

Step 6: Add Value::List to lumina-runtime/src/value.rs.

Step 7: Add built-in fn dispatch in eval_expr() Call handler.

Step 8: Add eval_to_list() and eval_to_num_list() helpers to Evaluator.

Step 9: Activate R004 error message in RuntimeError Display.

Step 10: cargo test --workspace.

Step 11: Test: entity Fleet { readings: Number\[\] } and verify len(), append() work.

**Chapter 26**

**Go FFI Wrapper**

_Implementation guide - cgo wrapper over liblumina_ffi.so_

The Go wrapper is a pure file-addition task. It does not modify any Rust code. It uses cgo to call the existing C API in lumina.h. The Rust library must be compiled with cargo build --release -p lumina-ffi before the Go wrapper can be built or tested.

# **26.1 What Exists (Do Not Modify)**

The following files already exist and must NOT be changed for the Go wrapper:

| **Existing file** | **What it provides** |
| --- | --- |
| crates/lumina-ffi/lumina.h | C header with all 6 FFI function signatures |
| crates/lumina-ffi/lumina_py.py | Python ctypes wrapper - reference for Go impl |
| target/release/liblumina_ffi.so | Compiled shared library (Linux) |
| target/release/liblumina_ffi.dylib | Compiled shared library (macOS) |

# **26.2 Files To Create**

| **File path** | **Purpose** |
| --- | --- |
| crates/lumina-ffi/lumina_go/lumina.go | Go package - cgo wrapper |
| crates/lumina-ffi/lumina_go/lumina_test.go | Go test suite (5 tests) |
| crates/lumina-ffi/lumina_go/README.md | Build and usage instructions |

**⚠️ Do NOT create go.mod in lumina_go/**

lumina_go/ is NOT a Go module. It is a package imported by user projects.

Do not run "go mod init" inside lumina_go/.

Users of the wrapper create their own go.mod and import lumina_go/ by path.

# **26.3 crates/lumina-ffi/lumina_go/lumina.go - Complete File**

| **lumina.go** |
| --- |
| package lumina<br><br>/\*<br><br>#cgo linux LDFLAGS: -L../../../target/release -llumina_ffi -Wl,-rpath,../../../target/release<br><br>#cgo darwin LDFLAGS: -L../../../target/release -llumina_ffi<br><br>#include "../lumina.h"<br><br>#include &lt;stdlib.h&gt;<br><br>\*/<br><br>import "C"<br><br>import (<br><br>"encoding/json"<br><br>"errors"<br><br>"fmt"<br><br>"strings"<br><br>"unsafe"<br><br>)<br><br>// Runtime wraps the opaque LuminaRuntime C pointer.<br><br>// Always call Close() when done - it frees the C-side memory.<br><br>type Runtime struct {<br><br>ptr \*C.LuminaRuntime<br><br>}<br><br>// FromSource creates a Runtime from a Lumina source string.<br><br>// Returns an error if parsing or analysis fails.<br><br>func FromSource(source string) (\*Runtime, error) {<br><br>cs := C.CString(source)<br><br>defer C.free(unsafe.Pointer(cs))<br><br>ptr := C.lumina_create(cs)<br><br>if ptr == nil {<br><br>return nil, errors.New("lumina: failed to create runtime (parse/analyze error)")<br><br>}<br><br>return &Runtime{ptr: ptr}, nil<br><br>}<br><br>// ApplyEvent sets instance.field = value.<br><br>// valueJSON must be a valid JSON-encoded string:<br><br>// Number: "42" or "3.14"<br><br>// Text: "\\"hello\\"" (extra quotes - it is JSON)<br><br>// Boolean: "true" or "false"<br><br>// Returns the PropResult as a map, or an error on rollback.<br><br>func (r \*Runtime) ApplyEvent(instance, field, valueJSON string) (map\[string\]any, error) {<br><br>ci := C.CString(instance)<br><br>cf := C.CString(field)<br><br>cv := C.CString(valueJSON)<br><br>defer C.free(unsafe.Pointer(ci))<br><br>defer C.free(unsafe.Pointer(cf))<br><br>defer C.free(unsafe.Pointer(cv))<br><br>raw := C.lumina_apply_event(r.ptr, ci, cf, cv)<br><br>if raw == nil { return nil, errors.New("lumina: null response from apply_event") }<br><br>defer C.lumina_free_string(raw)<br><br>result := C.GoString(raw)<br><br>// Rollback is signaled by "ERROR:{...}"<br><br>if strings.HasPrefix(result, "ERROR:") {<br><br>return nil, fmt.Errorf("lumina rollback: %s", result\[6:\])<br><br>}<br><br>var out map\[string\]any<br><br>if err := json.Unmarshal(\[\]byte(result), &out); err != nil {<br><br>return nil, fmt.Errorf("lumina: cannot parse response: %w", err)<br><br>}<br><br>return out, nil<br><br>}<br><br>// ExportState returns the full runtime state as a parsed map.<br><br>// Keys: "instances" -> map of instance name -> { "fields": {...} }<br><br>func (r \*Runtime) ExportState() (map\[string\]any, error) {<br><br>raw := C.lumina_export_state(r.ptr)<br><br>if raw == nil { return nil, errors.New("lumina: null response from export_state") }<br><br>defer C.lumina_free_string(raw)<br><br>var out map\[string\]any<br><br>err := json.Unmarshal(\[\]byte(C.GoString(raw)), &out)<br><br>return out, err<br><br>}<br><br>// Tick advances all timers. Returns a slice of fired events.<br><br>// Call on a time.Ticker for every/for rules.<br><br>func (r \*Runtime) Tick() (\[\]map\[string\]any, error) {<br><br>raw := C.lumina_tick(r.ptr)<br><br>if raw == nil { return nil, errors.New("lumina: null response from tick") }<br><br>defer C.lumina_free_string(raw)<br><br>var events \[\]map\[string\]any<br><br>err := json.Unmarshal(\[\]byte(C.GoString(raw)), &events)<br><br>return events, err<br><br>}<br><br>// Close destroys the runtime and frees all C-side memory.<br><br>// After Close(), the Runtime must not be used.<br><br>func (r \*Runtime) Close() {<br><br>if r.ptr != nil {<br><br>C.lumina_destroy(r.ptr)<br><br>r.ptr = nil<br><br>}<br><br>} |

# **26.4 crates/lumina-ffi/lumina_go/lumina_test.go - Complete File**

| **lumina_test.go** |
| --- |
| package lumina_test<br><br>import (<br><br>"testing"<br><br>lumina "." // import the local package<br><br>)<br><br>const basicSource = \`<br><br>entity Moto {<br><br>battery: Number<br><br>isLowBattery := battery < 20<br><br>}<br><br>let moto1 = Moto { battery: 80 }<br><br>\`<br><br>func TestFromSource(t \*testing.T) {<br><br>rt, err := lumina.FromSource(basicSource)<br><br>if err != nil { t.Fatalf("FromSource failed: %v", err) }<br><br>defer rt.Close()<br><br>}<br><br>func TestFromSourceInvalid(t \*testing.T) {<br><br>\_, err := lumina.FromSource("this is not valid lumina @@@@")<br><br>if err == nil { t.Fatal("expected error for invalid source") }<br><br>}<br><br>func TestApplyEventAndExportState(t \*testing.T) {<br><br>rt, err := lumina.FromSource(basicSource)<br><br>if err != nil { t.Fatal(err) }<br><br>defer rt.Close()<br><br>\_, err = rt.ApplyEvent("moto1", "battery", "15")<br><br>if err != nil { t.Fatalf("ApplyEvent failed: %v", err) }<br><br>state, err := rt.ExportState()<br><br>if err != nil { t.Fatalf("ExportState failed: %v", err) }<br><br>instances := state\["instances"\].(map\[string\]any)<br><br>moto1 := instances\["moto1"\].(map\[string\]any)<br><br>fields := moto1\["fields"\].(map\[string\]any)<br><br>isLow := fields\["isLowBattery"\].(bool)<br><br>if !isLow { t.Error("expected isLowBattery=true after setting battery=15") }<br><br>}<br><br>func TestRollbackOnDerivedField(t \*testing.T) {<br><br>rt, err := lumina.FromSource(basicSource)<br><br>if err != nil { t.Fatal(err) }<br><br>defer rt.Close()<br><br>\_, err = rt.ApplyEvent("moto1", "isLowBattery", "true")<br><br>if err == nil { t.Fatal("expected rollback error R009 for derived field write") }<br><br>}<br><br>func TestTick(t \*testing.T) {<br><br>rt, err := lumina.FromSource(basicSource)<br><br>if err != nil { t.Fatal(err) }<br><br>defer rt.Close()<br><br>events, err := rt.Tick()<br><br>if err != nil { t.Fatalf("Tick failed: %v", err) }<br><br>// No every/for rules in basicSource - events should be empty<br><br>if len(events) != 0 { t.Errorf("expected 0 events, got %d", len(events)) }<br><br>} |

# **26.5 Build and Test Commands**

| **Full build and test sequence** |
| --- |
| \# Step 1: Build the Rust shared library (MUST do this first)<br><br>cargo build --release -p lumina-ffi<br><br>\# Step 2: Set library path so cgo can find liblumina_ffi.so<br><br>\# Linux:<br><br>export LD_LIBRARY_PATH=\$(pwd)/target/release:\$LD_LIBRARY_PATH<br><br>\# macOS:<br><br>export DYLD_LIBRARY_PATH=\$(pwd)/target/release:\$DYLD_LIBRARY_PATH<br><br>\# Step 3: Run Go tests<br><br>cd crates/lumina-ffi/lumina_go<br><br>go test ./... -v<br><br>\# Expected output:<br><br>\# --- PASS: TestFromSource (0.00s)<br><br>\# --- PASS: TestFromSourceInvalid (0.00s)<br><br>\# --- PASS: TestApplyEventAndExportState (0.00s)<br><br>\# --- PASS: TestRollbackOnDerivedField (0.00s)<br><br>\# --- PASS: TestTick (0.00s)<br><br>\# PASS<br><br>\# Step 4: Verify Rust tests still pass<br><br>cd ../../..<br><br>cargo test --workspace |

**🚫 CRITICAL: Memory Rules for Go Wrapper**

Every C.CString() call MUST have a matching defer C.free(unsafe.Pointer(cs)).

Every string returned by Rust MUST be freed with defer C.lumina_free_string(raw).

Never call C.free() on a Rust-returned string - use lumina_free_string() only.

Never call C.lumina_free_string() on a C.CString() you allocated - use C.free().

The Runtime.ptr must be set to nil after Close() to prevent double-free.

Never use a Runtime after Close() - add a nil check at the start of each method.

**Appendix**

**Complete v1.4 Build Sequence**

_The exact order to implement all 8 features - for Antigravity AI_

Implement features in this exact order. Each phase ends with cargo test --workspace passing. Do not start the next phase until the current one is green.

# **Phase 1 - Foundation (Chapter 19)**

**▶ lumina-diagnostics crate**

1\. Create crates/lumina-diagnostics/ with Cargo.toml, lib.rs, location.rs, render.rs.

2\. Add to workspace Cargo.toml members.

3\. cargo build -p lumina-diagnostics

4\. Add as dependency to lumina-analyzer.

5\. Update analyze() to return Vec&lt;Diagnostic&gt;.

6\. cargo test --workspace \[must show 40 passing\]

# **Phase 2 - REPL v2 (Chapter 20)**

**▶ Persistent session**

1\. Add Evaluator::new_empty() and describe_schema() to lumina-runtime.

2\. Create lumina-cli/src/repl.rs and lumina-cli/src/commands.rs.

3\. Replace the old REPL loop in main.rs.

4\. cargo test --workspace \[must show 40 passing\]

5\. Manual test: lumina repl + multi-line entity + :state

# **Phase 3 - VS Code Extension (Chapter 21)**

**▶ Grammar and snippets - no Rust changes**

1\. Create extensions/lumina-vscode/ with all files from Chapter 21.

2\. vsce package to verify the extension builds.

3\. Install locally and open a .lum file to verify highlighting.

4\. cargo test --workspace \[still 40 passing - no Rust changes\]

# **Phase 4 - fn keyword (Chapter 22)**

**▶ Pure functions**

1\. Add Token::Fn to lexer.

2\. Add FnDecl, FnParam, Expr::Call to parser AST.

3\. Add parse_fn_decl() and call parsing to parser.

4\. Add L011-L015 checks and fn_defs to analyzer.

5\. Add functions field, eval_expr_local(), Call eval to runtime.

6\. cargo test --workspace \[40+ passing\]

# **Phase 5 - import (Chapter 23)**

**▶ Module system**

1\. Add Token::Import to lexer.

2\. Add ImportDecl, Program::imports() to parser.

3\. Create lumina-cli/src/loader.rs with ModuleLoader.

4\. Update main.rs run/check to use ModuleLoader.

5\. Add allow_imports flag to analyze().

6\. cargo test --workspace \[40+ passing\]

# **Phase 6 - String Interpolation (Chapter 24)**

**▶ {expr} inside strings**

1\. Add interpolation tokens to lexer + expand_interpolations() pass.

2\. Add InterpolatedString, StringSegment to parser AST.

3\. Add parsing for InterpStringStart sequence.

4\. Add type inference in analyzer (always Text).

5\. Add eval_expr case in runtime.

6\. cargo test --workspace \[40+ passing\]

# **Phase 7 - List Types (Chapter 25)**

**▶ Number\[\], Text\[\], Boolean\[\]**

1\. Add LuminaType::List, Expr::ListLiteral, Expr::Index to parser.

2\. Update parse_type() for \[\] suffix, parse_primary() for list literals.

3\. Add type inference for lists in analyzer.

4\. Add Value::List to runtime.

5\. Add built-in fn dispatch (len/min/max/sum/append/head/tail/at).

6\. Activate R004 error message.

7\. cargo test --workspace \[40+ passing\]

# **Phase 8 - Go FFI (Chapter 26)**

**▶ Go wrapper - no Rust changes**

1\. cargo build --release -p lumina-ffi

2\. Create crates/lumina-ffi/lumina_go/lumina.go

3\. Create crates/lumina-ffi/lumina_go/lumina_test.go

4\. export LD_LIBRARY_PATH + go test ./... -v \[5 tests pass\]

5\. cargo test --workspace \[all Rust tests still pass\]

**✅ v1.4 Complete - Definition of Done**

cargo test --workspace shows all tests passing (40 from v1.3 + new v1.4 tests).

lumina run on a file with fn, import, string interpolation, and list fields works.

lumina repl supports multi-line entities and :state/:schema/:load/:save commands.

VS Code opens a .lum file with full syntax highlighting and snippets.

go test ./... in lumina_go/ shows 5 passing tests.

No v1.3 tests were removed or modified.