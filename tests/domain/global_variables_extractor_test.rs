extern crate troll;

use troll::domain::global_variable::*;
use troll::domain::global_variables_extractor::*;
use troll::domain::type_entry::*;
use troll::domain::type_entry_repository::TypeEntryRepository;
use troll::domain::variable_declaration_repository::VariableDeclarationRepository;
use troll::library::dwarf::{DwarfInfo, DwarfInfoBuilder, DwarfTag, Location, Offset};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

fn extract_test(
    infos: Vec<DwarfInfo>,
    expected_variables: Vec<GlobalVariable>,
    expected_types: Vec<TypeEntry>,
    expected_decs: Vec<VariableDeclarationEntry>,
) {
    init();

    let mut type_entry_repository = TypeEntryRepository::new();
    let mut variable_declaration_repository = VariableDeclarationRepository::new();
    let mut global_variables_extractor = GlobalVariablesExtractor::new(
        &mut type_entry_repository,
        &mut variable_declaration_repository,
    );

    let got_variables = global_variables_extractor.extract(infos.into_iter());
    assert_eq!(expected_variables, got_variables);
    for expected_type in expected_types {
        let got_type = type_entry_repository
            .find_by_id(&expected_type.id())
            .map(TypeEntry::clone);
        assert_eq!(Some(expected_type), got_type);
    }
    for expected_dec in expected_decs {
        let got_dec = variable_declaration_repository
            .find_by_id(&expected_dec.id)
            .map(VariableDeclarationEntry::clone);
        assert_eq!(Some(expected_dec), got_dec);
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

    let expected_variables = vec![GlobalVariable::new_variable(
        Some(Address::new(Location::new(8196))),
        String::from("c"),
        TypeEntryId::new(Offset::new(72)),
    )];
    let expected_types = vec![
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(65)), String::from("int"), 4),
        TypeEntry::new_const_type_entry(
            TypeEntryId::new(Offset::new(72)),
            TypeEntryId::new(Offset::new(65)),
        ),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
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

    let expected_variables = vec![GlobalVariable::new_variable(
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
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(71)), String::from("int"), 4),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
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

    let expected_variables = vec![GlobalVariable::new_variable(
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
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(114)), String::from("int"), 4),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
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

    let expected_variables = vec![GlobalVariable::new_variable(
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
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(68)), String::from("int"), 4),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
}

#[test]
fn extract_enum() {
    let infos = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_enumeration_type)
            .name("AB")
            .byte_size(4)
            .type_offset(Offset::new(71))
            .children(vec![
                DwarfInfoBuilder::new()
                    .offset(Offset::new(62))
                    .tag(DwarfTag::DW_TAG_enumerator)
                    .name("A")
                    .const_value(0)
                    .build(),
                DwarfInfoBuilder::new()
                    .offset(Offset::new(66))
                    .tag(DwarfTag::DW_TAG_enumerator)
                    .name("B")
                    .const_value(1)
                    .build(),
            ])
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(71))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("unsigned int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(78))
            .tag(DwarfTag::DW_TAG_variable)
            .name("ab")
            .type_offset(Offset::new(45))
            .location(Location::new(16428))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(99))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(129))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(129))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
    ];

    let expected_variables = vec![GlobalVariable::new_variable(
        Some(Address::new(Location::new(16428))),
        String::from("ab"),
        TypeEntryId::new(Offset::new(45)),
    )];
    let expected_types = vec![
        TypeEntry::new_enum_type_entry(
            TypeEntryId::new(Offset::new(45)),
            Some(String::from("AB")),
            TypeEntryId::new(Offset::new(71)),
            vec![
                EnumeratorEntry {
                    name: String::from("A"),
                    value: 0,
                },
                EnumeratorEntry {
                    name: String::from("B"),
                    value: 1,
                },
            ],
        ),
        TypeEntry::new_base_type_entry(
            TypeEntryId::new(Offset::new(71)),
            String::from("unsigned int"),
            4,
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(129)), String::from("int"), 4),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
}

