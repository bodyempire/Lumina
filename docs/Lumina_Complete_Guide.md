# Lumina: The Complete Guide (v1.5) 🛰️

_"Describe what is true. Lumina figures out what to do."_

---

## Table of Contents

### **1. Fundamentals**
*   **Philosophy**: Truth vs Procedure
*   **Getting Started**: CLI, REPL, and Playground
*   **Entities & Fields**: Declaring State
*   **Data Types**: Primitives and Lists

### **2. Reactive Logic**
*   **Rules & Triggers**: when and every
*   **Condition Logic**: becomes and Durations
*   **Actions**: alerts and State Mutation
*   **Recovery**: on clear Blocks

### **3. Advanced Context**
*   **Historical State**: The prev() Operator
*   **Pure Functions**: Encapsulating Logic
*   **Fleet Operations**: any/all and aggregates

### **4. Integration & Ecosystem**
*   **Modules**: The import system
*   **External Data**: Adapters and Sync
*   **FFI**: Python, Go, and C
*   **WebAssembly**: Browser Integration
*   **IDE Tooling**: LSP and VS Code

### **5. Reference**
*   **Architecture**: The Snapshot VM
*   **Error Codes**: Diagnostic Reference

---

## **1. Fundamentals**

### **Philosophy: Truth vs Procedure**
Most programming languages are **imperative**: you tell the computer *how* to change state (e.g., `if x > 10 then set y = true`). 
Lumina is **declarative and reactive**: you tell the computer *what is true* (e.g., `y := x > 10`).

In Lumina, you don't "update" derived state. You define the relationship, and the engine ensures it is always correct. This prevents stale data, race conditions, and synchronization bugs.

### **Getting Started**

#### **The CLI**
The `lumina` binary is your main entry point. 
*   `lumina check <file>`: Validates syntax and types.
*   `lumina run <file>`: Executes a program and exports final state.
*   `lumina repl`: Launches the interactive shell.

#### **REPL v2**
The REPL maintains persistent state. You can define an entity, create an instance, and then manipulate it line-by-line.
*   `:state`: View current entity instances as JSON.
*   `:schema`: View all declared entity types.
*   `:load <file>`: Import a script into your session.

#### **Playground v2**
For visual simulation, use the React-based Playground. It provides a live **State Panel**, an **Alert Timeline**, and a **Virtual Clock** to speed up or pause time.

### **Entities & Fields**

Entities are the blueprints for your system state. Fields come in two flavors:
1.  **Stored Fields (`name: Type`)**: These hold data that is set externally (e.g., via a sensor or user input).
2.  **Derived Fields (`name := Expression`)**: These are automatically calculated by the engine whenever their dependencies change.

#### **Metadata**
Add context to fields with @ tags:
#### **List Literals & Indexing**
Create lists with brackets: `let my_list = [1, 2, 3]`. Access elements with 0-based indexing: `val := my_list[0]`.
Out-of-bounds access triggers an **R004** runtime error and immediate state rollback for safety.

> [!NOTE]
> **Technical Note on Types**:
> *   **Number Precision**: Lumina uses the IEEE 754-2008 standard (64-bit floats).
> *   **Text Encoding**: Strings are immutable UTF-8 sequences. Interpolation (`"x is {x}"`) is performed during the propagation phase.
> *   **Empty Lists**: A list type is fixed at declaration (e.g., `Number[]`). Mixing types in a single list literal will trigger an **L004** type mismatch.

### **Deep Dive: The Reactive Graph**
Lumina's performance comes from its static dependency analysis. During the **Analysis Phase**, the compiler builds a Directed Acyclic Graph (DAG) of all derived fields.
*   **Topological Sorting**: Fields are updated in an order that guarantees all dependencies are calculated *before* their dependents.
*   **No Cycles**: If you define `a := b + 1` and `b := a + 1`, the compiler will throw **L004: Circular Dependency** and block execution. This ensures the engine never enters an infinite loop.

---

## **2. Reactive Logic**

### **Rules & Triggers**

Rules are the engine of automation. They are defined with a **trigger** and a set of **actions**.

#### **State-Based Triggers (`when`)**
Fires when a condition evaluates to `true`. Lumina uses **edge detection**: a rule with `when X becomes true` fires exactly once when the transition occurs, not every time the engine ticks.

#### **Time-Based Triggers (`every`)**
Fires on a fixed interval (e.g., `rule "Heartbeat" every 1h`). These are managed by an internal timer heap and fire independently of state changes.

### **Condition Logic**

#### **The `becomes` Keyword**
`becomes` is the secret to stable reactive systems. It compares the current state to the **previous committed state**.
*   `when Sensor.temp becomes > 100`: Fires only at the moment the temperature crosses the threshold.

