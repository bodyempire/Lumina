**LUMINA**

**v1.6 Deep Documentation**

**The Infrastructure Release**

_ref | Multi-Condition Triggers | Frequency Conditions | LSP v2 | write | Timestamp_

_"Describe what is true. Lumina figures out what to do."_

_2026 | Chapters 35-40 | Builds on v1.5 | Designed and authored by Isaac Ishimwe_

**Why v1.6**

**The Infrastructure Release**

_What v1.5 could not yet express -- and why v1.6 closes those gaps_

By the end of v1.5, Lumina could connect to real sensors, watch fleet conditions, fire structured alerts, and track cooldowns. It was production-ready for IoT. But one domain remained just out of reach: infrastructure management.

The gap was not features. The gap was expressiveness. Lumina could describe individual entities but could not express how they relate to each other. It could react to single conditions but not compound truths. It could observe but not write back. v1.6 closes all of those gaps.

# **The Four Gaps v1.6 Closes**

| **Gap in v1.5**                                       | **v1.6 solution**                                       |
| ----------------------------------------------------- | ------------------------------------------------------- |
| Entities cannot reference each other                  | ref keyword -- structural relationships are truth       |
| Rules react to one condition only                     | when...and... -- compound conditions are single truths  |
| No memory of how often something happened             | N times within -- frequency is observable truth         |
| Lumina can read external state but not write back     | write action -- commands to the physical world          |
| No native time type -- engineers fake it with Numbers | Timestamp type with .age accessor                       |
| LSP is functional but not production-grade            | LSP v2 -- rename, references, code actions, inlay hints |

| **Chapter** | **Feature**                 | **The truth it unlocks**                            |
| ----------- | --------------------------- | --------------------------------------------------- |
| 35          | Multi-condition triggers    | Compound system pressure is a single fact           |
| 36          | ref -- Entity relationships | One entity depending on another is structural truth |
| 37          | Frequency conditions        | How often something happens is observable history   |
| 38          | LSP v2                      | Tooling for maintaining truth at scale              |
| 39          | write action                | Lumina commands the physical world                  |
| 40          | Timestamp type              | When something last happened is temporal truth      |

**Chapter 35**

**Multi-Condition Triggers**

_when ... and ... -- compound truths as single rule triggers_

A rule in v1.5 reacts to one condition becoming true. But real systems have compound failures. A server overheating while its cooling unit is failing is a single fact about the system -- more specific and more urgent than either condition alone. v1.6 makes compound conditions expressible as a single rule trigger.

**CORE Why Multi-Condition Triggers Belong in Lumina**

"Server is overheating AND cooling unit is failing" is ONE fact about the system.

It is true or false. It is observable. It deserves a single rule.

Two separate rules watching each condition separately cannot express the compound truth.

They can only express: "overheating is true" and "cooling is failing" independently.

The compound fact -- both true simultaneously -- requires a compound trigger.

This is not logic. This is declaring a more precise truth.

# **35.1 Syntax**

| **when ... and ... trigger syntax**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Two conditions that must both be true simultaneously<br><br>rule CascadeRisk for (s: Server)<br><br>when s.isOverheating becomes true<br><br>and s.cooling.isFailing becomes true {<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "thermal cascade risk -- heat and cooling failure simultaneous"<br><br>}<br><br>\-- Fleet-level compound condition<br><br>rule DataCenterEmergency<br><br>when RackStatus.anyOverheat becomes true<br><br>and CoolingStatus.anyFailing becomes true {<br><br>alert severity: "critical",<br><br>source: "datacenter",<br><br>message: "compound failure -- overheating and cooling loss"<br><br>}<br><br>\-- Three conditions (v1.6 supports up to 3 and clauses)<br><br>rule FullSystemStress<br><br>when ClusterHealth.avgCpu becomes 85<br><br>and ClusterHealth.avgLatency becomes 500<br><br>and CoolingStatus.anyFailing becomes true {<br><br>alert severity: "critical",<br><br>source: "cluster",<br><br>message: "full system stress -- cpu, latency, and cooling all degraded"<br><br>}<br><br>\-- Multi-condition with for duration<br><br>rule SustainedCompoundPressure<br><br>when RackStatus.avgTemp becomes 80<br><br>and PowerStatus.totalLoad becomes 14000<br><br>for 5m {<br><br>alert severity: "critical",<br><br>source: "datacenter",<br><br>message: "compound pressure sustained 5 minutes"<br><br>}<br><br>\-- Multi-condition with on clear<br><br>rule CompoundRisk<br><br>when RackStatus.anyOverheat becomes true<br><br>and CoolingStatus.anyFailing becomes true {<br><br>alert severity: "critical", source: "datacenter", message: "compound risk"<br><br>} on clear {<br><br>alert severity: "resolved", source: "datacenter", message: "compound risk resolved"<br><br>} |

