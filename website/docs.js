// ─── Lumina Documentation Content ───
// All documentation tabs and examples are defined here.

export const DOCS = {
  getting_started: {
    title: "Getting Started",
    sections: [
      {
        heading: "Welcome to Lumina",
        text: `Lumina is a statically typed, declarative, and reactive language designed from the ground up to model state-driven systems, IoT networks, smart environments, and data pipelines. With Lumina, you don't write control flow — you write truth, and the runtime enforces it.`,
      },
      {
        heading: "Installation",
        code: `# Clone and build from source
git clone https://github.com/IshimweIsaac/Lumina
cd Lumina && cargo build --release

# Run the REPL
cargo run -p lumina-cli -- repl`,
        file: "terminal"
      },
      {
        heading: "Your First Lumina Program",
        text: `In Lumina, data is structured into <code>entity</code> definitions. Derived fields update automatically when inputs change, and <code>rule</code> blocks fire when precise state edges occur.`,
        code: `entity Switch {
  isOn: Boolean
}

entity Lightbulb {
  switchA: Switch
  switchB: Switch
  
  -- The light is on if EITHER switch is on
  isLit := switchA.isOn or switchB.isOn
}

let s1 = Switch { isOn: false }
let s2 = Switch { isOn: false }
let light = Lightbulb { switchA: s1, switchB: s2 }

rule "Light Alert" {
  when Lightbulb.isLit becomes true
  then show "The light turned ON!"
}

-- Turn a switch on -> rule triggers automatically
update s1.isOn to true`,
        file: "hello_world.lum"
      }
    ]
  },
  entities: {
    title: "Entities & Fields",
    sections: [
      {
        heading: "Entity Declaration",
        text: `Entities define the structure of state contexts. Fields are partitioned into <strong>stored</strong> (basal) and <strong>derived</strong> categories. Stored fields hold mutable state. Derived fields (<code>:=</code>) are continuously re-evaluated whenever their dependencies change, using Kahn's topological sorting algorithm.`,
        code: `entity Moto {
  @doc "Battery capacity in watt-hours"
  @range 0 to 100
  battery: Number
  isBusy: Boolean
  status: Text

  -- Derived fields (always up-to-date)
  priority    := if battery < 10 then 1 else 2
  isCritical  := battery < 5
  isAvailable := not isBusy and battery > 15
  description := "Status: {status} ({battery}%)"
}`,
        file: "entity.lum"
      },
      {
        heading: "Metadata Annotations",
        text: `Lumina supports metadata annotations on fields to add constraints and documentation. These are checked at both analysis time and runtime.`,
        table: [
          ["Annotation", "Purpose", "Example"],
          ["<code>@doc</code>", "Documents a field (shown in LSP hover)", '<code>@doc "Temperature in Celsius"</code>'],
          ["<code>@range</code>", "Constrains a Number field to a range", "<code>@range 0 to 100</code>"],
          ["<code>@affects</code>", "Declares side-effect intent", "<code>@affects status, priority</code>"]
        ]
      },
      {
        heading: "Instance Creation",
        text: `Instances are created with the <code>let</code> keyword. All stored fields must be provided. Derived fields are computed automatically.`,
        code: `let moto1 = Moto { battery: 80, isBusy: false, status: "available" }
let moto2 = Moto { battery: 15, isBusy: true, status: "in-use" }

show "Moto1 available: {moto1.isAvailable}"  -- true
show "Moto2 critical: {moto2.isCritical}"    -- false`,
        file: "instances.lum"
      },
      {
        heading: "Updating State",
        text: `State is mutated via <code>update</code> actions. Only stored fields can be updated — attempting to write to a derived field raises <code>R009</code>. After an update, all dependent derived fields re-evaluate automatically.`,
        code: `update moto1.battery to 3
-- isCritical is now true (derived re-evaluated)
-- priority is now 1

update moto1.status to "maintenance"
-- description auto-updates to "Status: maintenance (3%)"`,
        file: "updates.lum"
      },
      {
        heading: "prev() — Previous Field Value",
        badge: "v1.5",
        text: `The <code>prev()</code> function gives derived fields access to the <strong>last committed value</strong> of any stored field, enabling drift detection, rate-of-change monitoring, and trend analysis.`,
        code: `entity Sensor {
  reading: Number

  -- State drift detection
  delta      := reading - prev(reading)
  isSpike    := delta > 20
  trend      := if reading > prev(reading) then "rising"
                else if reading < prev(reading) then "falling"
                else "stable"
}`,
        file: "prev.lum"
      }
    ]
  },

  rules: {
    title: "Rules & Reactive Triggers",
    sections: [
      {
        heading: "Basic Rules with becomes",
        text: `Rules fire on <strong>edge transitions</strong>. The <code>becomes</code> keyword ensures a rule only triggers when a condition flips from false to true (positive-edge detection), preventing redundant execution.`,
        code: `rule "Low Battery Alert" {
  when Moto.isLowBattery becomes true
  then show "ALERT: battery low"
  then update Moto.status to "charging"
}

rule "Critical Shutdown" {
  when Moto.isCritical becomes true
  then update Moto.status to "maintenance"
  then show "CRITICAL: pulled from service"
}`,
        file: "rules.lum"
      },
      {
        heading: "Temporal Logic — for and every",
        text: `<code>for</code> requires a condition to be sustained for a duration before firing. <code>every</code> creates interval-based recurring rules. Duration units: <code>s</code> (seconds), <code>m</code> (minutes), <code>h</code> (hours), <code>d</code> (days).`,
        code: `-- Fires ONLY after condition holds for 5 minutes
rule "Sustained Critical" {
  when Sensor.isCritical becomes true for 5 m
  then show "EMERGENCY: Sustained anomaly for 5+ minutes"
}

-- Fires every 30 seconds, regardless of state
rule "Heartbeat" {
  every 30 s
  then show "System alive: {Sensor.reading}°C"
}`,
        file: "temporal.lum"
      },
      {
        heading: "alert + on clear",
        badge: "v1.5",
        text: `<code>alert</code> produces structured signals with severity, source, and message. <code>on clear</code> fires automatically when the triggering condition recovers — expressing the complete lifecycle of a monitoring event.`,
        code: `rule "Overheat" {
  when Sensor.isCritical becomes true
  alert severity: "critical",
        source: "temp-sensor",
        message: "overheating: {Sensor.reading}°C"
} on clear {
  alert severity: "resolved",
        source: "temp-sensor",
        message: "temperature recovered"
}`,
        file: "alerts.lum"
      },
      {
        heading: "Rule Cooldown",
        badge: "v1.5",
        text: `<code>cooldown</code> declares a silence period to prevent alert storms. After firing, the rule cannot fire again for the specified duration. Cooldown is per-instance for parameterized rules.`,
        code: `-- Cannot fire again for 5 minutes
rule "Battery Alert"
  when Moto.isLowBattery becomes true
  cooldown 5 m {
    alert severity: "warning",
          source: "fleet",
          message: "low battery: {Moto.battery}%"
  }`,
        file: "cooldown.lum"
      },
      {
        heading: "when any / when all",
        badge: "v1.5",
        text: `Fleet-level reactive conditions. <code>when any</code> fires when at least one instance satisfies the condition. <code>when all</code> fires when every instance satisfies it.`,
        code: `-- At least one moto is low
rule "Any Low" {
  when any Moto.isLowBattery becomes true
  then show "WARNING: at least one moto is low"
}

-- Entire fleet is offline
rule "Fleet Offline" {
  when all Moto.isOnline becomes false for 5 m
  alert severity: "critical",
        source: "fleet",
        message: "entire fleet offline 5+ minutes"
} on clear {
  alert severity: "resolved",
        source: "fleet",
        message: "fleet back online"
}`,
        file: "fleet_rules.lum"
      }
    ]
  },

  functions: {
    title: "Pure Functions",
    sections: [
      {
        heading: "fn — Pure, Stateless Functions",
        text: `Functions are declared with <code>fn</code>. They are pure expressions — no access to entity state, no side effects. They can be called from derived fields and rule conditions. Supported types: <code>Number</code>, <code>Text</code>, <code>Boolean</code>.`,
        code: `fn clamp(value: Number, min: Number, max: Number) -> Number {
  if value < min then min
  else if value > max then max
  else value
}

fn classify(battery: Number) -> Text {
  if battery < 5 then "critical"
  else if battery < 20 then "low"
  else if battery < 80 then "normal"
  else "full"
}

fn is_healthy(temp: Number, humidity: Number) -> Boolean {
  temp > 15 and temp < 35 and humidity > 30 and humidity < 80
}`,
        file: "functions.lum"
      },
      {
        heading: "Using Functions in Entities",
        text: `Functions integrate naturally with derived fields, keeping complex logic reusable and testable.`,
        code: `entity Moto {
  battery: Number
  safeBattery := clamp(battery, 0, 100)
  category    := classify(battery)
}

entity Environment {
  temperature: Number
  humidity: Number
  isHealthy := is_healthy(temperature, humidity)
}`,
        file: "fn_usage.lum"
      },
      {
        heading: "Function Rules",
        table: [
          ["Rule", "Error Code", "Description"],
          ["Duplicate fn name", "L011", "Two fn declarations share the same name"],
          ["Unknown fn called", "L012", "Call references a fn that was not declared"],
          ["Argument count mismatch", "L013", "Wrong number of arguments"],
          ["Argument type mismatch", "L004", "Argument type doesn't match parameter type"],
          ["Return type mismatch", "L014", "Body type doesn't match -> return type"],
          ["fn accesses entity state", "L015", "fn bodies cannot reference entity instances"]
        ]
      }
    ]
  },

  types: {
    title: "Type System & Lists",
    sections: [
      {
        heading: "Static Type System",
        text: `Lumina is statically typed. Every field and expression has a type known at analysis time. Type mismatches are caught before execution.`,
        table: [
          ["Type", "Description", "Literal Example"],
          ["<code>Number</code>", "64-bit floating point (f64)", "<code>42</code>, <code>3.14</code>, <code>-10</code>"],
          ["<code>Text</code>", 'String of characters', '<code>"hello"</code>, <code>"Battery: {x}%"</code>'],
          ["<code>Boolean</code>", "Logical true or false", "<code>true</code>, <code>false</code>"],
          ["<code>Number[]</code>", "Ordered list of numbers", "<code>[80, 60, 40, 20]</code>"],
          ["<code>Text[]</code>", "Ordered list of strings", '<code>["north", "south"]</code>'],
          ["<code>Boolean[]</code>", "Ordered list of booleans", "<code>[true, false, true]</code>"]
        ]
      },
      {
        heading: "String Interpolation",
        text: `Embed any expression inside text literals with <code>{expr}</code>. Works with Numbers, Text, and Booleans. Numbers are automatically formatted. No nested interpolation allowed (L019).`,
        code: `show "Battery level: {moto1.battery}%"
show "Half charge: {moto1.battery / 2}"
show "Status: {moto1.isAvailable}"

entity Report {
  label: Text
  battery: Number
  summary := "Unit [{label}] battery={battery}%"
}`,
        file: "interpolation.lum"
      },
      {
        heading: "List Types",
        text: `Lists are ordered, immutable collections. <code>append()</code> returns a new list — never mutates in place. Out-of-bounds access triggers <code>R004</code> and automatic rollback.`,
        code: `entity Fleet {
  readings: Number[]
  labels:   Text[]

  count   := len(readings)
  lowest  := min(readings)
  highest := max(readings)
  total   := sum(readings)
}

let fleet = Fleet {
  readings: [80, 60, 40, 20],
  labels: ["north", "south", "east", "west"]
}

show fleet.readings[0]  -- 80
show fleet.count         -- 4`,
        file: "lists.lum"
      },
      {
        heading: "Built-in List Functions",
        table: [
          ["Function", "Signature", "Description"],
          ["<code>len(list)</code>", "T[] → Number", "Number of elements"],
          ["<code>min(list)</code>", "Number[] → Number", "Minimum value (R004 if empty)"],
          ["<code>max(list)</code>", "Number[] → Number", "Maximum value (R004 if empty)"],
          ["<code>sum(list)</code>", "Number[] → Number", "Sum of all values"],
          ["<code>append(list, val)</code>", "(T[], T) → T[]", "Returns new list with value added"],
          ["<code>head(list)</code>", "T[] → T", "First element (R004 if empty)"],
          ["<code>tail(list)</code>", "T[] → T[]", "All except first (R004 if empty)"],
          ["<code>at(list, i)</code>", "(T[], Number) → T", "Element at index i (R004 if OOB)"]
        ]
      },
      {
        heading: "Operators",
        table: [
          ["Category", "Operators", "Precedence (high→low)"],
          ["Unary", "<code>not</code>, <code>-</code> (negation)", "Highest"],
          ["Multiplicative", "<code>*</code>, <code>/</code>, <code>mod</code>", "High"],
          ["Additive", "<code>+</code>, <code>-</code>", "Medium"],
          ["Comparison", "<code>==</code> <code>!=</code> <code>&gt;</code> <code>&lt;</code> <code>&gt;=</code> <code>&lt;=</code>", "Low"],
          ["Logical AND", "<code>and</code>", "Lower"],
          ["Logical OR", "<code>or</code>", "Lowest"]
        ]
      }
    ]
  },

  modules: {
    title: "Module System",
    sections: [
      {
        heading: "import — Multi-File Projects",
        text: `Split programs across files with the <code>import</code> keyword. All entity declarations, fn declarations, and let bindings from the imported file become visible. Paths are resolved relative to the importing file.`,
        code: `-- File: shared/types.lum
entity Moto {
  battery: Number
  isLowBattery := battery < 20
}

fn clamp(v: Number, lo: Number, hi: Number) -> Number {
  if v < lo then lo else if v > hi then hi else v
}

-- File: fleet_os.lum
import "shared/types.lum"

let moto1 = Moto { battery: 80 }

rule "Alert" {
  when Moto.isLowBattery becomes true
  then show "Alert: {moto1.battery}%"
}`,
        file: "modules.lum"
      },
      {
        heading: "Resolution Rules",
        table: [
          ["Scenario", "Behavior", "Error"],
          ["Relative path", 'Resolves from current file directory', "—"],
          ["Circular import", "A imports B, B imports A (directly or transitively)", "L016"],
          ["File not found", "Path does not exist on disk", "L017"],
          ["Duplicate declaration", "Imported file redeclares existing entity", "L001"],
          ["WASM / Playground", "import not supported in single-file mode", "L018"]
        ]
      }
    ]
  },

  external: {
    title: "External Entities",
    sections: [
      {
        heading: "Connecting to Real Data Sources",
        badge: "v1.5",
        text: `External entities connect Lumina's reactive rules to real-world sensors, devices, and APIs. An adapter connects an external entity to a data source. When new data arrives, the runtime propagates it exactly as if a rule action had fired.`,
        code: `external entity TemperatureSensor {
  reading: Number
  isOnline: Boolean
  isCritical := reading > 90
  isWarning  := reading > 75 and reading <= 90
} sync on reading

rule "Overheat" {
  when TemperatureSensor.isCritical becomes true
  alert severity: "critical",
        source: "temp-sensor",
        message: "overheating: {TemperatureSensor.reading}°C"
}`,
        file: "external.lum"
      },
      {
        heading: "Built-in Adapters",
        table: [
          ["Adapter", "Protocol", "Use Case"],
          ["<code>MqttAdapter</code>", "MQTT v3.1 / v5", "IoT sensors, industrial equipment, smart devices"],
          ["<code>HttpPollAdapter</code>", "HTTP GET (JSON)", "REST APIs, cloud sensor platforms"],
          ["<code>ChannelAdapter</code>", "Rust mpsc channel", "Embedding Lumina in a Rust application"],
          ["<code>StaticAdapter</code>", "In-memory queue", "Testing, simulation, historical replay"],
          ["<code>FileWatchAdapter</code>", "File system watch", "Log files, CSV exports, config changes"]
        ]
      },
      {
        heading: "sync on — Controlling Propagation",
        text: `The <code>sync on</code> clause declares which field change triggers rule propagation. Without it, every adapter poll triggers a full cycle. With <code>sync on</code>, Lumina only reacts when meaningful data changes.`,
        code: `-- Only propagate when reading changes
external entity Sensor { reading: Number } sync on reading

-- Only propagate when connection status changes
external entity Device {
  signalStrength: Number
  isOnline: Boolean
} sync on isOnline`,
        file: "sync.lum"
      }
    ]
  },

  fleet: {
    title: "Fleet & Aggregates",
    sections: [
      {
        heading: "aggregate — Fleet-Wide Derived Values",
        badge: "v1.5",
        text: `The <code>aggregate</code> block declares fleet-level truths as named reactive values. They recompute automatically whenever any instance of the target entity changes — no polling, no explicit refresh.`,
        code: `aggregate FleetStatus over Moto {
  avgBattery   := avg(battery)
  minBattery   := min(battery)
  onlineCount  := count(isOnline)
  anyLow       := any(isLowBattery)
  allOnline    := all(isOnline)
  totalUnits   := count()
}

rule "Fleet Critical" {
  when FleetStatus.anyLow becomes true
  alert severity: "warning",
        source: "fleet",
        message: "fleet avg battery: {FleetStatus.avgBattery}%"
}`,
        file: "aggregate.lum"
      },
      {
        heading: "Aggregate Functions",
        table: [
          ["Function", "Input Type", "Returns"],
          ["<code>avg(field)</code>", "Number", "Mean across all instances"],
          ["<code>min(field)</code>", "Number", "Minimum across all instances"],
          ["<code>max(field)</code>", "Number", "Maximum across all instances"],
          ["<code>sum(field)</code>", "Number", "Sum across all instances"],
          ["<code>count()</code>", "any", "Total number of instances"],
          ["<code>count(field)</code>", "Boolean", "Number where field is true"],
          ["<code>any(field)</code>", "Boolean", "True if any instance is true"],
          ["<code>all(field)</code>", "Boolean", "True if all instances are true"]
        ]
      }
    ]
  },

  tooling: {
    title: "Tooling & Integration",
    sections: [
      {
        heading: "CLI Commands",
        table: [
          ["Command", "Description"],
          ["<code>cargo run -p lumina-cli -- run file.lum</code>", "Execute a Lumina program"],
          ["<code>cargo run -p lumina-cli -- check file.lum</code>", "Static analysis without execution"],
          ["<code>cargo run -p lumina-cli -- repl</code>", "Interactive REPL with persistent state"]
        ]
      },
      {
        heading: "REPL v2 — Inspector Commands",
        text: `The REPL maintains a single Evaluator across all inputs and supports multi-line constructs via brace-depth tracking.`,
        table: [
          ["Command", "Description"],
          ["<code>:state</code>", "Print current state as pretty JSON"],
          ["<code>:schema</code>", "Print all declared entities and fields"],
          ["<code>:load file.lum</code>", "Load and execute a file into the session"],
          ["<code>:save file.lum</code>", "Save session source to a file"],
          ["<code>:clear</code>", "Reset the session"],
          ["<code>:help</code>", "List all commands"],
          ["<code>:quit</code>", "Exit the REPL"]
        ]
      },
      {
        heading: "Language Server (LSP)",
        badge: "v1.5",
        text: `The <code>lumina-lsp</code> binary provides real-time language intelligence over stdin/stdout. Compatible with VS Code, Neovim, Emacs, Helix, Zed, and any LSP client.`,
        table: [
          ["Capability", "What You See"],
          ["Real-time diagnostics", "Red squiggles on errors as you type"],
          ["Hover tooltips", "Field type and @doc text on hover"],
          ["Go-to-definition", "Ctrl+Click jumps to entity declaration"],
          ["Document symbols", "Outline panel lists entities and rules"],
          ["Completion", "Entity and field name suggestions"]
        ]
      },
      {
        heading: "VS Code Extension",
        text: `First-class <code>.lum</code> file support: TextMate grammar for syntax highlighting, snippet completions for entity/rule boilerplate, bracket matching, and comment toggling. Install via VSIX or the extensions marketplace.`,
        code: `# Package and install
cd extensions/lumina-vscode
npm install -g @vscode/vsce
vsce package
code --install-extension lumina-language-0.1.0.vsix`,
        file: "terminal"
      },
      {
        heading: "WebAssembly",
        text: `Lumina cross-compiles to WASM via <code>wasm-pack</code> and <code>wasm-bindgen</code>. The playground runs entirely in the browser with no server required.`,
        code: `# Build WASM package
cd crates/lumina-wasm
wasm-pack build --target web --release`,
        file: "terminal"
      },
      {
        heading: "FFI — C / Python / Go",
        text: `Lumina exposes a stable C ABI via <code>liblumina_ffi.so</code>. Python wrapper uses <code>ctypes</code>, Go wrapper uses <code>cgo</code>. JSON is the interchange format for state.`,
        code: `# Build the shared library
cargo build --release -p lumina-ffi

# Python usage
from lumina_py import LuminaRuntime
rt = LuminaRuntime.from_source(source)
rt.apply_event("moto1", "battery", "15")
state = rt.export_state()

# Go usage
rt, _ := lumina.FromSource(source)
defer rt.Close()
rt.ApplyEvent("moto1", "battery", "15")
state, _ := rt.ExportState()`,
        file: "ffi_usage"
      },
      {
        heading: "Error Codes Reference",
        text: `Lumina produces Rust-style error messages with source context, caret highlighting, and help text.`,
        table: [
          ["Code", "Description"],
          ["L001", "Duplicate entity name"],
          ["L002", "Unknown entity referenced"],
          ["L003", "Derived field cycle detected"],
          ["L004", "Type mismatch"],
          ["L005", "Unknown field on entity"],
          ["L006", "Invalid @range metadata"],
          ["L007", "Rule trigger entity unknown"],
          ["L008", "Action targets unknown instance"],
          ["L009", "Duplicate instance name"],
          ["L011–L015", "Function errors (duplicate, unknown, arg/return mismatch, purity)"],
          ["L016", "Circular import"],
          ["L017", "Import file not found"],
          ["L018", "Import not supported in WASM mode"],
          ["L019", "Nested string interpolation"],
          ["L024–L025", "prev() on derived field / nested prev()"],
          ["L026–L027", "when any/all on non-Boolean / zero instances"],
          ["L028–L030", "Alert severity / on clear / payload errors"],
          ["L031–L033", "Aggregate type / name / zero-instance errors"],
          ["L034", "Cooldown duration zero or negative"],
          ["R004", "List index out of bounds"],
          ["R006", "@range violation"],
          ["R009", "Write to derived field"]
        ]
      }
    ]
  }
};

