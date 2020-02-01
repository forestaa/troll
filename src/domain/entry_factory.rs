use super::global_variable::*;
use super::type_entry::*;
use super::variable_declaration_entry::*;
use crate::library::dwarf::{DwarfInfo, DwarfTag};

pub struct EntryFactory;

pub enum FromDwarfInfoOutput {
    None,
    GlobalVariable(GlobalVariable),
    TypeEntry {
        entry: TypeEntry,
        children_warnings: Vec<String>,
    },
    VariableDeclarationEntry(VariableDeclarationEntry),
}

impl FromDwarfInfoOutput {
    fn new_type_entry_with_children_warnings(
        entry: TypeEntry,
        children_warnings: Vec<String>,
    ) -> Self {
        Self::TypeEntry {
            entry,
            children_warnings,
        }
    }

    fn new_type_entry_with_no_children_warnings(entry: TypeEntry) -> Self {
        Self::TypeEntry {
            entry,
            children_warnings: Vec::new(),
        }
    }
}

impl EntryFactory {
    pub fn from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        match entry.tag() {
            DwarfTag::DW_TAG_variable => Self::variable_from_dwarf_info(entry),
            DwarfTag::DW_TAG_typedef => Self::typedef_from_dwarf_info(entry),
            DwarfTag::DW_TAG_volatile_type => Self::volatile_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_const_type => Self::const_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_pointer_type => Self::pointer_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_base_type => Self::base_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_enumeration_type => Self::enumeration_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_structure_type => Self::structure_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_union_type => Self::union_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_array_type => Self::array_type_from_dwarf_info(entry),
            DwarfTag::DW_TAG_subroutine_type => Ok(Self::function_type_from_dwarf_info(entry)),
            DwarfTag::DW_TAG_enumerator => Ok(FromDwarfInfoOutput::None),
            DwarfTag::DW_TAG_subrange_type => Ok(FromDwarfInfoOutput::None),
            DwarfTag::DW_TAG_formal_parameter => Ok(FromDwarfInfoOutput::None),
            DwarfTag::DW_TAG_unimplemented => Ok(FromDwarfInfoOutput::None),
        }
    }

    fn variable_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        match entry.declaration() {
            None => Self::variable_without_declaration_from_dwarf_info(entry)
                .map(|global_variable| FromDwarfInfoOutput::GlobalVariable(global_variable)),
            Some(_) => Self::variable_with_declaration_from_dwarf_info(entry)
                .map(|dec| FromDwarfInfoOutput::VariableDeclarationEntry(dec)),
        }
    }

    fn variable_without_declaration_from_dwarf_info(
        entry: &DwarfInfo,
    ) -> Result<GlobalVariable, String> {
        let address = entry.location().map(Address::new);
        match entry.specification() {
            None => {
                let name = match entry.name() {
                    Some(name) => Ok(name),
                    None => Err("variable entry should have name"),
                }?;
                let type_ref = match entry.type_offset() {
                    Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
                    None => Err("variable entry should have type"),
                }?;
                Ok(GlobalVariable::new_variable(address, name, type_ref))
            }
            Some(dec_ref) => {
                let spec = VariableDeclarationEntryId::new(dec_ref);
                Ok(GlobalVariable::new_variable_with_spec(address, spec))
            }
        }
    }

    fn variable_with_declaration_from_dwarf_info(
        entry: &DwarfInfo,
    ) -> Result<VariableDeclarationEntry, String> {
        let id = VariableDeclarationEntryId::new(entry.offset());
        let name = match entry.name() {
            Some(name) => Ok(name),
            None => Err("variable entry with declaration should have name"),
        }?;
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("variable entry with declaration should have type"),
        }?;
        Ok(VariableDeclarationEntry::new(id, name, type_ref))
    }

    fn typedef_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = match entry.name() {
            Some(name) => Ok(name),
            None => Err("typedef entry should have name"),
        }?;
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("typedef entry should have type"),
        }?;

        let entry = TypeEntry::new_typedef_entry(id, name, type_ref);
        Ok(FromDwarfInfoOutput::new_type_entry_with_no_children_warnings(entry))
    }

    fn volatile_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("volatile_type entry should have type"),
        }?;

        let entry = TypeEntry::new_volatile_type_entry(id, type_ref);
        Ok(FromDwarfInfoOutput::new_type_entry_with_no_children_warnings(entry))
    }

    fn const_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("const_type entry should have type"),
        }?;

        let entry = TypeEntry::new_const_type_entry(id, type_ref);
        Ok(FromDwarfInfoOutput::new_type_entry_with_no_children_warnings(entry))
    }

    fn pointer_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let size = match entry.byte_size() {
            Some(size) => Ok(size),
            None => Err("pointer_type entry should have size"),
        }?;
        let type_ref = entry.type_offset().map(TypeEntryId::new);

        let entry = TypeEntry::new_pointer_type_entry(id, size, type_ref);
        Ok(FromDwarfInfoOutput::new_type_entry_with_no_children_warnings(entry))
    }

    fn base_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = match entry.name() {
            Some(name) => Ok(name),
            None => Err("base_type entry should have name"),
        }?;

        let size = match entry.byte_size() {
            Some(size) => Ok(size),
            None => Err("base_type entry should have size"),
        }?;

        let entry = TypeEntry::new_base_type_entry(id, name, size);
        Ok(FromDwarfInfoOutput::new_type_entry_with_no_children_warnings(entry))
    }

    fn enumeration_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = entry.name();
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("enumeration_type entry should have type"),
        }?;

        let mut children_warnings = Vec::new();
        let enumerators = entry
            .children()
            .iter()
            .flat_map(|entry| {
                let name = entry.name().or_else(|| {
                    children_warnings.push(String::from("enumerator entry should have name"));
                    None
                })?;
                let value = entry.const_value().or_else(|| {
                    children_warnings
                        .push(String::from("enumerator entry should have const_value"));
                    None
                })?;

                Some(EnumeratorEntry { name, value })
            })
            .collect();

        let entry = TypeEntry::new_enum_type_entry(id, name, type_ref, enumerators);
        Ok(FromDwarfInfoOutput::new_type_entry_with_children_warnings(
            entry,
            children_warnings,
        ))
    }

    fn structure_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());

        let name = entry.name();
        let size = match entry.byte_size() {
            Some(size) => Ok(size),
            None => Err("structure_type entry should have size"),
        }?;
        let mut children_warnings = Vec::new();
        let members = entry
            .children()
            .iter()
            .flat_map(|entry| {
                let name = entry.name().or_else(|| {
                    children_warnings.push(String::from("member entry should have name"));
                    None
                })?;
                let location = entry.data_member_location().or_else(|| {
                    children_warnings.push(String::from(
                        "member entry should have data_member_location",
                    ));
                    None
                })?;
                let type_ref = match entry.type_offset() {
                    Some(type_ref) => Some(TypeEntryId::new(type_ref)),
                    None => {
                        children_warnings.push(String::from("member entry should have type"));
                        None
                    }
                }?;

                let bit_size = entry.bit_size();
                let bit_offset = entry.bit_offset();

                Some(StructureTypeMemberEntry::new(
                    name, location, type_ref, bit_size, bit_offset,
                ))
            })
            .collect();

        let entry = TypeEntry::new_structure_type_entry(id, name, size, members);
        Ok(FromDwarfInfoOutput::new_type_entry_with_children_warnings(
            entry,
            children_warnings,
        ))
    }

    fn union_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let name = entry.name();
        let size = match entry.byte_size() {
            Some(size) => Ok(size),
            None => Err("structure_type entry should have size"),
        }?;
        let mut children_warnings = Vec::new();
        let members = entry
            .children()
            .iter()
            .flat_map(|entry| {
                let name = entry.name().or_else(|| {
                    children_warnings.push(String::from("member entry should have name"));
                    None
                })?;
                let type_ref = match entry.type_offset() {
                    Some(type_ref) => Some(TypeEntryId::new(type_ref)),
                    None => {
                        children_warnings.push(String::from("member entry should have type"));
                        None
                    }
                }?;

                let bit_size = entry.bit_size();
                let bit_offset = entry.bit_offset();

                Some(UnionTypeMemberEntry::new(
                    name, type_ref, bit_size, bit_offset,
                ))
            })
            .collect();

        let entry = TypeEntry::new_union_type_entry(id, name, size, members);
        Ok(FromDwarfInfoOutput::new_type_entry_with_children_warnings(
            entry,
            children_warnings,
        ))
    }

    fn array_type_from_dwarf_info(entry: &DwarfInfo) -> Result<FromDwarfInfoOutput, String> {
        let id = TypeEntryId::new(entry.offset());
        let type_ref = match entry.type_offset() {
            Some(type_ref) => Ok(TypeEntryId::new(type_ref)),
            None => Err("array_type entry should have type"),
        }?;
        let upper_bound = entry.children().iter().find_map(|child| match child.tag() {
            DwarfTag::DW_TAG_subrange_type => child.upper_bound(),
            _ => None,
        });

        let entry = TypeEntry::new_array_type_entry(id, type_ref, upper_bound);
        Ok(FromDwarfInfoOutput::new_type_entry_with_no_children_warnings(entry))
    }

    fn function_type_from_dwarf_info(entry: &DwarfInfo) -> FromDwarfInfoOutput {
        let id = TypeEntryId::new(entry.offset());
        let mut children_warnings = Vec::new();
        let argument_type_ref = entry
            .children()
            .iter()
            .flat_map(|entry| match (entry.tag(), entry.type_offset()) {
                (DwarfTag::DW_TAG_formal_parameter, Some(type_ref)) => {
                    Some(TypeEntryId::new(type_ref))
                }
                (DwarfTag::DW_TAG_formal_parameter, None) => {
                    children_warnings.push(String::from("formal_parameter entry should have type"));
                    None
                }
                _ => None,
            })
            .collect();
        let return_type_ref = entry
            .type_offset()
            .map(|type_ref| TypeEntryId::new(type_ref));

        let entry = TypeEntry::new_function_type_entry(id, argument_type_ref, return_type_ref);
        FromDwarfInfoOutput::new_type_entry_with_children_warnings(entry, children_warnings)
    }
}
