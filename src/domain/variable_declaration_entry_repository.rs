use std::ops::{Deref, DerefMut};

use super::entity_repository::Repository;
use super::variable_declaration_entry::VariableDeclarationEntry;

pub struct VariableDeclarationEntryRepository(Repository<VariableDeclarationEntry>);

impl VariableDeclarationEntryRepository {
    pub fn new() -> Self {
        Self(Repository::new())
    }
}

impl Deref for VariableDeclarationEntryRepository {
    type Target = Repository<VariableDeclarationEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariableDeclarationEntryRepository {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