#[test]
fn extract_anonymous_enum() {
    let infos = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_enumeration_type)
            .byte_size(4)
            .type_offset(Offset::new(68))
            .children(vec![
                DwarfInfoBuilder::new()
                    .offset(Offset::new(59))
                    .tag(DwarfTag::DW_TAG_enumerator)
                    .name("A")
                    .const_value(0)
                    .build(),
                DwarfInfoBuilder::new()
                    .offset(Offset::new(63))
                    .tag(DwarfTag::DW_TAG_enumerator)
                    .name("B")
                    .const_value(1)
                    .build(),
            ])
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(68))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("unsigned int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(75))
            .tag(DwarfTag::DW_TAG_variable)
            .name("ab")
            .type_offset(Offset::new(45))
            .location(Location::new(16428))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(96))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(126))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(126))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
    ];

    let expected_variables = vec![GlobalVariable::new_variable(
        Some(Address::new(Location::new(16428))),
        String::from("ab"),
        TypeEntryId::new(Offset::new(45)),
    )];
    let expected_types = vec![
        TypeEntry::new_enum_type_entry(
            TypeEntryId::new(Offset::new(45)),
            None,
            TypeEntryId::new(Offset::new(68)),
            vec![
                EnumeratorEntry {
                    name: String::from("A"),
                    value: 0,
                },
                EnumeratorEntry {
                    name: String::from("B"),
                    value: 1,
                },
            ],
        ),
        TypeEntry::new_base_type_entry(
            TypeEntryId::new(Offset::new(68)),
            String::from("unsigned int"),
            4,
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(126)), String::from("int"), 4),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
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
                    .bit_size(1)
                    .bit_offset(23)
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

    let expected_variables = vec![GlobalVariable::new_variable(
        Some(Address::new(Location::new(16432))),
        String::from("hoge"),
        TypeEntryId::new(Offset::new(45)),
    )];
    let expected_types = vec![
        TypeEntry::new_structure_type_entry(
            TypeEntryId::new(Offset::new(45)),
            Some(String::from("hoge")),
            8,
            vec![
                StructureTypeMemberEntryBuilder::new()
                    .name("hoge")
                    .location(0)
                    .type_ref(TypeEntryId::new(Offset::new(101)))
                    .build(),
                StructureTypeMemberEntryBuilder::new()
                    .name("fuga")
                    .location(4)
                    .type_ref(TypeEntryId::new(Offset::new(108)))
                    .build(),
                StructureTypeMemberEntryBuilder::new()
                    .name("pohe")
                    .location(4)
                    .type_ref(TypeEntryId::new(Offset::new(115)))
                    .bit_size(1)
                    .bit_offset(23)
                    .build(),
            ],
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(101)), String::from("int"), 4),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(108)), String::from("char"), 1),
        TypeEntry::new_base_type_entry(
            TypeEntryId::new(Offset::new(115)),
            String::from("unsigned int"),
            4,
        ),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
}

#[test]
fn extract_union() {
    let infos = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_union_type)
            .name("book")
            .byte_size(4)
            .children(vec![
                DwarfInfoBuilder::new()
                    .offset(Offset::new(58))
                    .tag(DwarfTag::DW_TAG_unimplemented)
                    .name("name")
                    .type_offset(Offset::new(83))
                    .build(),
                DwarfInfoBuilder::new()
                    .offset(Offset::new(70))
                    .tag(DwarfTag::DW_TAG_unimplemented)
                    .name("price")
                    .type_offset(Offset::new(90))
                    .build(),
            ])
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(83))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(1)
            .name("char")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(90))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(97))
            .tag(DwarfTag::DW_TAG_variable)
            .name("book")
            .type_offset(Offset::new(45))
            .location(Location::new(16428))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(119))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(90))
            .build(),
    ];

    let expected_variables = vec![GlobalVariable::new_variable(
        Some(Address::new(Location::new(16428))),
        String::from("book"),
        TypeEntryId::new(Offset::new(45)),
    )];

    let expected_types = vec![
        TypeEntry::new_union_type_entry(
            TypeEntryId::new(Offset::new(45)),
            Some(String::from("book")),
            4,
            vec![
                UnionTypeMemberEntry {
                    name: String::from("name"),
                    type_ref: TypeEntryId::new(Offset::new(83)),
                },
                UnionTypeMemberEntry {
                    name: String::from("price"),
                    type_ref: TypeEntryId::new(Offset::new(90)),
                },
            ],
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(83)), String::from("char"), 1),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(90)), String::from("int"), 4),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
}

