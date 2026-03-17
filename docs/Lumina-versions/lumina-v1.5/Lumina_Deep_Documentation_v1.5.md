**LUMINA**

**v1.5 Deep Documentation**

_Built for Reactive Automation | IoT | Monitoring | Real-Time Operations_

_LSP | External Entities | prev() | when any/all | alert + on clear | aggregate | cooldown | Playground v2_

_"Describe what is true. Lumina figures out what to do."_

_2026 | Chapters 27-34 | Designed and authored by Isaac Ishimwe_

**Core Philosophy**

**What Lumina Is and What It Will Never Be**

_The north star that guides every feature decision in every version_

**CORE The One Rule That Guards the Language**

Every feature must pass this test: Does it help an engineer describe what is TRUE about their system?

If YES -- it belongs in Lumina.

If NO -- it belongs in the host language: Rust, Python, or Go.

This is not a guideline. It is the line between a focused language and a dead one.

# **What Lumina Is**

Lumina is a declarative reactive language for systems that watch the world and respond to it. Its natural domain is anywhere sensors produce state, rules govern behavior, and humans should not write event loops, polling logic, or propagation chains by hand.

Reactive automation. IoT device fleets. Infrastructure monitoring. Real-time operations. Smart buildings. Industrial control. These are environments where the question is always the same: "When this becomes true, what should happen?" Lumina answers that question directly, without ceremony.

# **What Lumina Is Not**

| **Lumina will NEVER be this**  | **Because**                                                            |
| ------------------------------ | ---------------------------------------------------------------------- |
| A general-purpose language     | Python and Rust already exist. They are better at general computation. |
| A scripting language           | Scripts encode procedure. Lumina encodes truth.                        |
| A math library                 | If you need sqrt() in a reactive rule, your architecture is wrong.     |
| A string manipulation language | Text processing belongs in the host language or an adapter.            |
| A data transformation language | SQL and Pandas exist for that. Lumina is not a query engine.           |
| A replacement for your backend | Lumina is the reactive brain. The backend is the body.                 |

# **Why v1.5 Features Were Chosen**

Every feature in this document was evaluated against the one rule before inclusion. Each one helps engineers describe truth about a reactive system -- nothing more.

| **Feature**       | **The truth it expresses**                              | **Why it passed**                                      |
| ----------------- | ------------------------------------------------------- | ------------------------------------------------------ |
| LSP               | Not a language feature -- tooling                       | Helps write truth faster without changing the language |
| External Entities | Real sensor state is connected truth                    | The physical world is a truth source                   |
| prev()            | The previous value of a field is observable truth       | State drift and velocity are real system facts         |
| when any/all      | Fleet-level conditions are truth about the collective   | IoT is always a fleet, never one device                |
| alert + on clear  | A raised and cleared condition is system truth          | Recovery is as real as failure                         |
| aggregate         | Fleet-wide averages and counts are derived truth        | avg(battery) is a real fact about the fleet            |
| cooldown          | A rule that fired recently is truth about its own state | Firing history is observable state                     |
| Playground v2     | Not a language feature -- tooling                       | Helps test truth interactively                         |

**Chapter 27**

**Language Server Protocol**

_Real-time diagnostics, hover, and go-to-definition in any LSP editor_

v1.4 gave VS Code syntax highlighting and snippets. v1.5 adds a full Language Server: real-time error squiggles as you type, hover tooltips showing field types and @doc text, go-to-definition for entity declarations, and a document outline. The server runs as a standalone binary: lumina-lsp.

# **27.1 What the Language Server Provides**

| **LSP Capability**    | **What the engineer sees**                 | **Powered by**               |
| --------------------- | ------------------------------------------ | ---------------------------- |
| Real-time diagnostics | Red squiggles on errors as you type        | lumina-analyzer output       |
| Hover tooltips        | Field type and @doc text on hover          | EntityDecl field metadata    |
| Go-to-definition      | Ctrl+Click jumps to entity declaration     | SourceLocation from analyzer |
| Document symbols      | Outline panel lists all entities and rules | Program AST statements       |
| Completion            | Entity and field name suggestions          | EntityDecl registry          |

# **27.2 The @doc Metadata -- Now Visible in Hover**

The @doc annotation from v1.3 was stored but never surfaced to the engineer. With the language server, hovering over any annotated field shows its documentation inline. This makes Lumina programs self-documenting.

