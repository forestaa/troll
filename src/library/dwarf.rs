use log::error;
use object::Object;
use std::{borrow, fs};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Offset(usize);
impl Offset {
    pub fn new(size: usize) -> Offset {
        Offset(size)
    }
}

impl Into<usize> for Offset {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location(usize);
impl Location {
    pub fn new(size: usize) -> Location {
        Location(size)
    }

    pub fn add(&mut self, size: usize) {
        self.0 += size;
    }
}

impl Into<usize> for Location {
    fn into(self) -> usize {
        self.0
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum DwarfTag {
    DW_TAG_variable,
    DW_TAG_typedef,
    DW_TAG_const_type,
    DW_TAG_pointer_type,
    DW_TAG_base_type,
    DW_TAG_structure_type,
    DW_TAG_array_type,
    DW_TAG_subrange_type,
    DW_TAG_unimplemented,
}

impl From<gimli::DwTag> for DwarfTag {
    fn from(tag: gimli::DwTag) -> DwarfTag {
        match tag {
            gimli::DW_TAG_variable => DwarfTag::DW_TAG_variable,
            gimli::DW_TAG_typedef => DwarfTag::DW_TAG_typedef,
            gimli::DW_TAG_const_type => DwarfTag::DW_TAG_const_type,
            gimli::DW_TAG_pointer_type => DwarfTag::DW_TAG_pointer_type,
            gimli::DW_TAG_base_type => DwarfTag::DW_TAG_base_type,
            gimli::DW_TAG_structure_type => DwarfTag::DW_TAG_structure_type,
            gimli::DW_TAG_array_type => DwarfTag::DW_TAG_array_type,
            gimli::DW_TAG_subrange_type => DwarfTag::DW_TAG_subrange_type,
            _ => DwarfTag::DW_TAG_unimplemented,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DwarfInfo {
    offset: Offset,
    tag: DwarfTag,
    name: Option<String>,
    type_offset: Option<Offset>,
    byte_size: Option<usize>,
    location: Option<Location>,
    upper_bound: Option<usize>,
    data_member_location: Option<usize>,
    children: Vec<DwarfInfo>,
}

impl DwarfInfo {
    pub fn offset(&self) -> Offset {
        self.offset.clone()
    }

    pub fn tag(&self) -> DwarfTag {
        self.tag.clone()
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn type_offset(&self) -> Option<Offset> {
        self.type_offset.clone()
    }

    pub fn size(&self) -> Option<usize> {
        self.byte_size
    }

    pub fn location(&self) -> Option<Location> {
        self.location.clone()
    }

    pub fn upper_bound(&self) -> Option<usize> {
        self.upper_bound
    }

    pub fn data_member_location(&self) -> Option<usize> {
        self.data_member_location
    }

    pub fn children(&self) -> &Vec<DwarfInfo> {
        &self.children
    }
}

pub struct DwarfInfoIntoIterator {
    elf_path: String,
}

impl DwarfInfoIntoIterator {
    pub fn new(elf_path: String) -> DwarfInfoIntoIterator {
        DwarfInfoIntoIterator { elf_path }
    }

    fn next_info<'input, 'abbrev, 'unit>(
        header: &gimli::CompilationUnitHeader<
            gimli::read::EndianSlice<'input, gimli::RunTimeEndian>,
        >,
        dwarf: &gimli::read::Dwarf<gimli::read::EndianSlice<'input, gimli::RunTimeEndian>>,
        encoding: gimli::Encoding,
        entries: &mut gimli::read::EntriesCursor<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<DwarfInfo> {
        let _ = entries.next_entry();
        match entries.current() {
            None => None,
            Some(entry) => {
                let offset = Self::get_offset(header, entry);
                let tag = DwarfTag::from(entry.tag());
                let name = Self::get_name(dwarf, entry);
                let type_offset = Self::get_type_offset(header, entry);
                let byte_size = Self::get_byte_size(entry);
                let location = Self::get_location(encoding, entry);
                let upper_bound = Self::get_upper_bound(entry);
                let data_member_location = Self::get_data_member_location(entry);

                let mut children = Vec::new();
                if entry.has_children() {
                    while let Some(info) = Self::next_info(header, dwarf, encoding, entries) {
                        children.push(info);
                    }
                }
                Some(DwarfInfo {
                    offset,
                    tag,
                    name,
                    type_offset,
                    byte_size,
                    location,
                    upper_bound,
                    data_member_location,
                    children: children,
                })
            }
        }
    }

    fn get_offset<'input, 'abbrev, 'unit>(
        header: &gimli::CompilationUnitHeader<
            gimli::read::EndianSlice<'input, gimli::RunTimeEndian>,
        >,
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Offset {
        Offset::new(entry.offset().to_debug_info_offset(header).0)
    }

    fn get_name<'input, 'abbrev, 'unit>(
        dwarf: &gimli::read::Dwarf<gimli::read::EndianSlice<'input, gimli::RunTimeEndian>>,
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<String> {
        entry
            .attr_value(gimli::DW_AT_name)
            .unwrap()
            .and_then(|value| value.string_value(&dwarf.debug_str))
            .map(|r| r.to_string().unwrap())
            .map(String::from)
    }

    fn get_type_offset<'input, 'abbrev, 'unit>(
        header: &gimli::CompilationUnitHeader<
            gimli::read::EndianSlice<'input, gimli::RunTimeEndian>,
        >,
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<Offset> {
        if let Some(gimli::read::AttributeValue::UnitRef(offset)) =
            entry.attr_value(gimli::DW_AT_type).unwrap()
        {
            Some(Offset::new(offset.to_debug_info_offset(header).0))
        } else {
            None
        }
    }

    fn get_byte_size<'abbrev, 'unit>(
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<usize> {
        entry
            .attr_value(gimli::DW_AT_byte_size)
            .unwrap()
            .and_then(|value| value.udata_value())
            .map(|byte_size| byte_size as usize)
    }

    fn get_location<'abbrev, 'unit>(
        encoding: gimli::Encoding,
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<Location> {
        // TODO: always should get location
        // Currently not because handling RequiresFrameBase from Evaluation is needed
        match DwarfTag::from(entry.tag()) {
            DwarfTag::DW_TAG_variable => entry
                .attr_value(gimli::DW_AT_location)
                .unwrap()
                .map(|location| {
                    let mut eval = location
                        .exprloc_value()
                        .expect(&Self::expect_error_message(
                            "location attribute should be exprloc",
                            &entry,
                        ))
                        .evaluation(encoding);
                    let mut result = eval.evaluate().unwrap();
                    while result != gimli::EvaluationResult::Complete {
                        match result {
                            gimli::EvaluationResult::RequiresRelocatedAddress(address) => {
                                result = eval.resume_with_relocated_address(address).unwrap()
                            }
                            result => {
                                error!("Evaluation requires more information: {:?}", result);
                                unimplemented!()
                            }
                        }
                    }

                    let result = eval.result();
                    if let Some(gimli::Location::Address { address }) =
                        result.get(0).map(|piece| piece.location)
                    {
                        address
                    } else {
                        error!(
                            "The head of Evaluation result is not address: results is {:?}",
                            result
                        );
                        unimplemented!()
                    }
                })
                .map(|size| Location::new(size as usize)),
            _ => None,
        }
    }

    fn get_upper_bound<'abbrev, 'unit>(
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<usize> {
        if let Some(gimli::read::AttributeValue::Data1(upper_bound)) =
            entry.attr_value(gimli::DW_AT_upper_bound).unwrap()
        {
            Some(upper_bound as usize)
        } else {
            None
        }
    }

    fn get_data_member_location<'abbrev, 'unit>(
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<usize> {
        if let Some(gimli::read::AttributeValue::Udata(location)) =
            entry.attr_value(gimli::DW_AT_data_member_location).unwrap()
        {
            Some(location as usize)
        } else {
            None
        }
    }

    fn expect_error_message(
        message: &str,
        entry: &gimli::DebuggingInformationEntry<gimli::read::EndianSlice<gimli::RunTimeEndian>>,
    ) -> String {
        format!("{}: offset = {:#x}", message, entry.offset().0)
    }
}

impl IntoIterator for DwarfInfoIntoIterator {
    type Item = DwarfInfo;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let file = fs::File::open(&self.elf_path).unwrap();
        let mmap = unsafe { memmap::Mmap::map(&file).unwrap() };
        let object = object::File::parse(&*mmap).unwrap();
        let endian = if object.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
            Ok(object
                .section_data_by_name(id.name())
                .unwrap_or(borrow::Cow::Borrowed(&[][..])))
        };
        // Load a supplementary section. We don't have a supplementary object file,
        // so always return an empty slice.
        let load_section_sup = |_| Ok(borrow::Cow::Borrowed(&[][..]));

        // Load all of the sections.
        let dwarf_cow = gimli::Dwarf::load(&load_section, &load_section_sup).unwrap();

        // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
        let borrow_section: &dyn for<'b> Fn(
            &'b borrow::Cow<[u8]>,
        )
            -> gimli::EndianSlice<'b, gimli::RunTimeEndian> =
            &|section| gimli::EndianSlice::new(&*section, endian);

        // Create `EndianSlice`s for all of the sections.
        let dwarf = dwarf_cow.borrow(&borrow_section);

        // Iterate over the compilation units.
        let mut units = dwarf.units();
        let mut infos = Vec::new();
        while let Some(header) = units.next().unwrap() {
            let unit = dwarf.unit(header).unwrap();
            let mut entries = unit.entries();
            let _ = entries.next_entry(); // skip compilatoin unit entry
            while let Some(info) = Self::next_info(&header, &dwarf, unit.encoding(), &mut entries) {
                infos.push(info);
            }
        }

        infos.into_iter()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct DwarfInfoBuilder<OffsetP, TagP> {
        offset: OffsetP,
        tag: TagP,
        name: Option<String>,
        type_offset: Option<Offset>,
        byte_size: Option<usize>,
        location: Option<Location>,
        upper_bound: Option<usize>,
        data_member_location: Option<usize>,
        children: Vec<DwarfInfo>,
    }

    impl DwarfInfoBuilder<(), ()> {
        pub fn new() -> Self {
            DwarfInfoBuilder {
                offset: (),
                tag: (),
                name: None,
                type_offset: None,
                byte_size: None,
                location: None,
                upper_bound: None,
                data_member_location: None,
                children: Vec::new(),
            }
        }
    }

    impl DwarfInfoBuilder<Offset, DwarfTag> {
        pub fn build(self) -> DwarfInfo {
            DwarfInfo {
                offset: self.offset,
                tag: self.tag,
                name: self.name,
                type_offset: self.type_offset,
                byte_size: self.byte_size,
                location: self.location,
                upper_bound: self.upper_bound,
                data_member_location: self.data_member_location,
                children: self.children,
            }
        }
    }

    impl<OffsetP> DwarfInfoBuilder<OffsetP, ()> {
        pub fn tag(self, tag: DwarfTag) -> DwarfInfoBuilder<OffsetP, DwarfTag> {
            DwarfInfoBuilder {
                offset: self.offset,
                tag: tag,
                name: self.name,
                type_offset: self.type_offset,
                byte_size: self.byte_size,
                location: self.location,
                upper_bound: self.upper_bound,
                data_member_location: self.data_member_location,
                children: self.children,
            }
        }
    }

    impl<TagP> DwarfInfoBuilder<(), TagP> {
        pub fn offset(self, offset: Offset) -> DwarfInfoBuilder<Offset, TagP> {
            DwarfInfoBuilder {
                offset: offset,
                tag: self.tag,
                name: self.name,
                type_offset: self.type_offset,
                byte_size: self.byte_size,
                location: self.location,
                upper_bound: self.upper_bound,
                data_member_location: self.data_member_location,
                children: self.children,
            }
        }
    }

    impl<OffsetP, TagP> DwarfInfoBuilder<OffsetP, TagP> {
        pub fn name<S: Into<String>>(mut self, name: S) -> Self {
            self.name = Some(name.into());
            self
        }

        pub fn type_offset(mut self, type_offset: Offset) -> Self {
            self.type_offset = Some(type_offset);
            self
        }

        pub fn byte_size(mut self, size: usize) -> Self {
            self.byte_size = Some(size);
            self
        }

        pub fn location(mut self, location: Location) -> Self {
            self.location = Some(location);
            self
        }

        pub fn upper_bound(mut self, upper_bound: usize) -> Self {
            self.upper_bound = Some(upper_bound);
            self
        }

        pub fn data_member_location(mut self, data_member_location: usize) -> Self {
            self.data_member_location = Some(data_member_location);
            self
        }

        pub fn children(mut self, children: Vec<DwarfInfo>) -> Self {
            self.children = children;
            self
        }
    }

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
                .offset(Offset(45))
                .tag(DwarfTag::DW_TAG_variable)
                .name("c")
                .type_offset(Offset(72))
                .location(Location(8196))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(65))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(72))
                .tag(DwarfTag::DW_TAG_const_type)
                .type_offset(Offset(65))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(77))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset(65))
                .build(),
        ];

        dwarf_info_intoiterator_test("examples/const", expected);
    }

    #[test]
    #[ignore]
    fn dwarf_info_pointer() {
        let expected = vec![
            DwarfInfoBuilder::new()
                .offset(Offset(45))
                .tag(DwarfTag::DW_TAG_variable)
                .name("p")
                .type_offset(Offset(65))
                .location(Location(16432))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(65))
                .tag(DwarfTag::DW_TAG_pointer_type)
                .byte_size(8)
                .type_offset(Offset(71))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(71))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(78))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset(71))
                .build(),
        ];

