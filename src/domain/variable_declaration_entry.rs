use crate::library::dwarf;

use super::entity::Entity;
use super::type_entry::TypeEntryId;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct VariableDeclarationEntryId(dwarf::Offset);
impl VariableDeclarationEntryId {
    pub fn new(offset: dwarf::Offset) -> Self {
        Self(offset)
    }
}

impl Into<dwarf::Offset> for VariableDeclarationEntryId {
    fn into(self) -> dwarf::Offset {
        self.0
    }
}

impl Into<usize> for VariableDeclarationEntryId {
    fn into(self) -> usize {
        let offset: dwarf::Offset = self.into();
        offset.into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDeclarationEntry {
    pub id: VariableDeclarationEntryId,
    pub name: String,
    pub type_ref: TypeEntryId,
}

impl VariableDeclarationEntry {
    pub fn new(id: VariableDeclarationEntryId, name: String, type_ref: TypeEntryId) -> Self {
        Self { id, name, type_ref }
    }
}

impl Entity for VariableDeclarationEntry {
    type Id = VariableDeclarationEntryId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
