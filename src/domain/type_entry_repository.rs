use std::collections::HashMap;

use super::type_entry::{TypeEntry, TypeEntryId};

#[derive(Debug)]
pub struct TypeEntryRepository {
    map: HashMap<TypeEntryId, TypeEntry>,
}

impl TypeEntryRepository {
    pub fn new() -> TypeEntryRepository {
        TypeEntryRepository {
            map: HashMap::new(),
        }
    }

    pub fn save(&mut self, value: TypeEntry) {
        self.map.insert(value.id(), value);
    }

    pub fn find_by_id(&self, id: &TypeEntryId) -> Option<&TypeEntry> {
        self.map.get(id)
    }
}