#[test]
fn extract_anonymous_union_structure() {
    let infos = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_structure_type)
            .byte_size(4)
            .children(vec![DwarfInfoBuilder::new()
                .offset(Offset::new(54))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("a")
                .type_offset(Offset::new(66))
                .data_member_location(0)
                .build()])
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(66))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(73))
            .tag(DwarfTag::DW_TAG_variable)
            .name("a")
            .type_offset(Offset::new(45))
            .location(Location::new(16428))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(93))
            .tag(DwarfTag::DW_TAG_union_type)
            .byte_size(4)
            .children(vec![
                DwarfInfoBuilder::new()
                    .offset(Offset::new(102))
                    .tag(DwarfTag::DW_TAG_unimplemented)
                    .name("a")
                    .type_offset(Offset::new(66))
                    .build(),
                DwarfInfoBuilder::new()
                    .offset(Offset::new(112))
                    .tag(DwarfTag::DW_TAG_unimplemented)
                    .name("b")
                    .type_offset(Offset::new(123))
                    .build(),
            ])
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(123))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(1)
            .name("char")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(130))
            .tag(DwarfTag::DW_TAG_variable)
            .name("ab")
            .type_offset(Offset::new(93))
            .location(Location::new(16432))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(151))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(66))
            .build(),
    ];

    let expected_variables = vec![
        GlobalVariable::new_variable(
            Some(Address::new(Location::new(16428))),
            String::from("a"),
            TypeEntryId::new(Offset::new(45)),
        ),
        GlobalVariable::new_variable(
            Some(Address::new(Location::new(16432))),
            String::from("ab"),
            TypeEntryId::new(Offset::new(93)),
        ),
    ];

    let expected_types = vec![
        TypeEntry::new_structure_type_entry(
            TypeEntryId::new(Offset::new(45)),
            None,
            4,
            vec![StructureTypeMemberEntryBuilder::new()
                .name("a")
                .type_ref(TypeEntryId::new(Offset::new(66)))
                .location(0)
                .build()],
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(66)), String::from("int"), 4),
        TypeEntry::new_union_type_entry(
            TypeEntryId::new(Offset::new(93)),
            None,
            4,
            vec![
                UnionTypeMemberEntry {
                    name: String::from("a"),
                    type_ref: TypeEntryId::new(Offset::new(66)),
                },
                UnionTypeMemberEntry {
                    name: String::from("b"),
                    type_ref: TypeEntryId::new(Offset::new(123)),
                },
            ],
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(123)), String::from("char"), 1),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
}

#[test]
fn extract_function_pointer() {
    let infos = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_subroutine_type)
            .type_offset(Offset::new(65))
            .children(vec![
                DwarfInfoBuilder::new()
                    .offset(Offset::new(54))
                    .tag(DwarfTag::DW_TAG_formal_parameter)
                    .type_offset(Offset::new(65))
                    .build(),
                DwarfInfoBuilder::new()
                    .offset(Offset::new(59))
                    .tag(DwarfTag::DW_TAG_formal_parameter)
                    .type_offset(Offset::new(72))
                    .build(),
            ])
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(65))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(72))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(1)
            .name("char")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(79))
            .tag(DwarfTag::DW_TAG_variable)
            .name("sub2")
            .type_offset(Offset::new(101))
            .location(Location::new(16424))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(101))
            .tag(DwarfTag::DW_TAG_pointer_type)
            .byte_size(8)
            .type_offset(Offset::new(45))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(107))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(65))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(137))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("sub1")
            .type_offset(Offset::new(65))
            .children(vec![
                DwarfInfoBuilder::new()
                    .offset(Offset::new(167))
                    .tag(DwarfTag::DW_TAG_formal_parameter)
                    .name("i")
                    .type_offset(Offset::new(65))
                    .build(),
                DwarfInfoBuilder::new()
                    .offset(Offset::new(179))
                    .tag(DwarfTag::DW_TAG_formal_parameter)
                    .name("c")
                    .type_offset(Offset::new(72))
                    .build(),
            ])
            .build(),
    ];

    let expected_variables = vec![GlobalVariable::new_variable(
        Some(Address::new(Location::new(16424))),
        String::from("sub2"),
        TypeEntryId::new(Offset::new(101)),
    )];
    let expected_types = vec![
        TypeEntry::new_function_type_entry(
            TypeEntryId::new(Offset::new(45)),
            vec![
                TypeEntryId::new(Offset::new(65)),
                TypeEntryId::new(Offset::new(72)),
            ],
            Some(TypeEntryId::new(Offset::new(65))),
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(65)), String::from("int"), 4),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(72)), String::from("char"), 1),
        TypeEntry::new_pointer_type_entry(
            TypeEntryId::new(Offset::new(101)),
            8,
            Some(TypeEntryId::new(Offset::new(45))),
        ),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
}

