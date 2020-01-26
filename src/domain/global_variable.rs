use super::entity::Entity;
use crate::library::dwarf;

use super::type_entry::TypeEntryId;

#[derive(Debug, Clone, PartialEq)]
pub struct Address(dwarf::Location);
impl Address {
    pub fn new(location: dwarf::Location) -> Address {
        Address(location)
    }

    pub fn add(&mut self, size: usize) {
        self.0.add(size);
    }
}

impl Into<usize> for Address {
    fn into(self) -> usize {
        self.0.into()
    }
}

#[derive(Debug, PartialEq)]
pub enum GlobalVariable {
    NoSpec {
        address: Option<Address>,
        name: String,
        type_ref: TypeEntryId,
    },
    HasSpec {
        address: Option<Address>,
        spec: VariableDeclarationEntryId,
    },
}

impl GlobalVariable {
    pub fn new_variable(address: Option<Address>, name: String, type_ref: TypeEntryId) -> Self {
        Self::NoSpec {
            address,
            name,
            type_ref,
        }
    }

    pub fn new_variable_with_spec(
        address: Option<Address>,
        spec: VariableDeclarationEntryId,
    ) -> Self {
        Self::HasSpec { address, spec }
    }
}

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