# **35.2 Semantics**

| **Behavior**                                        | **Detail**                                                    |
| --------------------------------------------------- | ------------------------------------------------------------- |
| Both conditions must be true simultaneously         | Not sequential -- both must be true at the same moment        |
| Rule fires on the rising edge of the compound truth | When the LAST condition becomes true, the rule fires          |
| Either condition returning to false clears the rule | The compound truth is false if any part is false              |
| on clear fires when either condition clears         | Recovery of any part = recovery of the compound truth         |
| for duration applies to the compound condition      | Both must be true continuously for the full duration          |
| cooldown applies to the whole rule                  | Not per-condition -- the whole compound rule has one cooldown |
| Named parameters work with multi-condition          | for (s: Server) when s.isOverheating and s.cooling.isFailing  |

# **35.3 Why This Is Not General Logic**

Multi-condition triggers are deliberately limited. They express "both of these truths are simultaneously true" -- nothing more. There is no OR condition. There is no NOT condition. There is no nesting. These limitations are intentional.

**NOTE What Multi-Condition Is NOT**

NOT: if (A and B) or C -- that is logic programming.

NOT: when A becomes true, then check B -- that is procedural sequencing.

NOT: a rule that fires when any combination of conditions is met -- that is a query.

IS: when A AND B are simultaneously true -- that is a compound observable fact.

The compound fact either exists or it does not. It transitions like any other truth.

**Chapter 36**

**Entity Relationships -- ref**

_Structural truth -- one entity associated with another as a declared fact_

In real systems, entities relate to each other structurally. A server belongs to a rack. An API service depends on a database. A temperature sensor monitors a specific zone. These relationships are truth about the system -- observable, stable, and consequential. v1.6 makes them expressible with the ref keyword.

**CORE Why ref Belongs in Lumina**

"This server is cooled by this cooling unit" is a structural truth.

"This API service depends on this database" is a structural truth.

These relationships exist in the real world. Lumina should be able to declare them.

ref is not object-oriented programming. It is not inheritance or polymorphism.

ref is: this entity is structurally associated with that entity.

That association is a fact. It can be traversed in derived fields and rules.

Nothing more. Nothing less.

# **36.1 Syntax**

| **ref keyword -- declaring structural relationships**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| entity CoolingUnit {<br><br>isRunning: Boolean<br><br>thermalLoad: Number<br><br>isFailing := not isRunning<br><br>isStressed := thermalLoad > 15<br><br>}<br><br>entity Server {<br><br>cpuTemp: Number<br><br>powerDraw: Number<br><br>cooling: ref CoolingUnit -- structural relationship<br><br>\-- Derived fields can traverse the relationship<br><br>isAtRisk := isOverheating and cooling.isFailing<br><br>thermalPressure := cpuTemp + cooling.thermalLoad<br><br>isOverheating := cpuTemp > 85<br><br>}<br><br>\-- Rules traverse ref relationships naturally<br><br>rule ServerAtRisk for (s: Server)<br><br>when s.isAtRisk becomes true {<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "server {s.label} at risk -- cooling unit failing"<br><br>}<br><br>\-- Multi-condition uses ref traversal<br><br>rule CascadeRisk for (s: Server)<br><br>when s.isOverheating becomes true<br><br>and s.cooling.isFailing becomes true {<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "cascade risk -- heat and cooling failure"<br><br>}<br><br>\-- Infrastructure example<br><br>entity DatabaseService {<br><br>connectionCount: Number<br><br>queryLatency: Number<br><br>isOnline: Boolean<br><br>isOverloaded := connectionCount > 400<br><br>}<br><br>entity ApiService {<br><br>cpuPercent: Number<br><br>latencyMs: Number<br><br>db: ref DatabaseService<br><br>isBottlenecked := latencyMs > 500 and db.isOverloaded<br><br>isDatabaseLost := not db.isOnline<br><br>}<br><br>rule ApiDegraded for (a: ApiService)<br><br>when a.isBottlenecked becomes true {<br><br>alert severity: "warning",<br><br>source: a.label,<br><br>message: "api degraded -- database bottleneck"<br><br>} |

