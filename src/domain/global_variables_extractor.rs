use log::warn;

use super::global_variable::{Address, GlobalVariable};
use super::type_entry::*;
use super::type_entry_repository::TypeEntryRepository;
use super::variable_declaration_repository::VariableDeclarationRepository;
use crate::library::dwarf::{DwarfInfo, DwarfTag};

pub struct GlobalVariablesExtractor<'type_repo, 'dec_repo> {
    type_entry_repository: &'type_repo mut TypeEntryRepository,
    variable_declaration_repository: &'dec_repo mut VariableDeclarationRepository,
}

impl<'type_repo, 'dec_repo> GlobalVariablesExtractor<'type_repo, 'dec_repo> {
    pub fn new(
        type_entry_repository: &'type_repo mut TypeEntryRepository,
        variable_declaration_repository: &'dec_repo mut VariableDeclarationRepository,
    ) -> Self {
        Self {
            type_entry_repository,
            variable_declaration_repository,
        }
    }

    pub fn extract(&mut self, entries: impl Iterator<Item = DwarfInfo>) -> Vec<GlobalVariable> {
        let mut global_variables = Vec::new();
        for entry in entries {
            let result = match entry.tag() {
                DwarfTag::DW_TAG_variable => Self::extract_variable(&entry)
                    .map(|global_variable| global_variables.push(global_variable)),
                DwarfTag::DW_TAG_typedef => Self::extract_typedef(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_const_type => Self::extract_const_type(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_pointer_type => Self::extract_pointer_type(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_base_type => Self::extract_base_type(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_enumeration_type => Self::extract_enumeration_type(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_structure_type => Self::extract_structure_type(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_union_type => Self::extract_union_type(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_array_type => Self::extract_array_type(&entry)
                    .map(|type_entry| self.type_entry_repository.save(type_entry)),
                DwarfTag::DW_TAG_subroutine_type => Ok(self
                    .type_entry_repository
                    .save(Self::extract_function_type(&entry))),
                DwarfTag::DW_TAG_enumerator => Ok(()),
                DwarfTag::DW_TAG_subrange_type => Ok(()),
                DwarfTag::DW_TAG_formal_parameter => Ok(()),
                DwarfTag::DW_TAG_unimplemented => Ok(()),
            };
            match result {
                Err(msg) => Self::warning_no_expected_attribute(msg, &entry),
                Ok(()) => (),
            }
        }
        global_variables
    }

    fn extract_variable(entry: &DwarfInfo) -> Result<GlobalVariable, String> {
        let name = match entry.name() {
            Some(name) => Ok(name),
            None => Err("variable entry should have name"),
        }?;
        let address = entry.location().map(Address::new);
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("variable entry should have type"),
        }?;
        Ok(GlobalVariable::new_variable(address, name, type_ref))
    }

    fn extract_typedef(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = match entry.name() {
            Some(name) => Ok(name),
            None => Err("typedef entry should have name"),
        }?;
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("typedef entry should have type"),
        }?;
        Ok(TypeEntry::new_typedef_entry(id, name, type_ref))
    }

    fn extract_const_type(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("const_type entry should have type"),
        }?;

        Ok(TypeEntry::new_const_type_entry(id, type_ref))
    }

    fn extract_pointer_type(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());
        let size = match entry.size() {
            Some(size) => Ok(size),
            None => Err("pointer_type entry should have size"),
        }?;
        let type_ref = entry.type_offset().map(TypeEntryId::new);
        Ok(TypeEntry::new_pointer_type_entry(id, size, type_ref))
    }

    fn extract_base_type(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = match entry.name() {
            Some(name) => Ok(name),
            None => Err("base_type entry should have name"),
        }?;

        let size = match entry.size() {
            Some(size) => Ok(size),
            None => Err("base_type entry should have size"),
        }?;
        Ok(TypeEntry::new_base_type_entry(id, name, size))
    }

    fn extract_enumeration_type(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = entry.name();
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("enumeration_type entry should have type"),
        }?;

        let enumerators = entry
            .children()
            .iter()
            .flat_map(|entry| {
                let name = entry.name().or_else(|| {
                    Self::warning_no_expected_attribute(
                        String::from("enumerator entry should have name"),
                        &entry,
                    );
                    None
                })?;
                let value = entry.const_value().or_else(|| {
                    Self::warning_no_expected_attribute(
                        String::from("enumerator entry should have const_value"),
                        &entry,
                    );
                    None
                })?;

                Some(EnumeratorEntry { name, value })
            })
            .collect();
        Ok(TypeEntry::new_enum_type_entry(
            id,
            name,
            type_ref,
            enumerators,
        ))
    }

    fn extract_structure_type(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());

        let name = entry.name();
        let size = match entry.size() {
            Some(size) => Ok(size),
            None => Err("structure_type entry should have size"),
        }?;
        let members = entry
            .children()
            .iter()
            .flat_map(|entry| {
                let name = entry.name().or_else(|| {
                    Self::warning_no_expected_attribute(
                        String::from("member entry should have name"),
                        &entry,
                    );
                    None
                })?;
                let location = entry.data_member_location().or_else(|| {
                    Self::warning_no_expected_attribute(
                        String::from("member entry should have data_member_location"),
                        &entry,
                    );
                    None
                })?;
                let type_ref = match entry.type_offset() {
                    Some(type_ref) => Some(TypeEntryId::new(type_ref)),
                    None => {
                        Self::warning_no_expected_attribute(
                            String::from("member entry should have type"),
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
        Ok(TypeEntry::new_structure_type_entry(id, name, size, members))
    }

    fn extract_union_type(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = entry.name();
        let size = match entry.size() {
            Some(size) => Ok(size),
            None => Err("structure_type entry should have size"),
        }?;
        let members = entry
            .children()
            .iter()
            .flat_map(|entry| {
                let name = entry.name().or_else(|| {
                    Self::warning_no_expected_attribute(
                        String::from("member entry should have name"),
                        &entry,
                    );
                    None
                })?;
                let type_ref = match entry.type_offset() {
                    Some(type_ref) => Some(TypeEntryId::new(type_ref)),
                    None => {
                        Self::warning_no_expected_attribute(
                            String::from("member entry should have type"),
                            &entry,
                        );
                        None
                    }
                }?;

                Some(UnionTypeMemberEntry { name, type_ref })
            })
            .collect();
        Ok(TypeEntry::new_union_type_entry(id, name, size, members))
    }

    fn extract_array_type(entry: &DwarfInfo) -> Result<TypeEntry, String> {
        let id = TypeEntryId::new(entry.offset());
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("array_type entry should have type"),
        }?;
        let upper_bound = entry.children().iter().find_map(|child| match child.tag() {
            DwarfTag::DW_TAG_subrange_type => child.upper_bound(),
            _ => None,
        });
        Ok(TypeEntry::new_array_type_entry(id, type_ref, upper_bound))
    }

    fn extract_function_type(entry: &DwarfInfo) -> TypeEntry {
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
                        String::from("formal_parameter entry should have type"),
                        &entry,
                    );
                    None
                }
                _ => None,
            })
            .collect();
        let return_type_ref = entry
            .type_offset()
            .map(|type_ref| TypeEntryId::new(type_ref));
        TypeEntry::new_function_type_entry(id, argument_type_ref, return_type_ref)
    }

    fn warning_no_expected_attribute(message: String, entry: &DwarfInfo) {
        let offset: usize = entry.offset().into();
        warn!("Skip this entry: {}: offset = {:#x}", message, offset);
    }
}
