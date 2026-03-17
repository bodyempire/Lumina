**◈**

**LUMINA**

**v1.4 Deep Documentation**

_Functions · Modules · Enhanced Errors · REPL v2 · VS Code · String Interpolation · Lists · Go FFI_

_"Describe what is true. Lumina figures out what to do."_

2026 · Rust Runtime · Built on v1.3 · Chapters 19-26

_Designed and authored by Isaac Ishimwe_

**Chapter 19**

**Enhanced Error Messages**

_The lumina-diagnostics crate - Rust-style errors with source context_

v1.3 error messages report a code and a line number. v1.4 introduces the lumina-diagnostics crate, which renders errors the way Rust and Elm do: with the offending source line printed, a caret pointing to the exact column, and a plain-English suggestion. Every crate in the workspace is upgraded to produce Diagnostic values instead of raw strings.

# **19.1 What Changes**

Three things change across the workspace in v1.4:

First, a new crate lumina-diagnostics is added. It owns the Diagnostic struct, SourceLocation, and DiagnosticRenderer. Second, lumina-analyzer and lumina-runtime are upgraded to return Diagnostic values everywhere they previously returned AnalyzerError or RuntimeError strings. Third, the CLI output is piped through DiagnosticRenderer::render() so users see formatted output instead of raw codes.

| **Before (v1.3) vs After (v1.4)** |
| --- |
| \-- v1.3 output<br><br>error L003: derived field cycle detected at line 4<br><br>\-- v1.4 output<br><br>error\[L003\]: derived field cycle detected<br><br>\--> fleet.lum:4:3<br><br>\|<br><br>4 \| speed := distance / time<br><br>\| ^^^^^ this derived field depends on itself (transitively)<br><br>\|<br><br>\= help: break the cycle by making one field stored |

# **19.2 New Crate: lumina-diagnostics**

Add a new crate at crates/lumina-diagnostics/. It has no dependencies on any other Lumina crate - all other crates depend on it.

| **lumina-diagnostics public API** |
| --- |
| // crates/lumina-diagnostics/src/lib.rs<br><br>pub struct SourceLocation {<br><br>pub file: String,<br><br>pub line: u32,<br><br>pub col: u32,<br><br>pub len: u32, // highlight length in chars<br><br>}<br><br>pub struct Diagnostic {<br><br>pub code: String, // "L003" or "R006"<br><br>pub message: String, // short message<br><br>pub location: SourceLocation,<br><br>pub source_line: String, // the raw source text at that line<br><br>pub help: Option&lt;String&gt;, // "help: ..." suggestion<br><br>}<br><br>pub struct DiagnosticRenderer;<br><br>impl DiagnosticRenderer {<br><br>// Renders to the multi-line Rust-style string<br><br>pub fn render(diag: &Diagnostic) -> String<br><br>// Renders multiple diagnostics separated by blank lines<br><br>pub fn render_all(diags: &\[Diagnostic\]) -> String<br><br>} |

# **19.3 Implementing render()**

The renderer produces exactly four lines: the error header, the file location arrow, the source context block with line number gutter, and the help line if present.