# **36.2 ref Semantics**

| **Property**                            | **Detail**                                                                         |
| --------------------------------------- | ---------------------------------------------------------------------------------- |
| ref is a stored field                   | It holds a reference to exactly one instance of the target entity                  |
| ref fields are set at instance creation | let server1 = Server { cooling: coolingUnit1, ... }                                |
| ref fields can be updated               | update server1.cooling = coolingUnit2 -- reassignment is valid                     |
| Traversal in derived fields             | cooling.isFailing reads the current value of that field on the referenced instance |
| Traversal in rules                      | s.cooling.isFailing in a trigger checks the referenced instance live               |
| Circular refs are not allowed           | Server ref CoolingUnit ref Server would be rejected by analyzer                    |
| ref to external entity is valid         | An internal entity can ref an external entity                                      |

# **36.3 What ref Is NOT**

**NEVER ref Does Not Make Lumina Object-Oriented**

ref does not add inheritance. There is no parent/child entity relationship.

ref does not add polymorphism. A ref field holds exactly one entity type.

ref does not add methods. Entities have derived fields -- not methods.

ref does not add encapsulation. All fields remain observable from rules.

ref is one thing only: this entity is structurally associated with that entity.

The relationship is a fact. Derived fields and rules can read that fact.

Nothing more belongs in ref.

**Chapter 37**

**Frequency Conditions**

_N times within -- how often something happens is observable truth_

v1.5 prev() made the previous state observable. Frequency conditions make the history of state transitions observable. A sensor that reconnects 3 times in 10 minutes is flapping. A pod that crashes 5 times in an hour is unstable. These are truths about patterns over time -- and they deserve first-class expression in Lumina.

**CORE Why Frequency Conditions Belong in Lumina**

"This device reconnected 3 times in 10 minutes" is observable truth about its history.

"This rule has fired 5 times in the last hour" is truth about the system's behavior.

prev() describes the last state. Frequency describes how often state has changed.

Both are observations. Both help engineers describe what is really happening.

A flapping sensor is not the same as a failing sensor. Frequency captures the difference.

# **37.1 Syntax**

| **N times within -- frequency condition syntax**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Fires when condition has become true 3+ times in 10 minutes<br><br>rule SensorFlapping for (s: TemperatureSensor)<br><br>when s.isOnline becomes true<br><br>3 times within 10m {<br><br>alert severity: "warning",<br><br>source: s.label,<br><br>message: "sensor flapping -- reconnected 3 times in 10 minutes"<br><br>}<br><br>\-- Pod instability detection<br><br>rule PodUnstable for (s: ApiService)<br><br>when s.isOnline becomes false<br><br>5 times within 1h {<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "pod crashing repeatedly -- 5 failures in 1 hour"<br><br>}<br><br>\-- Chronic overheating -- not just one spike but a pattern<br><br>rule ChronicOverheat for (sv: Server)<br><br>when sv.isOverheating becomes true<br><br>5 times within 1h<br><br>cooldown 2h {<br><br>alert severity: "critical",<br><br>source: sv.label,<br><br>message: "chronic overheating -- hardware investigation required"<br><br>}<br><br>\-- Alert storm detection -- too many alerts from one source<br><br>rule AlertStorm<br><br>when FleetStatus.anyLow becomes true<br><br>10 times within 5m<br><br>cooldown 30m {<br><br>alert severity: "warning",<br><br>source: "fleet",<br><br>message: "alert storm detected -- fleet instability"<br><br>}<br><br>\-- Frequency window units: s (seconds), m (minutes), h (hours)<br><br>\-- N must be a positive integer >= 2<br><br>\-- The window is a sliding window -- not a fixed bucket |

# **37.2 Frequency Semantics**

