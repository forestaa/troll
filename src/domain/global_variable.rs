use crate::library::dwarf;

use super::type_entry::TypeEntryId;
use super::variable_declaration_entry::VariableDeclarationEntryId;

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
