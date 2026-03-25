**LUMINA**

**v1.6 Implementation Guide**

_For Antigravity AI -- exact file paths, complete Rust code, build order,_

_anti-hallucination guards, and verification for all 6 v1.6 features._

_"Describe what is true. Lumina figures out what to do."_

Chapters 35-40 | 6 Features | Rust + LSP | The Infrastructure Release | 2026

_Designed and authored by Isaac Ishimwe_

**Preface**

**How To Use This Document**

_Rules for Antigravity AI -- read every word before writing any code_

**CRITICAL Core Philosophy Guard -- Absolute and Non-Negotiable**

Lumina is a declarative reactive language. "Describe what is true. Lumina figures out what to do."

Every feature must help engineers describe what is TRUE about their reactive system.

Do NOT add general computation, math functions, string manipulation, or data processing.

Do NOT make Lumina generate Terraform or Kubernetes configs -- that is not its job.

Do NOT add stop, exit, while, or any procedural flow control.

When in doubt: "Does this describe truth about a reactive system?" If not -- stop.

**CRITICAL Implementation Rules -- Non-Negotiable**

RULE 1: v1.5 complete, all phases green, cargo test --workspace passing before starting v1.6.

RULE 2: Every file path in this document is exact. Never invent paths.

RULE 3: cargo test --workspace must pass at the end of every phase.

RULE 4: ref is a new field type in the AST -- not a pointer, not a smart pointer.

RULE 5: Multi-condition triggers extend the existing Trigger enum -- no new rule type.

RULE 6: Frequency tracking lives in the runtime -- not the analyzer, not the parser.

RULE 7: write action routes through the existing LuminaAdapter on_write() -- no new trait.

RULE 8: Timestamp is a new Value variant -- not a String, not a Number.

RULE 9: now() is only valid in update and write actions -- reject it in derived fields.

RULE 10: LSP v2 updates lumina-lsp in place -- do NOT create a new binary.

**NOTE v1.5 Workspace Entering v1.6**

crates/lumina-lexer/ -- all v1.5 tokens present including Prev, Any, All, Cooldown

crates/lumina-parser/ -- AggregateDecl, AlertAction, on_clear, RuleParam all in AST

crates/lumina-analyzer/ -- L001-L034 active, all v1.5 checks passing

crates/lumina-diagnostics/ -- DiagnosticRenderer, SourceLocation

crates/lumina-runtime/ -- Evaluator with adapters, FleetState, AggregateStore, AlertEvent

crates/lumina-ffi/ -- C API, Python bindings, Go wrapper

crates/lumina-wasm/ -- WASM target, Playground v2

crates/lumina-cli/ -- run/check/repl, ModuleLoader, ReplSession

crates/lumina-lsp/ -- lumina-lsp binary, diagnostics/hover/goto/symbols/completion

extensions/lumina-vscode/ -- grammar + snippets + LSP client

All v1.5 tests passing. cargo test --workspace must stay green throughout all 6 phases.

**Chapter 35**

**Multi-Condition Triggers**

_Implementation -- extending Trigger with AndBecomes variants_

Multi-condition triggers extend the existing Trigger enum with a new variant that holds a Vec of conditions. The parser recognizes "and" after a trigger clause. The runtime evaluates all conditions and fires only when all are simultaneously true, tracking the compound state across ticks.

# **35.1 Lexer -- One New Token**

**NOTE Check First**

Token::And may already exist for "and" in boolean expressions (x > 5 and y < 10).

If it does -- do NOT add a duplicate. Reuse the existing Token::And.

If it does not exist -- add it now.

| **crates/lumina-lexer/src/token.rs -- verify or add**                                                                                                                                                                                           |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Check if Token::And already exists.<br><br>// If not, add:<br><br>// #\[token("and")\]<br><br>// And,<br><br>// Do NOT add Token::And if it already exists for boolean expressions.<br><br>// The parser will disambiguate based on context. |

# **35.2 Parser -- Extend Trigger Enum**