| **Property**                         | **Detail**                                                             |
| ------------------------------------ | ---------------------------------------------------------------------- |
| Sliding window                       | The window moves with time -- not a fixed bucket reset every N minutes |
| Counts rising edges only             | Each time the condition becomes true counts as one occurrence          |
| Cooldown interacts naturally         | After firing, cooldown prevents re-firing during the silence period    |
| Window resets after cooldown         | Occurrences before the cooldown expiry do not count after it expires   |
| Per-instance for parameterized rules | moto1 and moto2 frequency counts are tracked independently             |
| Fleet-level for aggregate triggers   | when any/all + frequency shares the fleet-level count                  |
| on clear is not applicable           | Frequency conditions have no natural "cleared" state                   |

# **37.3 Frequency vs cooldown vs for Duration**

| **Clause**         | **What it measures**                             | **Use case**                       |
| ------------------ | ------------------------------------------------ | ---------------------------------- |
| for 5m             | Condition sustained continuously for 5 minutes   | Avoiding transient noise           |
| cooldown 5m        | Time since the rule last fired                   | Preventing alert storms            |
| 3 times within 10m | How many times condition became true in a window | Detecting flapping and instability |

**Chapter 38**

**LSP v2**

_Production-grade language server -- rename, references, code actions, semantic tokens_

The v1.5 LSP gave engineers real-time diagnostics, hover, go-to-definition, symbols, and completion. That is enough to write Lumina programs. But as programs grow to dozens of entities and hundreds of rules, engineers need to refactor them. v1.6 LSP v2 adds the capabilities that make large Lumina programs maintainable.

# **38.1 What LSP v2 Adds**

| **New capability**  | **What the engineer can do**                                                           |
| ------------------- | -------------------------------------------------------------------------------------- |
| Rename symbol       | Rename an entity or field -- all references update automatically across all files      |
| Find all references | See every rule and derived field that reads a specific field                           |
| Code actions        | Quick-fix suggestions for common errors -- press the lightbulb to fix L001             |
| Semantic tokens     | Richer syntax highlighting beyond grammar -- derived fields in different color         |
| Inlay hints         | Field types shown inline without hovering -- see Number after battery without hovering |

# **38.2 Why Rename and References Matter at Scale**

In a small Lumina program -- one entity, five rules -- rename does not matter much. But in a data center monitoring program with 20 entity types, 150 rules, and 10 aggregate declarations, renaming a field like cpuTemp to cpuCelsius means finding and updating every derived field and rule trigger that references it. Without LSP rename, this is error-prone manual work. With it, it is one keyboard shortcut.

Find all references complements rename. Before renaming a field, an engineer wants to see exactly what will change. Find all references answers that question instantly -- showing every usage in the program with file and line number.

# **38.3 Code Actions -- Quick Fixes for Common Errors**

| **Code action examples triggered by error codes**                                                                                                                                                                                                                                                                                                                                                                                |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- L001: Entity not found<br><br>\-- Code action: "Did you mean TemperatureSensor?" (fuzzy match suggestion)<br><br>\-- L024: prev() on derived field<br><br>\-- Code action: "Change to stored field declaration"<br><br>\-- L028: Invalid severity level<br><br>\-- Code action: "Replace with nearest valid severity: warning"<br><br>\-- L034: Cooldown duration is zero<br><br>\-- Code action: "Set minimum cooldown: 1s" |

# **38.4 Semantic Tokens and Inlay Hints**

Semantic tokens give the editor richer information than the grammar alone. Stored fields appear in one color. Derived fields appear in another. Entity names in rules appear differently from field names. This visual distinction helps engineers read complex programs faster.

Inlay hints show type information inline. Rather than hovering to see that battery is a Number, the editor shows battery: Number directly in the source. For derived fields, the inlay hint shows the computed type. This is especially useful for long programs where engineers cannot remember every field type.

**NOTE LSP v2 Scope**

v1.6 LSP v2 provides: rename, find references, code actions, semantic tokens, inlay hints.

The v1.5 capabilities remain: diagnostics, hover, go-to-def, symbols, completion.

LSP v2 is a superset of v1.5 LSP -- nothing is removed.

The binary remains lumina-lsp -- it is updated in place, not a new binary.

**Chapter 39**

**write Action**

_Lumina commands the physical world -- structured write-back to external entities_

v1.5 External Entities made the world a truth source. Lumina could read from sensors, devices, and services. v1.6 closes the loop. The write action lets Lumina send commands back to external entities through their adapters. Opening a valve, scaling a service, toggling a relay -- these are now expressible as Lumina actions.

