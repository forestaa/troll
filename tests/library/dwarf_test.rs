extern crate troll;

use troll::library::dwarf::*;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

fn dwarf_info_intoiterator_test<S: Into<String>>(elf_path: S, expected: Vec<DwarfInfo>) {
    init();

    let got: Vec<DwarfInfo> = DwarfInfoIntoIterator::new(elf_path.into())
        .into_iter()
        .collect();
    assert_eq!(expected, got);
}

#[test]
#[ignore]
fn dwarf_info_const() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/const", expected);
}

#[test]
#[ignore]
fn dwarf_info_pointer() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/pointer", expected);
}

#[test]
#[ignore]
fn dwarf_info_typedef() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/typedef", expected);
}

#[test]
#[ignore]
fn dwarf_info_array() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/array", expected);
}

#[test]
#[ignore]
fn dwarf_info_enum() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/enum", expected);
}

#[test]
#[ignore]
fn dwarf_info_anonymous_enum() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/anonymous-enum", expected);
}

#[test]
#[ignore]
fn dwarf_info_structure() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/structure", expected);
}

#[test]
#[ignore]
fn dwarf_info_union() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/union", expected);
}

#[test]
#[ignore]
fn dwarf_info_anonymous_union_structure() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/anonymous-union-structure", expected);
}

#[test]
#[ignore]
fn dwarf_info_function_pointer() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/function-pointer", expected);
}

#[test]
#[ignore]
fn dwarf_info_void_function_pointer() {
    let expected = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_subroutine_type)
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(46))
            .tag(DwarfTag::DW_TAG_variable)
            .name("callback")
            .type_offset(Offset::new(68))
            .location(Location::new(16432))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(68))
            .tag(DwarfTag::DW_TAG_pointer_type)
            .byte_size(8)
            .type_offset(Offset::new(45))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(74))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(104))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(104))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
    ];

    dwarf_info_intoiterator_test("examples/void-function-pointer", expected);
}

#[test]
#[ignore]
fn dwarf_info_complex_structure() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/complex-structure", expected);
}

#[test]
#[ignore]
fn dwarf_info_many_compilation_units() {
    let expected = vec![
        DwarfInfoBuilder::new()
            .offset(Offset::new(45))
            .tag(DwarfTag::DW_TAG_variable)
            .name("c")
            .type_offset(Offset::new(65))
            .location(Location::new(16424))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(65))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(72))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("main")
            .type_offset(Offset::new(65))
            .children(vec![DwarfInfoBuilder::new()
                .offset(Offset::new(106))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .build()])
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(120))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("sub1")
            .declaration(true)
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(182))
            .tag(DwarfTag::DW_TAG_variable)
            .name("c")
            .type_offset(Offset::new(202))
            .location(Location::new(16424))
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(202))
            .tag(DwarfTag::DW_TAG_base_type)
            .byte_size(4)
            .name("int")
            .build(),
        DwarfInfoBuilder::new()
            .offset(Offset::new(209))
            .tag(DwarfTag::DW_TAG_unimplemented)
            .name("sub1")
            .type_offset(Offset::new(202))
            .children(vec![DwarfInfoBuilder::new()
                .offset(Offset::new(239))
                .tag(DwarfTag::DW_TAG_formal_parameter)
                .name("i")
                .type_offset(Offset::new(202))
                .build()])
            .build(),
    ];

    dwarf_info_intoiterator_test("examples/many-compilation-units", expected);
}

#[test]
#[ignore]
fn dwarf_info_extern() {
    let expected = vec![
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

    dwarf_info_intoiterator_test("examples/extern", expected);
}
