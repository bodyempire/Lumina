**◈**

**LUMINA**

**v1.3 Implementation Reference**

What was built · How it works · What to never break · How to extend it

_"Describe what is true. Lumina figures out what to do."_

March 2026 · Rust Runtime · 40 Tests Passing · 985KB Native Library

_Designed and authored by Isaac Ishimwe_

**Section 1**

**What Is Built**

_The exact state of the Lumina v1.3 runtime_

This document is the ground truth for Lumina v1.3. It describes what exists in code right now - not what is planned, not what the language spec describes as future work. Every crate, every test count, every API surface here reflects the actual implementation.

# **1.1 Workspace Overview**

The Lumina runtime is a Cargo workspace located at lumina/ with 8 crates. Each crate has exactly one responsibility.

| **Workspace Layout** |
| --- |
| lumina/<br><br>Cargo.toml # workspace manifest<br><br>crates/<br><br>lumina-lexer/ # tokenizer<br><br>lumina-parser/ # AST + parser<br><br>lumina-analyzer/ # type checker + dependency graph<br><br>lumina-runtime/ # evaluator + reactive engine<br><br>lumina-ffi/ # C API (.so / .dll / .a)<br><br>lumina-wasm/ # WebAssembly target<br><br>lumina-cli/ # CLI binary<br><br>lumina-diagnostics/ # (v1.4 - not built yet)<br><br>tests/<br><br>spec/fleet.lum # end-to-end spec test<br><br>spec/errors.lum # L003 error test<br><br>oracle/integration_test.rs<br><br>playground/<br><br>index.html # browser IDE |

# **1.2 Test Counts by Crate**

| **Crate** | **Tests** | **What They Cover** |
| --- | --- | --- |
| lumina-lexer | 1   | Tokenizes a full entity declaration correctly |
| lumina-parser | 6   | Entity parsing, rule parsing, expression parsing, error recovery |
| lumina-analyzer | 6   | L001-L010 error codes, type inference, cycle detection |
| lumina-runtime | 19  | Store, snapshot, evaluator, rules, timer heap |
| lumina-cli | 3   | Integration tests running the binary against spec files |
| lumina-ffi | 5   | FFI load, apply_event, derived recompute, rollback, export_state |
| TOTAL | 40  | All passing - cargo test --workspace |

# **1.3 Build Artifacts**

| **Artifact** | **Location** | **Size** | **What It Is** |
| --- | --- | --- | --- |
| liblumina_ffi.so | target/release/liblumina_ffi.so | 985 KB | Native shared library - callable from Python, Go, C, anything |
| lumina-cli | target/release/lumina | ~3 MB | CLI binary - run / check / repl commands |
| lumina_wasm_bg.wasm | crates/lumina-wasm/pkg/ | ~400 KB | WASM module - runs in browser and edge runtimes |
| lumina_wasm.js | crates/lumina-wasm/pkg/ | ~30 KB | JS bindings for the WASM module |
| lumina.h | crates/lumina-ffi/lumina.h | -   | C header - all FFI function signatures |
| lumina_py.py | crates/lumina-ffi/lumina_py.py | -   | Python ctypes wrapper over liblumina_ffi.so |

**Section 2**

**Crate-by-Crate Reference**

_What each crate contains and what it does_

# **2.1 lumina-lexer**

Tokenizes Lumina source text into a stream of SpannedTokens. Uses the logos crate for fast, pattern-driven tokenization.

| **lumina-lexer API** |
| --- |
| // Public API - the only function callers use<br><br>pub fn tokenize(source: &str, filename: &str) -> Result&lt;Vec<SpannedToken&gt;, LexError><br><br>// Token types (27 keywords, 19 operators)<br><br>pub enum Token {<br><br>// Keywords<br><br>Entity, Rule, When, Then, Let, Show, Update, To, Create, Delete,<br><br>Becomes, For, Every, And, Or, Not, Is, External, Sync, On,<br><br>Text, Number, Boolean, True, False, If, Else,<br><br>// Literals<br><br>NumberLit(f64), StringLit(String), Ident(String),<br><br>// Operators & punctuation<br><br>Assign, DeriveAssign, Dot, Comma, Colon, LBrace, RBrace,<br><br>LParen, RParen, Plus, Minus, Star, Slash, Percent,<br><br>Eq, Ne, Gt, Lt, Ge, Le, Arrow, At, Newline, Eof,<br><br>}<br><br>pub struct Span { pub start: u32, pub end: u32, pub line: u32, pub col: u32 }<br><br>pub struct SpannedToken { pub token: Token, pub span: Span } |

# **2.2 lumina-parser**

Parses a Vec&lt;SpannedToken&gt; into a typed AST. Uses recursive descent for statements and Pratt parsing for expressions. Every AST node carries a Span for error reporting.