        dwarf_info_intoiterator_test("examples/pointer", expected);
    }

    #[test]
    #[ignore]
    fn dwarf_info_typedef() {
        let expected = vec![
            DwarfInfoBuilder::new()
                .offset(Offset(45))
                .tag(DwarfTag::DW_TAG_typedef)
                .name("uint8")
                .type_offset(Offset(57))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(57))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(64))
                .tag(DwarfTag::DW_TAG_variable)
                .name("a")
                .type_offset(Offset(45))
                .location(Location(16428))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(84))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset(114))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(114))
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
                .offset(Offset(45))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset(68))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset(54))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset(61))
                    .upper_bound(2)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(61))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(8)
                .name("long unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(68))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(75))
                .tag(DwarfTag::DW_TAG_variable)
                .name("hoges")
                .type_offset(Offset(45))
                .location(Location(16432))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(97))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset(68))
                .build(),
        ];

        dwarf_info_intoiterator_test("examples/array", expected);
    }

    #[test]
    #[ignore]
    fn dwarf_info_structure() {
        let expected = vec![
            DwarfInfoBuilder::new()
                .offset(Offset(45))
                .tag(DwarfTag::DW_TAG_structure_type)
                .name("hoge")
                .byte_size(8)
                .children(vec![
                    DwarfInfoBuilder::new()
                        .offset(Offset(58))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("hoge")
                        .type_offset(Offset(101))
                        .data_member_location(0)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset(71))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("fuga")
                        .type_offset(Offset(108))
                        .data_member_location(4)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset(84))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("pohe")
                        .type_offset(Offset(115))
                        .byte_size(4)
                        .data_member_location(4)
                        .build(),
                ])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(101))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(108))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(1)
                .name("char")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(115))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(122))
                .tag(DwarfTag::DW_TAG_variable)
                .name("hoge")
                .type_offset(Offset(45))
                .location(Location(16432))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(144))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset(101))
                .build(),
        ];

        dwarf_info_intoiterator_test("examples/structure", expected);
    }

    #[test]
    #[ignore]
    fn dwarf_info_complex_structure() {
        let expected = vec![
            DwarfInfoBuilder::new()
                .offset(Offset(45))
                .tag(DwarfTag::DW_TAG_structure_type)
                .name("student")
                .byte_size(16)
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset(58))
                    .tag(DwarfTag::DW_TAG_unimplemented)
                    .name("name")
                    .type_offset(Offset(72))
                    .data_member_location(0)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(72))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset(95))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset(81))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset(88))
                    .upper_bound(15)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(88))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(8)
                .name("long unsigned int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(95))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(1)
                .name("char")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(102))
                .tag(DwarfTag::DW_TAG_structure_type)
                .name("hoge")
                .byte_size(32)
                .children(vec![
                    DwarfInfoBuilder::new()
                        .offset(Offset(115))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("hoge")
                        .type_offset(Offset(155))
                        .data_member_location(0)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset(128))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("array")
                        .type_offset(Offset(168))
                        .data_member_location(8)
                        .build(),
                    DwarfInfoBuilder::new()
                        .offset(Offset(141))
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .name("student")
                        .type_offset(Offset(45))
                        .data_member_location(16)
                        .build(),
                ])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(155))
                .tag(DwarfTag::DW_TAG_pointer_type)
                .byte_size(8)
                .type_offset(Offset(161))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(161))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(168))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset(161))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset(177))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset(88))
                    .upper_bound(1)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(184))
                .tag(DwarfTag::DW_TAG_array_type)
                .type_offset(Offset(102))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset(193))
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .type_offset(Offset(88))
                    .upper_bound(2)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(200))
                .tag(DwarfTag::DW_TAG_variable)
                .name("hoge")
                .type_offset(Offset(184))
                .location(Location(16480))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(222))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset(161))
                .build(),
        ];

        dwarf_info_intoiterator_test("examples/complex-structure", expected);
    }

    #[test]
    #[ignore]
    fn dwarf_info_many_compilation_units() {
        let expected = vec![
            DwarfInfoBuilder::new()
                .offset(Offset(45))
                .tag(DwarfTag::DW_TAG_variable)
                .name("c")
                .type_offset(Offset(65))
                .location(Location(16424))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(65))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(72))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("main")
                .type_offset(Offset(65))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset(106))
                    .tag(DwarfTag::DW_TAG_unimplemented)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(120))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("sub1")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(182))
                .tag(DwarfTag::DW_TAG_variable)
                .name("c")
                .type_offset(Offset(202))
                .location(Location(16424))
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(202))
                .tag(DwarfTag::DW_TAG_base_type)
                .byte_size(4)
                .name("int")
                .build(),
            DwarfInfoBuilder::new()
                .offset(Offset(209))
                .tag(DwarfTag::DW_TAG_unimplemented)
                .name("sub1")
                .type_offset(Offset(202))
                .children(vec![DwarfInfoBuilder::new()
                    .offset(Offset(239))
                    .tag(DwarfTag::DW_TAG_formal_parameter)
                    .name("i")
                    .type_offset(Offset(202))
                    .build()])
                .build(),
        ];

        dwarf_info_intoiterator_test("examples/many-compilation-units", expected);
    }
}