| **@doc shown in hover tooltip**                                                                                                                                                                                                                                                                                                                                      |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| entity TemperatureSensor {<br><br>@doc("Core temperature in Celsius")<br><br>@range(-40, 150)<br><br>reading: Number<br><br>@doc("True when temperature exceeds safe threshold")<br><br>isCritical := reading > 90<br><br>}<br><br>\-- Hovering over "reading" shows:<br><br>\-- reading: Number<br><br>\-- Core temperature in Celsius<br><br>\-- Range: -40 to 150 |

# **27.3 Editor Support**

lumina-lsp communicates over stdin/stdout using the LSP JSON-RPC protocol. Any LSP-compatible editor works: VS Code, Neovim, Emacs, Helix, Zed. No editor-specific plugins beyond a standard LSP client are needed.

**NOTE v1.5 LSP Scope**

v1.5 provides: diagnostics, hover, go-to-definition, document symbols, completion.

v1.5 does NOT provide: code actions, rename, references, semantic tokens.

Those are planned for v1.6. The server communicates over stdin/stdout only.

**Chapter 28**

**External Entities**

_Connecting Lumina reactive rules to real sensors, devices, and data sources_

External entity syntax has been valid since v1.3 but the runtime ignored adapters entirely. v1.5 makes external entities fully functional. An adapter connects an external entity to a real data source. When new data arrives, the runtime propagates it exactly as if a rule action had fired.

**CORE Why External Entities Are Core to the Philosophy**

A temperature sensor reading is truth about the physical world.

A GPS coordinate is truth about where a device is right now.

Without external entities, Lumina can only reason about synthetic state.

External entities make the world itself a truth source.

# **28.1 Syntax**

| **External entity connected to a real MQTT sensor**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| external entity TemperatureSensor {<br><br>reading: Number<br><br>isOnline: Boolean<br><br>isCritical := reading > 90<br><br>isWarning := reading > 75 and reading <= 90<br><br>} sync on reading<br><br>rule Overheat when TemperatureSensor.isCritical becomes true {<br><br>alert severity: "critical", source: "temp-sensor", message: "overheating"<br><br>}<br><br>rule SensorLost when TemperatureSensor.isOnline becomes false {<br><br>alert severity: "critical", source: "temp-sensor", message: "sensor offline"<br><br>} |

# **28.2 Built-in Adapters**

| **Adapter**      | **Protocol**      | **Typical use case**                             |
| ---------------- | ----------------- | ------------------------------------------------ |
| MqttAdapter      | MQTT v3.1 / v5    | IoT sensors, industrial equipment, smart devices |
| HttpPollAdapter  | HTTP GET (JSON)   | REST APIs, cloud sensor platforms                |
| ChannelAdapter   | Rust mpsc channel | Embedding Lumina in a Rust application           |
| StaticAdapter    | In-memory queue   | Testing, simulation, historical replay           |
| FileWatchAdapter | File system watch | Log files, CSV exports, config changes           |

# **28.3 sync on -- Controlling Propagation**

The sync on clause declares which field change triggers rule propagation. Without it, every adapter poll triggers a full cycle. With sync on, Lumina only reacts when meaningful data changes -- preventing unnecessary rule evaluations on noisy sensor feeds.

| **sync on examples**                                                                                                                                                                                                                                                                                                              |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Only propagate when reading value changes<br><br>external entity TemperatureSensor { reading: Number } sync on reading<br><br>\-- Only propagate when connection status changes (ignore signal noise)<br><br>external entity NetworkDevice {<br><br>signalStrength: Number<br><br>isOnline: Boolean<br><br>} sync on isOnline |

**Chapter 29**

**prev() -- Previous Field Value**

_Expressing truth about state drift, velocity, and change magnitude_

In reactive monitoring, the current value alone is often not enough. The rate of change, the direction of drift, and the magnitude of a drop are observable truths about a system. prev(field) gives derived fields access to the last committed value of any stored field.

**CORE Why prev() Belongs in Lumina**

"Battery dropped 30% in one update" is a fact about the system -- observable truth.

"Temperature is rising" is truth about direction, not just magnitude.

prev() does not add logic. It adds a new dimension of observable state.

An engineer does not calculate drift -- they observe it. prev() makes drift observable.