**CORE Why write Belongs in Lumina**

"Open the pressure relief valve" is a command driven by a truth: pressure is too high.

"Scale this service up" is a command driven by a truth: CPU load is sustained above threshold.

The truth was already expressible in Lumina. The command was not.

write does not make Lumina an orchestration tool.

write makes Lumina able to respond to truth with physical action.

The truth drives the command. The adapter executes it.

Lumina declares what should happen. The world decides how.

# **39.1 Syntax**

| **write action -- commanding external entities**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| external entity SmartValve {<br><br>isOpen: Boolean<br><br>pressure: Number<br><br>isPressureHigh := pressure > 80<br><br>} sync on pressure<br><br>rule RelievePressure for (v: SmartValve)<br><br>when v.isPressureHigh becomes true<br><br>cooldown 30s {<br><br>write v.isOpen = true -- command through adapter on_write()<br><br>alert severity: "warning",<br><br>source: v.label,<br><br>message: "pressure relief valve opened"<br><br>} on clear {<br><br>write v.isOpen = false<br><br>alert severity: "resolved",<br><br>source: v.label,<br><br>message: "pressure normalized -- valve closed"<br><br>}<br><br>\-- Infrastructure: scaling a service<br><br>external entity ApiService {<br><br>cpuPercent: Number<br><br>replicaCount: Number<br><br>isUnderLoad := cpuPercent > 75<br><br>} sync on cpuPercent<br><br>rule ScaleUp for (s: ApiService)<br><br>when s.isUnderLoad becomes true for 2m<br><br>cooldown 10m {<br><br>write s.replicaCount = s.replicaCount + 2<br><br>alert severity: "info",<br><br>source: s.label,<br><br>message: "scaling up -- load sustained 2 minutes"<br><br>} on clear {<br><br>write s.replicaCount = s.replicaCount - 2<br><br>alert severity: "resolved",<br><br>source: s.label,<br><br>message: "load normalized -- scaling down"<br><br>}<br><br>\-- write can also set to a literal value<br><br>rule EmergencyShutdown for (s: Server)<br><br>when s.cpuTemp becomes 95 {<br><br>write s.throttlePercent = 50<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "emergency throttle -- cpu critical"<br><br>} |

# **39.2 How write Works**

write routes through the adapter registered for that external entity. Specifically it calls the adapter's on_write(field, value) method. The adapter decides how to translate that into a physical command -- an MQTT publish, an HTTP request, a Kubernetes API call, a hardware signal.

| **Property**                               | **Detail**                                                            |
| ------------------------------------------ | --------------------------------------------------------------------- |
| write only works on external entity fields | Cannot write to internal entity stored fields -- use update for those |
| write routes through the adapter           | on_write(field, value) is called on the registered adapter            |
| The adapter decides execution              | Lumina declares intent. The adapter translates to the real world.     |
| write can reference other fields           | write s.replicaCount = s.replicaCount + 2 is valid                    |
| Multiple writes in one rule body           | A rule can write multiple fields in sequence                          |
| write + alert in same rule                 | Both actions fire -- the alert notifies, the write acts               |

# **39.3 What write Is NOT**

**NEVER write Does Not Make Lumina an Orchestration Tool**

write does not replace Terraform. Infrastructure provisioning is not Luminas job.

write does not replace Kubernetes operators. Workload scheduling is not Luminas job.

write does not create side effects outside the adapter contract.

write is: when this truth is observed, send this command to the adapter.

The adapter executes. Lumina observes. The cycle continues.

Lumina never owns the execution. It owns the decision.

**Chapter 40**

**Timestamp Type**

_Temporal truth -- when something last happened as a first-class observable fact_

Time is fundamental to reactive systems. How long has a device been silent? How long has a condition persisted beyond what the for clause covers? When did this sensor last report? These questions cannot be answered with Number fields. v1.6 adds the Timestamp type with a built-in .age accessor that makes temporal truth first-class in Lumina.

**CORE Why Timestamp Belongs in Lumina**

"This device has been silent for 30 minutes" is observable truth.

"This condition has persisted for 3 hours" is observable truth.

"The last reading was at 14:32 UTC" is observable truth.

Before Timestamp, engineers faked this with Number fields and manual time math.

That forces computation into a language designed to avoid it.