| **lumina-parser AST** |
| --- |
| // Public API<br><br>pub fn parse(source: &str) -> Result&lt;Program, ParseError&gt;<br><br>// Top-level AST<br><br>pub struct Program { pub statements: Vec&lt;Statement&gt; }<br><br>pub enum Statement {<br><br>Entity(EntityDecl),<br><br>ExternalEntity(ExternalEntityDecl),<br><br>Let(LetStmt),<br><br>Rule(RuleDecl),<br><br>Action(Action),<br><br>}<br><br>// Entity<br><br>pub struct EntityDecl { pub name: String, pub fields: Vec&lt;Field&gt;, pub span: Span }<br><br>pub enum Field { Stored(StoredField), Derived(DerivedField) }<br><br>pub struct StoredField { pub name: String, pub type_: LuminaType, pub metadata: FieldMetadata, pub span: Span }<br><br>pub struct DerivedField { pub name: String, pub expr: Expr, pub span: Span }<br><br>// Rule<br><br>pub struct RuleDecl {<br><br>pub name: String,<br><br>pub trigger: RuleTrigger,<br><br>pub actions: Vec&lt;Action&gt;,<br><br>pub span: Span,<br><br>}<br><br>pub enum RuleTrigger {<br><br>When(Condition),<br><br>Every(Duration),<br><br>}<br><br>pub struct Condition {<br><br>pub entity: String,<br><br>pub field: String,<br><br>pub becomes: Option&lt;Expr&gt;,<br><br>pub for_duration: Option&lt;Duration&gt;,<br><br>}<br><br>pub struct Duration { pub value: f64, pub unit: TimeUnit } // s/m/h/d<br><br>// Actions<br><br>pub enum Action {<br><br>Show(Expr),<br><br>Update { target: FieldAccess, value: Expr },<br><br>Create { entity: String, instance: String, fields: Vec&lt;(String, Expr)&gt; },<br><br>Delete(String),<br><br>}<br><br>// Expressions - Pratt parser handles precedence<br><br>pub enum Expr {<br><br>Number(f64), Text(String), Bool(bool),<br><br>Ident(String),<br><br>FieldAccess { obj: Box&lt;Expr&gt;, field: String },<br><br>Binary { op: BinOp, left: Box&lt;Expr&gt;, right: Box&lt;Expr&gt; },<br><br>Unary { op: UnOp, operand: Box&lt;Expr&gt; },<br><br>If { cond: Box&lt;Expr&gt;, then_: Box&lt;Expr&gt;, else_: Box&lt;Expr&gt; },<br><br>Interpolated { segments: Vec&lt;Segment&gt; },<br><br>}<br><br>pub enum Segment { Literal(String), Expr(Expr) }<br><br>pub enum BinOp { Add, Sub, Mul, Div, Mod, Eq, Ne, Gt, Lt, Ge, Le, And, Or }<br><br>pub enum UnOp { Neg, Not } |

# **2.3 lumina-analyzer**

Two-pass analysis over the AST. First pass builds the type schema and dependency graph. Second pass type-checks all expressions and validates all rules. Returns a typed AnalyzedProgram or a Vec&lt;AnalyzerError&gt;.

| **lumina-analyzer API** |
| --- |
| // Public API<br><br>pub fn analyze(program: Program) -> Result&lt;AnalyzedProgram, Vec<AnalyzerError&gt;><br><br>pub struct AnalyzedProgram {<br><br>pub program: Program, // the original AST<br><br>pub schema: Schema, // entity type map<br><br>pub graph: DependencyGraph,<br><br>pub rules: Vec&lt;RuleDecl&gt;,<br><br>}<br><br>// Schema - maps entity name -> field names + types<br><br>pub struct Schema {<br><br>pub entities: HashMap&lt;String, EntitySchema&gt;,<br><br>}<br><br>pub struct EntitySchema {<br><br>pub fields: HashMap&lt;String, FieldSchema&gt;,<br><br>}<br><br>pub struct FieldSchema {<br><br>pub type_: LuminaType,<br><br>pub derived: bool,<br><br>pub metadata: FieldMetadata,<br><br>}<br><br>// Dependency graph - NodeId = u32 (flat indices, no pointers)<br><br>pub type NodeId = u32;<br><br>pub struct DependencyGraph {<br><br>interner: HashMap&lt;(String, String), NodeId&gt;,<br><br>nodes: Vec&lt;(String, String)&gt;, // NodeId -> (entity, field)<br><br>dependents: Vec&lt;Vec<NodeId&gt;>,<br><br>dependencies: Vec&lt;Vec<NodeId&gt;>,<br><br>topo_order: Vec&lt;NodeId&gt;, // Kahn's algorithm output<br><br>derived_mask: Vec&lt;bool&gt;,<br><br>}<br><br>pub struct AnalyzerError { pub code: String, pub message: String, pub span: Span } |

| **Error Code** | **Meaning** |
| --- | --- |
| L001 | Unknown identifier - not defined as entity, let binding, or field |
| L002 | Type mismatch - expression type does not match expected type |
| L003 | Cannot assign to derived field - it recomputes automatically |
| L004 | Circular dependency - derived fields form a cycle |
| L005 | Duplicate entity name |
| L006 | Duplicate field name within an entity |
| L007 | Missing required stored field in create block |
| L008 | Unknown entity name |
| L009 | Cannot reassign a let binding - it is immutable |
| L010 | Unknown field on a known entity |

# **2.4 lumina-runtime**

The reactive engine. The most complex crate - contains 6 source files and 19 tests. This is the heart of Lumina.

| **lumina-runtime files** |
| --- |
| crates/lumina-runtime/src/<br><br>lib.rs # RuntimeError enum (R001-R009) + re-exports<br><br>value.rs # Value enum (Number/Text/Bool)<br><br>store.rs # Instance + EntityStore<br><br>snapshot.rs # Snapshot, SnapshotStack, PropResult, RollbackResult, Diagnostic<br><br>engine.rs # Evaluator - the reactive core<br><br>rules.rs # condition_is_met() helper<br><br>timers.rs # ForTimer, EveryTimer, TimerHeap |

