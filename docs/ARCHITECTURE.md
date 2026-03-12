# Lumina System Architecture (v1.4)

The Lumina runtime engine is designed for absolute correctness and deterministic reactivity. Version 1.4 introduces a decentralized module system, pure function support, and enhanced diagnostic tracking.

## 1. Compiler Pipeline Overview

Lumina programs are processed through a strictly ordered pipeline before execution. Every stage is designed for zero-allocation performance and strong safety guarantees.

### 2.1 Lexical Analysis (`lumina-lexer`)
Tokenization is performed via the `logos` crate, generating a high-performance DFA. Version 1.4 introduces complex string interpolation support, where literals are post-processed into `InterpolatedString` AST nodes when `{}` markers are detected.

### 2.2 Syntax Analysis (`lumina-parser`)
The parser maps tokens into an Abstract Syntax Tree (AST):
*   **Recursive Descent**: Handles declarative constructs (`entity`, `rule`, `fn`).
*   **Pratt Parsing**: For expressions, ensuring correct operator precedence for complex mathematical and boolean logic.

### 2.3 Module Resolution (`lumina-cli::loader`)
New in v1.4, the `ModuleLoader` handles recursive file discovery. 
*   **Path Resolution**: Resolves relative `import` paths into a flat source map.
*   **Dependency Deduplication**: Prevents circular imports and ensures each module is parsed exactly once.
*   **AST Merging**: Combines multiple files into a single `Program` context for the analyzer.

### 2.4 Semantic Analysis (`lumina-analyzer`)
Analysis is performed in two distinct passes:
1.  **Declaration Registration**: Records all entities, fields, and pure functions. It builds the primary `Schema` and detects naming collisions.
2.  **Typechecking & Graph Construction**: Validates every expression against the schema and constructs a topological `DependencyGraph` using Kahn's algorithm. 

The output is an `AnalyzedProgram` containing the validated AST, Schema, and Jump-Ordered Graph.

## 3. The Reactive Engine (`lumina-runtime`)

### 3.1 Closed Evaluation Model
The `Evaluator` executes state changes in deterministic topological order based on the `DependencyGraph`.
*   **Basal Updates**: Direct mutations to stored fields trigger downstream re-evaluation.
*   **Topological Re-evaluation**: Derived `:=` fields are recomputed exactly as needed, ensuring no stale state.
*   **Transition Rules**: The `becomes` modifier detects strictly positive-edge transitions (false -> true), preventing redundant action firing.

### 3.2 State Snapshots & Safety
Lumina implements a **Self-Healing Guarantee**. Before any `update`, `create`, or `delete` action:
1.  The VM takes a complete memory **Snapshot**.
2.  Evaluation proceeds. If a recursion limit (100 levels) is breached or an `@range` invariant is violated, the runtime **Automatically Rolls Back** to the snapshot.
3.  A structured `RuntimeError` is emitted, pinpointing the specific rule that caused the violation.

## 4. Host Communications & Boundary
The engine maintains purity by restricting I/O to defined boundaries:
*   **FFI (C ABI)**: Exposes a stable C interface for integration into Python (via `ctypes`) or Go systems.
*   **WASM**: Cross-compiles to WebAssembly with a tailored `wasm-bindgen` layer for browser execution.
*   **JSON Serialization**: Both `export_state()` and `apply_event()` utilize JSON as the interchange format for system state persistence.

## 5. Technical Stack Considerations
*   **Rust**: Chosen for its lack of a Garbage Collector, ensuring predictable tail latencies (p99) during intensive rule cascades.
*   **Logos & Serde**: Native-speed tokenization and serialization ensure minimum overhead at the I/O boundary.
*   **Memory Safety**: The borrow checker eliminates data races in the topological graph, critical for reactive systems where state dependencies are complex.
