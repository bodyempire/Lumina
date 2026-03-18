use crate::value::Value;

/// Trait for connecting Lumina external entities to real-world data sources.
///
/// Implementors supply data on each `poll()` call and receive write-backs
/// when a rule action updates an external entity field.
pub trait LuminaAdapter: Send + Sync {
    /// The external entity name this adapter serves.
    /// Must match: `external entity <Name> { ... }`
    fn entity_name(&self) -> &str;

    /// Called on every tick(). Return `Some((field, value))` if a new value is ready.
    fn poll(&mut self) -> Option<(String, Value)>;

    /// Called when a rule action writes to an external entity field.
    fn on_write(&mut self, field: &str, value: &Value);
}
