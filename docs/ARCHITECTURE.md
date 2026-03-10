# Lumina System Architecture

The Lumina compiler and runtime are designed using modern systems programming principles, capitalizing on Rust's type safety and concurrency features to build a robust, predictable reactive engine.

## 1. Compiler Pipeline

The translation from Lumina source text into an actionable state machine occurs in precisely defined phases:

### 1.1 Lexical Analysis (`lumina-lexer`)
Lumina uses `logos` to generate a high-performance Deterministic Finite Automaton (DFA) at compile-time. The lexer maps utf-8 byte streams into semantic tokens, stripping whitespace and matching complex numeric/string literals in a single pass.

### 1.2 Syntax Analysis (`lumina-parser`)
The parser employs a hybrid strategy:
*   **Recursive Descent**: Used for top-level declarations (Entities, Rules, Updates).
*   **Pratt Parsing**: Used for expressions to handle varying operator precedences (`and`, `or`, `==`, `+`, etc.) seamlessly without deeply nested grammar rules and stack exhaustion overheads.

### 1.3 Semantic Analysis (`lumina-analyzer`)
Before executing any state transition, the AST is validated:
*   **Type Checking**: Ensures deterministic typing (e.g., stopping a `Boolean` from being assigned a `Number`).
*   **Acyclic Dependency Graphs**: Checks for circular references in derived fields (e.g., `A := B` and `B := A`) preventing infinite recursion during variable cascades.

## 2. Runtime Virtual Environment (`lumina-runtime`)

### 2.1 State Representation and Transactional Updates
State mutations occurring via `update` commands are non-destructive in transit.
Lumina uses a **Snapshot and Rollback** mechanism. When an event fires, the VM clones the current state topological view. Changes are applied transactionally. If an invariant (`@range` or similar constraint) is violated mid-computation, the entire block is aborted, ensuring the system state never remains corrupted or inconsistent.

### 2.2 Forward Chaining and Evaluation
Once an entity field is updated, a forward-chaining evaluation resolves all downstream derived fields. Rules monitoring those fields are evaluated along state transitions (`becomes true`), reducing continuous overhead to a single edge-detection boundary.

### 2.3 Temporal Schedulers
Conditions with `for X s` or `every X h` utilize asynchronous tick monitors. Time intervals register callbacks within the VM’s event loop which evaluate the persistence of preceding conditions sequentially.