Timestamp makes temporal truth native -- observable and declarable without arithmetic.

# **40.1 Syntax**

| **Timestamp type and .age accessor**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| entity NetworkDevice {<br><br>lastSeen: Timestamp<br><br>isOnline: Boolean<br><br>\-- .age gives the duration since the Timestamp was last set<br><br>silenceDuration := lastSeen.age<br><br>isStale := lastSeen.age > 5m<br><br>isLost := lastSeen.age > 30m<br><br>}<br><br>rule DeviceStale for (d: NetworkDevice)<br><br>when d.isStale becomes true<br><br>cooldown 10m {<br><br>alert severity: "warning",<br><br>source: d.label,<br><br>message: "device silent for {d.lastSeen.age}"<br><br>}<br><br>rule DeviceLost for (d: NetworkDevice)<br><br>when d.isLost becomes true {<br><br>alert severity: "critical",<br><br>source: d.label,<br><br>message: "device lost -- silent for {d.lastSeen.age}"<br><br>}<br><br>\-- Updating a Timestamp field -- use now()<br><br>\-- now() is the only built-in function Lumina ever needs for time<br><br>rule RecordContact for (d: NetworkDevice)<br><br>when d.isOnline becomes true {<br><br>update d.lastSeen = now()<br><br>}<br><br>\-- External entities can push Timestamp values via adapters<br><br>external entity TemperatureSensor {<br><br>reading: Number<br><br>lastSeen: Timestamp<br><br>isStale := lastSeen.age > 2m<br><br>} sync on reading<br><br>\-- Infrastructure: service health with time awareness<br><br>entity ApiService {<br><br>cpuPercent: Number<br><br>lastHealthyAt: Timestamp<br><br>isDegraded := cpuPercent > 80<br><br>degradedDuration := lastHealthyAt.age<br><br>isChronicallySick := lastHealthyAt.age > 1h<br><br>}<br><br>rule ChronicDegradation for (s: ApiService)<br><br>when s.isChronicallySick becomes true<br><br>cooldown 2h {<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "service degraded for {s.lastHealthyAt.age} -- intervention needed"<br><br>} |

# **40.2 Timestamp Semantics**

| **Property**                     | **Detail**                                                                   |
| -------------------------------- | ---------------------------------------------------------------------------- |
| Timestamp stores a UTC moment    | Internally stored as Unix timestamp in milliseconds                          |
| .age returns a duration          | The elapsed time since the Timestamp value -- updates continuously           |
| now() is the only time function  | Sets a Timestamp to the current moment. No other time math needed.           |
| .age in derived fields           | isStale := lastSeen.age > 5m -- age is compared to a duration literal        |
| .age in interpolated strings     | "{lastSeen.age}" formats as human-readable: "4m 32s"                         |
| Duration comparisons             | lastSeen.age > 5m, lastSeen.age > 1h, lastSeen.age > 30s                     |
| Timestamp fields can be external | Adapters can push Timestamp values when devices report in                    |
| Unset Timestamp                  | A Timestamp field that has never been set has age = infinity -- always stale |

# **40.3 now() -- The Only Time Built-in**

now() is the only built-in function Lumina ever needs that touches time. It returns the current UTC moment as a Timestamp value. It can only be used in update and write actions -- not in derived field expressions. This is intentional: derived fields should describe static structural truth, not dynamic time-dependent computation.

**NOTE Why now() Is Not Allowed in Derived Fields**

A derived field is recomputed when its dependencies change.

If now() were allowed in derived fields, every field containing it

would need to recompute every millisecond -- not on state changes.

The .age accessor is different: it is a property of the Timestamp value itself.

The runtime tracks .age continuously without recomputing the derived field.

now() in update/write is fine: it runs once when the action fires.

**Appendix**

**v1.6 Quick Reference**

_New syntax, new error codes, complete feature summary_

# **Complete New Syntax in v1.6**

