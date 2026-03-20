use std::collections::HashMap;

/// Tracks Boolean field true-counts per (entity, field) for fleet-level triggers.
///
/// Instead of scanning all instances on every update, FleetState maintains
/// a running counter that is updated on each Boolean field write, enabling
/// O(1) `any_true()` and `all_true()` evaluation.
#[derive(Debug, Clone)]
pub struct FleetState {
    /// (entity_name, field_name) -> (true_count, total_count)
    counts: HashMap<(String, String), (usize, usize)>,
}

impl FleetState {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Called after every Boolean field write.
    /// `new_val` is the new Boolean value, `old_val` is the previous value.
    /// `total` is the total number of instances of this entity.
    pub fn update(
        &mut self,
        entity: &str,
        field: &str,
        old_val: bool,
        new_val: bool,
        total: usize,
    ) {
        let key = (entity.to_string(), field.to_string());
        let entry = self.counts.entry(key).or_insert((0, total));
        entry.1 = total;

        // Only adjust counter if the value actually changed
        if old_val != new_val {
            if new_val {
                entry.0 = entry.0.saturating_add(1);
            } else {
                entry.0 = entry.0.saturating_sub(1);
            }
        }
    }

    /// Initialize counts for all instances during startup.
    /// Call this once per (entity, field) after all instances are created.
    pub fn initialize(&mut self, entity: &str, field: &str, true_count: usize, total: usize) {
        let key = (entity.to_string(), field.to_string());
        self.counts.insert(key, (true_count, total));
    }

    /// Returns true if at least one instance has the field set to true.
    pub fn any_true(&self, entity: &str, field: &str) -> bool {
        self.counts
            .get(&(entity.to_string(), field.to_string()))
            .map(|(t, _)| *t > 0)
            .unwrap_or(false)
    }

    /// Returns true if ALL instances have the field set to true.
    pub fn all_true(&self, entity: &str, field: &str) -> bool {
        self.counts
            .get(&(entity.to_string(), field.to_string()))
            .map(|(t, total)| *t == *total && *total > 0)
            .unwrap_or(false)
    }

    /// Returns the current (true_count, total_count) for a given entity+field.
    pub fn get_counts(&self, entity: &str, field: &str) -> (usize, usize) {
        self.counts
            .get(&(entity.to_string(), field.to_string()))
            .copied()
            .unwrap_or((0, 0))
    }
}

impl Default for FleetState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_any_true() {
        let mut fs = FleetState::new();
        fs.initialize("Moto", "isLow", 0, 3);
        assert!(!fs.any_true("Moto", "isLow"));

        fs.update("Moto", "isLow", false, true, 3);
        assert!(fs.any_true("Moto", "isLow"));
    }

    #[test]
    fn test_all_true() {
        let mut fs = FleetState::new();
        fs.initialize("Moto", "isLow", 0, 3);

        fs.update("Moto", "isLow", false, true, 3);
        assert!(!fs.all_true("Moto", "isLow"));

        fs.update("Moto", "isLow", false, true, 3);
        assert!(!fs.all_true("Moto", "isLow"));

        fs.update("Moto", "isLow", false, true, 3);
        assert!(fs.all_true("Moto", "isLow"));
    }

    #[test]
    fn test_no_change_doesnt_alter_count() {
        let mut fs = FleetState::new();
        fs.initialize("Moto", "isLow", 1, 3);

        // Setting true -> true should not change count
        fs.update("Moto", "isLow", true, true, 3);
        assert_eq!(fs.get_counts("Moto", "isLow"), (1, 3));
    }

    #[test]
    fn test_decrement() {
        let mut fs = FleetState::new();
        fs.initialize("Moto", "isLow", 3, 3);
        assert!(fs.all_true("Moto", "isLow"));

        fs.update("Moto", "isLow", true, false, 3);
        assert!(!fs.all_true("Moto", "isLow"));
        assert!(fs.any_true("Moto", "isLow"));
    }
}
