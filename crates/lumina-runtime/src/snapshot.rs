use crate::store::EntityStore;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version:   u64,
    pub store:     EntityStore,
}

#[derive(Debug)]
pub struct SnapshotStack {
    snapshots: Vec<Snapshot>,
    version:   u64,
}

impl SnapshotStack {
    pub fn new() -> Self {
        Self { snapshots: vec![], version: 0 }
    }

    /// Take a snapshot of the current store state
    pub fn take(&mut self, store: &EntityStore) -> Snapshot {
        self.version += 1;
        Snapshot {
            version: self.version,
            store:   store.clone(),
        }
    }

    /// Push a snapshot onto the stack for nested operations
    pub fn push(&mut self, snap: Snapshot) {
        self.snapshots.push(snap);
    }

    /// Pop and return the most recent snapshot
    pub fn pop(&mut self) -> Option<Snapshot> {
        self.snapshots.pop()
    }

    /// Current version number
    pub fn current_version(&self) -> u64 {
        self.version
    }
}

/// The result of a propagation cycle
#[derive(Debug, Serialize, Deserialize)]
pub struct PropResult {
    pub success:      bool,
    pub events_fired: Vec<FiredEvent>,
    pub version:      u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiredEvent {
    pub rule:     String,
    pub instance: String,
    pub severity: String,
    pub message:  String,
    pub ts:       f64,
}

/// Returned when a propagation cycle is rolled back
#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackResult {
    pub diagnostic: Diagnostic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub version:          u64,
    pub rolled_back:      bool,
    pub error_code:       String,
    pub message:          String,
    pub suggested_fix:    String,
    pub affected_rules:   Vec<String>,
}

impl Diagnostic {
    pub fn from_runtime_error(
        code: &str,
        message: &str,
        version: u64,
        affected_rules: Vec<String>,
    ) -> Self {
        let suggested_fix = match code {
            "R003" => "A rule loop was detected. Check that the rule's actions do not re-trigger its own condition. Make conditions mutually exclusive.".to_string(),
            "R002" => "Division by zero occurred. Add a guard condition: check the divisor is not zero before performing division.".to_string(),
            "R006" => "@range violation. The value assigned is outside the declared valid range for this field.".to_string(),
            "R007" => "External entity sync failed. Check the adapter connection and credentials.".to_string(),
            _      => format!("Runtime error {code}. Review the rule logic and field types."),
        };
        Self {
            version,
            rolled_back: true,
            error_code: code.to_string(),
            message: message.to_string(),
            suggested_fix,
            affected_rules,
        }
    }
}
