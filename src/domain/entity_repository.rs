use std::collections::HashMap;

use super::entity::Entity;

pub struct Repository<E: Entity> {
    map: HashMap<E::Id, E>,
}

impl<E: Entity> Repository<E> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn save(&mut self, entity: E) {
        self.map.insert(entity.id().clone(), entity);
    }

    pub fn find_by_id(&self, id: &E::Id) -> Option<&E> {
        self.map.get(id)
    }
}
