pub mod value;
pub mod store;
pub mod snapshot;
pub mod engine;
pub mod rules;
pub mod timers;
pub mod adapter;
pub mod adapters;

pub use value::Value;
pub use store::{Instance, EntityStore};
pub use snapshot::{Snapshot, SnapshotStack, PropResult, FiredEvent, RollbackResult, Diagnostic};
pub use adapter::LuminaAdapter;

#[derive(Debug)]
pub enum RuntimeError {
    R001 { instance: String },
    R002,
    R003 { depth: usize },
    R004 { index: usize, len: usize },
    R005 { instance: String, field: String },
    R006 { field: String, value: f64, min: f64, max: f64 },
    R007 { entity: String, reason: String },
    R008 { rule: String },
    R009 { field: String },
}

impl RuntimeError {
    pub fn code(&self) -> &'static str {
        match self {
            RuntimeError::R001 { .. } => "R001",
            RuntimeError::R002        => "R002",
            RuntimeError::R003 { .. } => "R003",
            RuntimeError::R004 { .. } => "R004",
            RuntimeError::R005 { .. } => "R005",
            RuntimeError::R006 { .. } => "R006",
            RuntimeError::R007 { .. } => "R007",
            RuntimeError::R008 { .. } => "R008",
            RuntimeError::R009 { .. } => "R009",
        }
    }

    pub fn message(&self) -> String {
        match self {
            RuntimeError::R001 { instance }   => format!("Access to deleted instance: '{instance}'"),
            RuntimeError::R002                => "Division by zero".to_string(),
            RuntimeError::R003 { depth }      => format!("Rule re-entrancy limit exceeded ({depth})"),
            RuntimeError::R004 { index, len } => format!("List index out of bounds: {index} of {len}"),
            RuntimeError::R005 { instance, field } => format!("Null field access: '{instance}.{field}'"),
            RuntimeError::R006 { field, value, min, max } => format!("@range violation: {field} = {value}, expected {min}–{max}"),
            RuntimeError::R007 { entity, reason }  => format!("External entity sync failed: {entity} ({reason})"),
            RuntimeError::R008 { rule }            => format!("Timer conflict: rule '{rule}' already has a pending timer"),
            RuntimeError::R009 { field }             => format!("Cannot update derived field '{field}' — it is computed automatically"),
        }
    }
}
