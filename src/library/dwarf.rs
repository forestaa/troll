use log::info;
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
    DW_TAG_enumeration_type,
    DW_TAG_enumerator,
    DW_TAG_structure_type,
    DW_TAG_union_type,
    DW_TAG_array_type,
    DW_TAG_subroutine_type,
    DW_TAG_subrange_type,
    DW_TAG_volatile_type,
    DW_TAG_formal_parameter,
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
            gimli::DW_TAG_enumeration_type => DwarfTag::DW_TAG_enumeration_type,
            gimli::DW_TAG_enumerator => DwarfTag::DW_TAG_enumerator,
            gimli::DW_TAG_structure_type => DwarfTag::DW_TAG_structure_type,
            gimli::DW_TAG_union_type => DwarfTag::DW_TAG_union_type,
            gimli::DW_TAG_array_type => DwarfTag::DW_TAG_array_type,
            gimli::DW_TAG_subroutine_type => DwarfTag::DW_TAG_subroutine_type,
            gimli::DW_TAG_subrange_type => DwarfTag::DW_TAG_subrange_type,
            gimli::DW_TAG_volatile_type => DwarfTag::DW_TAG_volatile_type,
            gimli::DW_TAG_formal_parameter => DwarfTag::DW_TAG_formal_parameter,
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
    bit_size: Option<usize>,
    bit_offset: Option<usize>,
    location: Option<Location>,
    upper_bound: Option<usize>,
    const_value: Option<isize>,
    data_member_location: Option<usize>,
    declaration: Option<bool>,
    specification: Option<Offset>,
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

    pub fn byte_size(&self) -> Option<usize> {
        self.byte_size
    }

    pub fn bit_size(&self) -> Option<usize> {
        self.bit_size
    }

    pub fn bit_offset(&self) -> Option<usize> {
        self.bit_offset
    }

    pub fn location(&self) -> Option<Location> {
        self.location.clone()
    }

    pub fn upper_bound(&self) -> Option<usize> {
        self.upper_bound
    }

    pub fn const_value(&self) -> Option<isize> {
        self.const_value
    }

    pub fn data_member_location(&self) -> Option<usize> {
        self.data_member_location
    }

    pub fn declaration(&self) -> Option<bool> {
        self.declaration
    }

    pub fn specification(&self) -> Option<Offset> {
        self.specification.clone()
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
                let bit_size = Self::get_bit_size(entry);
                let bit_offset = Self::get_bit_offset(entry);
                let location = Self::get_location(header, encoding, entry);
                let upper_bound = Self::get_upper_bound(entry);
                let const_value = Self::get_const_value(entry);
                let data_member_location = Self::get_data_member_location(entry);
                let declaration = Self::get_declaration(entry);
                let specification = Self::get_specification(header, entry);

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
                    bit_size,
                    bit_offset,
                    location,
                    upper_bound,
                    const_value,
                    data_member_location,
                    declaration,
                    specification,
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

    fn get_bit_size<'abbrev, 'unit>(
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<usize> {
        entry
            .attr_value(gimli::DW_AT_bit_size)
            .unwrap()
            .and_then(|value| value.udata_value())
            .map(|byte_size| byte_size as usize)
    }

    fn get_bit_offset<'abbrev, 'unit>(
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<usize> {
        entry
            .attr_value(gimli::DW_AT_bit_offset)
            .unwrap()
            .and_then(|value| value.udata_value())
            .map(|byte_size| byte_size as usize)
    }

    fn get_location<'input, 'abbrev, 'unit>(
        header: &gimli::CompilationUnitHeader<
            gimli::read::EndianSlice<'input, gimli::RunTimeEndian>,
        >,
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
                .and_then(|location| {
                    let mut eval = match location.exprloc_value() {
                        Some(value) => Some(value.evaluation(encoding)),
                        None => {
                            info!("location attribute  which is not exprloc is not supported yet: offset = {:#x}", entry.offset().to_debug_info_offset(header).0);
                            None
                        }
                    }?;
                    let mut result = eval.evaluate().unwrap();
                    while result != gimli::EvaluationResult::Complete {
                        match result {
                            gimli::EvaluationResult::RequiresRelocatedAddress(address) => {
                                result = eval.resume_with_relocated_address(address).unwrap()
                            }
                            result => {
                                info!("Evaluation requires more information: {:?}", result);
                                return None
                            }
                        }
                    }

                    let result = eval.result();
                    if let Some(gimli::Location::Address { address }) =
                        result.get(0).map(|piece| piece.location)
                    {
                        Some(address)
                    } else {
                        info!(
                            "The head of Evaluation result is not address: results is {:?}",
                            result
                        );
                        None
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
        entry
            .attr_value(gimli::DW_AT_upper_bound)
            .unwrap()
            .and_then(|value| value.udata_value())
            .map(|byte_size| byte_size as usize)
    }

    fn get_const_value<'abbrev, 'unit>(
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<isize> {
        entry
            .attr_value(gimli::DW_AT_const_value)
            .unwrap()
            .and_then(|value| value.sdata_value())
            .map(|const_value| const_value as isize)
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

    fn get_declaration<'input, 'abbrev, 'unit>(
        entry: &gimli::DebuggingInformationEntry<
            'abbrev,
            'unit,
            gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
        >,
    ) -> Option<bool> {
        if let Some(gimli::read::AttributeValue::Flag(flag)) =
            entry.attr_value(gimli::DW_AT_declaration).unwrap()
        {
            Some(flag)
        } else {
            None
        }
    }

    fn get_specification<'input, 'abbrev, 'unit>(
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
            entry.attr_value(gimli::DW_AT_specification).unwrap()
        {
            Some(Offset::new(offset.to_debug_info_offset(header).0))
        } else {
            None
        }
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

pub struct DwarfInfoBuilder<OffsetP, TagP> {
    offset: OffsetP,
    tag: TagP,
    name: Option<String>,
    type_offset: Option<Offset>,
    byte_size: Option<usize>,
    bit_size: Option<usize>,
    bit_offset: Option<usize>,
    location: Option<Location>,
    upper_bound: Option<usize>,
    const_value: Option<isize>,
    data_member_location: Option<usize>,
    declaration: Option<bool>,
    specification: Option<Offset>,
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
            bit_size: None,
            bit_offset: None,
            location: None,
            upper_bound: None,
            const_value: None,
            data_member_location: None,
            declaration: None,
            specification: None,
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
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            location: self.location,
            upper_bound: self.upper_bound,
            const_value: self.const_value,
            data_member_location: self.data_member_location,
            declaration: self.declaration,
            specification: self.specification,
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
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            location: self.location,
            upper_bound: self.upper_bound,
            const_value: self.const_value,
            data_member_location: self.data_member_location,
            declaration: self.declaration,
            specification: self.specification,
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
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            location: self.location,
            upper_bound: self.upper_bound,
            const_value: self.const_value,
            data_member_location: self.data_member_location,
            declaration: self.declaration,
            specification: self.specification,
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

    pub fn bit_size(mut self, size: usize) -> Self {
        self.bit_size = Some(size);
        self
    }

    pub fn bit_offset(mut self, offset: usize) -> Self {
        self.bit_offset = Some(offset);
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

    pub fn const_value(mut self, const_value: isize) -> Self {
        self.const_value = Some(const_value);
        self
    }

    pub fn data_member_location(mut self, data_member_location: usize) -> Self {
        self.data_member_location = Some(data_member_location);
        self
    }

    pub fn declaration(mut self, declaration: bool) -> Self {
        self.declaration = Some(declaration);
        self
    }

    pub fn specification(mut self, specification: Offset) -> Self {
        self.specification = Some(specification);
        self
    }

    pub fn children(mut self, children: Vec<DwarfInfo>) -> Self {
        self.children = children;
        self
    }
}