# **29.1 Syntax**

| **prev() in derived field declarations**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| entity Moto {<br><br>battery: Number<br><br>\-- How much battery was lost since the last update<br><br>batteryDrop := prev(battery) - battery<br><br>\-- True when draining faster than 10% per update<br><br>droppingFast := batteryDrop > 10<br><br>\-- Direction of change<br><br>trend := if battery > prev(battery) then "rising"<br><br>else if battery < prev(battery) then "falling"<br><br>else "stable"<br><br>}<br><br>entity TemperatureSensor {<br><br>reading: Number<br><br>delta := reading - prev(reading)<br><br>isSpike := delta > 20<br><br>}<br><br>rule RapidDrain when Moto.droppingFast becomes true {<br><br>alert severity: "warning", source: "battery", message: "rapid drain detected"<br><br>}<br><br>rule TempSpike when TemperatureSensor.isSpike becomes true {<br><br>alert severity: "critical", source: "temperature", message: "temperature spike"<br><br>} |

# **29.2 prev() Semantics**

| **Scenario**                            | **prev() returns**                        |
| --------------------------------------- | ----------------------------------------- |
| First update after instance creation    | The initial field value (same as current) |
| After any update action or adapter push | The value before that update              |
| prev() of a derived field               | Not allowed -- L024                       |
| Nested prev(prev(x))                    | Not allowed -- L025                       |

# **29.3 New Error Codes**

| **Code** | **Description**                                                              |
| -------- | ---------------------------------------------------------------------------- |
| L024     | prev() applied to a derived field -- only stored fields have previous values |
| L025     | Nested prev() call -- prev(prev(x)) is not permitted                         |

**Chapter 30**

**when any / when all**

_Fleet-level reactive conditions -- truth about the collective, not just the individual_

IoT systems are never about one device. A fleet of drones, a network of sensors, a grid of smart meters -- the most critical conditions are fleet-level facts. when any and when all bring these collective truths into Lumina as first-class rule triggers.

**CORE Why Fleet Conditions Belong in Lumina**

"All motos are offline" is a fact about the system -- true or false. It belongs in Lumina.

"Any sensor is critical" is observable state -- not a loop, not a query. Just truth.

These are not aggregations. They are boolean truths about the collective.

No general language expresses this as cleanly. This is Luminas natural territory.

# **30.1 Syntax**

| **when any and when all trigger syntax**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Fire when AT LEAST ONE instance satisfies the condition<br><br>rule AnyMotoLow when any Moto.isLowBattery becomes true {<br><br>alert severity: "warning", source: "fleet", message: "at least one moto is low"<br><br>}<br><br>\-- Fire when ALL instances satisfy the condition<br><br>rule FleetOffline when all Moto.isOnline becomes false {<br><br>alert severity: "critical", source: "fleet", message: "entire fleet is offline"<br><br>}<br><br>\-- With for duration<br><br>rule SustainedOutage when all Moto.isOnline becomes false for 5m {<br><br>alert severity: "critical", source: "fleet", message: "fleet outage 5+ minutes"<br><br>}<br><br>\-- on clear works with any/all<br><br>rule FleetOffline when all Moto.isOnline becomes false {<br><br>alert severity: "critical", source: "fleet", message: "fleet offline"<br><br>} on clear {<br><br>alert severity: "resolved", source: "fleet", message: "fleet back online"<br><br>} |

# **30.2 Semantics**

| **Trigger**                    | **Fires when**                                         |
| ------------------------------ | ------------------------------------------------------ |
| when any E.field becomes true  | Count of true instances transitions from 0 to 1        |
| when any E.field becomes false | Count of false instances transitions from 0 to 1       |
| when all E.field becomes true  | Every instance has field=true (last one tips it over)  |
| when all E.field becomes false | Every instance has field=false (last one tips it over) |

# **30.3 New Error Codes**

| **Code** | **Description**                                               |
| -------- | ------------------------------------------------------------- |
| L026     | when any / when all applied to a non-Boolean derived field    |
| L027     | when all used with an entity that has zero declared instances |

**Chapter 31**

**alert + on clear**

_Structured signals and recovery detection -- the full lifecycle of a monitoring event_

The show action prints a string. Production monitoring systems need structured output with severity, source, and a machine-readable payload. v1.5 adds the alert action and an on clear block that fires automatically when the triggering condition recovers. Together they express the complete lifecycle: raised, sustained, and resolved.