| **Core runtime types** |
| --- |
| // Value - the three Lumina primitive types<br><br>pub enum Value { Number(f64), Text(String), Bool(bool) }<br><br>// Instance - one entity instance in the store<br><br>pub struct Instance {<br><br>pub entity_name: String,<br><br>pub fields: HashMap&lt;String, Value&gt;,<br><br>pub prev_fields: HashMap&lt;String, Value&gt;, // for becomes detection<br><br>}<br><br>// EntityStore - all active instances<br><br>pub struct EntityStore {<br><br>instances: HashMap&lt;String, Instance&gt;,<br><br>}<br><br>// Evaluator - owns all runtime state<br><br>pub struct Evaluator {<br><br>pub schema: Schema,<br><br>pub graph: DependencyGraph,<br><br>pub rules: Vec&lt;RuleDecl&gt;,<br><br>pub store: EntityStore,<br><br>pub snapshots: SnapshotStack,<br><br>pub env: HashMap&lt;String, Value&gt;, // global let bindings<br><br>pub instances: HashMap&lt;String, String&gt;, // instance_name -> entity_name<br><br>pub timers: TimerHeap,<br><br>pub output: Vec&lt;String&gt;, // captured show output (WASM)<br><br>depth: usize, // re-entrancy depth counter<br><br>}<br><br>pub const MAX_DEPTH: usize = 100; |

| **Evaluator API** |
| --- |
| // Evaluator public API<br><br>impl Evaluator {<br><br>pub fn new(schema: Schema, graph: DependencyGraph, rules: Vec&lt;RuleDecl&gt;) -> Self<br><br>pub fn exec_statement(&mut self, stmt: &Statement) -> Result&lt;(), RuntimeError&gt;<br><br>pub fn exec_action(&mut self, action: &Action) -> Result&lt;Vec<FiredEvent&gt;, RuntimeError><br><br>pub fn apply_update(&mut self, instance: &str, field: &str, value: Value)<br><br>\-> Result&lt;Vec<FiredEvent&gt;, RuntimeError><br><br>pub fn apply_event(&mut self, instance: &str, field: &str, value: Value)<br><br>\-> Result&lt;PropResult, RollbackResult&gt;<br><br>pub fn export_state(&self) -> serde_json::Value<br><br>pub fn tick(&mut self) -> Result&lt;Vec<FiredEvent&gt;, RollbackResult><br><br>pub fn drain_output(&mut self) -> Vec&lt;String&gt; // WASM show capture<br><br>} |

**🔑 KEY PATTERN: apply_update - The Reactive Core**

apply_update is where all reactivity happens. Steps in exact order:

1\. Take snapshot of EntityStore

2\. Apply the field update to the instance

3\. Check @range - R006 if violated, restore snapshot