#### **Durations (`for`)**
Rules can require a condition to hold for a period before firing:
*   `when Moto.isIdle becomes true for 15m`: Prevents "flapping" by waiting for the state to stabilize.

### **Actions: Alerts and Mutation**

When a rule fires, it executes one or more actions:
*   **`show <expr>`**: Prints a message to stdout (or the Playground console).
*   **`update <inst>.<field> to <expr>`**: Modifies a stored field, triggering a new reactive cycle.
*   **`create <Entity> { ... }`**: Spawns a new instance with an auto-generated ID.
*   **`delete <inst>`**: Removes an instance from the store.
*   **`alert severity: "...", message: "..."`**: Sends a structured signal to the host system. High-severity alerts appear in red on the Playground timeline.

### **Recovery: `on clear` Blocks**

Modern monitoring requires knowing when a problem is **resolved**. The `on clear` block executes actions when a rule's condition becomes `false` after having been `true`.
```lumina
rule "High Temp" when Sensor.temp > 100 {
  alert severity: "critical", message: "Overheating!"
} on clear {
  alert severity: "info", message: "Temp stabilized."
}
```

> [!TIP]
> **Technical Note on Recovery**:
> *   **Internal Naming**: The engine automatically labels recovery events as `{rule_name}_clear`. This allows host systems to correlate alerts with their resolutions.
> *   **Edge Detection**: `on clear` only fires on the **falling edge** (true -> false). It does not fire if the rule was already inactive.

### **Deep Dive: The Evaluation Pipeline**
Every engine tick follows a strict order of operations to ensure state consistency:
1.  **Ingest**: External adapters (MQTT, HTTP) push new basal values.
2.  **Snapshot**: The VM takes a bit-level copy of the entire memory store.
3.  **Propagation**: Derived fields are updated based on the topological order.
4.  **Short-Circuiting**: During rule evaluation, `and` / `or` conditions are short-circuited. If a rule condition is `when Sensor.isOnline and Sensor.temp > 100`, the temperature is never even checked if the sensor is offline.
5.  **Edge Detection**: The engine compares the **new state** against the **committed snapshot** from the previous tick to detect `becomes` transitions and `on clear` events.
6.  **Commit**: If no runtime errors (R-codes) occurred, the snapshot is committed and becomes the new "previous state".

