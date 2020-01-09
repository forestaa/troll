use crate::library::dwarf;

use super::type_entry::TypeEntryId;

#[derive(Debug, Clone)]
pub struct Address(dwarf::Location);
impl Address {
    pub fn new(location: dwarf::Location) -> Address {
        Address(location)
    }

    pub fn add(&self, size: usize) -> Address {
        Address(self.0.add(size))
    }
}

impl Into<usize> for Address {
    fn into(self) -> usize {
        self.0.into()
    }
}

#[derive(Debug)]
pub struct GlobalVariable {
    pub address: Option<Address>,
    pub name: String,
    pub type_ref: TypeEntryId,
}