#[test]
fn extract_complex_structure() {
    let infos = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_structure_type)
            .name("student")
            .byte_size(4)
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
                .upper_bound(3)
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
            .byte_size(24)
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
                .upper_bound(1)
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

    let expected_variables = vec![GlobalVariable::new_variable(
        Some(Address::new(Location::new(16480))),
        String::from("hoge"),
        TypeEntryId::new(Offset::new(184)),
    )];
    let expected_types = vec![
        TypeEntry::new_structure_type_entry(
            TypeEntryId::new(Offset::new(45)),
            Some(String::from("student")),
            4,
            vec![StructureTypeMemberEntryBuilder::new()
                .name("name")
                .location(0)
                .type_ref(TypeEntryId::new(Offset::new(72)))
                .build()],
        ),
        TypeEntry::new_array_type_entry(
            TypeEntryId::new(Offset::new(72)),
            TypeEntryId::new(Offset::new(95)),
            Some(3),
        ),
        TypeEntry::new_base_type_entry(
            TypeEntryId::new(Offset::new(88)),
            String::from("long unsigned int"),
            8,
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(95)), String::from("char"), 1),
        TypeEntry::new_structure_type_entry(
            TypeEntryId::new(Offset::new(102)),
            Some(String::from("hoge")),
            24,
            vec![
                StructureTypeMemberEntryBuilder::new()
                    .name("hoge")
                    .location(0)
                    .type_ref(TypeEntryId::new(Offset::new(155)))
                    .build(),
                StructureTypeMemberEntryBuilder::new()
                    .name("array")
                    .location(8)
                    .type_ref(TypeEntryId::new(Offset::new(168)))
                    .build(),
                StructureTypeMemberEntryBuilder::new()
                    .name("student")
                    .location(16)
                    .type_ref(TypeEntryId::new(Offset::new(45)))
                    .build(),
            ],
        ),
        TypeEntry::new_pointer_type_entry(
            TypeEntryId::new(Offset::new(155)),
            8,
            Some(TypeEntryId::new(Offset::new(161))),
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(161)), String::from("int"), 4),
        TypeEntry::new_array_type_entry(
            TypeEntryId::new(Offset::new(168)),
            TypeEntryId::new(Offset::new(161)),
            Some(1),
        ),
        TypeEntry::new_array_type_entry(
            TypeEntryId::new(Offset::new(184)),
            TypeEntryId::new(Offset::new(102)),
            Some(1),
        ),
    ];

    extract_test(infos, expected_variables, expected_types, Vec::new());
}

#[test]
fn extract_extern() {
    let infos = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_variable)
            .name("c")
            .type_offset(Offset::new(55))
            .declaration(true)
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(55))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(62))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(55))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(126))
            .tag(DwarfTag::DW_TAG_variable)
            .name("c")
            .type_offset(Offset::new(136))
            .declaration(true)
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(136))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(143))
            .tag(DwarfTag::DW_TAG_variable)
            .specification(Offset::new(126))
            .location(Location::new(16428))
            .build(),
    ];

    let expected_variables = vec![GlobalVariable::new_variable_with_spec(
        Some(Address::new(Location::new(16428))),
        VariableDeclarationEntryId::new(Offset::new(126)),
    )];
    let expected_types = vec![
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(55)), String::from("int"), 4),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(136)), String::from("int"), 4),
    ];
    let expected_decs = vec![
        VariableDeclarationEntry::new(
            VariableDeclarationEntryId::new(Offset::new(45)),
            String::from("c"),
            TypeEntryId::new(Offset::new(55)),
        ),
        VariableDeclarationEntry::new(
            VariableDeclarationEntryId::new(Offset::new(126)),
            String::from("c"),
            TypeEntryId::new(Offset::new(136)),
        ),
    ];

    extract_test(infos, expected_variables, expected_types, expected_decs);
}