**CORE Why alert and on clear Belong in Lumina**

"An alert is raised" is truth. "An alert is cleared" is truth.

The full lifecycle of a condition -- onset to recovery -- is observable system state.

Lumina should express that lifecycle directly, not require two separate rules.

show is for development. alert is for production. Both coexist.

# **31.1 The alert Action**

| **alert syntax and fields**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Required: severity, message<br><br>\-- Optional: source, code, payload<br><br>rule CriticalBattery for (m: Moto) when m.battery becomes 0 {<br><br>alert severity: "critical",<br><br>source: m.label,<br><br>message: "battery depleted on {m.label}",<br><br>code: "BATTERY_DEAD",<br><br>payload: { battery: m.battery, label: m.label }<br><br>}<br><br>\-- Valid severity levels:<br><br>\-- "info" -- informational, no action required<br><br>\-- "warning" -- action recommended<br><br>\-- "critical" -- immediate action required<br><br>\-- "resolved" -- used by on clear only, cannot be set manually |

# **31.2 on clear -- Automatic Recovery Detection**

| **on clear block syntax**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| rule LowBattery for (m: Moto) when m.isLowBattery becomes true {<br><br>alert severity: "warning",<br><br>source: m.label,<br><br>message: "low battery: {m.battery}%"<br><br>} on clear {<br><br>alert severity: "resolved",<br><br>source: m.label,<br><br>message: "battery recovered: {m.battery}%"<br><br>}<br><br>\-- on clear also works with when any/all<br><br>rule FleetOffline when all Moto.isOnline becomes false {<br><br>alert severity: "critical", source: "fleet", message: "fleet offline"<br><br>} on clear {<br><br>alert severity: "resolved", source: "fleet", message: "fleet back online"<br><br>} |

# **31.3 Alert Delivery to Host**

Alerts are delivered as structured events to a registered handler on the Evaluator. Each alert carries: severity, source, message, code, payload, UTC timestamp, and the rule name that fired it.

| **Alert JSON delivered to host application**                                                                                                                                                                                                                                                  |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| {<br><br>"severity": "critical",<br><br>"source": "moto-north-4",<br><br>"message": "battery depleted on moto-north-4",<br><br>"code": "BATTERY_DEAD",<br><br>"payload": { "battery": 0, "label": "moto-north-4" },<br><br>"timestamp": 1741824000,<br><br>"rule": "CriticalBattery"<br><br>} |

# **31.4 New Error Codes**

| **Code** | **Description**                                                 |
| -------- | --------------------------------------------------------------- |
| L028     | Invalid severity level -- must be info, warning, or critical    |
| L029     | on clear block without a matching when trigger in the same rule |
| L030     | payload references an unknown variable or field                 |

**Chapter 32**

**aggregate**

_Fleet-wide derived values -- truth about the collective state of a device group_

Rules and derived fields operate on individual instances. But monitoring systems constantly need fleet-level facts: average battery across 50 drones, number of offline sensors, whether any device is critical. The aggregate block declares these fleet-level truths as named reactive values, available to rules exactly like entity fields.

**CORE Why aggregate Belongs in Lumina**

"Average battery across the fleet is 34%" is a fact about the system. It is true.

"3 out of 50 sensors are offline" is observable truth about the collective.

These are not queries. They are derived facts that the system maintains reactively.

aggregate gives these facts a name and keeps them up to date automatically.

# **32.1 Syntax**

| **aggregate block declaration and usage**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| aggregate FleetStatus over Moto {<br><br>avgBattery := avg(battery)<br><br>minBattery := min(battery)<br><br>onlineCount := count(isOnline)<br><br>offlineCount := count(isOnline)<br><br>anyLow := any(isLowBattery)<br><br>allOnline := all(isOnline)<br><br>totalUnits := count()<br><br>}<br><br>\-- Aggregate values used in rules like any other field<br><br>rule FleetCritical when FleetStatus.anyLow becomes true {<br><br>alert severity: "warning", source: "fleet",<br><br>message: "fleet avg battery: {FleetStatus.avgBattery}%"<br><br>}<br><br>rule AllOffline when FleetStatus.allOnline becomes false {<br><br>alert severity: "critical", source: "fleet", message: "entire fleet offline"<br><br>}<br><br>\-- Multiple aggregates over different entities<br><br>aggregate SensorNetwork over TemperatureSensor {<br><br>avgTemp := avg(reading)<br><br>maxTemp := max(reading)<br><br>critCount := count(isCritical)<br><br>} |

