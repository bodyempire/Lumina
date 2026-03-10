# Lumina: A Declarative and Reactive Programming Language for State-Driven Systems

## Abstract

Lumina is a statically typed, declarative, and reactive programming language engineered in Rust, designed specifically for modeling complex state-driven systems. Unlike traditional imperative languages where state transitions are manually coordinated, Lumina employs a continuous evaluation model. Domain entities are defined with intrinsic storage and derived mathematical relationships. Reactive rule cascades autonomously compute state changes, temporal triggers enforce duration and interval-based logic, and invariant assertions ensure computational consistency. The resulting execution model provides a robust framework for asynchronous and deterministic system state resolution, suitable for both embedded logic and high-level behavioral orchestration.

## 1. Introduction

Modern software architecture frequently struggles with the synchronization of distributed state and the unintended consequences of imperative control flow across varying subsystems. Lumina mitigates these systemic challenges by inverting the control paradigm. Instead of explicitly instructing the system *when* and *how* to update context, developers declare the inherent relationships between properties and the subsequent reactive triggers that must fire upon specific state variations.

### 1.1 Key Paradigms
*   **Declarative Entity Modeling**: Fields are partitioned into basal and derived categories. Derived fields (`:=`) maintain a strictly guaranteed relationship with their dependencies through continuous topological re-evaluation.
*   **Deterministic Reactive Automata**: State mutations are constrained to atomic intervals. The runtime leverages a directed acyclic evaluation graph to cascade variable updates without triggering divergent recursion limits.
*   **Temporal Logic Triggers**: The inclusion of `for` and `every` reactive clauses enables the execution of interval-based and sustained-duration logic, natively offloading manual timer management to the runtime orchestrator.
*   **Robust Boundary and Type Checking**: Compile-time semantic analysis combined with runtime bounds constraints (`@range`) minimizes invalid state allocations.

## 2. Architecture and Execution Model

The Lumina compiler and runtime (`lumina-cli`) are implemented in Rust, exploiting zero-cost abstractions and strict memory safety guarantees. The underlying virtual machine utilizes a snapshot-and-rollback transactional mechanism for state updates, preserving atomicity in event application.

### 2.1 Toolchain Components
*   `lumina-lexer`: A high-throughput deterministic finite automaton tokenizer powered by `logos`.
*   `lumina-parser`: A hybrid recursive-descent and Pratt parser optimized for context-free grammar evaluation and operator precedence.
*   `lumina-analyzer`: Constructs a dependency graph, enforces static type safety, and detects circular dependency vectors before runtime.
*   `lumina-runtime`: The core evaluation substrate handling state allocation, rule execution, and temporal scheduling.

## 3. Language Specification

### 3.1 Entity Schemas

Entities define the polymorphic structure of state contexts within the runtime environment.

```lua
entity Moto {
  @doc "Battery capacity measured in watt-hours (Wh)"
  @range 0 to 100
  battery: Number
  isBusy: Boolean
  status: Text
  
  -- Derived fields autonomously calculate their value topologically
  isLowBattery := battery < 20
  isAvailable  := not isBusy and battery > 15
}
```

### 3.2 Polymorphic Initialization and Instantiation

Instantiation within Lumina requires the explicit declaration of non-derived fields. Derived fields are structurally computed immediately upon object allocation.

```lua
let transport = Moto { battery: 80, isBusy: false, status: "available" }
```

### 3.3 Rule Cascades and Forward Chaining

The control logic in Lumina relies upon the `rule` keyword, evaluating preconditions incrementally.

```lua
rule "Low Power Heuristics" {
  when Moto.isLowBattery becomes true
  then show "SYSTEM ALERT: Battery reserves have dropped below structural thresholds."
}
```

The trigger modifier `becomes` ensures rule execution operates strictly on edge transitions (low to high), minimizing computational redundancy during steady-state sequences.

### 3.4 Temporal Semantics

Lumina provides native capabilities for evaluating sustained conditions and periodic intervals.

```lua
rule "Thermodynamic Cooldown Constraint" {
  when Sensor.isHot becomes true for 30 s
  then show "Diagnostic: Thermal mitigation failed. Temperature remains sustained over timeframe."
}

rule "Diagnostic Telemetry Interval" {
  every 1 h
  then show "Transmitting hourly systemic telemetry."
}
```

### 3.5 Primitive Types and Operators

| Type Classification | Permitted Inputs |
|---------------------|------------------|
| `Number`            | 64-bit IEEE 754 Floating-Point Precision (`42`, `3.14159`) |
| `Text`              | UTF-8 encoded String Literals (`"Diagnostic Start"`) |
| `Boolean`           | `true`, `false` |

**Operators**:
Lumina supports a full spectrum of Boolean algebra and arithmetic operators (`+`, `-`, `*`, `/`, `==`, `!=`, `>`, `<`, `>=`, `<=`, `and`, `or`, `not`), as well as the specialized state transition operator (`becomes`). String interpolation is intrinsically supported natively (e.g., `"{entity.variable}"`).

## 4. Compilation and Environmental Configuration

### 4.1 System Prerequisites
*   Rust Toolchain (`rustc` >= 1.70.0)
*   Cargo Build Manager

### 4.2 Installation

**Quick Install (Linux/macOS)**

The simplest way to install the Lumina toolchain locally is via our installation script, which orchestrates the downloading, compilation, and PATH provisioning automatically.

```bash
curl -fsSL https://raw.githubusercontent.com/bodyempire/Lumina/main/install.sh | bash
```

**Building from Source**

If you prefer to compile manually:

```bash
# Clone the repository and compile the target binary
git clone https://github.com/bodyempire/Lumina.git
cd Lumina
cargo build --release --bin lumina-cli

# Symlink the binary into the global execution path
cp target/release/lumina-cli ~/.local/bin/lumina
```

### 4.3 Command Line Interface (CLI)
```bash
# Execute abstract syntax tree evaluation
lumina run target_program.lum

# Perform static analysis and dependency validation
lumina check target_program.lum

# Initiate interactive Read-Eval-Print Loop (REPL)
lumina repl
```

## 5. System Interoperability (FFI)

Lumina implements a Foreign Function Interface (FFI) permitting native linkage and execution from external environments such as Python or C via a cross-platform shared library.

```python
from lumina_py import LuminaRuntime

# Initialize runtime and parse logical schema
rt = LuminaRuntime.from_source("""
entity Iterator { index: Number }
let Iterator = Iterator { index: 0 }
""")

# Apply extrinsic variables and export delta state
rt.apply_event("Iterator", "index", 42)
print(rt.export_state())
```

## 6. WebAssembly Integration

For sandboxed and distributed deployments, Lumina cross-compiles to WebAssembly (`wasm32-unknown-unknown`), operating robustly within the V8 Javascript execution environment.

```bash
cd crates/lumina-wasm
wasm-pack build --target web --out-dir pkg --release
```

## 7. Quality Assurance and Testing

The repository maintains a rigorous test suite covering lexer validation, structural parsing, AST transformations, and runtime semantic execution.

```bash
cargo test --workspace
```

## License

This software and associated documentation files are distributed under the MIT License. Reference [`LICENSE`](LICENSE) for complete legal stipulations.
