extern crate troll;

use troll::domain::global_variable::*;
use troll::domain::global_variable_view::*;
use troll::domain::global_variable_view_factory::*;
use troll::domain::type_entry::*;
use troll::domain::type_entry_repository::TypeEntryRepository;
use troll::domain::variable_declaration_repository::VariableDeclarationRepository;
use troll::library::dwarf::{Location, Offset};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

fn from_global_variable_test(
    defined_types: Vec<TypeEntry>,
    variable_decs: Vec<VariableDeclarationEntry>,
    global_variable: GlobalVariable,
    expected_view: GlobalVariableView,
) {
    from_global_variables_test(
        defined_types,
        variable_decs,
        vec![global_variable],
        vec![expected_view],
    );
}

fn from_global_variables_test(
    defined_types: Vec<TypeEntry>,
    variable_decs: Vec<VariableDeclarationEntry>,
    global_variables: Vec<GlobalVariable>,
    expected_views: Vec<GlobalVariableView>,
) {
    init();

    let mut type_entry_repository = TypeEntryRepository::new();
    for defined_type in defined_types {
        type_entry_repository.save(defined_type);
    }
    let mut variable_declaration_repository = VariableDeclarationRepository::new();
    for variable_dec in variable_decs {
        variable_declaration_repository.save(variable_dec);
    }

    let factory =
        GlobalVariableViewFactory::new(&type_entry_repository, &variable_declaration_repository);

    let got_views: Vec<GlobalVariableView> = global_variables
        .into_iter()
        .flat_map(|global_variable| factory.from_global_variable(global_variable))
        .collect();
    assert_eq!(expected_views, got_views);
}