# **32.2 Aggregate Functions**

| **Function** | **Input type** | **Returns**                             |
| ------------ | -------------- | --------------------------------------- |
| avg(field)   | Number         | Mean value across all instances         |
| min(field)   | Number         | Minimum value across all instances      |
| max(field)   | Number         | Maximum value across all instances      |
| sum(field)   | Number         | Sum across all instances                |
| count()      | any            | Total number of instances               |
| count(field) | Boolean        | Number of instances where field is true |
| any(field)   | Boolean        | True if any instance has field=true     |
| all(field)   | Boolean        | True if all instances have field=true   |

# **32.3 Reactivity**

Aggregate values recompute automatically whenever any instance of the target entity changes. There is no polling, no explicit refresh. When a moto battery updates, avgBattery and minBattery recompute in the same tick. Rules watching aggregate values fire instantly when a threshold is crossed.

# **32.4 New Error Codes**

| **Code** | **Description**                                                     |
| -------- | ------------------------------------------------------------------- |
| L031     | avg/min/max/sum applied to a non-Number field                       |
| L032     | Aggregate name conflicts with an existing entity or let binding     |
| L033     | Aggregate over entity with zero declared instances at analysis time |

**Chapter 33**

**Rule Cooldown**

_Preventing alert storms -- silence periods as part of rule truth_

Sensors flap. Devices reconnect and disconnect rapidly. Without cooldown, a rule fires every single time its trigger condition is met, producing an alert storm. cooldown declares a silence period as part of the rule itself: after firing, the rule cannot fire again for the declared duration.

**CORE Why cooldown Belongs in Lumina**

"This rule fired recently" is truth about the rule itself -- observable state.

A rule in its cooldown period is in a distinct state: silenced.

Engineers should declare debounce intent in the rule, not in external logic.

The cooldown period is not a hack -- it is a first-class fact about rule behavior.

# **33.1 Syntax**

| **cooldown syntax and examples**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Rule cannot fire again for 5 minutes after firing<br><br>rule BatteryAlert for (m: Moto)<br><br>when m.isLowBattery becomes true<br><br>cooldown 5m {<br><br>alert severity: "warning", source: m.label, message: "low battery: {m.battery}%"<br><br>}<br><br>\-- Cooldown in seconds<br><br>rule SensorSpike when TemperatureSensor.isSpike becomes true cooldown 30s {<br><br>alert severity: "warning", source: "sensor", message: "temperature spike"<br><br>}<br><br>\-- Cooldown with for duration<br><br>rule SustainedOverheat<br><br>when TemperatureSensor.isCritical becomes true for 10s<br><br>cooldown 2m {<br><br>alert severity: "critical", source: "sensor", message: "sustained overheat"<br><br>}<br><br>\-- Cooldown units: s (seconds), m (minutes), h (hours)<br><br>\-- Cooldown is per-instance for parameterized rules<br><br>\-- moto1 and moto2 cooldowns are tracked independently |

# **33.2 Cooldown Semantics**

| **Scenario**                           | **Behavior**                                                    |
| -------------------------------------- | --------------------------------------------------------------- |
| Rule fires for moto1                   | moto1 enters cooldown. moto2 is unaffected.                     |
| Trigger fires again during cooldown    | Rule body does NOT fire. Silently ignored.                      |
| Cooldown expires, condition still true | Rule fires again on next trigger evaluation                     |
| on clear during cooldown               | on clear ALWAYS fires -- cooldown only suppresses the main body |
| when all/any with cooldown             | Cooldown is fleet-level -- shared, not per-instance             |

# **33.3 New Error Code**

| **Code** | **Description**                       |
| -------- | ------------------------------------- |
| L034     | Cooldown duration is zero or negative |

**Chapter 34**

**Lumina Playground v2**

_Interactive IoT simulation -- live state, virtual clock, alert timeline, shareable URLs_

The v1.4 playground was a single-file editor with a run button. v1.5 upgrades it to an interactive reactive system simulator: live state panel, virtual clock for timer rules, alert timeline showing every fired alert with severity, and shareable URLs encoding the full program.

