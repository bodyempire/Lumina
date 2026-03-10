# ⚡ Lumina

**A declarative, reactive programming language built in Rust.**

Lumina lets you define entities, declare reactive rules, and watch state propagate automatically. When a field changes, derived fields recompute, rules fire, and the system stays consistent — all without imperative control flow.

```lua
entity Moto {
  battery: Number
  isBusy: Boolean
  status: Text
  isLowBattery := battery < 20
  isAvailable  := not isBusy and battery > 15
}

rule "low battery alert" {
  when Moto.isLowBattery becomes true
  then show "ALERT: battery low!"
}

let Moto = Moto { battery: 80, isBusy: false, status: "available" }
update Moto.battery to 12
-- → ALERT: battery low!
```

## Features

- **Declarative entities** with stored and derived fields
- **Reactive rules** that fire on state transitions (`becomes`)
- **Temporal triggers** — `for` duration and `every` interval rules
- **Type checking** with compile-time error detection
- **`@range` constraints** with automatic validation
- **String interpolation** — `"Hello {person.name}"`
- **Snapshot & rollback** for atomic state changes
- **Browser playground** via WebAssembly
- **C FFI** — callable from Python, C, Go, and any language
- **Interactive REPL** for live experimentation

## Installation

```bash
git clone https://github.com/bodyempire/Lumina.git
cd Lumina
cargo build --release --bin lumina-cli

# Optional: install globally
cp target/release/lumina-cli ~/.local/bin/lumina
```

## Usage

```bash
# Run a Lumina program
lumina run myprogram.lum

# Type-check without running
lumina check myprogram.lum

# Interactive REPL
lumina repl
```

## Language Reference

### Entities

```lua
entity Person {
  name: Text
  age: Number
  isAdult := age >= 18          -- derived field
  grade   := if age >= 18 then "adult" else "minor"
}
```

### Metadata

```lua
entity Sensor {
  @doc "Temperature in Celsius"
  @range 0 to 100
  temp: Number
}
```

### Instances & Updates

```lua
let p = Person { name: "Isaac", age: 25 }
update p.age to 26
show "Name: {p.name}, Grade: {p.grade}"
```

### Rules

```lua
rule "overheat" {
  when Sensor.temp becomes true
  then show "WARNING: overheating!"
}

rule "cooldown timer" {
  when Sensor.isHot becomes true for 30 s
  then show "Still hot after 30s"
}

rule "periodic check" {
  every 1 h
  then show "Hourly status check"
}
```

### Types

| Type | Example |
|------|---------|
| `Number` | `42`, `3.14` |
| `Text` | `"hello"` |
| `Boolean` | `true`, `false` |

### Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/` |
| Comparison | `==`, `!=`, `>`, `<`, `>=`, `<=` |
| Logical | `and`, `or`, `not` |
| Transition | `becomes` |

## Browser Playground

Try Lumina in your browser — no installation needed:

```bash
# Build the WASM module
cd crates/lumina-wasm
wasm-pack build --target web --out-dir pkg --release

# Serve the playground
cd ../..
python3 -m http.server 8080
# Open http://localhost:8080/playground/index.html
```

## FFI — Use Lumina from Python

```python
from lumina_py import LuminaRuntime

rt = LuminaRuntime.from_source("""
entity Counter { value: Number }
let Counter = Counter { value: 0 }
""")

rt.apply_event("Counter", "value", 42)
print(rt.export_state())
```

## Project Structure

```
Lumina/
├── crates/
│   ├── lumina-lexer/       # Tokenizer (logos)
│   ├── lumina-parser/      # Recursive descent + Pratt parsing
│   ├── lumina-analyzer/    # Type checker & dependency graph
│   ├── lumina-runtime/     # Evaluator, store, rules, timers
│   ├── lumina-cli/         # Command-line interface
│   ├── lumina-ffi/         # C shared library + Python bindings
│   └── lumina-wasm/        # WebAssembly module
├── playground/             # Browser IDE
└── tests/spec/             # Integration test programs
```

## Running Tests

```bash
cargo test --workspace
```

## License

MIT License — see [LICENSE](LICENSE) for details.
