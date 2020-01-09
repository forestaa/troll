use crate::library::dwarf::{DwarfInfo, DwarfTag};

use super::global_variable::{Address, GlobalVariable};
use super::type_entry::{StructureTypeMemberEntry, TypeEntry, TypeEntryId};
use super::type_entry_repository::TypeEntryRepository;

pub struct GlobalVariablesExtractor<'repo> {
    type_entry_repository: &'repo mut TypeEntryRepository,
}

impl<'repo> GlobalVariablesExtractor<'repo> {
    pub fn new(type_entry_repository: &mut TypeEntryRepository) -> GlobalVariablesExtractor {
        GlobalVariablesExtractor {
            type_entry_repository,
        }
    }

    pub fn extract(&mut self, entries: impl Iterator<Item = DwarfInfo>) -> Vec<GlobalVariable> {
        let mut global_variables = Vec::new();
        for entry in entries {
            match entry.tag() {
                DwarfTag::DW_TAG_variable => {
                    let name = entry.name().unwrap();
                    let address = entry.location().map(Address::new);
                    let type_ref = TypeEntryId::new(entry.type_offset().unwrap());
                    global_variables.push(GlobalVariable::new(address, name, type_ref));
                }
                DwarfTag::DW_TAG_typedef => {
                    let id = TypeEntryId::new(entry.offset());
                    let name = entry.name().unwrap();
                    if let Some(type_ref) = entry.type_offset() {
                        let type_ref = TypeEntryId::new(type_ref);
                        let type_entry = TypeEntry::new_typedef_entry(id, name, type_ref);
                        self.type_entry_repository.save(type_entry);
                    }
                }
                DwarfTag::DW_TAG_const_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let type_ref = TypeEntryId::new(entry.type_offset().unwrap());
                    let type_entry = TypeEntry::new_const_type_entry(id, type_ref);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_pointer_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let size = entry.size().unwrap();
                    let type_ref = entry.type_offset().map(TypeEntryId::new);
                    let type_entry = TypeEntry::new_pointer_type_entry(id, size, type_ref);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_base_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let name = entry.name().unwrap();
                    let size = entry.size().unwrap();
                    let type_entry = TypeEntry::new_base_type_entry(id, name, size);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_structure_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let name = entry.name().unwrap();
                    let members = entry
                        .children()
                        .iter()
                        .map(|entry| {
                            let name = entry.name().unwrap();
                            let type_ref = TypeEntryId::new(entry.type_offset().unwrap());
                            StructureTypeMemberEntry { name, type_ref }
                        })
                        .collect();
                    let type_entry = TypeEntry::new_structure_type_entry(id, name, members);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_array_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let type_ref = TypeEntryId::new(entry.type_offset().unwrap());
                    let upper_bound = entry.children().iter().find_map(|child| match child.tag() {
                        DwarfTag::DW_TAG_subrange_type => child.upper_bound(),
                        _ => None,
                    });
                    let type_entry = TypeEntry::new_array_type_entry(id, type_ref, upper_bound);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_subrange_type => (),
                DwarfTag::DW_TAG_unimplemented => (),
            }
        }
        global_variables
    }
}