# **34.1 What Changes**

| **v1.4 Playground**      | **v1.5 Playground v2**                             |
| ------------------------ | -------------------------------------------------- |
| Single file, run button  | Multi-file tabs with simulated import              |
| Static output in console | Live state panel -- fields update as rules fire    |
| No timer simulation      | Virtual clock -- 1x / 10x / 100x speed             |
| show output only         | Alert timeline -- severity color-coded, filterable |
| No sharing               | Shareable URL -- full program in URL fragment      |
| Single instance only     | Add/remove instances interactively                 |

# **34.2 Live State Panel**

Every entity instance appears as a card. Stored fields have editable inputs -- change a value and the WASM runtime fires apply_event() instantly. Derived fields update in a different color. Alert badges appear on instance cards when the instance has an active alert.

# **34.3 Alert Timeline**

Every alert fired since the program started appears in the timeline with: timestamp, severity (color coded), rule name, source, and message. on clear events appear as resolved entries paired with their original alert.

# **34.4 Virtual Clock**

The virtual clock drives every and for rules. Speed options: 1x (real time), 10x, 100x. Engineers can test rules with 5-minute cooldowns or 30-minute for durations in seconds of real time.

# **34.5 Shareable URLs**

The full program source is LZ-compressed into the URL fragment. Sharing the URL gives the recipient the exact same program in their browser -- no login, no server, no account required.

**DONE v1.5 Playground v2 -- What IoT Engineers Can Do**

Write a fleet monitoring program and watch state update in real time.

Drag battery sliders down and watch fleet-level aggregates react instantly.

Set virtual clock to 100x and test cooldowns and sustained conditions in seconds.

Filter the alert timeline by severity to trace the sequence of events.

Share a URL with a colleague -- they see the same reactive system running.

Test on clear recovery by bringing a condition back within range.

**Appendix**

**v1.5 Quick Reference**

_New syntax, new error codes, feature summary_

# **Complete New Syntax**

| **All v1.5 syntax additions**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- prev(): previous stored field value<br><br>field := prev(storedField) - storedField<br><br>\-- when any / when all: fleet-level triggers<br><br>rule Name when any Entity.boolField becomes true { ... }<br><br>rule Name when all Entity.boolField becomes false for 30s { ... }<br><br>\-- alert action<br><br>alert severity: "critical", source: label, message: "text {field}"<br><br>\-- on clear: recovery block<br><br>rule Name when ... { alert ... } on clear { alert severity: "resolved", ... }<br><br>\-- aggregate block<br><br>aggregate Name over EntityType {<br><br>field := avg(numericField)<br><br>field := count(boolField)<br><br>field := any(boolField)<br><br>}<br><br>\-- cooldown clause<br><br>rule Name when ... cooldown 5m { ... }<br><br>rule Name when ... for 10s cooldown 2m { ... } |

# **New Error Codes**

| **Code** | **Description**                                          |
| -------- | -------------------------------------------------------- |
| L024     | prev() applied to a derived field                        |
| L025     | Nested prev() call                                       |
| L026     | when any/all applied to a non-Boolean field              |
| L027     | when all with zero declared instances                    |
| L028     | Invalid alert severity                                   |
| L029     | on clear without matching when trigger                   |
| L030     | alert payload references unknown field                   |
| L031     | avg/min/max/sum on non-Number field                      |
| L032     | Aggregate name conflicts with existing entity or binding |
| L033     | Aggregate over entity with zero instances                |
| L034     | Rule cooldown duration is zero or negative               |

# **Feature Summary**

| **Chapter** | **Feature**                                                         |
| ----------- | ------------------------------------------------------------------- |
| 27          | Language Server -- diagnostics, hover, go-to-definition, symbols    |
| 28          | External Entities -- MQTT, HTTP, channel adapters fully functional  |
| 29          | prev() -- previous stored field value for drift and velocity        |
| 30          | when any / when all -- fleet-level reactive conditions              |
| 31          | alert + on clear -- structured signals with recovery detection      |
| 32          | aggregate -- fleet-wide derived values                              |
| 33          | Rule cooldown -- silence periods to prevent alert storms            |
| 34          | Playground v2 -- live state, virtual clock, alert timeline, sharing |