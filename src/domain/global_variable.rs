use crate::library::dwarf;

use super::type_entry::TypeEntryId;

#[derive(Debug, Clone)]
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

#[derive(Debug)]
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
