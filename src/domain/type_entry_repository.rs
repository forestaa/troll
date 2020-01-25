use std::ops::{Deref, DerefMut};

use super::entity_repository::Repository;
use super::type_entry::TypeEntry;

pub struct TypeEntryRepository(Repository<TypeEntry>);

impl TypeEntryRepository {
    pub fn new() -> Self {
        Self(Repository::new())
    }
}

impl Deref for TypeEntryRepository {
    type Target = Repository<TypeEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TypeEntryRepository {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