```

---

## **3. Advanced Context**

### **Historical State: The `prev()` Operator**

Lumina is unique in its ability to reason about the **past**. The `prev(identifier)` operator returns the value of a field from the **previous successful engine tick**.
*   **Drift Detection**: `drift := temperature - prev(temperature)`
*   **State Transitions**: `becameOnline := isOnline and not prev(isOnline)`

This historical context is maintained automatically by the engine's snapshot manager.

### **Pure Functions: Encapsulating Logic**

For complex calculations, use the `fn` keyword. Functions are **pure**, meaning they cannot access entity state directly—they only operate on their parameters. This ensures they are safe to call from any derived field.
```lumina
fn isCritical(battery: Number, temp: Number) -> Boolean {
  battery < 10 or temp > 120
}
```

### **Fleet Operations: `any`, `all`, and `aggregate`**

In large systems, you often need to reason about a **collection** of entities.

#### **Fleet Triggers (`any` / `all`)**
Rules can monitor the state of an entire entity class:
*   `when any Moto.isCritical becomes true`: Fires if *one or more* units enter a critical state.
*   `when all Moto.isOnline becomes false`: Fires if the *entire fleet* goes dark.

#### **Aggregates**
The `aggregate` block defines top-level values that summarize the entire fleet reactively.
```lumina
aggregate FleetStats over Moto {
  avgBattery := avg(battery)
  offlineCount := count(not isOnline)
}
```
Lumina maintains these values with **O(1)** read performance by updating counters incrementally as units join, leave, or change.

#### **Cooldowns**
Prevent "alert fatigue" by rate-limiting rules. The `cooldown` keyword enforces a silence period after a firing.
*   `rule "Overheat" when Temp > 100 cooldown 15m { ... }`

> [!CAUTION]
> **Technical Note on Advanced Logic**:
> *   **`prev()` Initialization**: On the very first tick of an entity's existence, `prev(x)` is equal to the initial value of `x`.
> *   **Aggregate Performance**: Aggregates are updated **incrementally** during the propagation phase. This ensures `O(1)` read performance even for thousands of instances.
> *   **Function Purity**: Functions cannot access `prev()`, call other functions, or use `any`/`all` triggers. This strictness is enforced at the AST level (**L015**) to ensure safe, side-effect-free evaluation.

### **Deep Dive: The Fleet State Manager**
When you monitor an entire class of entities with `any` / `all` triggers, Lumina uses its internal **Fleet State Manager**.
*   **Transitions**: Fleet triggers only execute their actions when the total fleet state transitions (e.g., from `Any=false` to `Any=true`).
*   **No Context**: Unlike instance rules, fleet rules execute in a **Neutral Context** (`ctx=None`). This means actions like `show` or `update` cannot reference individual unit fields without an aggregate or identifier.
*   **Boolean Only**: Fleet-level triggers only work on fields of type `Boolean`. For numeric triggers, use an `aggregate` first.

---

## **4. Integration & Ecosystem**

### **The Module System: `import`**

As your projects grow, split your code into multiple files. The `import` keyword allows including logic from other `.lum` files.
*   `import "shared/types.lum"`: Imports all entities and functions from the specified path.
*   **Resolution**: Paths are relative to the importing file.
*   **Safety**: Lumina automatically detects and blocks circular imports.

### **External Data: Connecting the Real World**

Lumina acts as a "Digital Twin" of your hardware or database. The **Adapter Protocol** allows you to synchronize entity state with external sources.

#### **External Entity Declaration**
```lumina
external entity PowerMeter {
  voltage: Number
  current: Number
  
  sync: "mqtt://broker.local/sensors/meter1"
  on: poll
  poll_interval: 10s
}
```
*   **`sync`**: The connection string for the adapter.
*   **`on`**: The synchronization mode (`realtime`, `poll`, or `webhook`).

### **Polyglot FFI: Calling Lumina from Any Language**

The Lumina core is written in Rust, but it is accessible from anywhere. Every release ships with a **C ABI shared library** (`.so` / `.dll`).

#### **Official Wrappers**
*   **Python**: A thin `ctypes` wrapper for data science and automation.
*   **Go**: A high-performance `cgo` implementation.
*   **C**: Direct header integration for embedded systems.

### **WebAssembly: Lumina in the Browser**

Lumina compiles to highly optimized **WASM**. This allows you to run the entire reactive engine inside a browser tab with no backend required. It powers the **Lumina Playground**, enabling sub-millisecond local simulation.

> [!IMPORTANT]
> **Technical Note on Integration**:
> *   **C ABI Stability**: The Lumina core exports a stable C-compatible interface (`liblumina.so` / `lumina.dll`). This ensures that your Python or Go apps won't crash when you upgrade the engine.
> *   **Adapter Sync**: While `poll` is the default, the `realtime` mode uses a dedicated background thread for sub-millisecond event ingestion.
> *   **LSP Scope**: The LSP performs **Semantic Analysis** across all imported files, meaning it can detect a type mismatch even if the error occurs in a different module.

### **Deep Dive: The Adapter Architecture**
To connect Lumina to your physical environment, you use **Adapters**.
*   **The Trait**: Internally, an adapter is any Rust struct that implements the `LuminaAdapter` trait.
*   **Registry**: At startup, you call `eval.register_adapter(Box::new(MyAdapter::new()))`.
*   **Tick Lifecycle**: During each engine tick, the `Evaluator` calls `adapter.poll()` to gather new values before starting the propagation phase.

---

## **5. Reference**

### **Architecture: The Snapshot VM**

Lumina is built on a custom **Snapshot-based Virtual Machine**. Every update cycle is atomic:
1.  **Ingest**: Collect external events or timer triggers.
2.  **Snapshot**: Copy the entire store state.
3.  **Propagate**: Re-evaluate all derived fields and rules.
4.  **Validate**: Ensure no invariants (like `@range`) are broken.
5.  **Commit**: Make the changes permanent and sync `prev()` store.

If any step fails (e.g., division by zero or recursion limit reached), the engine performs a **zero-cost rollback** to the snapshot.

### **Error Reference (L-Codes & R-Codes)**

Lumina diagnostics provide clear, actionable feedback with Rust-style source pointers.

#### **Analysis Errors (L-Codes)**
*   **L001-L003**: Naming and cycle detection errors.
*   **L004**: Circular dependency in derived fields.
*   **L005-L006**: Duplicate entity or field names.
*   **L011-L015**: Function declaration and purity violations.
*   **L018**: Import not supported in single-file (WASM) mode.
*   **L026**: Unknown entity referenced in fleet trigger.
*   **L027**: Fleet trigger field must be Boolean.

#### **Runtime Errors (R-Codes)**
*   **R001**: Null or deleted instance access.
*   **R002**: Mathematical error (e.g., Division by Zero).
*   **R003**: Recursion depth exceeded (default: 100).
*   **R004**: List index out of bounds.
*   **R005**: Field not found on existing instance.
*   **R006**: `@range` metadata violation.
*   **R009**: Illegal attempt to write to a derived field.

---
_Lumina v1.5 Definitive Documentation | 2026 | Isaac Ishimwe_