#[test]
fn from_global_variable_const() {
    let defined_types = vec![
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(65)), String::from("int"), 4),
        TypeEntry::new_const_type_entry(
            TypeEntryId::new(Offset::new(72)),
            TypeEntryId::new(Offset::new(65)),
        ),
    ];

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(8196))),
        String::from("c"),
        TypeEntryId::new(Offset::new(72)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("c"),
        Some(Address::new(Location::new(8196))),
        4,
        TypeView::new_const_type_view(TypeView::new_base_type_view("int")),
        vec![],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_pointer() {
    let defined_types = vec![
        TypeEntry::new_pointer_type_entry(
            TypeEntryId::new(Offset::new(65)),
            8,
            Some(TypeEntryId::new(Offset::new(71))),
        ),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(71)), String::from("int"), 4),
    ];

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16432))),
        String::from("p"),
        TypeEntryId::new(Offset::new(65)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("p"),
        Some(Address::new(Location::new(16432))),
        8,
        TypeView::new_pointer_type_view(TypeView::new_base_type_view("int")),
        vec![],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_typedef() {
    let defined_types = vec![
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

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16428))),
        String::from("a"),
        TypeEntryId::new(Offset::new(45)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("a"),
        Some(Address::new(Location::new(16428))),
        4,
        TypeView::new_typedef_type_view("uint8", TypeView::new_base_type_view("unsigned int")),
        vec![],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_array() {
    let defined_types = vec![
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

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16432))),
        String::from("hoges"),
        TypeEntryId::new(Offset::new(45)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("hoges"),
        Some(Address::new(Location::new(16432))),
        12,
        TypeView::new_array_type_view(TypeView::new_base_type_view("int"), Some(2)),
        vec![
            GlobalVariableView::new(
                String::from("0"),
                Some(Address::new(Location::new(16432))),
                4,
                TypeView::new_base_type_view("int"),
                vec![],
            ),
            GlobalVariableView::new(
                String::from("1"),
                Some(Address::new(Location::new(16436))),
                4,
                TypeView::new_base_type_view("int"),
                vec![],
            ),
            GlobalVariableView::new(
                String::from("2"),
                Some(Address::new(Location::new(16440))),
                4,
                TypeView::new_base_type_view("int"),
                vec![],
            ),
        ],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_enum() {
    let defined_types = vec![
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

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16428))),
        String::from("ab"),
        TypeEntryId::new(Offset::new(45)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("ab"),
        Some(Address::new(Location::new(16428))),
        4,
        TypeView::new_enum_type_view(
            Some("AB"),
            TypeView::new_base_type_view("unsigned int"),
            vec![
                Enumerator {
                    name: String::from("A"),
                    value: 0,
                },
                Enumerator {
                    name: String::from("B"),
                    value: 1,
                },
            ],
        ),
        vec![],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_anonymous_enum() {
    let defined_types = vec![
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

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16428))),
        String::from("ab"),
        TypeEntryId::new(Offset::new(45)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("ab"),
        Some(Address::new(Location::new(16428))),
        4,
        TypeView::new_enum_type_view::<String>(
            None,
            TypeView::new_base_type_view("unsigned int"),
            vec![
                Enumerator {
                    name: String::from("A"),
                    value: 0,
                },
                Enumerator {
                    name: String::from("B"),
                    value: 1,
                },
            ],
        ),
        vec![],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_structure() {
    let defined_types = vec![
        TypeEntry::new_structure_type_entry(
            TypeEntryId::new(Offset::new(45)),
            Some(String::from("hoge")),
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
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(101)), String::from("int"), 4),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(108)), String::from("char"), 1),
        TypeEntry::new_base_type_entry(
            TypeEntryId::new(Offset::new(115)),
            String::from("unsigned int"),
            4,
        ),
    ];

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16432))),
        String::from("hoge"),
        TypeEntryId::new(Offset::new(45)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("hoge"),
        Some(Address::new(Location::new(16432))),
        8,
        TypeView::new_structure_type_view(Some("hoge")),
        vec![
            GlobalVariableView::new(
                String::from("hoge"),
                Some(Address::new(Location::new(16432))),
                4,
                TypeView::new_base_type_view("int"),
                vec![],
            ),
            GlobalVariableView::new(
                String::from("fuga"),
                Some(Address::new(Location::new(16436))),
                1,
                TypeView::new_base_type_view("char"),
                vec![],
            ),
            GlobalVariableView::new(
                String::from("pohe"),
                Some(Address::new(Location::new(16436))),
                4,
                TypeView::new_base_type_view("unsigned int"),
                vec![],
            ),
        ],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_union() {
    let defined_types = vec![
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

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16428))),
        String::from("book"),
        TypeEntryId::new(Offset::new(45)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("book"),
        Some(Address::new(Location::new(16428))),
        4,
        TypeView::new_union_type_view(Some("book")),
        vec![
            GlobalVariableView::new(
                String::from("name"),
                Some(Address::new(Location::new(16428))),
                1,
                TypeView::new_base_type_view("char"),
                vec![],
            ),
            GlobalVariableView::new(
                String::from("price"),
                Some(Address::new(Location::new(16428))),
                4,
                TypeView::new_base_type_view("int"),
                vec![],
            ),
        ],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_anonymous_union_structure() {
    let defined_types = vec![
        TypeEntry::new_structure_type_entry(
            TypeEntryId::new(Offset::new(45)),
            None,
            4,
            vec![StructureTypeMemberEntry {
                name: String::from("a"),
                type_ref: TypeEntryId::new(Offset::new(66)),
                location: 0,
            }],
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

    let global_variables = vec![
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

    let expected_views = vec![
        GlobalVariableView::new(
            String::from("a"),
            Some(Address::new(Location::new(16428))),
            4,
            TypeView::new_structure_type_view::<String>(None),
            vec![GlobalVariableView::new(
                String::from("a"),
                Some(Address::new(Location::new(16428))),
                4,
                TypeView::new_base_type_view("int"),
                vec![],
            )],
        ),
        GlobalVariableView::new(
            String::from("ab"),
            Some(Address::new(Location::new(16432))),
            4,
            TypeView::new_union_type_view::<String>(None),
            vec![
                GlobalVariableView::new(
                    String::from("a"),
                    Some(Address::new(Location::new(16432))),
                    4,
                    TypeView::new_base_type_view("int"),
                    vec![],
                ),
                GlobalVariableView::new(
                    String::from("b"),
                    Some(Address::new(Location::new(16432))),
                    1,
                    TypeView::new_base_type_view("char"),
                    vec![],
                ),
            ],
        ),
    ];

    from_global_variables_test(defined_types, Vec::new(), global_variables, expected_views);
}

#[test]
fn from_global_variable_function_pointer() {
    let defined_types = vec![
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

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16424))),
        String::from("sub2"),
        TypeEntryId::new(Offset::new(101)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("sub2"),
        Some(Address::new(Location::new(16424))),
        8,
        TypeView::new_pointer_type_view(TypeView::new_function_type_view()),
        vec![],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_complex_structure() {
    let defined_types = vec![
        TypeEntry::new_structure_type_entry(
            TypeEntryId::new(Offset::new(45)),
            Some(String::from("student")),
            4,
            vec![StructureTypeMemberEntry {
                name: String::from("name"),
                location: 0,
                type_ref: TypeEntryId::new(Offset::new(72)),
            }],
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

    let global_variable = GlobalVariable::new_variable(
        Some(Address::new(Location::new(16480))),
        String::from("hoge"),
        TypeEntryId::new(Offset::new(184)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("hoge"),
        Some(Address::new(Location::new(16480))),
        48,
        TypeView::new_array_type_view(TypeView::new_structure_type_view(Some("hoge")), Some(1)),
        vec![
            GlobalVariableView::new(
                String::from("0"),
                Some(Address::new(Location::new(16480))),
                24,
                TypeView::new_structure_type_view(Some("hoge")),
                vec![
                    GlobalVariableView::new(
                        String::from("hoge"),
                        Some(Address::new(Location::new(16480))),
                        8,
                        TypeView::new_pointer_type_view(TypeView::new_base_type_view("int")),
                        vec![],
                    ),
                    GlobalVariableView::new(
                        String::from("array"),
                        Some(Address::new(Location::new(16488))),
                        8,
                        TypeView::new_array_type_view(TypeView::new_base_type_view("int"), Some(1)),
                        vec![
                            GlobalVariableView::new(
                                String::from("0"),
                                Some(Address::new(Location::new(16488))),
                                4,
                                TypeView::new_base_type_view("int"),
                                vec![],
                            ),
                            GlobalVariableView::new(
                                String::from("1"),
                                Some(Address::new(Location::new(16492))),
                                4,
                                TypeView::new_base_type_view("int"),
                                vec![],
                            ),
                        ],
                    ),
                    GlobalVariableView::new(
                        String::from("student"),
                        Some(Address::new(Location::new(16496))),
                        4,
                        TypeView::new_structure_type_view(Some("student")),
                        vec![GlobalVariableView::new(
                            String::from("name"),
                            Some(Address::new(Location::new(16496))),
                            4,
                            TypeView::new_array_type_view(
                                TypeView::new_base_type_view("char"),
                                Some(3),
                            ),
                            vec![
                                GlobalVariableView::new(
                                    String::from("0"),
                                    Some(Address::new(Location::new(16496))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                                GlobalVariableView::new(
                                    String::from("1"),
                                    Some(Address::new(Location::new(16497))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                                GlobalVariableView::new(
                                    String::from("2"),
                                    Some(Address::new(Location::new(16498))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                                GlobalVariableView::new(
                                    String::from("3"),
                                    Some(Address::new(Location::new(16499))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
            GlobalVariableView::new(
                String::from("1"),
                Some(Address::new(Location::new(16504))),
                24,
                TypeView::new_structure_type_view(Some("hoge")),
                vec![
                    GlobalVariableView::new(
                        String::from("hoge"),
                        Some(Address::new(Location::new(16504))),
                        8,
                        TypeView::new_pointer_type_view(TypeView::new_base_type_view("int")),
                        vec![],
                    ),
                    GlobalVariableView::new(
                        String::from("array"),
                        Some(Address::new(Location::new(16512))),
                        8,
                        TypeView::new_array_type_view(TypeView::new_base_type_view("int"), Some(1)),
                        vec![
                            GlobalVariableView::new(
                                String::from("0"),
                                Some(Address::new(Location::new(16512))),
                                4,
                                TypeView::new_base_type_view("int"),
                                vec![],
                            ),
                            GlobalVariableView::new(
                                String::from("1"),
                                Some(Address::new(Location::new(16516))),
                                4,
                                TypeView::new_base_type_view("int"),
                                vec![],
                            ),
                        ],
                    ),
                    GlobalVariableView::new(
                        String::from("student"),
                        Some(Address::new(Location::new(16520))),
                        4,
                        TypeView::new_structure_type_view(Some("student")),
                        vec![GlobalVariableView::new(
                            String::from("name"),
                            Some(Address::new(Location::new(16520))),
                            4,
                            TypeView::new_array_type_view(
                                TypeView::new_base_type_view("char"),
                                Some(3),
                            ),
                            vec![
                                GlobalVariableView::new(
                                    String::from("0"),
                                    Some(Address::new(Location::new(16520))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                                GlobalVariableView::new(
                                    String::from("1"),
                                    Some(Address::new(Location::new(16521))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                                GlobalVariableView::new(
                                    String::from("2"),
                                    Some(Address::new(Location::new(16522))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                                GlobalVariableView::new(
                                    String::from("3"),
                                    Some(Address::new(Location::new(16523))),
                                    1,
                                    TypeView::new_base_type_view("char"),
                                    vec![],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );

    from_global_variable_test(defined_types, Vec::new(), global_variable, expected_view);
}

#[test]
fn from_global_variable_extern() {
    let defined_types = vec![
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(55)), String::from("int"), 4),
        TypeEntry::new_base_type_entry(TypeEntryId::new(Offset::new(136)), String::from("int"), 4),
    ];

    let variable_decs = vec![
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

    let global_variable = GlobalVariable::new_variable_with_spec(
        Some(Address::new(Location::new(16428))),
        VariableDeclarationEntryId::new(Offset::new(126)),
    );

    let expected_view = GlobalVariableView::new(
        String::from("c"),
        Some(Address::new(Location::new(16428))),
        4,
        TypeView::new_base_type_view("int"),
        vec![],
    );

    from_global_variable_test(defined_types, variable_decs, global_variable, expected_view);
}
