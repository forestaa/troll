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
    use crate::library::dwarf::tests::DwarfInfoBuilder;
    use crate::library::dwarf::{DwarfTag, Location, Offset};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn extract_test(
        infos: Vec<DwarfInfo>,
        expected_variables: Vec<GlobalVariable>,
        expected_types: Vec<TypeEntry>,
    ) {
        init();

        let mut type_entry_repository = TypeEntryRepository::new();
        let mut global_variables_extractor =
            GlobalVariablesExtractor::new(&mut type_entry_repository);

        let got_variables = global_variables_extractor.extract(infos.into_iter());
        assert_eq!(expected_variables, got_variables);
        for expected_type in expected_types {
            let got_type = type_entry_repository
                .find_by_id(&expected_type.id())
                .map(TypeEntry::clone);
            assert_eq!(Some(expected_type), got_type);
        }
    }

    #[test]
    fn extract_const() {
        let infos = vec![
            DwarfInfoBuilder::new()
                .offset(Offset::new(45))
                .tag(DwarfTag::DW_TAG_variable)
                .name("c")
                .type_offset(Offset::new(72))
                .location(Location::new(8196))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(65))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(72))
                .tag(DwarfTag::DW_TAG_const_type)
                .type_offset(Offset::new(65))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(77))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset::new(65))
                .build(),
        ];

        let expected_variables = vec![GlobalVariable::new(
            Some(Address::new(Location::new(8196))),
            String::from("c"),
            TypeEntryId::new(Offset::new(72)),
        )];
        let expected_types = vec![
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(65)),
                String::from("int"),
                4,
            ),
            TypeEntry::new_const_type_entry(
                TypeEntryId::new(Offset::new(72)),
                TypeEntryId::new(Offset::new(65)),
            ),
        ];

        extract_test(infos, expected_variables, expected_types);
    }

    #[test]
    fn extract_pointer() {
        let infos = vec![
            DwarfInfoBuilder::new()
                .offset(Offset::new(45))
                .tag(DwarfTag::DW_TAG_variable)
                .name("p")
                .type_offset(Offset::new(65))
                .location(Location::new(16432))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(65))
                .tag(DwarfTag::DW_TAG_pointer_type)
                .byte_size(8)
                .type_offset(Offset::new(71))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(71))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(78))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset::new(71))
                .build(),
        ];

        let expected_variables = vec![GlobalVariable::new(
            Some(Address::new(Location::new(16432))),
            String::from("p"),
            TypeEntryId::new(Offset::new(65)),
        )];
        let expected_types = vec![
            TypeEntry::new_pointer_type_entry(
                TypeEntryId::new(Offset::new(65)),
                8,
                Some(TypeEntryId::new(Offset::new(71))),
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(71)),
                String::from("int"),
                4,
            ),
        ];

        extract_test(infos, expected_variables, expected_types);
    }

    #[test]
    fn extract_typedef() {
        let infos = vec![
            DwarfInfoBuilder::new()
                .offset(Offset::new(45))
                .tag(DwarfTag::DW_TAG_typedef)
                .name("uint8")
                .type_offset(Offset::new(57))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(57))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(64))
                .tag(DwarfTag::DW_TAG_variable)
                .name("a")
                .type_offset(Offset::new(45))
                .location(Location::new(16428))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(84))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset::new(114))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(114))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
        ];

        let expected_variables = vec![GlobalVariable::new(
            Some(Address::new(Location::new(16428))),
            String::from("a"),
            TypeEntryId::new(Offset::new(45)),
        )];
        let expected_types = vec![
            TypeEntry::new_typedef_entry(
                TypeEntryId::new(Offset::new(45)),
                String::from("uint8"),
                TypeEntryId::new(Offset::new(57)),
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(57)),
                String::from("unsigned int"),
                4,
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(114)),
                String::from("int"),
                4,
            ),
        ];

        extract_test(infos, expected_variables, expected_types);
    }

    #[test]
    fn extract_array() {
        let infos = vec![
            DwarfInfoBuilder::new()
                .offset(Offset::new(45))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset::new(68))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset::new(54))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset::new(61))
                    .upper_bound(2)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(61))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(8)
                .name("long unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(68))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(75))
                .tag(DwarfTag::DW_TAG_variable)
                .name("hoges")
                .type_offset(Offset::new(45))
                .location(Location::new(16432))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(97))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset::new(68))
                .build(),
        ];

        let expected_variables = vec![GlobalVariable::new(
            Some(Address::new(Location::new(16432))),
            String::from("hoges"),
            TypeEntryId::new(Offset::new(45)),
        )];
        let expected_types = vec![
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(45)),
                TypeEntryId::new(Offset::new(68)),
                Some(2),
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(61)),
                String::from("long unsigned int"),
                8,
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(68)),
                String::from("int"),
                4,
            ),
        ];

        extract_test(infos, expected_variables, expected_types);
    }

    #[test]
    fn extract_structure() {
        let infos = vec![
            DwarfInfoBuilder::new()
                .offset(Offset::new(45))
                .tag(DwarfTag::DW_TAG_structure_type)
                .name("hoge")
                .byte_size(8)
                .children(vec![
                    DwarfInfoBuilder::new()
                        .offset(Offset::new(58))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("hoge")
                        .type_offset(Offset::new(101))
                        .data_member_location(0)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset::new(71))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("fuga")
                        .type_offset(Offset::new(108))
                        .data_member_location(4)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset::new(84))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("pohe")
                        .type_offset(Offset::new(115))
                        .byte_size(4)
                        .data_member_location(4)
                        .build(),
                ])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(101))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(108))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(1)
                .name("char")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(115))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(122))
                .tag(DwarfTag::DW_TAG_variable)
                .name("hoge")
                .type_offset(Offset::new(45))
                .location(Location::new(16432))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(144))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset::new(101))
                .build(),
        ];

        let expected_variables = vec![GlobalVariable::new(
            Some(Address::new(Location::new(16432))),
            String::from("hoge"),
            TypeEntryId::new(Offset::new(45)),
        )];
        let expected_types = vec![
            TypeEntry::new_structure_type_entry(
                TypeEntryId::new(Offset::new(45)),
                String::from("hoge"),
                8,
                vec![
                    StructureTypeMemberEntry {
                        name: String::from("hoge"),
                        location: 0,
                        type_ref: TypeEntryId::new(Offset::new(101)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("fuga"),
                        location: 4,
                        type_ref: TypeEntryId::new(Offset::new(108)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("pohe"),
                        location: 4,
                        type_ref: TypeEntryId::new(Offset::new(115)),
                    },
                ],
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(101)),
                String::from("int"),
                4,
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(108)),
                String::from("char"),
                1,
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(115)),
                String::from("unsigned int"),
                4,
            ),
        ];

        extract_test(infos, expected_variables, expected_types);
    }

    #[test]
    fn extract_complex_structure() {
        let infos = vec![
            DwarfInfoBuilder::new()
                .offset(Offset::new(45))
                .tag(DwarfTag::DW_TAG_structure_type)
                .name("student")
                .byte_size(16)
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset::new(58))
                    .tag(DwarfTag::DW_TAG_unimplemented)
                    .name("name")
                    .type_offset(Offset::new(72))
                    .data_member_location(0)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(72))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset::new(95))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset::new(81))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset::new(88))
                    .upper_bound(15)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(88))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(8)
                .name("long unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(95))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(1)
                .name("char")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(102))
                .tag(DwarfTag::DW_TAG_structure_type)
                .name("hoge")
                .byte_size(32)
                .children(vec![
                    DwarfInfoBuilder::new()
                        .offset(Offset::new(115))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("hoge")
                        .type_offset(Offset::new(155))
                        .data_member_location(0)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset::new(128))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("array")
                        .type_offset(Offset::new(168))
                        .data_member_location(8)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset::new(141))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("student")
                        .type_offset(Offset::new(45))
                        .data_member_location(16)
                        .build(),
                ])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(155))
                .tag(DwarfTag::DW_TAG_pointer_type)
                .byte_size(8)
                .type_offset(Offset::new(161))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(161))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(168))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset::new(161))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset::new(177))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset::new(88))
                    .upper_bound(1)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(184))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset::new(102))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset::new(193))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset::new(88))
                    .upper_bound(2)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(200))
                .tag(DwarfTag::DW_TAG_variable)
                .name("hoge")
                .type_offset(Offset::new(184))
                .location(Location::new(16480))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset::new(222))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset::new(161))
                .build(),
        ];

        let expected_variables = vec![GlobalVariable::new(
            Some(Address::new(Location::new(16480))),
            String::from("hoge"),
            TypeEntryId::new(Offset::new(184)),
        )];
        let expected_types = vec![
            TypeEntry::new_structure_type_entry(
                TypeEntryId::new(Offset::new(45)),
                String::from("student"),
                16,
                vec![StructureTypeMemberEntry {
                    name: String::from("name"),
                    location: 0,
                    type_ref: TypeEntryId::new(Offset::new(72)),
                }],
            ),
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(72)),
                TypeEntryId::new(Offset::new(95)),
                Some(15),
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(88)),
                String::from("long unsigned int"),
                8,
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(95)),
                String::from("char"),
                1,
            ),
            TypeEntry::new_structure_type_entry(
                TypeEntryId::new(Offset::new(102)),
                String::from("hoge"),
                32,
                vec![
                    StructureTypeMemberEntry {
                        name: String::from("hoge"),
                        location: 0,
                        type_ref: TypeEntryId::new(Offset::new(155)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("array"),
                        location: 8,
                        type_ref: TypeEntryId::new(Offset::new(168)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("student"),
                        location: 16,
                        type_ref: TypeEntryId::new(Offset::new(45)),
                    },
                ],
            ),
            TypeEntry::new_pointer_type_entry(
                TypeEntryId::new(Offset::new(155)),
                8,
                Some(TypeEntryId::new(Offset::new(161))),
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(161)),
                String::from("int"),
                4,
            ),
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(168)),
                TypeEntryId::new(Offset::new(161)),
                Some(1),
            ),
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(184)),
                TypeEntryId::new(Offset::new(102)),
                Some(2),
            ),
        ];

        extract_test(infos, expected_variables, expected_types);
    }
}
