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
                DwarfTag::DW_TAG_subrange_type => (),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::dwarf::{DwarfInfoBuilder, DwarfTag, Location, Offset};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn extract_test() {
        init();

        let mut type_entry_repository = TypeEntryRepository::new();
        let mut global_variables_extractor =
            GlobalVariablesExtractor::new(&mut type_entry_repository);
        let dwarf_info_iterators = vec![
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_structure_type)
                .offset(Offset::new(45))
                .name("hoge")
                .byte_size(16)
                .children(vec![
                    DwarfInfoBuilder::new()
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .offset(Offset::new(58))
                        .name("hoge")
                        .type_offset(Offset::new(98))
                        .data_member_location(0)
                        .build(),
                    DwarfInfoBuilder::new()
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .offset(Offset::new(71))
                        .name("hogehoge")
                        .type_offset(Offset::new(105))
                        .data_member_location(4)
                        .build(),
                    DwarfInfoBuilder::new()
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .offset(Offset::new(84))
                        .name("array")
                        .type_offset(Offset::new(112))
                        .data_member_location(8)
                        .build(),
                ])
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_base_type)
                .offset(Offset::new(98))
                .name("int")
                .byte_size(4)
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_base_type)
                .offset(Offset::new(105))
                .name("char")
                .byte_size(1)
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_array_type)
                .offset(Offset::new(112))
                .type_offset(Offset::new(98))
                .children(vec![DwarfInfoBuilder::new()
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .offset(Offset::new(121))
                    .type_offset(Offset::new(128))
                    .upper_bound(1)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_base_type)
                .offset(Offset::new(128))
                .name("long unsigned int")
                .byte_size(8)
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_typedef)
                .offset(Offset::new(135))
                .name("Hoge")
                .type_offset(Offset::new(45))
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_array_type)
                .offset(Offset::new(147))
                .type_offset(Offset::new(135))
                .children(vec![DwarfInfoBuilder::new()
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .offset(Offset::new(156))
                    .type_offset(Offset::new(128))
                    .upper_bound(2)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_variable)
                .offset(Offset::new(163))
                .name("hoges")
                .location(Location::new(16480))
                .type_offset(Offset::new(147))
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_unimplemented)
                .offset(Offset::new(185))
                .name("main")
                .type_offset(Offset::new(98))
                .build(),
        ];

        let expected_variables = vec![GlobalVariable::new(
            Some(Address::new(Location::new(16480))),
            String::from("hoges"),
            TypeEntryId::new(Offset::new(147)),
        )];
        let expected_types = vec![
            TypeEntry::new_structure_type_entry(
                TypeEntryId::new(Offset::new(45)),
                String::from("hoge"),
                16,
                vec![
                    StructureTypeMemberEntry {
                        name: String::from("hoge"),
                        location: 0,
                        type_ref: TypeEntryId::new(Offset::new(98)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("hogehoge"),
                        location: 4,
                        type_ref: TypeEntryId::new(Offset::new(105)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("array"),
                        location: 8,
                        type_ref: TypeEntryId::new(Offset::new(112)),
                    },
                ],
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(98)),
                String::from("int"),
                4,
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(105)),
                String::from("char"),
                1,
            ),
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(112)),
                TypeEntryId::new(Offset::new(98)),
                Some(1),
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(128)),
                String::from("long unsigned int"),
                8,
            ),
            TypeEntry::new_typedef_entry(
                TypeEntryId::new(Offset::new(135)),
                String::from("Hoge"),
                TypeEntryId::new(Offset::new(45)),
            ),
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(147)),
                TypeEntryId::new(Offset::new(135)),
                Some(2),
            ),
        ];
        let got_variables = global_variables_extractor.extract(dwarf_info_iterators.into_iter());
        assert_eq!(expected_variables, got_variables);
        for expected_type in expected_types {
            let got_type = type_entry_repository
                .find_by_id(&expected_type.id())
                .map(TypeEntry::clone);
            assert_eq!(Some(expected_type), got_type);
        }
    }
}
