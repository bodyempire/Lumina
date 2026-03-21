# Lumina System Architecture (v1.5)

The Lumina runtime engine is designed for absolute correctness and deterministic reactivity. Version 1.5 introduces advanced fleet-level triggers, historical state access, and full integration with external systems via the Adapter protocol.

## 1. Compiler Pipeline Overview

Lumina programs are processed through a strictly ordered pipeline. Every stage is designed for zero-allocation performance and strong safety guarantees.

### 2.1 Lexical Analysis (`lumina-lexer`)
Tokenization is performed via the `logos` crate. Version 1.5 adds specialized keywords for `prev()`, `when any/all`, and new aggregate operators.

### 2.2 Syntax Analysis (`lumina-parser`)
The parser maps tokens into an Abstract Syntax Tree (AST):
*   **Recursive Descent**: Handles declarative constructs (`entity`, `rule`, `fn`).
*   **Pratt Parsing**: For expressions, managing operator precedence including the new `prev()` state accessor.

### 2.3 Module Resolution (`lumina-cli::loader`)
The `ModuleLoader` handles recursive file discovery. 
*   **Path Resolution**: Resolves relative `import` paths into a flat source map.
*   **Dependency Deduplication**: Prevents circular imports and ensures each module is parsed exactly once.
*   **AST Merging**: Combines multiple files into a single `Program` context.

### 2.4 Semantic Analysis (`lumina-analyzer`)
Analysis is performed in two distinct passes:
1.  **Declaration Registration**: Records all entities, fields, and pure functions.
2.  **Typechecking & Graph Construction**: Validates expressions and constructs a topological `DependencyGraph`. Version 1.5 introduces complex fleet-level validation for `any/all` conditions.

## 3. The Reactive Engine (`lumina-runtime`)

### 3.1 Closed Evaluation Model
The `Evaluator` executes state changes in deterministic topological order.
*   **Basal Updates**: Direct mutations to stored fields trigger downstream re-evaluation.
*   **Topological Re-evaluation**: Derived `:=` fields are recomputed exactly as needed.
*   **History Access**: The `prev()` operator allows rules to compare current values against the previous engine tick's state.

### 3.2 Fleet Triggers
New in v1.5, fleet triggers allow monitoring state across all instances of an entity:
*   **Running Counters**: The `FleetState` component maintains O(1) true-counts for Boolean fields across the entire engine.
*   **True/False Aggregation**: Logic for `any` (count > 0) and `all` (count == total) is updated atomically after every basal field write.
*   **Edge Detection**: Transitions are detected by comparing current fleet state against the previous tick's state-cache.

### 3.3 Historical State (`prev()`)
Lumina provides deterministic access to the previous state of any entity:
*   **Tick-based Snapshots**: At the start of every update cycle, the engine clones the current `EntityStore` into a `prev_store`.
*   **Evaluation Context**: The `prev()` operator resolves identifiers against this historical snapshot, enabling rate-of-change and state-transition logic.

### 3.4 External Entities & Adapters
Lumina v1.5 can synchronize with external data sources:
*   **Polling Loop**: The runtime `tick()` cycle polls all registered adapters to ingest external data.
*   **Field Filtering**: The `sync on` clause allows filtering incoming external updates to specific trigger fields, preventing unnecessary rule cascades.
*   **Write-back**: Mutations to fields marked as external are pushed to the host system via the `LuminaAdapter` trait's `on_write` hook.

### 3.5 State Snapshots & Safety
Lumina implements a **Self-Healing Guarantee**. Before any destructive action:
1.  The VM takes a complete memory **Snapshot**.
2.  Evaluation proceeds. If a recursion limit (default: 100) is breached or an invariant is violated, the runtime **Automatically Rolls Back** to the snapshot.

---

## 4. v1.5 Implementation Status
All architectural components of the v1.5 specification are now fully operational:

*   **`AggregateStore`**: A specialized storage layer that maintains named, top-level fleet-wide facts like `avg` and `sum` with O(1) read performance.
*   **Structured Alerting**: Full support for `alert` actions, providing consistent metadata (severity, source, code) across FFI and WASM boundaries.
*   **Recovery Logic (`on clear`)**: Integrated support for rule "clear" events, enabling automated lifecycle tracking from incident to resolution.
*   **Rule Cooldown Engine**: A logic gate in the firing pipeline that enforces temporal gaps, preventing alert storms from flapping sensors.

---

## 5. Ecosystem & Tooling

### 5.1 Language Server (`lumina-lsp`)
New in v1.5, a dedicated LSP implementation provides:
*   Real-time diagnostics and error squiggles via the `lumina-analyzer`.
*   Hover tooltips for types and documentation.
*   Multi-file import resolution and cross-document symbol search.

### 5.2 Web Integration
*   **WASM**: Tailored `wasm-bindgen` layer for browser execution.
*   **JSON API**: Stable serialization for snapshots and state exports.

## 6. Technical Stack Considerations
*   **Rust**: Predictable performance and memory safety.
*   **Logos**: High-performance regex-based lexing.
*   **Pratt Parsing**: Elegant handling of complex expression precedence.
*   **Deterministic Execution**: Ensures a given set of events always results in the same final state.
