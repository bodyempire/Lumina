# Lumina: A Declarative and Reactive Programming Language for State-Driven Systems

## Abstract

Lumina is a statically typed, declarative, and reactive programming language engineered in Rust, designed specifically for modeling complex state-driven systems. Unlike traditional imperative languages where state transitions are manually coordinated, Lumina employs a continuous evaluation model. Domain entities are defined with intrinsic storage and derived mathematical relationships. Reactive rule cascades autonomously compute state changes, temporal triggers enforce duration and interval-based logic, and invariant assertions ensure computational consistency. The resulting execution model provides a robust framework for asynchronous and deterministic system state resolution, suitable for both embedded logic and high-level behavioral orchestration.

---

## 1. Introduction

Modern software architecture frequently struggles with the synchronization of distributed state and the unintended consequences of imperative control flow across varying subsystems. Lumina mitigates these systemic challenges by inverting the control paradigm. Instead of explicitly instructing the system when and how to update context, developers declare the inherent relationships between properties and the subsequent reactive triggers that must fire upon specific state variations.

### 1.1 Key Features in v1.4
*   **Declarative Entity Modeling**: Fields are partitioned into basal and derived categories. Derived fields (`:=`) maintain a strictly guaranteed relationship with their dependencies through continuous topological re-evaluation using Kahn's algorithm.
*   **Pure Functions (`fn`)**: Stateless, side-effect-free functions allow for complex logic encapsulation within derived fields and rule conditions.
*   **Module System (`import`)**: Support for multi-file projects with robust dependency resolution and circular import detection.
*   **String Interpolation**: Native support for embedding expressions within text literals using the `"{expr}"` syntax, including automatic number formatting.
*   **Deterministic Reactive Automata**: State mutations are constrained to atomic intervals. The runtime leverages a directed acyclic evaluation graph to cascade variable updates without triggering divergent recursion limits.
*   **Temporal Logic Triggers**: The inclusion of `for` and `every` reactive clauses enables the execution of interval-based and sustained-duration logic, natively offloading manual timer management to the runtime orchestrator.
*   **Self-Healing Snapshots**: Before every state-changing operation, the runtime takes a deep-copy snapshot. If anything fails (like a recursion limit breach or range violation), the snapshot is restored instantly, guaranteeing a stable state.

---

## 2. Architecture and Execution Model

The Lumina compiler and runtime pipeline is implemented in Rust, exploiting zero-cost abstractions and strict memory safety guarantees. Every program flows through this 5-stage pipeline:

1.  **`lumina-lexer`**: A high-throughput deterministic finite automaton tokenizer that generates `Vec<SpannedToken>`.
2.  **`lumina-parser`**: A hybrid recursive-descent and Pratt parser optimized for context-free grammar evaluation and operator precedence (`Ast`).
3.  **`Module Loader`**: Resolves `import` statements and builds a unified AST from multiple source files.
4.  **`lumina-analyzer`**: Constructs a dependency graph natively as flat `u32` `NodeId` arrays, enforces static type safety, and validates evaluation order statically.
5.  **`lumina-runtime`**: The core Snapshot VM (`Evaluator`) handling state allocation, `becomes` edge-transition detection, and temporal scheduling.

---

## 3. Language Specification (v1.4 Example)

### 3.1 Entity Schemas & Initialization
Entities define the structure of state contexts.

```lua
import "types.lum"

fn calculate_priority(battery: Number) -> Number {
  if battery < 10 then 1 else 2
}

entity Moto {
  @doc "Battery capacity measured in watt-hours (Wh)"
  @range 0 to 100
  battery: Number
  isBusy: Boolean
  status: Text
  
  -- Derived fields autonomously calculate their value topologically
  priority    := calculate_priority(battery)
  isCritical  := battery < 5
  description := "Unit status: {status} (Battery: {battery}%)"
}

-- Instantiate with explicit basal fields
let moto1 = Moto { battery: 80, isBusy: false, status: "available" }
```

### 3.2 Rule Cascades & Temporal Semantics
Control logic relies upon the `rule` keyword. The `becomes` modifier ensures rules execute strictly on edge transitions.

```lua
-- Fires exactly once when the condition transitions from false to true
rule "Critical Battery" {
  when Moto.isCritical becomes true
  then update moto1.status to "maintenance"
  then show "ALARM: {moto1.description}"
}

-- Fires after the condition holds continuously for the duration
rule "Auto-lock idle bike" {
  when Moto.isBusy becomes false for 15 m
  then update moto1.status to "locked"
}
```

---

## 4. Compilation and Usage

### Prerequisites
* Rust Toolchain (`rustc` >= 1.70.0)
* Cargo Build Manager

### 4.1 Command Line Interface (CLI)
```bash
# Build the CLI
cargo build --release -p lumina-cli

# Execute a Lumina program
cargo run -p lumina-cli -- run main.lum

# Perform static analysis without executing
cargo run -p lumina-cli -- check main.lum

# Interactive REPL (v2 with state persistence)
cargo run -p lumina-cli -- repl
```

---

## 5. Foreign Function Interface (FFI)

Lumina implements a secure C ABI for integration into external environments.

### 5.1 FFI Setup
```bash
# Build the shared library
cargo build --release -p lumina-ffi
```

---

## 6. WebAssembly Integration

Lumina cross-compiles to WebAssembly, operating inside standard JavaScript runtimes.

```bash
# Build WASM package
cd crates/lumina-wasm
wasm-pack build --target web --release
```

---

## 7. Known Limitations (Targeted for v1.5)
*   **External Entities:** Syntax is parsed but host-specific adapters (e.g. Supabase) are not yet implemented.
*   **List Types:** Support for `Number[]`, `Text[]`, and `Boolean[]` is planned for the next major phase.
*   **Source Context in Diagnostics**: While v1.4 added improved error messages, full column-caret highlighting in terminal output is still in refinement.

---

## 8. Development & Testing
Before committing to the codebase, read our detailed [CONTRIBUTING.md](./CONTRIBUTING.md) guide.

Run the full specification regression suite:
```bash
cargo test --workspace
```

## License
This software and associated documentation files are distributed under the MIT License. Reference [`LICENSE`](LICENSE) for complete legal stipulations.