| **crates/lumina-parser/src/ast.rs -- add CompoundBecomes**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pub enum Trigger {<br><br>WhenBecomes { entity: String, field: String, value: Expr, duration: Option&lt;Duration&gt; },<br><br>AnyBecomes { entity: String, field: String, value: Expr, duration: Option&lt;Duration&gt; },<br><br>AllBecomes { entity: String, field: String, value: Expr, duration: Option&lt;Duration&gt; },<br><br>Every { interval: Duration },<br><br>// NEW: compound trigger -- all conditions must be simultaneously true<br><br>CompoundBecomes {<br><br>conditions: Vec&lt;TriggerCondition&gt;,<br><br>duration: Option&lt;Duration&gt;,<br><br>},<br><br>}<br><br>// Each condition in a compound trigger<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct TriggerCondition {<br><br>pub entity: String,<br><br>pub field: String,<br><br>pub value: Expr,<br><br>pub span: Span,<br><br>} |

| **crates/lumina-parser/src/parser.rs -- parse compound trigger**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| // In parse_trigger(), after parsing the first WhenBecomes condition:<br><br>// Check if "and" follows -- if so, parse additional conditions<br><br>fn parse_trigger(&mut self) -> Result&lt;Trigger, ParseError&gt; {<br><br>// Parse first condition as normal WhenBecomes<br><br>let first = self.parse_single_condition()?;<br><br>let duration = self.parse_optional_for_duration()?;<br><br>// Check for "and" -- compound trigger<br><br>if self.check(Token::And) {<br><br>let mut conditions = vec!\[first\];<br><br>while self.check(Token::And) {<br><br>self.advance(); // consume "and"<br><br>conditions.push(self.parse_single_condition()?);<br><br>// Maximum 3 and clauses enforced by analyzer (L035)<br><br>}<br><br>let dur = self.parse_optional_for_duration()?;<br><br>return Ok(Trigger::CompoundBecomes { conditions, duration: dur });<br><br>}<br><br>// Single condition -- existing behavior<br><br>Ok(Trigger::WhenBecomes {<br><br>entity: first.entity,<br><br>field: first.field,<br><br>value: first.value,<br><br>duration,<br><br>})<br><br>}<br><br>fn parse_single_condition(&mut self) -> Result&lt;TriggerCondition, ParseError&gt; {<br><br>let span = self.current_span();<br><br>let entity = self.expect_ident("entity name")?;<br><br>self.expect(Token::Dot)?;<br><br>let field = self.expect_ident("field name")?;<br><br>self.expect(Token::Becomes)?;<br><br>let value = self.parse_expr()?;<br><br>Ok(TriggerCondition { entity, field, value, span })<br><br>} |

# **35.3 Analyzer -- L035 Check**

| **crates/lumina-analyzer/src/analyzer.rs -- L035**                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In analyze_trigger() for Trigger::CompoundBecomes:<br><br>Trigger::CompoundBecomes { conditions, .. } => {<br><br>// L035: maximum 3 and clauses<br><br>if conditions.len() > 3 {<br><br>self.error("L035",<br><br>"multi-condition trigger supports at most 3 and clauses",<br><br>conditions\[3\].span);<br><br>}<br><br>// Validate each condition field exists and is Boolean<br><br>for cond in conditions {<br><br>self.validate_trigger_field(&cond.entity, &cond.field, cond.span);<br><br>}<br><br>} |

# **35.4 Runtime -- Compound State Tracking**

| **crates/lumina-runtime/src/compound.rs -- new file**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use std::collections::HashMap;<br><br>/// Tracks the current truth value of each condition in a compound trigger.<br><br>/// Key: (rule_name, condition_index) -> bool<br><br>pub struct CompoundState {<br><br>states: HashMap&lt;(String, usize), bool&gt;,<br><br>}<br><br>impl CompoundState {<br><br>pub fn new() -> Self { Self { states: HashMap::new() } }<br><br>// Update the truth value of one condition in a compound trigger.<br><br>pub fn update(&mut self, rule: &str, idx: usize, val: bool) {<br><br>self.states.insert((rule.to_string(), idx), val);<br><br>}<br><br>// Returns true if ALL conditions in the rule are currently true.<br><br>pub fn all_true(&self, rule: &str, count: usize) -> bool {<br><br>(0..count).all(\|i\| {<br><br>self.states.get(&(rule.to_string(), i)).copied().unwrap_or(false)<br><br>})<br><br>}<br><br>// Returns the previous compound truth (before last update).<br><br>pub fn was_all_true(&self, rule: &str, count: usize, prev: &Self) -> bool {<br><br>(0..count).all(\|i\| {<br><br>prev.states.get(&(rule.to_string(), i)).copied().unwrap_or(false)<br><br>})<br><br>}<br><br>} |

| **crates/lumina-runtime/src/engine.rs -- wire compound state**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Evaluator struct:<br><br>compound_state: CompoundState,<br><br>compound_state_prev: CompoundState,<br><br>// In propagate() -- after any field update, check all compound triggers:<br><br>for rule in &self.program.rules {<br><br>if let Trigger::CompoundBecomes { conditions, .. } = &rule.trigger {<br><br>let count = conditions.len();<br><br>// Update state for any condition whose field just changed<br><br>for (idx, cond) in conditions.iter().enumerate() {<br><br>if cond.entity == updated_entity && cond.field == updated_field {<br><br>let target = eval_bool_literal(&cond.value);<br><br>let current_val = self.store.get_field(&instance, &cond.field)<br><br>.and_then(\|v\| v.as_bool().ok()).unwrap_or(false);<br><br>let matches = current_val == target;<br><br>self.compound_state.update(&rule.name, idx, matches);<br><br>}<br><br>}<br><br>// Check for rising edge: was not all true, now all true<br><br>let now_all = self.compound_state.all_true(&rule.name, count);<br><br>let was_all = self.compound_state_prev.all_true(&rule.name, count);<br><br>if now_all && !was_all {<br><br>self.fire_rule(rule, &instance)?;<br><br>}<br><br>// on clear: was all true, now not all true<br><br>if !now_all && was_all {<br><br>if let Some(clear_body) = &rule.on_clear {<br><br>self.exec_body(clear_body, &instance)?;<br><br>}<br><br>}<br><br>}<br><br>}<br><br>// Save prev state after processing<br><br>self.compound_state_prev = self.compound_state.clone(); |

# **35.5 Build Order**

**BUILD Chapter 35 -- exact sequence**

Step 1: Check if Token::And exists. Add only if missing. cargo build -p lumina-lexer.

Step 2: Add TriggerCondition struct and Trigger::CompoundBecomes to AST.

Step 3: Add parse_single_condition(). Update parse_trigger() to detect "and" chains.

Step 4: cargo build -p lumina-parser.

Step 5: Add L035 check to analyzer. cargo build -p lumina-analyzer.

Step 6: Create crates/lumina-runtime/src/compound.rs with CompoundState.

Step 7: Add compound_state + compound_state_prev to Evaluator.

Step 8: Wire compound state updates into propagate().

Step 9: Implement rising edge detection and on clear for compound triggers.

Step 10: cargo test --workspace.

Step 11: Test: two conditions, set first true (rule should NOT fire), set second true (rule MUST fire).

Step 12: Test: set either condition false, verify on clear fires.

**Chapter 36**

**Entity Relationships -- ref**

_Implementation -- ref as a new field type with traversal in derived fields and rules_

ref is a new field type. The AST gains a FieldType::Ref variant. The runtime stores ref fields as instance name strings in the EntityStore. Derived field evaluation traverses refs by looking up the referenced instance. The analyzer validates that ref targets exist and detects circular references.

# **36.1 Lexer -- One New Token**

| **crates/lumina-lexer/src/token.rs**                                                                                       |
| -------------------------------------------------------------------------------------------------------------------------- |
| // Add:<br><br>// #\[token("ref")\]<br><br>// Ref,<br><br>pub enum Token {<br><br>// ... existing ...<br><br>Ref,<br><br>} |

# **36.2 Parser -- FieldType::Ref**

| **crates/lumina-parser/src/ast.rs -- add Ref variant**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pub enum FieldType {<br><br>Number,<br><br>Boolean,<br><br>Text,<br><br>Timestamp, // added in Ch40<br><br>Ref(String), // NEW: ref EntityName<br><br>}<br><br>// FieldDecl already handles stored fields -- add ref parsing:<br><br>pub struct FieldDecl {<br><br>pub name: String,<br><br>pub field_type: FieldType, // now includes Ref(entity_name)<br><br>pub doc: Option&lt;String&gt;,<br><br>pub range: Option&lt;(f64, f64)&gt;,<br><br>pub is_derived: bool,<br><br>pub expr: Option&lt;Expr&gt;,<br><br>pub span: Span,<br><br>} |

| **crates/lumina-parser/src/parser.rs -- parse ref fields**                                                                                                                                                                                                                                                                                                                                                                                  |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In parse_field_type():<br><br>Token::Ref => {<br><br>self.advance(); // consume "ref"<br><br>let target = self.expect_ident("entity name")?;<br><br>Ok(FieldType::Ref(target))<br><br>}<br><br>// Ref fields are stored fields (not derived)<br><br>// They appear in entity declarations like:<br><br>// cooling: ref CoolingUnit<br><br>// parsed as: FieldDecl { name: "cooling", field_type: Ref("CoolingUnit"), is_derived: false } |

| **Expr::FieldAccess -- traversal in derived fields**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Traversal like cooling.isFailing is already parsed as chained field access.<br><br>// Ensure Expr::FieldAccess supports chaining:<br><br>pub enum Expr {<br><br>// ... existing ...<br><br>FieldAccess {<br><br>object: Box&lt;Expr&gt;, // the ref field name<br><br>field: String, // the field on the referenced entity<br><br>span: Span,<br><br>},<br><br>}<br><br>// s.cooling.isFailing parses as:<br><br>// FieldAccess {<br><br>// object: FieldAccess { object: Ident("s"), field: "cooling" },<br><br>// field: "isFailing"<br><br>// } |

# **36.3 Analyzer -- L036, L037, L038**

| **crates/lumina-analyzer/src/analyzer.rs -- ref validation**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In analyze_field() for FieldType::Ref:<br><br>FieldType::Ref(target_entity) => {<br><br>// L036: target entity must exist<br><br>if !self.entity_registry.contains(target_entity) {<br><br>self.error("L036",<br><br>&format!("ref target entity {} does not exist", target_entity),<br><br>field.span);<br><br>}<br><br>// L037: detect circular refs<br><br>// Build ref dependency graph and check for cycles<br><br>if self.would_create_cycle(&self.current_entity, target_entity) {<br><br>self.error("L037",<br><br>&format!("circular ref: {} -> {} -> ...", self.current_entity, target_entity),<br><br>field.span);<br><br>}<br><br>}<br><br>// In analyze_expr() for Expr::FieldAccess:<br><br>// Resolve the ref chain: s.cooling.isFailing<br><br>// 1. Look up "cooling" field on Server -- confirm it is FieldType::Ref(CoolingUnit)<br><br>// 2. Look up "isFailing" field on CoolingUnit -- confirm it exists<br><br>// 3. Return the type of "isFailing" (Boolean) |

# **36.4 Runtime -- ref Storage and Traversal**

| **crates/lumina-runtime/src/store.rs -- ref fields as instance name strings**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Ref fields are stored in the EntityStore as Value::Text(instance*name)<br><br>// This is the simplest correct representation:<br><br>// When engineer writes: let server1 = Server { cooling: coolingUnit1, ... }<br><br>// The runtime stores: server1.cooling = Value::Text("coolingUnit1")<br><br>// In eval_expr() for Expr::FieldAccess:<br><br>Expr::FieldAccess { object, field, .. } => {<br><br>// Evaluate the object to get the referenced instance name<br><br>let obj_val = self.eval_expr(object, ctx)?;<br><br>match obj_val {<br><br>Value::Text(instance_name) => {<br><br>// Look up the field on the referenced instance<br><br>self.store.get_field(&instance_name, field)<br><br>.ok_or(RuntimeError::R005 {<br><br>instance: instance_name.clone(),<br><br>field: field.clone(),<br><br>})<br><br>}<br><br>*=> Err(RuntimeError::R006 {<br><br>message: format!("ref field did not resolve to an instance name"),<br><br>}),<br><br>}<br><br>}<br><br>// Chained traversal works naturally:<br><br>// s.cooling.isFailing<br><br>// 1. eval s.cooling -> Value::Text("coolingUnit1")<br><br>// 2. store.get_field("coolingUnit1", "isFailing") -> Value::Boolean(true) |

| **Instance creation with ref fields**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In exec_create() -- handle ref field values<br><br>// When engineer writes:<br><br>// let server1 = Server { cooling: coolingUnit1, cpuTemp: 72 }<br><br>// The "coolingUnit1" identifier resolves to Value::Text("coolingUnit1")<br><br>// stored as the cooling field value.<br><br>// In exec_update() -- ref fields can be reassigned<br><br>// update server1.cooling = coolingUnit2<br><br>// This stores Value::Text("coolingUnit2") as server1.cooling<br><br>// Subsequent traversals automatically use the new target. |

# **36.5 Build Order**

**BUILD Chapter 36 -- exact sequence**

Step 1: Add Token::Ref to lexer. cargo build -p lumina-lexer.

Step 2: Add FieldType::Ref(String) to AST. Add Expr::FieldAccess if not already present.

Step 3: Add parse_field_type() case for Token::Ref.

Step 4: Ensure chained field access (s.cooling.isFailing) parses correctly.

Step 5: cargo build -p lumina-parser.

Step 6: Add L036/L037 checks to analyzer. Add ref dependency graph for cycle detection.

Step 7: Add FieldAccess type resolution in analyzer.

Step 8: cargo build -p lumina-analyzer.

Step 9: Implement Expr::FieldAccess in eval_expr() -- resolve ref then look up field.

Step 10: Handle ref fields in exec_create() and exec_update().

Step 11: cargo test --workspace.

Step 12: Test: create two entities with ref, verify cooling.isFailing reads from correct instance.

Step 13: Test L036: ref to nonexistent entity must report error.

Step 14: Test L037: circular ref A->B->A must report error.

Step 15: Test ref reassignment: update server.cooling = newUnit, verify traversal uses new target.

**Chapter 37**

**Frequency Conditions**

_Implementation -- sliding window occurrence counting per rule per instance_

Frequency conditions add an optional clause to rule declarations: N times within Duration. The parser adds two new fields to RuleDecl. The runtime maintains a sliding window of occurrence timestamps per rule per instance. On each trigger rising edge, it records the timestamp and checks if N or more occurrences fall within the window.

# **37.1 Parser -- Extend RuleDecl**

| **crates/lumina-parser/src/ast.rs -- add frequency fields to RuleDecl**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pub struct RuleDecl {<br><br>pub name: String,<br><br>pub params: Vec&lt;RuleParam&gt;,<br><br>pub trigger: Trigger,<br><br>pub cooldown: Option&lt;Duration&gt;,<br><br>pub frequency: Option&lt;FrequencyClause&gt;, // NEW<br><br>pub body: Vec&lt;Action&gt;,<br><br>pub on_clear: Option&lt;Vec<Action&gt;>,<br><br>pub span: Span,<br><br>}<br><br>#\[derive(Debug, Clone)\]<br><br>pub struct FrequencyClause {<br><br>pub count: u32, // N -- the number of occurrences<br><br>pub window: Duration, // within Duration<br><br>pub span: Span,<br><br>} |

| **crates/lumina-parser/src/parser.rs -- parse frequency clause**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In parse_rule(), after parsing trigger and cooldown:<br><br>// Parse optional "N times within Duration"<br><br>let frequency = if self.check(Token::Number) {<br><br>// Peek: is this "N times within"?<br><br>let span = self.current_span();<br><br>let n = self.parse_integer()? as u32;<br><br>self.expect_keyword("times")?;<br><br>self.expect_keyword("within")?;<br><br>let window = self.parse_duration()?;<br><br>Some(FrequencyClause { count: n, window, span })<br><br>} else {<br><br>None<br><br>};<br><br>// NOTE: "times" and "within" are contextual keywords.<br><br>// They are parsed as identifiers in this context.<br><br>// Do NOT add Token::Times or Token::Within -- use expect_ident() and match the string. |

# **37.2 Analyzer -- L039, L040**

| **crates/lumina-analyzer/src/analyzer.rs -- frequency validation**                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In analyze_rule() for rules with frequency clause:<br><br>if let Some(freq) = &rule.frequency {<br><br>// L039: count must be >= 2<br><br>if freq.count < 2 {<br><br>self.error("L039",<br><br>"frequency condition count must be at least 2",<br><br>freq.span);<br><br>}<br><br>// L040: window duration must be positive<br><br>if freq.window.is_zero_or_negative() {<br><br>self.error("L040",<br><br>"frequency condition window duration must be positive",<br><br>freq.span);<br><br>}<br><br>} |

# **37.3 Runtime -- Sliding Window Tracker**

| **crates/lumina-runtime/src/frequency.rs -- new file**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| use std::collections::HashMap;<br><br>use std::time::Instant;<br><br>/// Tracks occurrence timestamps for frequency conditions.<br><br>/// Key: (rule_name, instance_name) -> Vec&lt;Instant&gt;<br><br>pub struct FrequencyTracker {<br><br>windows: HashMap&lt;(String, String), Vec<Instant&gt;>,<br><br>}<br><br>impl FrequencyTracker {<br><br>pub fn new() -> Self { Self { windows: HashMap::new() } }<br><br>// Record one occurrence of a condition becoming true.<br><br>// Returns true if the count within the window >= required count.<br><br>pub fn record_and_check(<br><br>&mut self,<br><br>rule: &str,<br><br>instance: &str,<br><br>count: u32,<br><br>window: std::time::Duration,<br><br>) -> bool {<br><br>let key = (rule.to_string(), instance.to_string());<br><br>let now = Instant::now();<br><br>let times = self.windows.entry(key).or_insert_with(Vec::new);<br><br>// Add this occurrence<br><br>times.push(now);<br><br>// Remove occurrences outside the sliding window<br><br>times.retain(\|t\| now.duration_since(\*t) <= window);<br><br>// Check if we have enough occurrences<br><br>times.len() >= count as usize<br><br>}<br><br>// Clear the window for a rule+instance after cooldown expires.<br><br>pub fn clear(&mut self, rule: &str, instance: &str) {<br><br>self.windows.remove(&(rule.to_string(), instance.to_string()));<br><br>}<br><br>} |

| **crates/lumina-runtime/src/engine.rs -- wire frequency into fire_rule()**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add to Evaluator struct:<br><br>frequency_tracker: FrequencyTracker,<br><br>// In fire_rule() -- check frequency BEFORE executing body:<br><br>fn fire_rule(&mut self, rule: &RuleDecl, instance: &str) -> Result&lt;(), RuntimeError&gt; {<br><br>// Cooldown check (existing)<br><br>if !self.should_fire(rule, instance) {<br><br>return Ok(());<br><br>}<br><br>// Frequency check (NEW)<br><br>if let Some(freq) = &rule.frequency {<br><br>let threshold_met = self.frequency_tracker.record_and_check(<br><br>&rule.name,<br><br>instance,<br><br>freq.count,<br><br>freq.window.to_std_duration(),<br><br>);<br><br>if !threshold_met {<br><br>// Occurrence recorded but threshold not yet reached<br><br>return Ok(());<br><br>}<br><br>// Threshold met -- clear window and fire<br><br>self.frequency_tracker.clear(&rule.name, instance);<br><br>}<br><br>// Execute rule body<br><br>self.exec_body(&rule.body, instance)?;<br><br>self.record_firing(rule, instance);<br><br>Ok(())<br><br>}<br><br>// IMPORTANT: frequency.record_and_check() is called on RISING EDGE only.<br><br>// The trigger condition must transition from false to true to count.<br><br>// Sustained true does not count -- only each new transition counts. |

# **37.4 Build Order**

**BUILD Chapter 37 -- exact sequence**

Step 1: Add FrequencyClause struct to parser AST.

Step 2: Add frequency: Option&lt;FrequencyClause&gt; to RuleDecl.

Step 3: Parse "N times within Duration" as contextual keywords in parse_rule().

Step 4: cargo build -p lumina-parser.

Step 5: Add L039/L040 checks to analyzer. cargo build -p lumina-analyzer.

Step 6: Create crates/lumina-runtime/src/frequency.rs with FrequencyTracker.

Step 7: Add frequency_tracker to Evaluator.

Step 8: Wire frequency check into fire_rule() -- after cooldown, before body.

Step 9: Ensure frequency counts only rising edges -- not sustained true state.

Step 10: cargo test --workspace.

Step 11: Test: rule with 3 times within 10m. Fire trigger twice -- rule must NOT fire. Fire third time -- rule MUST fire.

Step 12: Test: verify window slides -- old occurrences outside window do not count.

Step 13: Test: frequency + cooldown together -- cooldown fires after frequency threshold met.

**Chapter 38**

**LSP v2**

_Implementation -- rename, find references, code actions, semantic tokens, inlay hints_

LSP v2 updates the existing lumina-lsp binary. No new crate is created. The LanguageServer trait implementation in backend.rs gains five new capability handlers. The ServerCapabilities declaration is updated to advertise all new capabilities.

# **38.1 Updated ServerCapabilities**

| **crates/lumina-lsp/src/backend.rs -- add new capabilities**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In initialize() -- update ServerCapabilities:<br><br>Ok(InitializeResult { capabilities: ServerCapabilities {<br><br>// Existing v1.5 capabilities:<br><br>text_document_sync: Some(...),<br><br>hover_provider: Some(...),<br><br>definition_provider: Some(...),<br><br>document_symbol_provider: Some(...),<br><br>completion_provider: Some(...),<br><br>// NEW v1.6 capabilities:<br><br>rename_provider: Some(OneOf::Left(true)),<br><br>references_provider: Some(OneOf::Left(true)),<br><br>code_action_provider: Some(CodeActionProviderCapability::Simple(true)),<br><br>semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(<br><br>SemanticTokensOptions {<br><br>legend: SemanticTokensLegend {<br><br>token_types: vec!\[<br><br>SemanticTokenType::new("entity"),<br><br>SemanticTokenType::new("storedField"),<br><br>SemanticTokenType::new("derivedField"),<br><br>SemanticTokenType::new("rule"),<br><br>SemanticTokenType::new("aggregate"),<br><br>\],<br><br>token_modifiers: vec!\[\],<br><br>},<br><br>full: Some(SemanticTokensFullOptions::Bool(true)),<br><br>range: None,<br><br>}<br><br>)),<br><br>inlay_hint_provider: Some(OneOf::Left(true)),<br><br>..Default::default()<br><br>}, ..Default::default() }) |

# **38.2 Rename Handler**

| **crates/lumina-lsp/src/rename.rs -- new file**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use tower*lsp::lsp_types::\*;<br><br>use lumina_parser::{Program, Statement};<br><br>use std::collections::HashMap;<br><br>/// Find all locations where a symbol (entity name or field name) is used.<br><br>/// Returns a WorkspaceEdit with all locations replaced by new_name.<br><br>pub fn rename_symbol(<br><br>prog: &Program,<br><br>uri: &Url,<br><br>pos: Position,<br><br>new_name: &str,<br><br>) -> Option&lt;WorkspaceEdit&gt; {<br><br>// 1. Identify what symbol is at pos (entity name or field name)<br><br>let symbol = find_symbol_at(prog, pos)?;<br><br>// 2. Find all references to that symbol in the program<br><br>let refs = find_all_refs(prog, &symbol);<br><br>// 3. Build TextEdit list for each reference<br><br>let edits: Vec&lt;TextEdit&gt; = refs.iter().map(\|r\| TextEdit {<br><br>range: \*r,<br><br>new_text: new_name.to_string(),<br><br>}).collect();<br><br>let mut changes = HashMap::new();<br><br>changes.insert(uri.clone(), edits);<br><br>Some(WorkspaceEdit { changes: Some(changes), ..Default::default() })<br><br>}<br><br>pub fn find_all_refs(prog: &Program, symbol: &str) -> Vec&lt;Range&gt; {<br><br>let mut refs = Vec::new();<br><br>for stmt in &prog.statements {<br><br>collect_refs(stmt, symbol, &mut refs);<br><br>}<br><br>refs<br><br>}<br><br>fn collect_refs(stmt: &lumina_parser::Statement, symbol: &str, out: &mut Vec&lt;Range&gt;) {<br><br>use lumina_parser::Statement;<br><br>match stmt {<br><br>Statement::Entity(e) => {<br><br>if e.name == symbol { out.push(ident_range(&e.name_span)); }<br><br>for f in &e.fields {<br><br>if f.name == symbol { out.push(ident_range(&f.span)); }<br><br>}<br><br>}<br><br>Statement::Rule(r) => {<br><br>collect_refs_in_trigger(&r.trigger, symbol, out);<br><br>collect_refs_in_body(&r.body, symbol, out);<br><br>}<br><br>*=> {}<br><br>}<br><br>}<br><br>fn ident_range(span: &lumina_parser::Span) -> Range {<br><br>let l = span.line.saturating_sub(1);<br><br>let c = span.col.saturating_sub(1);<br><br>Range { start: Position{line:l,character:c}, end: Position{line:l,character:c+span.len} }<br><br>} |

# **38.3 Code Actions Handler**

| **crates/lumina-lsp/src/code_actions.rs -- new file**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use tower*lsp::lsp_types::\*;<br><br>use lumina_diagnostics::Diagnostic;<br><br>/// Generate quick-fix code actions for known error codes.<br><br>pub fn code_actions_for(diags: &\[Diagnostic\], range: Range) -> Vec&lt;CodeAction&gt; {<br><br>diags.iter().filter_map(\|d\| {<br><br>match d.code.as_str() {<br><br>"L001" => Some(CodeAction {<br><br>title: format!("Did you mean {}?", d.suggestion.as_deref().unwrap_or("")),<br><br>kind: Some(CodeActionKind::QUICKFIX),<br><br>diagnostics: Some(vec!\[to_lsp_diag(d)\]),<br><br>edit: d.suggestion.as_ref().map(\|s\| make_edit(range, s)),<br><br>..Default::default()<br><br>}),<br><br>"L028" => Some(CodeAction {<br><br>title: "Replace with nearest valid severity".to_string(),<br><br>kind: Some(CodeActionKind::QUICKFIX),<br><br>diagnostics: Some(vec!\[to_lsp_diag(d)\]),<br><br>edit: Some(make_edit(range, "warning")),<br><br>..Default::default()<br><br>}),<br><br>"L034" => Some(CodeAction {<br><br>title: "Set minimum cooldown: 1s".to_string(),<br><br>kind: Some(CodeActionKind::QUICKFIX),<br><br>diagnostics: Some(vec!\[to_lsp_diag(d)\]),<br><br>edit: Some(make_edit(range, "1s")),<br><br>..Default::default()<br><br>}),<br><br>* => None,<br><br>}<br><br>}).collect()<br><br>}<br><br>fn make_edit(range: Range, text: &str) -> WorkspaceEdit {<br><br>WorkspaceEdit { changes: None, ..Default::default() } // stub -- implement with uri<br><br>} |

# **38.4 Inlay Hints Handler**

| **crates/lumina-lsp/src/inlay_hints.rs -- new file**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| use tower_lsp::lsp_types::\*;<br><br>use lumina_parser::{Program, Statement, FieldType};<br><br>/// Generate inlay hints showing field types inline.<br><br>pub fn inlay_hints(prog: &Program) -> Vec&lt;InlayHint&gt; {<br><br>let mut hints = Vec::new();<br><br>for stmt in &prog.statements {<br><br>if let Statement::Entity(e) = stmt {<br><br>for f in &e.fields {<br><br>let type_label = match &f.field_type {<br><br>FieldType::Number => ": Number",<br><br>FieldType::Boolean => ": Boolean",<br><br>FieldType::Text => ": Text",<br><br>FieldType::Timestamp => ": Timestamp",<br><br>FieldType::Ref(t) => &format!(": ref {}", t),<br><br>};<br><br>let line = f.span.line.saturating_sub(1);<br><br>let col = f.span.col.saturating_sub(1) + f.span.len;<br><br>hints.push(InlayHint {<br><br>position: Position { line, character: col },<br><br>label: InlayHintLabel::String(type_label.to_string()),<br><br>kind: Some(InlayHintKind::TYPE),<br><br>padding_left: Some(true),<br><br>padding_right: None,<br><br>text_edits: None,<br><br>tooltip: None,<br><br>data: None,<br><br>});<br><br>}<br><br>}<br><br>}<br><br>hints<br><br>} |

# **38.5 Build Order**

**BUILD Chapter 38 -- exact sequence**

Step 1: Add tower-lsp new dependencies if needed (semantic tokens, inlay hints types).

Step 2: Update ServerCapabilities in backend.rs to advertise all new capabilities.

Step 3: Create rename.rs. Add rename and references handlers to LanguageServer impl.

Step 4: Create code_actions.rs. Add code_action handler to LanguageServer impl.

Step 5: Create inlay_hints.rs. Add inlay_hints handler to LanguageServer impl.

Step 6: Add semantic tokens handler -- walk AST and emit token type per entity/field/rule.

Step 7: cargo build -p lumina-lsp.

Step 8: cargo install --path crates/lumina-lsp (updates binary in place).

Step 9: In VS Code: rename an entity -- verify all references update.

Step 10: Right-click a field -- Find All References -- verify all usages shown.

Step 11: Introduce L001 error -- verify lightbulb appears with suggestion.

Step 12: Verify inlay hints show field types inline without hovering.

**Chapter 39**

**write Action**

_Implementation -- new Action variant routing through LuminaAdapter on_write()_

write is a new Action variant in the AST. It is only valid on external entity fields. The runtime resolves the target instance, confirms it has a registered adapter, and calls adapter.on_write(field, value). The analyzer rejects write on internal entity fields with L038.

# **39.1 Lexer -- One New Token**

| **crates/lumina-lexer/src/token.rs**                                                                                             |
| -------------------------------------------------------------------------------------------------------------------------------- |
| // Add:<br><br>// #\[token("write")\]<br><br>// Write,<br><br>pub enum Token {<br><br>// ... existing ...<br><br>Write,<br><br>} |

# **39.2 Parser -- Action::Write**

| **crates/lumina-parser/src/ast.rs -- add Write variant**                                                                                                                                                                                                                                                                              |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pub enum Action {<br><br>Show(Expr),<br><br>Update { instance: String, field: String, value: Expr },<br><br>Create { entity: String, name: String, fields: Vec&lt;(String, Expr)&gt; },<br><br>Delete { instance: String },<br><br>Alert(AlertAction),<br><br>Write { instance: String, field: String, value: Expr }, // NEW<br><br>} |

| **crates/lumina-parser/src/parser.rs -- parse write action**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In parse_action():<br><br>Token::Write => {<br><br>let span = self.current_span();<br><br>self.advance(); // consume "write"<br><br>let instance = self.expect_ident("instance name")?;<br><br>self.expect(Token::Dot)?;<br><br>let field = self.expect_ident("field name")?;<br><br>self.expect(Token::Assign)?; // "="<br><br>let value = self.parse_expr()?;<br><br>Ok(Action::Write { instance, field, value })<br><br>}<br><br>// "write s.throttle = 50" parses as:<br><br>// Action::Write { instance: "s", field: "throttle", value: Expr::Number(50.0) }<br><br>// "write s.replicaCount = s.replicaCount + 2" parses as:<br><br>// Action::Write { instance: "s", field: "replicaCount",<br><br>// value: Expr::BinOp { op: Add,<br><br>// left: Expr::FieldAccess { object: Ident("s"), field: "replicaCount" },<br><br>// right: Expr::Number(2.0) } } |

# **39.3 Analyzer -- L038**

| **crates/lumina-analyzer/src/analyzer.rs -- L038**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In analyze_action() for Action::Write:<br><br>Action::Write { instance, field, value } => {<br><br>// Resolve the entity type of the instance<br><br>let entity_name = self.resolve_instance_entity(instance);<br><br>// L038: write only valid on external entity fields<br><br>if !self.is_external_entity(&entity_name) {<br><br>self.error("L038",<br><br>&format!("write only valid on external entity fields -- use update for {}", instance),<br><br>action_span);<br><br>}<br><br>// Validate field exists on the entity<br><br>self.validate_field_exists(&entity_name, field, action_span);<br><br>// Validate value type matches field type<br><br>self.validate_expr_type(value, &entity_name, field, action_span);<br><br>} |

# **39.4 Runtime -- exec_action() for Write**

| **crates/lumina-runtime/src/engine.rs -- Action::Write**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| // In exec_action():<br><br>Action::Write { instance, field, value } => {<br><br>// 1. Evaluate the value expression<br><br>let val = self.eval_expr(value, ctx)?;<br><br>// 2. Resolve the actual instance name<br><br>// (may be a rule parameter like "s" -> "server1")<br><br>let resolved = self.resolve_instance(instance, ctx);<br><br>// 3. Find the registered adapter for this instance's entity<br><br>let entity_name = self.store.entity_of(&resolved)<br><br>.ok_or(RuntimeError::R007 { instance: resolved.clone() })?;<br><br>// 4. Call on_write() on the adapter<br><br>let mut wrote = false;<br><br>for adapter in &mut self.adapters {<br><br>if adapter.entity_name() == entity_name {<br><br>adapter.on_write(field, &val);<br><br>wrote = true;<br><br>break;<br><br>}<br><br>}<br><br>// 5. If no adapter found -- runtime warning (not panic)<br><br>if !wrote {<br><br>eprintln!("\[lumina\] warning: write on {} -- no adapter registered", resolved);<br><br>}<br><br>Ok(())<br><br>}<br><br>// NOTE: write does NOT update the local EntityStore.<br><br>// The adapter handles the write. The store is updated only when the adapter<br><br>// pushes new values back through poll() on the next tick.<br><br>// This reflects reality: the external world decides what the new value is. |

# **39.5 Build Order**

**BUILD Chapter 39 -- exact sequence**

Step 1: Add Token::Write to lexer. cargo build -p lumina-lexer.

Step 2: Add Action::Write to AST.

Step 3: Add parse case for Token::Write in parse_action().

Step 4: cargo build -p lumina-parser.

Step 5: Add L038 check to analyzer. cargo build -p lumina-analyzer.

Step 6: Add Action::Write case to exec_action() in engine.rs.

Step 7: Implement adapter lookup and on_write() call.

Step 8: Ensure write does NOT update the local EntityStore directly.

Step 9: cargo test --workspace.

Step 10: Test: register StaticAdapter with on_write() recording calls.

Step 11: Test: fire a rule with write action -- verify on_write() called with correct field and value.

Step 12: Test L038: write on internal entity must report error.

Step 13: Test: write with expression (s.count + 1) -- verify expression evaluates correctly.

**Chapter 40**

**Timestamp Type**

_Implementation -- Value::Timestamp, .age accessor, now() in update/write_

Timestamp is a new Value variant storing Unix milliseconds as u64. The lexer adds Token::Timestamp for the type keyword. The parser adds FieldType::Timestamp and Expr::Now for the now() call. The runtime computes .age dynamically from the stored timestamp. The analyzer rejects now() in derived field expressions with L041.

# **40.1 Lexer -- New Tokens**

| **crates/lumina-lexer/src/token.rs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Add:<br><br>// #\[token("Timestamp")\] TimestampType, // the type keyword<br><br>// #\[token("now")\] Now, // now() function<br><br>pub enum Token {<br><br>// ... existing ...<br><br>TimestampType, // "Timestamp" -- the field type declaration<br><br>Now, // "now" -- used only in update/write actions<br><br>}<br><br>// NOTE: "age" is a field accessor like any other field name.<br><br>// It does NOT need a dedicated token.<br><br>// lastSeen.age parses as FieldAccess { object: Ident("lastSeen"), field: "age" }<br><br>// The runtime handles .age specially when the object is Value::Timestamp. |

# **40.2 Runtime -- Value::Timestamp**

| **crates/lumina-runtime/src/value.rs -- add Timestamp variant**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pub enum Value {<br><br>Number(f64),<br><br>Boolean(bool),<br><br>Text(String),<br><br>List(Vec&lt;Value&gt;),<br><br>Timestamp(u64), // NEW: Unix timestamp in milliseconds<br><br>}<br><br>impl Value {<br><br>pub fn as*timestamp(&self) -> Result&lt;u64, RuntimeError&gt; {<br><br>match self {<br><br>Value::Timestamp(t) => Ok(\*t),<br><br>* => Err(RuntimeError::TypeMismatch { expected: "Timestamp".into() }),<br><br>}<br><br>}<br><br>/// Format a duration in milliseconds as human-readable.<br><br>pub fn format_duration_ms(ms: u64) -> String {<br><br>let secs = ms / 1000;<br><br>if secs < 60 { return format!("{}s", secs); }<br><br>let mins = secs / 60;<br><br>let rem_secs = secs % 60;<br><br>if mins < 60 { return format!("{}m {}s", mins, rem_secs); }<br><br>let hours = mins / 60;<br><br>let rem_mins = mins % 60;<br><br>format!("{}h {}m", hours, rem_mins)<br><br>}<br><br>} |

# **40.3 Parser -- FieldType::Timestamp, Expr::Now**

| **crates/lumina-parser/src/ast.rs**                                                                                                                                                                                                                                                                                                                                                     |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // Already added FieldType::Timestamp in Ch36 AST section.<br><br>// Ensure it is present:<br><br>pub enum FieldType {<br><br>Number,<br><br>Boolean,<br><br>Text,<br><br>Timestamp, // present<br><br>Ref(String),<br><br>}<br><br>// Add Expr::Now:<br><br>pub enum Expr {<br><br>// ... existing ...<br><br>Now { span: Span }, // now() -- current UTC moment as Timestamp<br><br>} |

| **crates/lumina-parser/src/parser.rs -- parse now()**                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In parse_primary():<br><br>Token::Now => {<br><br>let span = self.current_span();<br><br>self.advance(); // consume "now"<br><br>self.expect(Token::LParen)?;<br><br>self.expect(Token::RParen)?;<br><br>Ok(Expr::Now { span })<br><br>}<br><br>// .age accessor:<br><br>// lastSeen.age parses as:<br><br>// FieldAccess { object: Ident("lastSeen"), field: "age" }<br><br>// No special parsing needed -- same as any other field access.<br><br>// The runtime handles it when object evaluates to Value::Timestamp. |

# **40.4 Analyzer -- L041, L042**

| **crates/lumina-analyzer/src/analyzer.rs -- Timestamp validation**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // L041: now() not allowed in derived field expressions<br><br>Expr::Now { span } => {<br><br>if self.in_derived_field_context {<br><br>self.error("L041",<br><br>"now() is not valid in derived field expressions -- use in update or write only",<br><br>\*span);<br><br>}<br><br>}<br><br>// L042: .age must be compared to a duration literal<br><br>// In analyze_expr() for FieldAccess where field == "age":<br><br>// Check that the comparison operand is a Duration literal (e.g., 5m, 30s, 1h)<br><br>// and that the object resolves to a Timestamp field.<br><br>// If compared to a plain number -- emit L042.<br><br>// Also: validate FieldType::Timestamp in parse_field_type():<br><br>Token::TimestampType => {<br><br>self.advance();<br><br>Ok(FieldType::Timestamp)<br><br>} |

# **40.5 Runtime -- .age and now()**

| **crates/lumina-runtime/src/engine.rs -- Timestamp evaluation**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // In eval*expr() for Expr::FieldAccess where field == "age":<br><br>Expr::FieldAccess { object, field, .. } if field == "age" => {<br><br>let obj_val = self.eval_expr(object, ctx)?;<br><br>match obj_val {<br><br>Value::Timestamp(stored_ms) => {<br><br>// Compute elapsed time in milliseconds<br><br>let now_ms = unix_now_ms(); // current Unix time in ms<br><br>let age_ms = now_ms.saturating_sub(stored_ms);<br><br>// Return as a Duration value for comparison with duration literals<br><br>Ok(Value::Number(age_ms as f64))<br><br>}<br><br>* => Err(RuntimeError::TypeMismatch { expected: "Timestamp".into() }),<br><br>}<br><br>}<br><br>// In eval_expr() for Expr::Now:<br><br>Expr::Now { .. } => {<br><br>Ok(Value::Timestamp(unix_now_ms()))<br><br>}<br><br>// unix_now_ms() helper:<br><br>fn unix_now_ms() -> u64 {<br><br>std::time::SystemTime::now()<br><br>.duration_since(std::time::UNIX_EPOCH)<br><br>.unwrap_or_default()<br><br>.as_millis() as u64<br><br>}<br><br>// Duration literal comparison:<br><br>// lastSeen.age > 5m<br><br>// "5m" parses as Duration { value: 5, unit: Minutes }<br><br>// Converts to milliseconds: 5 \* 60 \* 1000 = 300_000<br><br>// Compared with: age_ms > 300_000<br><br>// Interpolated string {lastSeen.age}:<br><br>// When formatting, detect Value::Number representing ms duration<br><br>// and format using Value::format_duration_ms()<br><br>// This requires context -- tag duration values or handle in string interpolation. |

| **Unset Timestamp -- age is infinity**                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| // A Timestamp field that has never been set should be treated as<br><br>// "infinitely old" -- isStale and isLost would immediately be true.<br><br>// Implementation: use a sentinel value of 0 for unset Timestamp.<br><br>// age of a Timestamp(0) = now_ms - 0 = very large number.<br><br>// This naturally makes age > 5m true for any unset Timestamp.<br><br>// In FieldDecl default values:<br><br>// Timestamp fields default to Value::Timestamp(0) when not explicitly set. |

# **40.6 Build Order**

**BUILD Chapter 40 -- exact sequence**

Step 1: Add Token::TimestampType and Token::Now to lexer. cargo build -p lumina-lexer.

Step 2: Add FieldType::Timestamp (if not already from Ch36) and Expr::Now to AST.

Step 3: Add parse case for Token::TimestampType in parse_field_type().

Step 4: Add parse case for Token::Now in parse_primary().

Step 5: cargo build -p lumina-parser.

Step 6: Add L041 check (now() in derived field). Add L042 check (.age vs non-duration).

Step 7: cargo build -p lumina-analyzer.

Step 8: Add Value::Timestamp(u64) to runtime Value enum.

Step 9: Implement .age evaluation in eval_expr() FieldAccess for "age" field.

Step 10: Implement Expr::Now evaluation returning Value::Timestamp(unix_now_ms()).

Step 11: Set default Timestamp(0) for unset Timestamp fields.

Step 12: Handle duration comparisons: convert duration literals to ms for comparison.

Step 13: Handle {lastSeen.age} in interpolated strings -- format as human-readable.

Step 14: cargo test --workspace.

Step 15: Test: set lastSeen = now(), wait, verify .age increases.

Step 16: Test: isStale := lastSeen.age > 5m -- verify becomes true after 5 minutes.

Step 17: Test: unset Timestamp -- verify isStale is immediately true.

Step 18: Test L041: now() in derived field := must report error.

**Appendix**

**Complete v1.6 Build Sequence**

_6 phases -- implement in this exact order, cargo test after every phase_

**BUILD Phase 1 -- Chapter 35: Multi-Condition Triggers**

1\. Verify or add Token::And.

2\. Add TriggerCondition + Trigger::CompoundBecomes to AST.

3\. Update parse_trigger() to detect and chain.

4\. Add L035 to analyzer.

5\. Create compound.rs with CompoundState.

6\. Wire into propagate() with rising edge detection.

7\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 2 -- Chapter 36: Entity Relationships (ref)**

1\. Add Token::Ref.

2\. Add FieldType::Ref(String) + Expr::FieldAccess to AST.

3\. Parse ref fields and chained traversal.

4\. Add L036/L037 to analyzer with cycle detection.

5\. Implement FieldAccess eval -- resolve ref then look up field.

6\. Handle ref in exec_create() and exec_update().

7\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 3 -- Chapter 37: Frequency Conditions**

1\. Add FrequencyClause to AST + RuleDecl.

2\. Parse "N times within Duration" as contextual keywords.

3\. Add L039/L040 to analyzer.

4\. Create frequency.rs with FrequencyTracker sliding window.

5\. Wire into fire_rule() after cooldown check.

6\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 4 -- Chapter 38: LSP v2**

1\. Update ServerCapabilities for all new capabilities.

2\. Create rename.rs. Add rename + references handlers.

3\. Create code_actions.rs. Add code action handler.

4\. Create inlay_hints.rs. Add inlay hints handler.

5\. Add semantic tokens handler.

6\. cargo build -p lumina-lsp. cargo install lumina-lsp.

7\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 5 -- Chapter 39: write Action**

1\. Add Token::Write.

2\. Add Action::Write to AST.

3\. Parse write action in parse_action().

4\. Add L038 to analyzer.

5\. Implement Action::Write in exec_action() -- adapter.on_write() call.

6\. Verify write does NOT update local EntityStore.

7\. cargo test --workspace \[MUST BE GREEN\].

**BUILD Phase 6 -- Chapter 40: Timestamp Type**

1\. Add Token::TimestampType and Token::Now.

2\. Add FieldType::Timestamp + Expr::Now to AST.

3\. Add parse cases for both.

4\. Add L041/L042 to analyzer.

5\. Add Value::Timestamp(u64) to runtime.

6\. Implement .age evaluation and now() evaluation.

7\. Set Timestamp(0) as default for unset fields.

8\. Handle duration comparisons and string interpolation.

9\. cargo test --workspace \[MUST BE GREEN\].

**DONE v1.6 Definition of Done -- 15 Verification Points**

1\. cargo test --workspace -- all tests pass, zero regressions from v1.5.

2\. Multi-condition: two conditions, first true alone -- rule does NOT fire.

3\. Multi-condition: both conditions true -- rule MUST fire.

4\. Multi-condition: either condition clears -- on clear fires.

5\. ref: s.cooling.isFailing reads from the correct referenced instance.

6\. ref: update server.cooling = newUnit -- traversal immediately uses new target.

7\. ref: L036 fires for ref to nonexistent entity.

8\. ref: L037 fires for circular ref A->B->A.

9\. Frequency: 3 times within 10m -- fires only on third occurrence.

10\. Frequency: old occurrences outside sliding window do not count.

11\. write: on_write() called on adapter with correct field and value.

12\. write: local EntityStore NOT updated by write (adapter owns the state).

13\. Timestamp: .age increases over time after lastSeen = now().

14\. Timestamp: unset Timestamp field is immediately stale.

15\. LSP v2: rename an entity -- all references update in editor.