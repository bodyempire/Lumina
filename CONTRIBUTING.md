# Contributing to Lumina

Welcome to the Lumina Systems team. Lumina is built for rigorous safety, deterministic reactions, and temporal stability. Because it is used as a backend reactive engine, we employ strict developmental methodologies.

This document serves as the deep technical guide for contributing to the Lumina v1.4 codebase.

---

## 1. Local Workstation Setup

If contributing to the production engine, ensure you have:
* The Rust Toolchain (>= 1.70.0)
* The Cargo dependency manager.
* `wasm-pack` (for WASM target compilation)
* `python3` (for testing the C FFI)

---

## 2. Architecture & Key Patterns

Every Lumina program flows through a strict 5-stage pipeline. Each stage is a separate crate with a clean API boundary:

1. **`lumina-lexer`**: Converts source UTF-8 to `Vec<SpannedToken>` using `logos`. It includes a post-processor for string interpolation detection.
2. **`lumina-parser`**: Maps tokens to a typed Abstract Syntax Tree (AST).
3. **`Module Loader`**: Handles recursive file resolution, dependency deduplication, and AST merging.
4. **`lumina-analyzer`**: Performs 2-pass semantic validation, constructs the dependency graph, and strictly enforces static typing.
5. **`lumina-runtime`**: The core Snapshot VM (`Evaluator`) that evaluates rules and commands.

### 2.1 The NodeId Pattern (Kahn's Algorithm)
Lumina guarantees acyclic evaluation using topological sorting. 
* The dependency graph uses flat `u32` indices (`NodeId`) rather than pointers or heap-allocated keys.
* Each `(entity_name, field_name)` pair is interned to a `NodeId`.
* Maintain this pattern to ensure O(1) dirty marking and memory locality during reactive cascades.

### 2.2 Re-entrancy Guard and Cascades
Rules can trigger updates which trigger other rules. A re-entrancy guard ensures safe rule cascading:
* The `Evaluator` tracks recursive `apply_update` calls via a `depth` counter.
* If `depth > MAX_DEPTH` (100), the runtime returns `R003` and rolls back to a safe snapshot.
* A rule can only fire once per propagation cycle to prevent divergent oscillation.

### 2.3 Edge Transition Detection
The `becomes` keyword detects transitions, not just current state.
* The `store` maintains both `prev_fields` and `fields`.
* `becomes` is only `true` when the current value matches the target AND the previous value did not.

---

## 3. Core Invariants

These invariants keep the runtime mathematically sound. Violation of these rules leads to silent state corruption.

### Invariant: commit_all() Timing
`store.commit_all()` must only be called at the outermost `apply_update` (`depth == 1`). It synchronizes `prev_fields` with `fields`. If called mid-propagation during a nested update, edge-transition detection is fundamentally broken.

### Invariant: Snapshot Before Every Mutation
Lumina guarantees self-healing. Every function that modifies `EntityStore` must take a snapshot before mutation:
* Normal updates (`apply_update`)
* Timer expirations (`tick()` for-timer firing and every-timer firing)
If you add a new mutating operation, it must follow the snapshot/restore pattern.

### Invariant: Topo Order is Read-Only
`graph.topo_order` is computed once by Kahn's algorithm in the Analyzer. It must never be mutated during runtime execution.

---

## 4. How to Extend the Runtime

### 4.1 Adding a New Keyword
1. Add the token variant to `Token` enum in `lumina-lexer/src/token.rs`.
2. Add the `logos` pattern to the lexer in `lumina-lexer/src/lib.rs`.
3. Add an AST node to `lumina-parser/src/ast.rs`.
4. Add parsing logic in `lumina-parser/src/parser.rs`.
5. Add type-checking in `lumina-analyzer/src/analyzer.rs`.
6. Add evaluation in `lumina-runtime/src/engine.rs`.

### 4.2 Adding a New Action
1. Add a variant to the `Action` enum in `lumina-parser/src/ast.rs`.
2. Map it in `parse_action()` in the parser.
3. Validate it in `check_action()` in the analyzer.
4. Execute it in `exec_action()` in `engine.rs`.
   * `exec_action` must return `Result<Vec<FiredEvent>, RuntimeError>`.
   * If the action mutates state, take a snapshot before, and restore on error.

### 4.3 Diagnostic Mapping
* Compile-time errors use L-codes (`L011`, `L012`, etc.) in `lumina-analyzer`.
* Runtime errors use R-codes (`R010`, `R011`, etc.) in `lumina-runtime`. They must be added to the `RuntimeError` enum and explicitly mapped in `Diagnostic::from_runtime_error()`.

---

## 5. Regression Test Checklist

Run this full suite before any pull request:

1. **Full workspace:** `cargo test --workspace`
2. **CLI E2E:** `cargo build --release` && `cargo run -p lumina-cli -- run tests/spec/fleet.lum`
3. **C FFI:** `cargo build --release -p lumina-ffi`
4. **WASM target:** `cd crates/lumina-wasm && wasm-pack build --target web --release`

## Engine Integrity
By following these strict constraints, you help maintain Lumina as a fault-tolerant, mathematically sound engine. 
