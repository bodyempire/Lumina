use std::collections::HashMap;
use lumina_parser::ast::{LuminaType, FieldMetadata};

/// The fully analyzed schema — all entity definitions after type checking
#[derive(Debug, Clone)]
pub struct Schema {
    pub entities: HashMap<String, EntitySchema>,
}

#[derive(Debug, Clone)]
pub struct EntitySchema {
    pub name:    String,
    pub fields:  HashMap<String, FieldSchema>,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name:       String,
    pub ty:         LuminaType,
    pub is_derived: bool,
    pub metadata:   FieldMetadata,
}

impl Schema {
    pub fn new() -> Self {
        Self { entities: HashMap::new() }
    }

    pub fn get_entity(&self, name: &str) -> Option<&EntitySchema> {
        self.entities.get(name)
    }

    pub fn get_field(&self, entity: &str, field: &str) -> Option<&FieldSchema> {
        self.entities.get(entity)?.fields.get(field)
    }
}
