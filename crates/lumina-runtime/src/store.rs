use std::collections::HashMap;
use crate::value::Value;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub entity_name:  String,
    pub fields:       HashMap<String, Value>,
    /// Previous field values — used by the rule engine for `becomes` detection
    pub prev_fields:  HashMap<String, Value>,
}

impl Instance {
    pub fn new(entity_name: impl Into<String>, fields: HashMap<String, Value>) -> Self {
        Self {
            entity_name: entity_name.into(),
            prev_fields: fields.clone(),
            fields,
        }
    }

    pub fn get(&self, field: &str) -> Option<&Value> {
        self.fields.get(field)
    }

    pub fn set(&mut self, field: &str, value: Value) {
        if let Some(old) = self.fields.get(field) {
            self.prev_fields.insert(field.to_string(), old.clone());
        }
        self.fields.insert(field.to_string(), value);
    }

    pub fn prev(&self, field: &str) -> Option<&Value> {
        self.prev_fields.get(field)
    }

    /// Commit current state — copies fields into prev_fields
    /// Called after a full propagation cycle completes stably
    pub fn commit(&mut self) {
        self.prev_fields = self.fields.clone();
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityStore {
    instances: HashMap<String, Instance>,
}

impl EntityStore {
    pub fn new() -> Self { Self::default() }

    pub fn insert(&mut self, name: impl Into<String>, instance: Instance) {
        self.instances.insert(name.into(), instance);
    }

    pub fn get(&self, name: &str) -> Option<&Instance> {
        self.instances.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Instance> {
        self.instances.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Instance> {
        self.instances.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.instances.contains_key(name)
    }

    pub fn all(&self) -> impl Iterator<Item = (&String, &Instance)> {
        self.instances.iter()
    }

    pub fn all_of_entity<'a>(
        &'a self,
        entity_name: &'a str,
    ) -> impl Iterator<Item = (&'a String, &'a Instance)> {
        self.instances.iter().filter(move |(_, i)| i.entity_name == entity_name)
    }

    /// Commit all instances — called after stable propagation
    pub fn commit_all(&mut self) {
        for instance in self.instances.values_mut() {
            instance.commit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::SnapshotStack;

    fn make_person(name: &str, age: f64) -> Instance {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::Text(name.to_string()));
        fields.insert("age".to_string(), Value::Number(age));
        Instance::new("Person", fields)
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        let inst = store.get("isaac").unwrap();
        assert_eq!(inst.entity_name, "Person");
        assert_eq!(inst.get("name"), Some(&Value::Text("Isaac".to_string())));
        assert_eq!(inst.get("age"), Some(&Value::Number(26.0)));
    }

    #[test]
    fn test_set_captures_prev() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        let inst = store.get_mut("isaac").unwrap();

        inst.set("age", Value::Number(27.0));

        assert_eq!(inst.get("age"), Some(&Value::Number(27.0)));
        assert_eq!(inst.prev("age"), Some(&Value::Number(26.0)));
    }

    #[test]
    fn test_commit_syncs_prev() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        let inst = store.get_mut("isaac").unwrap();

        inst.set("age", Value::Number(27.0));
        inst.commit();

        assert_eq!(inst.get("age"), Some(&Value::Number(27.0)));
        assert_eq!(inst.prev("age"), Some(&Value::Number(27.0)));
    }

    #[test]
    fn test_all_of_entity() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        store.insert("alice", make_person("Alice", 30.0));

        let mut bike_fields = HashMap::new();
        bike_fields.insert("model".to_string(), Value::Text("Trek".to_string()));
        store.insert("bike1", Instance::new("Bike", bike_fields));

        let people: Vec<_> = store.all_of_entity("Person").collect();
        assert_eq!(people.len(), 2);

        let bikes: Vec<_> = store.all_of_entity("Bike").collect();
        assert_eq!(bikes.len(), 1);
    }

    #[test]
    fn test_snapshot_take_and_restore() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));

        let mut stack = SnapshotStack::new();
        let snap = stack.take(&store);
        stack.push(snap);

        // Modify the store
        store.get_mut("isaac").unwrap().set("age", Value::Number(99.0));
        assert_eq!(
            store.get("isaac").unwrap().get("age"),
            Some(&Value::Number(99.0))
        );

        // Restore from snapshot
        let restored = stack.pop().unwrap();
        store = restored.store;
        assert_eq!(
            store.get("isaac").unwrap().get("age"),
            Some(&Value::Number(26.0))
        );
    }
}