export const EXAMPLES = [
  {
    title: "IoT Fleet Management",
    desc: "Monitor a fleet of electric motos with battery tracking, reactive alerts, and automatic status transitions.",
    code: `entity Moto {
  @range 0 to 100
  battery: Number
  isBusy: Boolean
  status: Text
  isLowBattery := battery < 20
  isCritical   := battery < 5
  isAvailable  := not isBusy and battery > 15
}

rule "Low Battery" {
  when Moto.isLowBattery becomes true
  then show "ALERT: battery low"
}

rule "Critical" {
  when Moto.isCritical becomes true
  then update Moto.status to "maintenance"
  then show "CRITICAL: pulled from service"
}

let moto1 = Moto { battery: 80, isBusy: false, status: "available" }
update moto1.battery to 18
update moto1.battery to 4`,
    file: "fleet.lum",
    tags: ["IoT", "Fleet", "Alerts"]
  },
  {
    title: "Smart Greenhouse",
    desc: "Multi-file greenhouse controller with temperature monitoring, automated irrigation, and daily heartbeats.",
    code: `import "types.lum"
import "logic.lum"

let env = Environment {
  temperature: 32.0, humidity: 40.0, soilMoisture: 50.0
}
let hardware = Actuators { fanOn: false, waterValveOpen: false }

rule "High Temperature" {
  when calculate_heat_index(env.temperature, env.humidity) > 30.0
  then update hardware.fanOn to true
  then update monitor.lastReport to "Cooling: {env.temperature}C"
}

rule "Irrigation" {
  when is_dry(env.soilMoisture) becomes true
  then update hardware.waterValveOpen to true
}

rule "Daily Heartbeat" {
  every 1 d
  then show "Temp={env.temperature} Hum={env.humidity}"
}`,
    file: "greenhouse.lum",
    tags: ["IoT", "Modules", "Temporal"]
  },
  {
    title: "Thermal Monitoring",
    desc: "Sensor system with range constraints, temporal triggers, and sustained emergency detection.",
    code: `entity Sensor {
  @doc "Temperature in Celsius"
  @range -50 to 150
  temp: Number

  isHot      := temp > 80
  isCritical := temp >= 100
}

rule "Thermal Warning" {
  when Sensor.isHot becomes true
  then show "WARNING: {Sensor.temp}°C"
}

rule "Emergency Shutdown" {
  when Sensor.isCritical becomes true for 5 s
  then show "CRITICAL: Emergency shutdown"
}

let node1 = Sensor { temp: 22 }
update node1.temp to 85
update node1.temp to 105`,
    file: "thermal.lum",
    tags: ["Monitoring", "Temporal", "@range"]
  },
  {
    title: "Process State Machine",
    desc: "Deterministic process lifecycle — allocation, execution, and garbage collection simulation.",
    code: `entity Process {
  state: Text

  isActive := state == "RUNNING"
  isZombie := state == "TERMINATED"
}

rule "Log Allocation" {
  when Process.isActive becomes true
  then show "Dispatched to CPU: {Process.state}"
}

rule "Reap Zombie" {
  when Process.isZombie becomes true
  then show "Reaping memory segments."
}

let worker = Process { state: "INITIALIZED" }
update worker.state to "RUNNING"
update worker.state to "TERMINATED"`,
    file: "state_machine.lum",
    tags: ["State Machine", "becomes"]
  }
];
