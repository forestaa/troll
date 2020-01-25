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
pub struct GlobalVariable {
    address: Option<Address>,
    name: String,
    type_ref: TypeEntryId,
}

impl GlobalVariable {
    pub fn new(address: Option<Address>, name: String, type_ref: TypeEntryId) -> Self {
        GlobalVariable {
            address,
            name,
            type_ref,
        }
    }

    pub fn address(&self) -> Option<Address> {
        self.address.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn type_ref(&self) -> &TypeEntryId {
        &self.type_ref
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

pub struct VariableDeclarationEntry {
    id: VariableDeclarationEntryId,
    name: String,
    type_ref: TypeEntryId,
}

impl VariableDeclarationEntry {
    pub fn new(id: VariableDeclarationEntryId, name: String, type_ref: TypeEntryId) -> Self {
        Self { id, name, type_ref }
    }

    pub fn id(&self) -> VariableDeclarationEntryId {
        self.id.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn type_ref(&self) -> &TypeEntryId {
        &self.type_ref
    }
}

impl Entity for VariableDeclarationEntry {
    type Id = VariableDeclarationEntryId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
