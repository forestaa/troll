use log::warn;

use super::global_variable::{Address, GlobalVariable};
use super::type_entry::{StructureTypeMemberEntry, TypeEntry, TypeEntryId};
use super::type_entry_repository::TypeEntryRepository;
use crate::library::dwarf::{DwarfInfo, DwarfTag};

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
                    let name = match entry.name() {
                        Some(name) => name,
                        None => {
                            Self::warning_no_expected_attribute(
                                "variable entry should have name",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let address = entry.location().map(Address::new);
                    let type_ref = match entry.type_offset() {
                        Some(type_ref) => TypeEntryId::new(type_ref),
                        None => {
                            Self::warning_no_expected_attribute(
                                "variable entry should have type",
                                &entry,
                            );
                            continue;
                        }
                    };
                    global_variables.push(GlobalVariable::new(address, name, type_ref));
                }
                DwarfTag::DW_TAG_typedef => {
                    let id = TypeEntryId::new(entry.offset());
                    let name = match entry.name() {
                        Some(name) => name,
                        None => {
                            Self::warning_no_expected_attribute(
                                "typedef entry should have name",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let type_ref = match entry.type_offset() {
                        Some(type_ref) => TypeEntryId::new(type_ref),
                        None => {
                            Self::warning_no_expected_attribute(
                                "typedef entry should have type",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let type_entry = TypeEntry::new_typedef_entry(id, name, type_ref);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_const_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let type_ref = match entry.type_offset() {
                        Some(type_ref) => TypeEntryId::new(type_ref),
                        None => {
                            Self::warning_no_expected_attribute(
                                "const_type entry should have type",
                                &entry,
                            );
                            continue;
                        }
                    };

                    let type_entry = TypeEntry::new_const_type_entry(id, type_ref);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_pointer_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let size = match entry.size() {
                        Some(size) => size,
                        None => {
                            Self::warning_no_expected_attribute(
                                "pointer_type entry should have size",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let type_ref = entry.type_offset().map(TypeEntryId::new);
                    let type_entry = TypeEntry::new_pointer_type_entry(id, size, type_ref);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_base_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let name = match entry.name() {
                        Some(name) => name,
                        None => {
                            Self::warning_no_expected_attribute(
                                "base_type entry should have name",
                                &entry,
                            );
                            continue;
                        }
                    };

                    let size = match entry.size() {
                        Some(size) => size,
                        None => {
                            Self::warning_no_expected_attribute(
                                "base_type entry should have size",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let type_entry = TypeEntry::new_base_type_entry(id, name, size);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_structure_type => {
                    let id = TypeEntryId::new(entry.offset());

                    let name = match entry.name() {
                        Some(name) => name,
                        None => {
                            Self::warning_no_expected_attribute(
                                "structure_type entry should have name",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let size = match entry.size() {
                        Some(size) => size,
                        None => {
                            Self::warning_no_expected_attribute(
                                "structure_type entry should have size",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let members = entry
                        .children()
                        .iter()
                        .flat_map(|entry| {
                            let name = entry.name().or_else(|| {
                                Self::warning_no_expected_attribute(
                                    "member entry should have name",
                                    &entry,
                                );
                                None
                            })?;
                            let location = entry.data_member_location().or_else(|| {
                                Self::warning_no_expected_attribute(
                                    "member entry should have data_member_location",
                                    &entry,
                                );
                                None
                            })?;
                            let type_ref = match entry.type_offset() {
                                Some(type_ref) => Some(TypeEntryId::new(type_ref)),
                                None => {
                                    Self::warning_no_expected_attribute(
                                        "member entry should have type",
                                        &entry,
                                    );
                                    None
                                }
                            }?;

                            Some(StructureTypeMemberEntry {
                                name,
                                location,
                                type_ref,
                            })
                        })
                        .collect();
                    let type_entry = TypeEntry::new_structure_type_entry(id, name, size, members);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_array_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let type_ref = match entry.type_offset() {
                        Some(type_ref) => TypeEntryId::new(type_ref),
                        None => {
                            Self::warning_no_expected_attribute(
                                "array_type entry should have type",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let upper_bound = entry.children().iter().find_map(|child| match child.tag() {
                        DwarfTag::DW_TAG_subrange_type => child.upper_bound(),
                        _ => None,
                    });
                    let type_entry = TypeEntry::new_array_type_entry(id, type_ref, upper_bound);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_subroutine_type => {
                    let id = TypeEntryId::new(entry.offset());
                    let argument_type_ref = entry
                        .children()
                        .iter()
                        .flat_map(|entry| match (entry.tag(), entry.type_offset()) {
                            (DwarfTag::DW_TAG_formal_parameter, Some(type_ref)) => {
                                Some(TypeEntryId::new(type_ref))
                            }
                            (DwarfTag::DW_TAG_formal_parameter, None) => {
                                Self::warning_no_expected_attribute(
                                    "formal_parameter entry should have type",
                                    &entry,
                                );
                                None
                            }
                            _ => None,
                        })
                        .collect();
                    let return_type_ref = match entry.type_offset() {
                        Some(type_ref) => TypeEntryId::new(type_ref),
                        None => {
                            Self::warning_no_expected_attribute(
                                "subroutine_type entry should have type",
                                &entry,
                            );
                            continue;
                        }
                    };
                    let type_entry =
                        TypeEntry::new_function_type_entry(id, argument_type_ref, return_type_ref);
                    self.type_entry_repository.save(type_entry);
                }
                DwarfTag::DW_TAG_subrange_type => (),
                DwarfTag::DW_TAG_formal_parameter => (),
                DwarfTag::DW_TAG_unimplemented => (),
            }
        }
        global_variables
    }

    fn warning_no_expected_attribute(message: &str, entry: &DwarfInfo) {
        let offset: usize = entry.offset().into();
        warn!("Skip this entry: {}: offset = {:#x}", message, offset);
    }
}