| **All v1.6 syntax additions**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- ref: entity relationship (structural truth)<br><br>entity Server {<br><br>cooling: ref CoolingUnit<br><br>isAtRisk := isOverheating and cooling.isFailing<br><br>}<br><br>\-- Multi-condition trigger<br><br>rule Name<br><br>when Entity.fieldA becomes true<br><br>and Entity.fieldB becomes true { ... }<br><br>\-- Frequency condition<br><br>rule Name when Entity.field becomes true<br><br>3 times within 10m { ... }<br><br>\-- write action (external entities only)<br><br>write entity.field = value<br><br>write entity.field = entity.field + 1<br><br>\-- Timestamp type<br><br>lastSeen: Timestamp<br><br>isStale := lastSeen.age > 5m<br><br>update d.lastSeen = now()<br><br>\-- Duration comparisons for .age<br><br>\-- lastSeen.age > 30s<br><br>\-- lastSeen.age > 5m<br><br>\-- lastSeen.age > 1h |

# **New Error Codes in v1.6**

| **Code** | **Description**                                                                |
| -------- | ------------------------------------------------------------------------------ |
| L035     | Multi-condition trigger has more than 3 and clauses                            |
| L036     | ref field references an entity type that does not exist                        |
| L037     | Circular ref detected -- A refs B refs A                                       |
| L038     | write action used on a non-external entity field -- use update instead         |
| L039     | Frequency condition N is less than 2                                           |
| L040     | Frequency window duration is zero or negative                                  |
| L041     | now() used in a derived field expression -- only valid in update/write actions |
| L042     | Timestamp .age compared to a non-duration value                                |

# **v1.6 Feature Summary**

| **Chapter** | **Feature**                                                                   |
| ----------- | ----------------------------------------------------------------------------- |
| 35          | Multi-condition triggers -- when...and... for compound truths                 |
| 36          | Entity relationships -- ref keyword for structural truth                      |
| 37          | Frequency conditions -- N times within for historical truth                   |
| 38          | LSP v2 -- rename, find references, code actions, semantic tokens, inlay hints |
| 39          | write action -- structured write-back to external entities                    |
| 40          | Timestamp type -- temporal truth with .age accessor and now()                 |

# **The Infrastructure Program -- v1.6 in Full**

| **Complete data center monitoring program using all v1.6 features**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Entities with relationships (ref)<br><br>external entity CoolingUnit {<br><br>isRunning: Boolean<br><br>thermalLoad: Number<br><br>lastSeen: Timestamp<br><br>isFailing := not isRunning<br><br>isStale := lastSeen.age > 2m<br><br>} sync on isRunning<br><br>external entity Server {<br><br>cpuTemp: Number<br><br>powerDraw: Number<br><br>throttle: Number<br><br>cooling: ref CoolingUnit<br><br>lastSeen: Timestamp<br><br>isOverheating := cpuTemp > 85<br><br>isAtRisk := isOverheating and cooling.isFailing<br><br>tempRising := prev(cpuTemp) - cpuTemp < -3<br><br>isLost := lastSeen.age > 5m<br><br>} sync on cpuTemp<br><br>\-- Aggregates<br><br>aggregate RackStatus over Server {<br><br>avgTemp := avg(cpuTemp)<br><br>maxTemp := max(cpuTemp)<br><br>anyOverheat := any(isOverheating)<br><br>anyAtRisk := any(isAtRisk)<br><br>totalPower := sum(powerDraw)<br><br>lostCount := count(isLost)<br><br>}<br><br>\-- Multi-condition trigger (Ch35)<br><br>rule CascadeRisk for (s: Server)<br><br>when s.isOverheating becomes true<br><br>and s.cooling.isFailing becomes true {<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "cascade risk -- heat and cooling failure"<br><br>} on clear {<br><br>alert severity: "resolved", source: s.label, message: "cascade risk cleared"<br><br>}<br><br>\-- write action (Ch39)<br><br>rule EmergencyThrottle for (s: Server)<br><br>when s.cpuTemp becomes 93<br><br>cooldown 5m {<br><br>write s.throttle = 50<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "emergency throttle applied"<br><br>}<br><br>\-- Frequency condition (Ch37)<br><br>rule CoolingFlapping for (c: CoolingUnit)<br><br>when c.isRunning becomes false<br><br>3 times within 15m {<br><br>alert severity: "critical",<br><br>source: c.label,<br><br>message: "cooling unit unstable -- failing repeatedly"<br><br>}<br><br>\-- Timestamp (Ch40)<br><br>rule ServerLost for (s: Server)<br><br>when s.isLost becomes true {<br><br>alert severity: "critical",<br><br>source: s.label,<br><br>message: "server lost -- silent for {s.lastSeen.age}"<br><br>} |