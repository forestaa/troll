use std::ops::{Deref, DerefMut};

use super::entity_repository::Repository;
use super::global_variable::VariableDeclarationEntry;

pub struct VariableDeclarationRepository(Repository<VariableDeclarationEntry>);

impl VariableDeclarationRepository {
    pub fn new() -> Self {
        Self(Repository::new())
    }
}

impl Deref for VariableDeclarationRepository {
    type Target = Repository<VariableDeclarationEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariableDeclarationRepository {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