4\. Propagate derived fields in topological order (Kahn's output)

5\. Evaluate all rules - fire those whose condition is met

6\. Check depth counter against MAX_DEPTH - R003 if exceeded, restore

7\. On any error: restore snapshot, build Diagnostic, return RollbackResult

8\. On success: commit_all() syncs prev_fields, return events

| **Runtime Error Codes** |
| --- |
| // Runtime error codes<br><br>pub enum RuntimeError {<br><br>R001 { instance: String }, // access to deleted instance<br><br>R002, // division by zero<br><br>R003 { depth: usize }, // rule re-entrancy / loop<br><br>R004 { index: usize, len: usize }, // list bounds (reserved)<br><br>R005 { instance: String, field: String },// null field access<br><br>R006 { field: String, value: f64, min: f64, max: f64 }, // @range violation<br><br>R007 { entity: String, reason: String }, // external sync failed<br><br>R008 { rule: String }, // timer conflict<br><br>R009, // derived field update attempt<br><br>} |

# **2.5 lumina-ffi**

Compiles to liblumina_ffi.so (Linux), .dylib (macOS), .dll (Windows) and liblumina_ffi.a (static). Exposes a flat C API that any language can call via FFI.

| **C FFI Surface - lumina.h** |
| --- |
| // C API - all functions exported with #\[no_mangle\] pub extern "C"<br><br>// Load source, execute all statements, return opaque handle (null on error)<br><br>LuminaRuntime\* lumina_create(const char\* source);<br><br>// Apply field update - value_json is JSON: "42", "true", "\\"hello\\""<br><br>// Returns JSON PropResult or "ERROR:{...}" on rollback<br><br>char\* lumina_apply_event(LuminaRuntime\* rt,<br><br>const char\* instance_name,<br><br>const char\* field_name,<br><br>const char\* value_json);<br><br>// Export current state as JSON - caller must free<br><br>char\* lumina_export_state(const LuminaRuntime\* rt);<br><br>// Advance timers - returns JSON array of fired events<br><br>char\* lumina_tick(LuminaRuntime\* rt);<br><br>// Get last error from lumina_create failure<br><br>char\* lumina_last_error(const LuminaRuntime\* rt);<br><br>// Free any string returned by the runtime - THE ONLY WAY TO FREE<br><br>void lumina_free_string(char\* s);<br><br>// Destroy the runtime, free all memory<br><br>void lumina_destroy(LuminaRuntime\* rt); |

**⚠️ DO NOT BREAK: Memory Ownership Rule**

Every char\* returned by the runtime is heap-allocated by Rust.

The caller MUST free it with lumina_free_string() - not free(), not delete.

Never pass the same pointer to lumina_free_string() twice.

The LuminaRuntime\* handle is owned by the caller - always call lumina_destroy() when done.

# **2.6 lumina-wasm**

Compiles to a WebAssembly package via wasm-pack. Exposes the same runtime capabilities as the FFI but to JavaScript/TypeScript in browser and edge environments.

| **WASM JavaScript API** |
| --- |
| // JavaScript API (wasm-bindgen generated)<br><br>import init, { LuminaRuntime } from './lumina_wasm.js';<br><br>await init();<br><br>const rt = new LuminaRuntime(source); // throws on parse/analyze error<br><br>rt.apply_event(instance, field, valueJson); // returns JSON string<br><br>rt.export_state(); // returns pretty JSON string<br><br>rt.tick(); // returns JSON array of events<br><br>rt.get_output(); // returns captured show output<br><br>LuminaRuntime.check(source); // static - returns errors or "" |

**📌 NOTE: WASM Differences from FFI**

show actions do not print to stdout in WASM - they write to output: Vec&lt;String&gt;.

Call rt.get_output() after run/apply_event to retrieve captured show text.

Instant is cfg-gated for WASM compatibility - timer precision differs from native.

tokio is excluded from WASM builds via target dependencies in lumina-runtime/Cargo.toml.

# **2.7 lumina-cli**

The CLI binary. Three commands. Entry point for all human interaction with the language.

| **Command** | **What It Does** |
| --- | --- |
| lumina run &lt;file.lum&gt; | Full pipeline: parse → analyze → execute all statements → print export_state() as JSON |
| lumina check &lt;file.lum&gt; | Parse and analyze only - prints errors with codes and line numbers, exits non-zero on error |
| lumina repl | Interactive loop - accumulates source, rebuilds evaluator on each input (known limitation: state does not persist across inputs - v1.4 fix) |

**Section 3**

**Language Features in v1.3**

_Every feature that exists and is implemented_

This is the definitive list. If a feature is not on this list, it does not exist in v1.3. Do not reference v1.4 features (functions, modules, enhanced errors) when working on v1.3 code.

# **3.1 Entities**

| **Entity Syntax** |
| --- |
| \-- Basic entity with stored and derived fields<br><br>entity Person {<br><br>name: Text<br><br>age: Number<br><br>isAdult := age >= 18<br><br>grade := if age >= 18 then "adult" else "minor"<br><br>}<br><br>\-- Entity with field metadata<br><br>entity Sensor {<br><br>@doc "Ambient temperature in Celsius"<br><br>@range -40 to 125<br><br>temperature: Number<br><br>@doc "Relative humidity as a percentage"<br><br>@range 0 to 100<br><br>@affects comfort, safetyScore<br><br>humidity: Number<br><br>safeTemp := if temperature > 100 then 100 else temperature<br><br>} |

# **3.2 Rules**

| **Rule Syntax** |
| --- |
| \-- Basic becomes rule<br><br>rule "notify on low battery" {<br><br>when Moto.isLowBattery becomes true<br><br>then show "Battery low on {moto1.status}"<br><br>}<br><br>\-- Multi-action rule<br><br>rule "critical battery" {<br><br>when Moto.isCritical becomes true<br><br>then update moto1.status to "maintenance"<br><br>then show "CRITICAL: Moto pulled from service"<br><br>}<br><br>\-- Temporal rule: fire after condition holds for duration<br><br>rule "auto-lock idle bike" {<br><br>when Moto.isIdle becomes true for 15m<br><br>then update moto1.status to "locked"<br><br>then show "Moto {moto1.id} locked after 15 minutes idle"<br><br>}<br><br>\-- Periodic rule: fire on a fixed schedule<br><br>rule "fleet heartbeat" {<br><br>every 1h<br><br>then show "Fleet check running"<br><br>} |

# **3.3 Actions**

| **Action** | **Syntax** | **What It Does** |
| --- | --- | --- |
| show | show expr | Evaluates expr, prints to stdout (or captures for WASM) |
| update | update instance.field to expr | Updates a stored field, triggers full reactive propagation |
| create | create EntityName { field: val, ... } | Creates a new instance at runtime with an auto-generated name |
| delete | delete instance_name | Removes an instance from the store |

# **3.4 Expressions**

| **Expression Examples** |
| --- |
| \-- Arithmetic<br><br>age + 5<br><br>price \* 1.16<br><br>total / count<br><br>items mod 10<br><br>\-- Comparison<br><br>battery < 20<br><br>age >= 18<br><br>name == "Isaac"<br><br>status != "inactive"<br><br>\-- Logical (short-circuit)<br><br>not isBusy and battery > 15<br><br>isAdmin or isModerator<br><br>\-- Conditional<br><br>if age >= 18 then "adult" else "minor"<br><br>if battery < 5 then "critical" else if battery < 20 then "low" else "ok"<br><br>\-- Text interpolation<br><br>"Battery at {moto1.battery}% - status: {moto1.status}"<br><br>\-- Field access<br><br>person.name<br><br>fleet.totalBikes |

# **3.5 Let Bindings**

| **Let Binding Syntax** |
| --- |
| \-- Scalar binding<br><br>let threshold = 20<br><br>\-- Entity instance binding<br><br>let moto1 = Moto { battery: 80, isBusy: false, status: "available" }<br><br>let isaac = Person { name: "Isaac", birthYear: 2000 } |

# **3.6 External Entities (Spec Only - Not Implemented)**

**⚠️ DO NOT BREAK: External Entities Are Not Built**

The external entity syntax is in the spec and the parser accepts it.

The runtime does NOT have working Supabase or Docker adapters.

Do not write code that depends on external entity sync in v1.3.

This is the only spec feature that is documented but not implemented.

# **3.7 Features That Do NOT Exist in v1.3**

The following are v1.4 and later features. They are not in the lexer, parser, analyzer, or runtime. Do not reference them when working with v1.3 code.

- fn - pure function declarations
- return - function return statement
- import / from - module system
- List&lt;T&gt; - typed list with aggregation
- Enhanced error messages with source context and carets
- REPL v2 with persistent state
- VS Code extension

**Section 4**

**Architecture & Key Patterns**

_The decisions that make everything work_

# **4.1 Pipeline: Source to Execution**

Every .lum program flows through the same 4-stage pipeline. Each stage is a separate crate with a clean API boundary.

| **Compilation Pipeline** |
| --- |
| source: &str<br><br>│<br><br>▼ lumina-lexer<br><br>Vec&lt;SpannedToken&gt;<br><br>│<br><br>▼ lumina-parser<br><br>Program (AST)<br><br>│<br><br>▼ lumina-analyzer<br><br>AnalyzedProgram { schema, graph, rules, program }<br><br>│<br><br>▼ lumina-runtime::Evaluator::new()<br><br>Evaluator<br><br>│<br><br>▼ exec_statement() for each stmt in program.statements<br><br>Running runtime - reactive state machine |

# **4.2 Reactive Propagation - Topological Order**

When a stored field changes, derived fields that depend on it must recompute. The order matters - if A depends on B which depends on C, C must compute before B, B before A. The dependency graph is built once at analysis time using Kahn's algorithm and stored as a pre-computed topo_order.

**🔑 KEY PATTERN: The NodeId Pattern**

The dependency graph uses flat u32 indices (NodeId) instead of pointers or String keys.

Each (entity_name, field_name) pair is interned to a NodeId.

Edges are stored as Vec&lt;Vec<NodeId&gt;> (adjacency lists).

This avoids lifetime issues, is cache-friendly, and enables O(1) dirty marking.

Never change this to use String keys or HashMap&lt;String, Node&gt; - it would break the topo sort.

# **4.3 becomes Detection**

The becomes keyword detects state transitions, not just current state. A rule with 'when Moto.isLowBattery becomes true' fires exactly once - when isLowBattery transitions from false to true - not every time it evaluates to true.

| **becomes Detection Pattern** |
| --- |
| // How becomes works in condition_is_met()<br><br>// Instance.prev_fields stores the previous value<br><br>// Instance.fields stores the current value<br><br>fn condition_is_met(eval: &Evaluator, cond: &Condition, instance: &str) -> bool {<br><br>let inst = eval.store.get(instance)?;<br><br>let current = inst.get(&cond.field)?;<br><br>let previous = inst.prev(&cond.field)?;<br><br>if let Some(target) = &cond.becomes {<br><br>// becomes: current matches target AND previous did not<br><br>let target_val = eval.eval_expr(target, Some(instance)).ok()?;<br><br>current == &target_val && previous != &target_val<br><br>} else {<br><br>// no becomes: condition is currently true<br><br>current.as_bool().unwrap_or(false)<br><br>}<br><br>}<br><br>// prev_fields syncs with fields ONLY after a full stable propagation:<br><br>// store.commit_all() is called at the outermost apply_update only.<br><br>// Nested calls do NOT call commit_all() - this is critical. |

# **4.4 Snapshot & Rollback**

Before every state-changing operation, the runtime takes a complete deep-copy snapshot of EntityStore. If anything fails during propagation, the snapshot is restored. This guarantees the runtime always remains at a known stable state.

| **Snapshot/Rollback Pattern** |
| --- |
| // Snapshot is a deep clone of EntityStore with a version number<br><br>pub struct Snapshot { pub version: u64, pub store: EntityStore }<br><br>// apply_update pattern - always paired snapshot/restore<br><br>let snap = self.snapshots.take(&self.store);<br><br>match self.execute_and_propagate(update) {<br><br>Ok(result) => {<br><br>self.store.commit_all(); // sync prev_fields<br><br>Ok(result)<br><br>}<br><br>Err(e) => {<br><br>self.store = snap.store; // guaranteed rollback<br><br>let diag = Diagnostic::from_runtime_error(e.code(), ...);<br><br>Err(RollbackResult { diagnostic: diag })<br><br>}<br><br>} |

# **4.5 Timer Architecture**

Two timer types. Both are managed by TimerHeap. The host calls tick() on a regular interval (recommended: every 100ms) to advance them.

| **Timer Type** | **Trigger** | **Lifecycle** |
| --- | --- | --- |
| ForTimer | when X becomes Y for &lt;duration&gt; | Starts when condition becomes true. Cancels if condition becomes false before duration elapses. Fires once when elapsed and condition is still true. |
| EveryTimer | every &lt;duration&gt; | Registered at Evaluator::new() for all every-rules. Fires on interval, resets automatically. Never cancelled unless the runtime is destroyed. |

**📌 NOTE: WASM Timer Caveat**

std::time::Instant is not available in WASM. lumina-runtime/src/timers.rs uses

# \[cfg(not(target_arch = "wasm32"))\] to gate Instant usage.

In WASM, timer precision is reduced. tick() still works but uses JS timestamps indirectly.

# **4.6 Re-entrancy Guard**

Rules can trigger updates which can trigger other rules. Without a guard, this would loop forever. The depth counter in Evaluator tracks recursive apply_update calls. If it exceeds MAX_DEPTH (100), R003 is returned and the snapshot is restored.

| **Re-entrancy Guard** |
| --- |
| // In apply_update - checked at start of each recursive call<br><br>self.depth += 1;<br><br>if self.depth > MAX_DEPTH {<br><br>self.depth -= 1;<br><br>return Err(RuntimeError::R003 { depth: self.depth });<br><br>}<br><br>// ... do work ...<br><br>self.depth -= 1;<br><br>// Also: fired_this_cycle: HashSet&lt;String&gt; prevents a rule from<br><br>// firing more than once in a single propagation cycle. |

# **4.7 WASM Output Capture**

In native builds, show actions call println!() directly. In WASM, println!() has no effect. The output: Vec&lt;String&gt; field on Evaluator captures show output instead.

| **WASM Output Capture** |
| --- |
| // In exec_action - show handling<br><br>#\[cfg(target_arch = "wasm32")\]<br><br>Action::Show(expr) => {<br><br>let val = self.eval_expr(expr, ctx)?;<br><br>self.output.push(val.to_string());<br><br>}<br><br>#\[cfg(not(target_arch = "wasm32"))\]<br><br>Action::Show(expr) => {<br><br>let val = self.eval_expr(expr, ctx)?;<br><br>println!("{val}");<br><br>}<br><br>// After run, JavaScript calls:<br><br>const output = rt.get_output(); // drains and clears the buffer |

**Section 5**

**What Never to Break**

_The invariants the runtime depends on_

These are the rules that, if violated, will silently corrupt the runtime's correctness. Tests may still pass but the language will behave incorrectly.

**⚠️ DO NOT BREAK: commit_all() Timing**

store.commit_all() MUST only be called at the outermost apply_update.

It syncs prev_fields with fields. If called mid-propagation, becomes detection breaks.

Nested apply_update calls (from rule actions) must NOT call commit_all().

The depth counter determines outermost vs nested: only commit when depth returns to 1.

**⚠️ DO NOT BREAK: Snapshot Before Every Mutation**

Every function that modifies EntityStore must take a snapshot first.

This includes apply_update, tick() for-timer firing, and tick() every-timer firing.

If you add a new mutating operation, it must follow the snapshot/restore pattern.

An operation that mutates without a snapshot cannot be rolled back.

**⚠️ DO NOT BREAK: Topo Order is Read-Only After Analysis**

graph.topo_order is computed once by Kahn's algorithm during analyze().

It must never be mutated during runtime. It is the source of truth for propagation order.

If you add support for dynamic entity registration at runtime, you must re-run analysis first.

**⚠️ DO NOT BREAK: lumina_free_string is the Only Free Path**

All strings returned by FFI functions are owned by Rust (CString::into_raw).

They must be freed with lumina_free_string() - not the system free().

Do not add any FFI function that returns a char\* without documenting this requirement.

Double-free will cause undefined behavior. Null-check before calling.

**⚠️ DO NOT BREAK: NodeId Stability**

NodeIds are assigned during analysis and must remain stable for the runtime lifetime.

If entity or field ordering changes between analysis and runtime, graph edges are wrong.

The (entity, field) -> NodeId interner must be the single source of truth.

# **5.1 Regression Test Checklist**

Run this after any change. All must pass before committing.

| **Regression Checklist** |
| --- |
| \# Full workspace tests<br><br>cargo test --workspace<br><br>\# CLI end-to-end<br><br>cargo build --release<br><br>cargo run --bin lumina -- run tests/spec/fleet.lum<br><br>\# Expected: ALERT fires, CRITICAL fires, JSON state has battery:4<br><br>cargo run --bin lumina -- check tests/spec/errors.lum<br><br>\# Expected: exits non-zero, stderr contains L003<br><br>\# FFI (Python)<br><br>cargo build --release -p lumina-ffi<br><br>cd crates/lumina-ffi && python test_ffi.py<br><br>\# Expected: all 4 Python tests print ✓<br><br>\# WASM<br><br>cd crates/lumina-wasm && wasm-pack build --target web --out-dir pkg --release<br><br>cd ../.. && python3 -m http.server 8080<br><br>\# Open <http://localhost:8080/playground/index.html><br><br>\# Verify: Run Fleet OS example, ALERT + CRITICAL appear in output panel |

**Section 6**

**Build & Dependencies**

_How to build every artifact from source_

# **6.1 Cargo Dependencies**

| **Crate** | **Dependency** | **Version** | **Purpose** |
| --- | --- | --- | --- |
| lumina-lexer | logos | 0.14 | Fast regex-based tokenizer |
| lumina-runtime | serde | 1   | JSON serialization |
| lumina-runtime | serde_json | 1   | JSON serialization |
| lumina-runtime | tokio (non-WASM only) | 1   | Async runtime for adapter tasks |
| lumina-ffi | libc | 0.2 | C FFI types (c_char, etc.) |
| lumina-ffi | serde_json | 1   | JSON encode/decode across FFI boundary |
| lumina-wasm | wasm-bindgen | 0.2 | Rust-to-JS bindings |
| lumina-wasm | console_error_panic_hook | 0.1 | Panic messages in browser console |
| lumina-wasm | serde_json | 1   | JSON encode/decode |

# **6.2 Build Commands**

| **Build Commands** |
| --- |
| \# Full workspace compile + test<br><br>cargo build --release<br><br>cargo test --workspace<br><br>\# Native shared library (FFI)<br><br>cargo build --release -p lumina-ffi<br><br>\# Produces: target/release/liblumina_ffi.so (Linux)<br><br>\# target/release/liblumina_ffi.dylib (macOS)<br><br>\# target/release/lumina_ffi.dll (Windows)<br><br>\# WASM package<br><br>cargo install wasm-pack # one-time<br><br>rustup target add wasm32-unknown-unknown # one-time<br><br>cd crates/lumina-wasm<br><br>wasm-pack build --target web --out-dir pkg --release<br><br>\# Produces: pkg/lumina_wasm.js + pkg/lumina_wasm_bg.wasm<br><br>\# Generate C header (optional - lumina.h is checked in)<br><br>cargo install cbindgen<br><br>cbindgen --config cbindgen.toml --crate lumina-ffi --output crates/lumina-ffi/lumina.h<br><br>\# CLI binary<br><br>cargo build --release -p lumina-cli<br><br>\# Produces: target/release/lumina<br><br>\# Install globally: cargo install --path crates/lumina-cli |

# **6.3 lumina-ffi Cargo.toml**

| **lumina-ffi Cargo.toml** |
| --- |
| \[package\]<br><br>name = "lumina-ffi"<br><br>version = "0.1.0"<br><br>edition = "2021"<br><br>\[lib\]<br><br>crate-type = \["cdylib", "staticlib"\] # .so and .a<br><br>\[dependencies\]<br><br>lumina-parser = { path = "../lumina-parser" }<br><br>lumina-analyzer = { path = "../lumina-analyzer" }<br><br>lumina-runtime = { path = "../lumina-runtime" }<br><br>libc = "0.2"<br><br>serde_json = "1" |

# **6.4 lumina-wasm Cargo.toml**

| **lumina-wasm Cargo.toml** |
| --- |
| \[package\]<br><br>name = "lumina-wasm"<br><br>version = "0.1.0"<br><br>edition = "2021"<br><br>\[lib\]<br><br>crate-type = \["cdylib"\] # .wasm<br><br>\[dependencies\]<br><br>lumina-parser = { path = "../lumina-parser" }<br><br>lumina-analyzer = { path = "../lumina-analyzer" }<br><br>lumina-runtime = { path = "../lumina-runtime" }<br><br>wasm-bindgen = "0.2"<br><br>serde_json = "1"<br><br>serde = { version = "1", features = \["derive"\] }<br><br>console_error_panic_hook = "0.1"<br><br>\[dev-dependencies\]<br><br>wasm-bindgen-test = "0.3" |

# **6.5 lumina-runtime Cargo.toml (Target-Gated Dependencies)**

| **lumina-runtime Cargo.toml** |
| --- |
| \[dependencies\]<br><br>lumina-parser = { path = "../lumina-parser" }<br><br>lumina-analyzer = { path = "../lumina-analyzer" }<br><br>serde = { version = "1", features = \["derive"\] }<br><br>serde_json = "1"<br><br>\# tokio only for native builds - excluded from WASM<br><br>\[target.'cfg(not(target_arch = "wasm32"))'.dependencies\]<br><br>tokio = { version = "1", features = \["full"\] } |

**Section 7**

**How to Extend the Runtime**

_The correct way to add new features_

Before adding any feature, read this section. Every feature addition must follow these patterns or it will break the existing invariants.

# **7.1 Adding a New Keyword**

- Add the token variant to Token enum in lumina-lexer/src/token.rs
- Add the logos pattern to the lexer in lumina-lexer/src/lib.rs
- Add an AST node to lumina-parser/src/ast.rs
- Add parsing logic in lumina-parser/src/parser.rs
- Add type-checking in lumina-analyzer/src/analyzer.rs
- Add evaluation in lumina-runtime/src/engine.rs
- Add at least 2 tests: one that parses correctly, one that analyzes correctly

# **7.2 Adding a New Action**

- Add a variant to the Action enum in lumina-parser/src/ast.rs
- Add parsing in the parse_action() function in the parser
- Add analysis in the check_action() function in the analyzer
- Add execution in exec_action() in engine.rs
- exec_action MUST return Result&lt;Vec<FiredEvent&gt;, RuntimeError>
- If the action mutates state: take a snapshot before, restore on error

# **7.3 Adding a New Error Code**

Compile-time errors use L-codes (lumina-analyzer). Runtime errors use R-codes (lumina-runtime).

| **Adding Error Codes** |
| --- |
| // Analyzer - add to AnalyzerError construction:<br><br>// Use the next available L-code (L011, L012, ...)<br><br>errors.push(AnalyzerError {<br><br>code: "L011".to_string(),<br><br>message: format!("your message here: {name}"),<br><br>span: node.span,<br><br>});<br><br>// Runtime - add to RuntimeError enum in lib.rs:<br><br>pub enum RuntimeError {<br><br>// ... existing ...<br><br>R009, // derive field update attempt (already exists)<br><br>R010 { your: String, fields: usize }, // next new one<br><br>}<br><br>// Add code() and message() match arms<br><br>// Add Diagnostic::from_runtime_error() pattern match |

# **7.4 Adding a New Expression Variant**

- Add variant to Expr enum in lumina-parser/src/ast.rs
- Add parsing in the Pratt parser's parse_primary() or parse_infix()
- Add type inference in infer_type() in lumina-analyzer/src/analyzer.rs
- Add evaluation in eval_expr() in lumina-runtime/src/engine.rs
- Handle all evaluation failure modes - return RuntimeError, not panic

# **7.5 Adding a New FFI Function**

| **New FFI Function Template** |
| --- |
| // Template for a new FFI function<br><br>#\[no_mangle\]<br><br>pub extern "C" fn lumina_your_function(<br><br>runtime: \*mut LuminaRuntime,<br><br>param: \*const libc::c_char,<br><br>) -> \*mut libc::c_char {<br><br>// 1. Null-check all pointer arguments<br><br>if runtime.is_null() \| param.is_null() { return std::ptr::null_mut(); }<br><br>// 2. Convert C strings to Rust strings<br><br>let rt = unsafe { &mut \*runtime };<br><br>let arg = unsafe { CStr::from_ptr(param) }.to_str().unwrap_or("");<br><br>// 3. Do work<br><br>let result = rt.evaluator.your_method(arg);<br><br>// 4. Return as CString - caller owns and must free with lumina_free_string<br><br>CString::new(result).unwrap().into_raw()<br><br>}<br><br>// 5. Add signature to lumina.h<br><br>// 6. Add wrapper method to lumina_py.py<br><br>// 7. Add a Python test in test_ffi.py |

**Section 8**

**Known Issues in v1.3**

_Limitations to fix in v1.4_

These are confirmed issues in the current implementation. They are not bugs that crash the runtime - they are limitations to be addressed in v1.4.

# **8.1 REPL Does Not Persist State**

The current REPL in lumina-cli rebuilds the entire Evaluator from scratch on every line of input. If you define an entity on line 1 and try to use it on line 2, it is not visible - the evaluator was rebuilt without it.

**📌 NOTE: v1.4 Fix: REPL v2**

REPL v2 will maintain a single Evaluator across all inputs.

Multi-line constructs (entity declarations, rule bodies) will be detected by brace depth.

The REPL will support :state, :schema, :load, :save commands.

# **8.2 Error Messages Lack Source Context**

Error messages in v1.3 report a code (L003), a message string, and a line number. They do not show the offending source line or point to the exact column with a caret. This makes debugging unfamiliar errors harder than it needs to be.

**📌 NOTE: v1.4 Fix: lumina-diagnostics Crate**

A new lumina-diagnostics crate will introduce Diagnostic with SourceLocation.

All error types across all crates will be upgraded to produce Diagnostic values.

DiagnosticRenderer::render() will produce the full Rust-style error format.

# **8.3 External Entities Not Implemented**

The parser accepts external entity syntax. The analyzer type-checks it. But the runtime has no working adapter - there is no Supabase realtime connection, no Docker health poller, and no REST webhook receiver. external entity declarations are silently ignored at runtime.

**📌 NOTE: Future Fix: Adapters**

Implement only when there is a concrete use case requiring it.

The MPSC channel architecture is already designed for this in the spec.

Build Supabase polling adapter first - it is simpler than realtime websocket.

# **8.4 No VS Code Syntax Highlighting**

There is no .lum language extension for VS Code. Writing Lumina code in an editor currently shows no syntax highlighting. This will be addressed in v1.4 as the VS Code extension chapter (Chapter 21 in the v1.4 spec doc).

# **8.5 WASM Timer Precision**

std::time::Instant is cfg-gated for WASM. The WASM build compiles but timer precision is approximate - for/every rules in the WASM playground will fire eventually but not with millisecond accuracy. This is acceptable for the playground but not for production use.

**Section 9**

**Using Lumina from Python**

_The lumina_py wrapper - complete usage guide_

The Python wrapper is the primary way to use Lumina from non-Rust code right now. It is a thin ctypes wrapper over liblumina_ffi.so with no external dependencies - just Python standard library.

# **9.1 Setup**

| **Python Setup** |
| --- |
| \# Step 1: Build the shared library<br><br>cargo build --release -p lumina-ffi<br><br>\# Step 2: The wrapper looks for the library at:<br><br>\# target/release/liblumina_ffi.so (Linux)<br><br>\# target/release/liblumina_ffi.dylib (macOS)<br><br>\# target/release/lumina_ffi.dll (Windows)<br><br>\# It searches target/release first, then target/debug<br><br>\# Step 3: Import<br><br>\# Run your Python scripts from the repo root OR<br><br>\# set LD_LIBRARY_PATH=target/release before running<br><br>from crates.lumina_ffi.lumina_py import LuminaRuntime |

# **9.2 API Reference**

| **Python API Usage** |
| --- |
| from lumina_py import LuminaRuntime<br><br>\# Load from source string<br><br>rt = LuminaRuntime.from_source("""<br><br>entity Moto {<br><br>battery: Number<br><br>isLowBattery := battery < 20<br><br>}<br><br>let moto1 = Moto { battery: 80 }<br><br>""")<br><br>\# Load from file<br><br>rt = LuminaRuntime.from_file("fleet_os.lum")<br><br>\# Apply a field update - value is a Python native type<br><br>result = rt.apply_event("moto1", "battery", 15)<br><br>\# result is a dict: { "success": True, "events_fired": \[...\], "version": N }<br><br>\# Apply multiple updates<br><br>rt.apply_event("moto1", "battery", 4)<br><br>rt.apply_event("moto1", "isBusy", True)<br><br>\# Get current state<br><br>state = rt.export_state()<br><br>\# state is a dict: { "instances": { ... }, "stable": True, "version": N }<br><br>print(state\["instances"\]\["moto1"\]\["fields"\]\["isLowBattery"\]) # True<br><br>\# Advance timers (call periodically for for/every rules)<br><br>events = rt.tick() # returns list of { "rule": ..., "instance": ... }<br><br>\# Rollback raises RuntimeError with message and suggested fix<br><br>try:<br><br>rt.apply_event("moto1", "isLowBattery", True) # R009: derived field<br><br>except RuntimeError as e:<br><br>print(e) # Lumina rollback: ... Fix: ... |

# **9.3 Value Type Mapping**

| **Python Type** | **Lumina Type** | **JSON Wire Format** |
| --- | --- | --- |
| int or float | Number | 42 or 3.14 |
| str | Text | "hello" (quoted in JSON) |
| bool | Boolean | true or false |

**📌 NOTE: JSON Encoding**

apply_event() internally calls json.dumps(value) before passing to the FFI.

You pass native Python values - int, float, str, bool - not JSON strings.

The wrapper handles encoding and decoding transparently.