| **DiagnosticRenderer::render() implementation** |
| --- |
| pub fn render(diag: &Diagnostic) -> String {<br><br>let mut out = String::new();<br><br>// Line 1: error\[CODE\]: message<br><br>out.push_str(&format!("error\[{}\]: {}\\n", diag.code, diag.message));<br><br>// Line 2: --> file:line:col<br><br>out.push_str(&format!(" --> {}:{}:{}\\n", diag.location.file,<br><br>diag.location.line, diag.location.col));<br><br>// Line 3: gutter + source line<br><br>let gutter = diag.location.line.to_string();<br><br>let pad = " ".repeat(gutter.len());<br><br>out.push_str(&format!("{} \|\\n", pad));<br><br>out.push_str(&format!("{} \| {}\\n", gutter, diag.source_line));<br><br>// Line 4: caret under the offending token<br><br>let spaces = " ".repeat(diag.location.col as usize - 1);<br><br>let carets = "^".repeat(diag.location.len.max(1) as usize);<br><br>out.push_str(&format!("{} \| {}{}\\n", pad, spaces, carets));<br><br>out.push_str(&format!("{} \|\\n", pad));<br><br>// Optional help<br><br>if let Some(help) = &diag.help {<br><br>out.push_str(&format!(" = help: {}\\n", help));<br><br>}<br><br>out<br><br>} |

# **19.4 Upgrading lumina-analyzer**

The analyzer now takes the original source string as an extra parameter so it can extract the source line for each error. Add a source: &str parameter to analyze() and store it in Analyzer.

| **Updated analyze() signature** |
| --- |
| // Before (v1.3)<br><br>pub fn analyze(program: &Program) -> Vec&lt;AnalyzerError&gt;<br><br>// After (v1.4)<br><br>pub fn analyze(program: &Program, source: &str) -> Vec&lt;Diagnostic&gt;<br><br>// Inside the analyzer, replace:<br><br>errors.push(AnalyzerError { code: "L003".into(), message: ..., span })<br><br>// With:<br><br>errors.push(Diagnostic {<br><br>code: "L003".into(),<br><br>message: "derived field cycle detected".into(),<br><br>location: SourceLocation::from_span(&span, source, filename),<br><br>source_line: extract_line(source, span.line),<br><br>help: Some("break the cycle by making one field stored".into()),<br><br>}) |

# **19.5 Help Text for Every Error Code**

| **Code** | **Short message** | **Help text** |
| --- | --- | --- |
| L001 | duplicate entity name | rename one of the entity declarations |
| L002 | unknown entity referenced | check spelling or add the entity declaration |
| L003 | derived field cycle | break the cycle by making one field stored |
| L004 | type mismatch | verify the field type and the literal type match |
| L005 | unknown field on entity | check field spelling or add the field declaration |
| L006 | invalid @range metadata | @range only applies to Number fields; ensure min < max |
| L007 | rule trigger entity unknown | check that the entity name in the when clause is correct |
| L008 | action targets unknown instance | add a let binding for the instance before using it |
| L009 | duplicate instance name | instance names must be globally unique in the program |
| L010 | invalid @affects metadata | @affects only applies to stored fields |
| R006 | @range violation | clamp the value to \[@range min, @range max\] before writing |
| R009 | derived field write attempt | only stored fields (field: Type) can be set externally |

**📌 NOTE: Backward Compatibility**

The FFI (lumina-ffi) and WASM (lumina-wasm) surface errors as JSON strings.

The JSON now includes a "diagnostic" key with the rendered string:

{ "success": false, "diagnostic": "error\[L003\]: ...", "code": "L003" }

Callers that only check result.startsWith("ERROR:") continue to work unchanged.

**Chapter 20**

**REPL v2**

_Persistent state, multi-line input, and inspector commands_

The v1.3 REPL rebuilds the entire Evaluator from scratch on every line. v1.4 fixes this by maintaining a single Evaluator across all inputs and detecting multi-line constructs by brace depth. A set of inspector commands (:state, :schema, :load, :save, :clear) makes the REPL useful for interactive development.

# **20.1 The Core Problem with v1.3 REPL**

| **v1.3 REPL - what breaks** |
| --- |
| \> entity Moto { battery: Number }<br><br>ok<br><br>\> let moto1 = Moto { battery: 80 }<br><br>error: unknown entity "Moto" -- Evaluator was rebuilt, forgot Moto |

The fix is simple: instead of rebuilding the Evaluator each loop iteration, maintain one Evaluator for the lifetime of the REPL session and append each parsed statement to it.

# **20.2 Architecture**

| **REPL v2 state machine** |
| --- |
| struct ReplSession {<br><br>evaluator: Evaluator,<br><br>source_accum: String, // accumulated source for multi-line input<br><br>brace_depth: i32, // tracks open { to detect complete constructs<br><br>history: Vec&lt;String&gt;,<br><br>}<br><br>impl ReplSession {<br><br>pub fn new() -> Self<br><br>pub fn feed(&mut self, line: &str) -> ReplResult<br><br>pub fn run_command(&mut self, cmd: &str) -> String<br><br>}<br><br>pub enum ReplResult {<br><br>NeedMore, // multi-line construct incomplete - show "..." prompt<br><br>Ok(String), // success - optional output to print<br><br>Error(Diagnostic), // error - render and print, then continue<br><br>} |

# **20.3 Multi-line Detection**

When the user types a line, brace_depth is updated. If brace_depth > 0 after the line, the prompt changes to "..." and more input is accumulated. When brace_depth returns to 0, the accumulated source is parsed and executed as a unit.

| **Brace depth tracking in feed()** |
| --- |
| pub fn feed(&mut self, line: &str) -> ReplResult {<br><br>// Track brace depth<br><br>for ch in line.chars() {<br><br>match ch {<br><br>'{' => self.brace_depth += 1,<br><br>'}' => self.brace_depth -= 1,<br><br>_=> {}<br><br>}<br><br>}<br><br>self.source_accum.push_str(line);<br><br>self.source_accum.push('\\n');<br><br>if self.brace_depth > 0 {<br><br>return ReplResult::NeedMore; // show "..." prompt<br><br>}<br><br>// Complete construct - parse and execute<br><br>let source = self.source_accum.drain(..).collect::&lt;String&gt;();<br><br>match lumina_parser::parse(&source) {<br><br>Err(e) => ReplResult::Error(e.into()),<br><br>Ok(program) => self.exec_program(program),<br><br>}<br><br>} |

# **20.4 Inspector Commands**

| **Command** | **What it does** | **Example output** |
| --- | --- | --- |
| :state | Print current state as pretty JSON | { "instances": { "moto1": { ... } } } |
| :schema | Print all declared entities and their fields | entity Moto { battery: Number, isLow := ... } |
| :load &lt;file&gt; | Load and execute a .lum file into the session | Loaded fleet.lum - 3 entities, 2 rules |
| :save &lt;file&gt; | Save accumulated session source to a .lum file | Saved to session.lum |
| :clear | Reset the session - new Evaluator, empty state | Session cleared |
| :help | List all commands | Available commands: :state :schema ... |
| :quit | Exit the REPL |     |

# **20.5 Updated CLI Entry Point**

| **lumina-cli REPL v2 main loop** |
| --- |
| // crates/lumina-cli/src/repl.rs<br><br>pub fn run_repl() {<br><br>let mut session = ReplSession::new();<br><br>let stdin = std::io::stdin();<br><br>loop {<br><br>let prompt = if session.brace_depth > 0 { "... " } else { ">>> " };<br><br>print!("{}", prompt);<br><br>std::io::Write::flush(&mut std::io::stdout()).ok();<br><br>let mut line = String::new();<br><br>if stdin.lock().read_line(&mut line).unwrap() == 0 { break; }<br><br>let line = line.trim_end_matches('\\n');<br><br>if line.starts_with(':') {<br><br>println!("{}", session.run_command(line));<br><br>continue;<br><br>}<br><br>match session.feed(line) {<br><br>ReplResult::NeedMore => {}<br><br>ReplResult::Ok(output) => { if !output.is_empty() { println!("{}", output); } }<br><br>ReplResult::Error(diag) => {<br><br>eprintln!("{}", DiagnosticRenderer::render(&diag));<br><br>}<br><br>}<br><br>}<br><br>} |

**📌 NOTE: State Persistence Across :load**

When :load is called, the loaded file is executed on top of the existing session state.

Entities and instances from prior input remain visible.

If the file re-declares an existing entity name, L001 (duplicate entity) is raised.

Use :clear before :load to start from a clean state.

**Chapter 21**

**VS Code Extension**

_Syntax highlighting and basic language support for .lum files_

The VS Code extension adds first-class .lum file support: syntax highlighting via a TextMate grammar, snippet completions for entity/rule boilerplate, and bracket matching. The extension is a standalone package in extensions/lumina-vscode/ and does not require a language server in v1.4.

# **21.1 Extension Structure**

| **Extension directory layout** |
| --- |
| extensions/lumina-vscode/<br><br>package.json # extension manifest<br><br>syntaxes/<br><br>lumina.tmLanguage.json # TextMate grammar<br><br>snippets/<br><br>lumina.json # code snippets<br><br>language-configuration.json # bracket matching, comments<br><br>README.md |

# **21.2 package.json Manifest**

| **package.json** |
| --- |
| {<br><br>"name": "lumina-language",<br><br>"displayName": "Lumina",<br><br>"description": "Lumina language support for VS Code",<br><br>"version": "0.1.0",<br><br>"engines": { "vscode": "^1.75.0" },<br><br>"categories": \["Programming Languages"\],<br><br>"contributes": {<br><br>"languages": \[{<br><br>"id": "lumina",<br><br>"aliases": \["Lumina", "lum"\],<br><br>"extensions": \[".lum"\],<br><br>"configuration": "./language-configuration.json"<br><br>}\],<br><br>"grammars": \[{<br><br>"language": "lumina",<br><br>"scopeName": "source.lumina",<br><br>"path": "./syntaxes/lumina.tmLanguage.json"<br><br>}\],<br><br>"snippets": \[{<br><br>"language": "lumina",<br><br>"path": "./snippets/lumina.json"<br><br>}\]<br><br>}<br><br>} |

# **21.3 TextMate Grammar Scopes**

| **Token type** | **TextMate scope** | **VS Code color** |
| --- | --- | --- |
| Keywords (entity rule when every for) | keyword.control.lumina | Purple |
| Type names (Number Text Boolean) | storage.type.lumina | Blue |
| Operators (:= := -> @) | keyword.operator.lumina | Cyan |
| String literals | string.quoted.double.lumina | Orange |
| Number literals | constant.numeric.lumina | Green |
| Boolean literals (true false) | constant.language.lumina | Blue |
| Comments (-- ...) | comment.line.double-dash.lumina | Grey |
| Entity names | entity.name.type.lumina | Yellow |
| Field names | variable.other.lumina | White |
| Derived operator := | keyword.operator.derive.lumina | Magenta |

# **21.4 Snippets**

| **lumina.json - code snippets** |
| --- |
| {<br><br>"Entity": {<br><br>"prefix": "entity",<br><br>"body": \[<br><br>"entity \${1:Name} {",<br><br>" \${2:field}: \${3:Number}",<br><br>" \${4:derived} := \${5:expr}",<br><br>"}"<br><br>\],<br><br>"description": "Declare a new entity"<br><br>},<br><br>"Rule when becomes": {<br><br>"prefix": "rule",<br><br>"body": \[<br><br>"rule \${1:Name} when \${2:Entity}.\${3:field} becomes \${4:true} {",<br><br>" \${5:show \\"fired\\"}",<br><br>"}"<br><br>\]<br><br>},<br><br>"Rule every": {<br><br>"prefix": "every",<br><br>"body": \[<br><br>"rule \${1:Name} every \${2:30}s {",<br><br>" \${3:show \\"tick\\"}",<br><br>"}"<br><br>\]<br><br>},<br><br>"Let binding": {<br><br>"prefix": "let",<br><br>"body": \["let \${1:name} = \${2:Entity} { \${3:field}: \${4:value} }"\]<br><br>}<br><br>} |

# **21.5 Installing Locally**

| **Install and test the extension** |
| --- |
| \# Package the extension<br><br>cd extensions/lumina-vscode<br><br>npm install -g @vscode/vsce<br><br>vsce package<br><br>\# Produces: lumina-language-0.1.0.vsix<br><br>\# Install in VS Code<br><br>code --install-extension lumina-language-0.1.0.vsix<br><br>\# Or: open VS Code → Extensions → "..." → Install from VSIX |

**📌 NOTE: v1.4 Scope - No Language Server**

The v1.4 extension provides syntax highlighting and snippets only.

There is no hover documentation, go-to-definition, or inline error squiggles.

These features require a Language Server Protocol implementation.

That is planned for v1.5 as a dedicated chapter.

**Chapter 22**

**Pure Functions**

_The fn keyword - reusable, side-effect-free expressions_

v1.4 introduces pure functions with the fn keyword. Functions are expressions - they take typed parameters and return a single value. They have no access to entity state or instances. They are called from derived fields and rule conditions. Functions cannot call rules or trigger side effects.

# **22.1 Syntax**

| **fn syntax** |
| --- |
| \-- Declaration<br><br>fn clamp(value: Number, min: Number, max: Number) -> Number {<br><br>if value < min then min<br><br>else if value > max then max<br><br>else value<br><br>}<br><br>\-- Used in a derived field<br><br>entity Moto {<br><br>battery: Number<br><br>safeBattery := clamp(battery, 0, 100)<br><br>}<br><br>\-- Used in a rule condition<br><br>fn isCritical(b: Number) -> Boolean { b < 5 }<br><br>rule Shutdown when Moto.battery becomes isCritical(Moto.battery) {<br><br>show "critical shutdown"<br><br>}<br><br>\-- String helper<br><br>fn greet(name: Text) -> Text { "Hello, " + name } |

# **22.2 Grammar Changes**

Functions are top-level statements. Add a FnDecl variant to the Statement enum in the parser.

| **AST additions - lumina-parser/src/ast.rs** |
| --- |
| // New Statement variant<br><br>pub enum Statement {<br><br>Entity(EntityDecl),<br><br>ExternalEntity(ExternalEntityDecl),<br><br>Let(LetStmt),<br><br>Rule(RuleDecl),<br><br>Action(Action),<br><br>Fn(FnDecl), // NEW<br><br>}<br><br>pub struct FnDecl {<br><br>pub name: String,<br><br>pub params: Vec&lt;FnParam&gt;,<br><br>pub returns: LuminaType,<br><br>pub body: Expr, // single expression - no statements in body<br><br>pub span: Span,<br><br>}<br><br>pub struct FnParam {<br><br>pub name: String,<br><br>pub type_: LuminaType,<br><br>} |

# **22.3 Analysis Rules**

| **Rule** | **Error code** | **Description** |
| --- | --- | --- |
| Duplicate fn name | L011 | Two fn declarations share the same name |
| Unknown fn called | L012 | A call site references a fn that was not declared |
| Argument count mismatch | L013 | Call passes wrong number of arguments |
| Argument type mismatch | L004 | Call argument type does not match parameter type |
| Return type mismatch | L014 | Body expression type does not match -> return type |
| fn calls entity fields | L015 | fn bodies cannot reference entity instances or fields |

# **22.4 Evaluation**

Functions are stored in a HashMap&lt;String, FnDecl&gt; on the Evaluator. When eval_expr() encounters a Call expression, it looks up the fn, binds the arguments to parameter names in a local scope, and evaluates the body expression. There is no recursion limit separate from the main depth counter.

| **Evaluating fn calls in eval_expr()** |
| --- |
| Expr::Call { name, args } => {<br><br>let decl = self.functions.get(name)<br><br>.ok_or(RuntimeError::UnknownFn { name: name.clone() })?;<br><br>// Evaluate argument expressions<br><br>let arg_vals: Vec&lt;Value&gt; = args.iter()<br><br>.map(\|a\| self.eval_expr(a, ctx))<br><br>.collect::&lt;Result<\_, \_&gt;>()?;<br><br>// Build local scope<br><br>let mut local = HashMap::new();<br><br>for (param, val) in decl.params.iter().zip(arg_vals) {<br><br>local.insert(param.name.clone(), val);<br><br>}<br><br>// Evaluate body with local scope (no entity access)<br><br>self.eval_expr_in_scope(&decl.body, &local)<br><br>} |

**⚠️ DO NOT BREAK: fn bodies are pure**

fn bodies must not access self.store (the entity store).

eval_expr_in_scope() takes only a local HashMap - no ctx parameter.

Any attempt to access entity state from a fn body must raise L015 at analysis time.

This purity guarantee is what makes functions safe to call from derived fields.

**Chapter 23**

**Modules**

_The import keyword - splitting programs across files_

v1.4 introduces modules. A .lum file can import another .lum file with the import keyword. All entity declarations, fn declarations, and let bindings from the imported file become visible in the importing file. Circular imports are detected and reported as a compile-time error.

# **23.1 Syntax**

| **import syntax** |
| --- |
| \-- File: shared/moto.lum<br><br>entity Moto {<br><br>battery: Number<br><br>isLowBattery := battery < 20<br><br>}<br><br>fn clamp(v: Number, lo: Number, hi: Number) -> Number {<br><br>if v &lt; lo then lo else if v &gt; hi then hi else v<br><br>}<br><br>\-- File: fleet_os.lum<br><br>import "shared/moto.lum"<br><br>let moto1 = Moto { battery: 80 }<br><br>rule AlertLow when Moto.isLowBattery becomes true {<br><br>show "ALERT: " + moto1.battery<br><br>} |

# **23.2 Resolution Rules**

Import paths are resolved relative to the importing file. The CLI passes the file's directory as the resolution root. The WASM build does not support import (no filesystem access) - the playground is single-file only.

| **Scenario** | **Behavior** | **Error if violated** |
| --- | --- | --- |
| Relative path | import "shared/moto.lum" resolves from current file dir | -   |
| Circular import | A imports B, B imports A (directly or transitively) | L016: circular import |
| File not found | The path does not exist on disk | L017: file not found |
| Duplicate declaration | Imported file declares entity that already exists | L001: duplicate entity |
| WASM / playground | import is a parse error in single-file mode | L018: import not supported |

# **23.3 Implementation: Module Loader**

Add a ModuleLoader struct to lumina-cli. It owns the import graph and is responsible for loading, parsing, and analyzing all files before handing a merged Program to the Evaluator.

| **ModuleLoader - lumina-cli/src/loader.rs** |
| --- |
| pub struct ModuleLoader {<br><br>loaded: HashMap&lt;PathBuf, Program&gt;, // path -> parsed AST<br><br>order: Vec&lt;PathBuf&gt;, // topological load order<br><br>}<br><br>impl ModuleLoader {<br><br>pub fn load_entry(entry: &Path) -> Result&lt;Program, Vec<Diagnostic&gt;> {<br><br>let mut loader = Self::new();<br><br>loader.load_recursive(entry)?;<br><br>// Merge all programs in topo order into one flat Program<br><br>Ok(loader.merge())<br><br>}<br><br>fn load_recursive(&mut self, path: &Path) -> Result&lt;(), Vec<Diagnostic&gt;> {<br><br>if self.loaded.contains_key(path) { return Ok(()); } // already loaded<br><br>if self.currently_loading.contains(path) {<br><br>return Err(vec!\[circular_import_error(path)\]); // cycle<br><br>}<br><br>self.currently_loading.insert(path.to_path_buf());<br><br>let source = fs::read_to_string(path)?;<br><br>let program = parse(&source)?;<br><br>for import in program.imports() {<br><br>let dep = path.parent().unwrap().join(&import.path);<br><br>self.load_recursive(&dep)?;<br><br>}<br><br>self.loaded.insert(path.to_path_buf(), program);<br><br>self.order.push(path.to_path_buf());<br><br>Ok(())<br><br>}<br><br>} |

# **23.4 AST Changes**

| **Import statement in the AST** |
| --- |
| // lumina-parser/src/ast.rs<br><br>pub enum Statement {<br><br>// ... existing variants ...<br><br>Import(ImportDecl), // NEW<br><br>}<br><br>pub struct ImportDecl {<br><br>pub path: String, // raw path string from source<br><br>pub span: Span,<br><br>}<br><br>// Program now exposes a helper<br><br>impl Program {<br><br>pub fn imports(&self) -> impl Iterator&lt;Item = &ImportDecl&gt; {<br><br>self.statements.iter().filter_map(\|s\| match s {<br><br>Statement::Import(i) => Some(i),<br><br>_ => None,<br><br>})<br><br>}<br><br>} |

**📌 NOTE: WASM and import**

The WASM runtime and browser playground are single-file only.

import statements in WASM mode produce error L018 at analysis time.

The lumina-wasm crate passes a no_filesystem flag to the analyzer to enforce this.

**Chapter 24**

**String Interpolation**

_Embedding expressions directly inside Text literals_

v1.4 adds string interpolation using the {expr} syntax inside double-quoted strings. Any expression that evaluates to a Number, Text, or Boolean can be embedded. The interpolated string is evaluated at the point it appears and produces a Text value.

# **24.1 Syntax**

| **String interpolation examples** |
| --- |
| \-- Basic field interpolation<br><br>show "Battery level: {moto1.battery}%"<br><br>\-- Boolean field<br><br>show "Low battery: {moto1.isLowBattery}"<br><br>\-- Arithmetic expression<br><br>show "Half charge: {moto1.battery / 2}"<br><br>\-- In a derived field<br><br>entity Moto {<br><br>battery: Number<br><br>label: Text<br><br>summary := "Moto \[{label}\] battery={battery}%"<br><br>}<br><br>\-- Nested fn call<br><br>show "Clamped: {clamp(moto1.battery, 0, 100)}"<br><br>\-- Escaping a literal brace<br><br>show "Use {{ and }} for literal braces" |

# **24.2 Lexer Changes**

The lexer needs to tokenize interpolated strings as a sequence: StringStart, then alternating StringPart / InterpolatedExpr segments, then StringEnd. This is a mode-switching lexer addition.

| **New token variants for string interpolation** |
| --- |
| // lumina-lexer/src/token.rs - add to Token enum<br><br>pub enum Token {<br><br>// ... existing ...<br><br>// String interpolation tokens<br><br>StringStart, // opening "<br><br>StringPart(String), // literal text segment<br><br>InterpStart, // {<br><br>InterpEnd, // }<br><br>StringEnd, // closing "<br><br>}<br><br>// Simple strings (no {}) still produce StringLit(String) - no change.<br><br>// Only strings containing { ... } produce the new token sequence. |

# **24.3 AST Changes**

| **InterpolatedString expression node** |
| --- |
| // lumina-parser/src/ast.rs<br><br>pub enum Expr {<br><br>// ... existing ...<br><br>InterpolatedString(Vec&lt;StringSegment&gt;), // NEW<br><br>}<br><br>pub enum StringSegment {<br><br>Literal(String), // plain text portion<br><br>Expr(Box&lt;Expr&gt;), // {expr} portion<br><br>} |

# **24.4 Evaluation**

| **eval_expr for InterpolatedString** |
| --- |
| Expr::InterpolatedString(segments) => {<br><br>let mut result = String::new();<br><br>for seg in segments {<br><br>match seg {<br><br>StringSegment::Literal(s) => result.push_str(s),<br><br>StringSegment::Expr(e) => {<br><br>let val = self.eval_expr(e, ctx)?;<br><br>result.push_str(&val.to_string()); // Number/Boolean -> string<br><br>}<br><br>}<br><br>}<br><br>Ok(Value::Text(result))<br><br>} |

**📌 NOTE: No nested interpolation**

Interpolated expressions cannot themselves contain interpolated strings.

"outer {"inner {x}"}" is a parse error - L019: nested interpolation.

Use fn declarations to build complex strings step by step instead.

**Chapter 25**

**List Types**

_Number\[\], Text\[\], Boolean\[\] - ordered collections of values_

v1.4 introduces list types. A stored field can be declared as Number\[\], Text\[\], or Boolean\[\]. Lists support indexing, length, append, and iteration in derived fields. R004, previously reserved for list bounds errors, is now active. Lists are immutable from outside the runtime - they can only be mutated through rule actions.

# **25.1 Syntax**

| **List field declarations and usage** |
| --- |
| entity Fleet {<br><br>batteryReadings: Number\[\] -- stored list field<br><br>labels: Text\[\]<br><br>count := len(batteryReadings) -- built-in fn: list length<br><br>lowest := min(batteryReadings) -- built-in fn: list min<br><br>highest := max(batteryReadings) -- built-in fn: list max<br><br>}<br><br>\-- Let binding with initial list<br><br>let fleet1 = Fleet {<br><br>batteryReadings: \[80, 60, 40, 20\],<br><br>labels: \["north", "south", "east", "west"\]<br><br>}<br><br>\-- Index access (0-based) - R004 if out of bounds<br><br>show fleet1.batteryReadings\[0\] -- 80<br><br>\-- Append via rule action<br><br>rule Record every 10s {<br><br>update fleet1.batteryReadings to append(fleet1.batteryReadings, moto1.battery)<br><br>} |

# **25.2 Built-in List Functions**

| **Function** | **Signature** | **Description** |
| --- | --- | --- |
| len | len(list: T\[\]) -> Number | Number of elements in the list |
| min | min(list: Number\[\]) -> Number | Minimum value - R004 if empty |
| max | max(list: Number\[\]) -> Number | Maximum value - R004 if empty |
| sum | sum(list: Number\[\]) -> Number | Sum of all values |
| append | append(list: T\[\], value: T) -> T\[\] | Returns new list with value added at end |
| head | head(list: T\[\]) -> T | First element - R004 if empty |
| tail | tail(list: T\[\]) -> T\[\] | All elements except first - R004 if empty |
| at  | at(list: T\[\], i: Number) -> T | Element at index i - R004 if out of bounds |

# **25.3 Type System Changes**

| **LuminaType extended for lists** |
| --- |
| // lumina-parser/src/ast.rs<br><br>pub enum LuminaType {<br><br>Number,<br><br>Text,<br><br>Boolean,<br><br>List(Box&lt;LuminaType&gt;), // NEW - Number\[\], Text\[\], Boolean\[\]<br><br>}<br><br>// lumina-runtime/src/value.rs<br><br>pub enum Value {<br><br>Number(f64),<br><br>Text(String),<br><br>Boolean(bool),<br><br>List(Vec&lt;Value&gt;), // NEW<br><br>Null,<br><br>} |

# **25.4 R004 - List Bounds Error**

R004 was reserved in v1.3. It is now active. Any index access, head(), tail(), min(), or max() on an empty or out-of-bounds list raises R004 and triggers the standard snapshot rollback.

| **R004 in practice** |
| --- |
| \-- Accessing index 5 on a 4-element list<br><br>show fleet1.batteryReadings\[5\]<br><br>\-- R004: index 5 out of bounds for list of length 4<br><br>\-- Calling min() on empty list<br><br>show min(\[\]) -- R004: min() called on empty list<br><br>\-- Guard pattern<br><br>minBattery := if len(batteryReadings) > 0 then min(batteryReadings) else 0 |

**⚠️ DO NOT BREAK: Lists are Values, not References**

append() does not mutate the existing list - it returns a new list.

"update fleet1.batteryReadings to append(...)" creates and stores a new list.

Two derived fields can both reference the same list field safely.

Never store a mutable reference to a list value across tick boundaries.

**Chapter 26**

**Go FFI Wrapper**

_Using Lumina from Go via cgo and liblumina_ffi.so_

v1.4 ships a Go wrapper for the Lumina runtime alongside the Python wrapper. It uses cgo to call liblumina_ffi.so directly. The Go API mirrors the Python API exactly: LuminaRuntime.FromSource(), ApplyEvent(), ExportState(), Tick(). No new FFI functions are needed - the existing C API is sufficient.

# **26.1 File Location**

| **Go wrapper location** |
| --- |
| crates/lumina-ffi/<br><br>lumina.h # existing C header<br><br>lumina_py.py # existing Python wrapper<br><br>lumina_go/ # NEW<br><br>lumina.go # Go package: package lumina<br><br>lumina_test.go # Go tests<br><br>README.md |

# **26.2 lumina.go - Full Implementation**

| **lumina.go** |
| --- |
| package lumina<br><br>/\*<br><br>#cgo LDFLAGS: -llumina_ffi -L../../../target/release<br><br>#include "../lumina.h"<br><br>#include &lt;stdlib.h&gt;<br><br>\*/<br><br>import "C"<br><br>import (<br><br>"encoding/json"<br><br>"errors"<br><br>"unsafe"<br><br>)<br><br>type Runtime struct {<br><br>ptr \*C.LuminaRuntime<br><br>}<br><br>// FromSource creates a new runtime from a Lumina source string.<br><br>func FromSource(source string) (\*Runtime, error) {<br><br>cs := C.CString(source)<br><br>defer C.free(unsafe.Pointer(cs))<br><br>ptr := C.lumina_create(cs)<br><br>if ptr == nil {<br><br>return nil, errors.New("lumina: failed to create runtime")<br><br>}<br><br>return &Runtime{ptr: ptr}, nil<br><br>}<br><br>// ApplyEvent sets a field value on an instance.<br><br>// value must be a JSON-encoded string: "42", "true", "\\"hello\\""<br><br>func (r \*Runtime) ApplyEvent(instance, field, valueJSON string) (map\[string\]any, error) {<br><br>ci := C.CString(instance)<br><br>cf := C.CString(field)<br><br>cv := C.CString(valueJSON)<br><br>defer C.free(unsafe.Pointer(ci))<br><br>defer C.free(unsafe.Pointer(cf))<br><br>defer C.free(unsafe.Pointer(cv))<br><br>raw := C.lumina_apply_event(r.ptr, ci, cf, cv)<br><br>defer C.lumina_free_string(raw)<br><br>result := C.GoString(raw)<br><br>if len(result) > 6 && result\[:6\] == "ERROR:" {<br><br>return nil, errors.New(result\[6:\])<br><br>}<br><br>var out map\[string\]any<br><br>json.Unmarshal(\[\]byte(result), &out)<br><br>return out, nil<br><br>}<br><br>// ExportState returns the current runtime state as a map.<br><br>func (r \*Runtime) ExportState() (map\[string\]any, error) {<br><br>raw := C.lumina_export_state(r.ptr)<br><br>defer C.lumina_free_string(raw)<br><br>var out map\[string\]any<br><br>err := json.Unmarshal(\[\]byte(C.GoString(raw)), &out)<br><br>return out, err<br><br>}<br><br>// Tick advances all timers.<br><br>func (r \*Runtime) Tick() (\[\]map\[string\]any, error) {<br><br>raw := C.lumina_tick(r.ptr)<br><br>defer C.lumina_free_string(raw)<br><br>var events \[\]map\[string\]any<br><br>err := json.Unmarshal(\[\]byte(C.GoString(raw)), &events)<br><br>return events, err<br><br>}<br><br>// Close destroys the runtime and frees memory.<br><br>func (r \*Runtime) Close() {<br><br>C.lumina_destroy(r.ptr)<br><br>r.ptr = nil<br><br>} |

# **26.3 Example Usage**

| **Using the Go wrapper** |
| --- |
| package main<br><br>import (<br><br>"fmt"<br><br>"log"<br><br>lumina "path/to/crates/lumina-ffi/lumina_go"<br><br>)<br><br>func main() {<br><br>rt, err := lumina.FromSource(\`<br><br>entity Moto {<br><br>battery: Number<br><br>isLowBattery := battery < 20<br><br>}<br><br>let moto1 = Moto { battery: 80 }<br><br>\`)<br><br>if err != nil { log.Fatal(err) }<br><br>defer rt.Close()<br><br>\_, err = rt.ApplyEvent("moto1", "battery", "15")<br><br>if err != nil { log.Fatal(err) }<br><br>state,_ := rt.ExportState()<br><br>instances := state\["instances"\].(map\[string\]any)<br><br>moto1 := instances\["moto1"\].(map\[string\]any)<br><br>fields := moto1\["fields"\].(map\[string\]any)<br><br>fmt.Println("isLowBattery:", fields\["isLowBattery"\]) // true<br><br>} |

# **26.4 Go Test Suite**

| **lumina_test.go** |
| --- |
| package lumina_test<br><br>import "testing"<br><br>import lumina "."<br><br>func TestFromSource(t \*testing.T) {<br><br>rt, err := lumina.FromSource(\`entity Moto { battery: Number }\`)<br><br>if err != nil { t.Fatal(err) }<br><br>defer rt.Close()<br><br>}<br><br>func TestApplyEvent(t \*testing.T) {<br><br>rt,_ := lumina.FromSource(\`<br><br>entity Moto { battery: Number; isLow := battery < 20 }<br><br>let m = Moto { battery: 80 }<br><br>\`)<br><br>defer rt.Close()<br><br>\_, err := rt.ApplyEvent("m", "battery", "10")<br><br>if err != nil { t.Fatal(err) }<br><br>state, _:= rt.ExportState()<br><br>// verify isLow == true in state<br><br>_ = state<br><br>}<br><br>func TestRollbackOnDerived(t \*testing.T) {<br><br>rt, _ := lumina.FromSource(\`<br><br>entity Moto { battery: Number; isLow := battery < 20 }<br><br>let m = Moto { battery: 80 }<br><br>\`)<br><br>defer rt.Close()<br><br>\_, err := rt.ApplyEvent("m", "isLow", "true")<br><br>if err == nil { t.Fatal("expected R009 error") }<br><br>} |

# **26.5 Build Requirements**

| **Building with Go FFI** |
| --- |
| \# Step 1: Build the shared library<br><br>cargo build --release -p lumina-ffi<br><br>\# Step 2: Set library path so cgo can find liblumina_ffi.so<br><br>export LD_LIBRARY_PATH=\$(pwd)/target/release:\$LD_LIBRARY_PATH # Linux<br><br>export DYLD_LIBRARY_PATH=\$(pwd)/target/release:\$DYLD_LIBRARY_PATH # macOS<br><br>\# Step 3: Run Go tests<br><br>cd crates/lumina-ffi/lumina_go<br><br>go test ./...<br><br>\# Step 4: Build a Go binary that uses Lumina<br><br>go build -o myapp main.go |

**📌 NOTE: Memory Safety in Go**

All C strings from the FFI are freed with lumina_free_string() inside the wrapper.

Callers never touch C memory directly - the wrapper handles all unsafe blocks.

Always call rt.Close() or use defer rt.Close() to prevent memory leaks.

The LuminaRuntime pointer is not safe to share across goroutines without a mutex.