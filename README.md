# Lumina: A Declarative and Reactive Programming Language for State-Driven Systems

## Abstract

Lumina is a statically typed, declarative, and reactive programming language engineered in Rust, designed specifically for modeling complex state-driven systems. Unlike traditional imperative languages where state transitions are manually coordinated, Lumina employs a continuous evaluation model. Domain entities are defined with intrinsic storage and derived mathematical relationships. Reactive rule cascades autonomously compute state changes, temporal triggers enforce duration and interval-based logic, and invariant assertions ensure computational consistency. The resulting execution model provides a robust framework for asynchronous and deterministic system state resolution, suitable for both embedded logic and high-level behavioral orchestration.

---

## 1. Introduction

Modern software architecture frequently struggles with the synchronization of distributed state and the unintended consequences of imperative control flow across varying subsystems. Lumina mitigates these systemic challenges by inverting the control paradigm. Instead of explicitly instructing the system when and how to update context, developers declare the inherent relationships between properties and the subsequent reactive triggers that must fire upon specific state variations.

### 1.1 Key Features in v1.5
*   **Declarative Entity Modeling**: Fields are partitioned into basal and derived categories. Derived fields (`:=`) maintain a strictly guaranteed relationship with their dependencies through continuous topological re-evaluation.
*   **Fleet-Level Triggers**: Support for `any` and `all` conditions across entity instances (e.g., `when any Moto.battery becomes < 10`).
*   **Aggregates**: Structural `aggregate` blocks for defining fleet-wide facts like `avg`, `sum`, `min`, `max`, and `count` with O(1) read performance.
*   **Structured Alerting**: Native `alert` actions with severity levels, source tracking, and consistent metadata.
*   **Recovery Logic (`on clear`)**: Automatic firing of actions when a rule condition is no longer met.
*   **Rule Cooldowns**: Silence periods to prevent alert storms from flapping sensors.
*   **Historical State Access**: Use the `prev()` keyword to access a field's value from the previous engine tick.
*   **External Entities & Adapters**: Native support for synchronizing state with external sources (e.g., Supabase, MQTT) via the Adapter protocol.
*   **List & Collection Types**: First-class support for list types (`type[]`), list literals (`[...]`), and element indexing (`list[0]`).
*   **Pure Functions (`fn`)**: Stateless, side-effect-free functions for complex logic encapsulation.
*   **Language Server Protocol (LSP)**: Full IDE support with real-time error squiggles, hover tooltips, and document symbol navigation.

---

## 2. Documentation

*   **[Lumina Complete Guide (v1.5)](./docs/Lumina_Complete_Guide.md)**: The definitive 34-chapter guide to the language, architecture, and tooling.
*   **[Language Specification](./docs/SPEC.md)**: Formal EBNF grammar.
*   **[Architecture Overview](./docs/ARCHITECTURE.md)**: Deep dive into the reactive engine and snapshot VM.

---

## 3. Architecture and Execution Model

The Lumina compiler and runtime pipeline is implemented in Rust, exploiting zero-cost abstractions and strict memory safety guarantees. Every program flows through this pipeline:

1.  **`lumina-lexer`**: A high-throughput deterministic finite automaton tokenizer.
2.  **`lumina-parser`**: A hybrid recursive-descent and Pratt parser.
3.  **`Module Loader`**: Resolves `import` statements and builds a unified AST.
4.  **`lumina-analyzer`**: Enforces static type safety and validates evaluation order.
5.  **`lumina-runtime`**: The core Snapshot VM (`Evaluator`) handling state allocation, fleet tracking, and temporal scheduling.
6.  **`lumina-lsp`**: Provides real-time developer feedback and cross-file navigation.

---

## 4. Language Specification (v1.5 Example)

### 4.1 Entity Schemas & Rules

```lua
import "types.lum"

entity Moto {
  battery: Number
  isCritical := battery < 5
}

-- Aggregate fleet state reactively
aggregate FleetStatus over Moto {
  avgBattery := avg(battery)
  onlineCount := count(isOnline)
}

-- Rule with structured alert, recovery logic, and cooldown
rule Overheat when any Temp.isHigh becomes true cooldown 5m {
  alert severity: "critical", message: "Unit is overheating!"
} on clear {
  alert severity: "resolved", message: "Temperature stabilized"
}
```

---

## 5. v1.6 Roadmap (Future Work)
*   **LSP Refactoring**: Go-to-definition and symbol renaming across modules.
*   **Visual Debugger**: Live inspection of the dependency graph in the Playground.
*   **Native Adapters**: Production-ready adapters for MQTT and Supabase.

---

## 5. Compilation and Usage

### Prerequisites
* Rust Toolchain (`rustc` >= 1.70.0)
* Cargo Build Manager

### 5.1 Command Line Interface (CLI)
```bash
# Build the CLI
cargo build --release -p lumina-cli

# Execute a Lumina program
cargo run -p lumina-cli -- run main.lum

# Interactive REPL
cargo run -p lumina-cli -- repl
```

### 5.2 Language Server (LSP)
Install the LSP globally to enable IDE features in VS Code:
```bash
cargo install --path crates/lumina-lsp
```

---

## 6. Foreign Function Interface (FFI)

Lumina implements a secure C ABI for integration into external environments.

```bash
# Build the shared library
cargo build --release -p lumina_ffi
```

---

## 7. WebAssembly Integration

Lumina cross-compiles to WebAssembly, operating inside standard JavaScript runtimes.

```bash
cd crates/lumina-wasm
wasm-pack build --target web --release
```

---

## 8. Development & Testing
Before committing to the codebase, read our detailed [CONTRIBUTING.md](./CONTRIBUTING.md) guide.

Run the full specification regression suite:
```bash
cargo test --workspace
```

## License
This software and associated documentation files are distributed under the MIT License. Reference [`LICENSE`](LICENSE) for complete legal stipulations.
