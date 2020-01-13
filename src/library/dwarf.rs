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
    tag: DwarfTag,
    offset: Offset,
    name: Option<String>,
    size: Option<usize>,
    location: Option<Location>,
    type_offset: Option<Offset>,
    upper_bound: Option<usize>,
    children: Vec<DwarfInfo>,
}

impl DwarfInfo {
    pub fn new(
        tag: DwarfTag,
        offset: Offset,
        name: Option<String>,
        size: Option<usize>,
        location: Option<Location>,
        type_offset: Option<Offset>,
        upper_bound: Option<usize>,
        children: Vec<DwarfInfo>,
    ) -> DwarfInfo {
        DwarfInfo {
            tag,
            offset,
            name,
            size,
            location,
            type_offset,
            upper_bound,
            children,
        }
    }
    pub fn tag(&self) -> DwarfTag {
        self.tag.clone()
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn size(&self) -> Option<usize> {
        self.size
    }

    pub fn offset(&self) -> Offset {
        self.offset.clone()
    }

    pub fn location(&self) -> Option<Location> {
        self.location.clone()
    }

    pub fn type_offset(&self) -> Option<Offset> {
        self.type_offset.clone()
    }

    pub fn upper_bound(&self) -> Option<usize> {
        self.upper_bound
    }

    pub fn children(&self) -> &Vec<DwarfInfo> {
        &self.children
    }
}

struct DwarfInfoBuilder<TagP, OffsetP> {
    tag: TagP,
    offset: OffsetP,
    name: Option<String>,
    size: Option<usize>,
    location: Option<Location>,
    type_offset: Option<Offset>,
    upper_bound: Option<usize>,
    children: Vec<DwarfInfo>,
}

impl DwarfInfoBuilder<(), ()> {
    pub fn new() -> Self {
        DwarfInfoBuilder {
            tag: (),
            offset: (),
            name: None,
            size: None,
            location: None,
            type_offset: None,
            upper_bound: None,
            children: Vec::new(),
        }
    }
}

impl DwarfInfoBuilder<DwarfTag, Offset> {
    pub fn build(self) -> DwarfInfo {
        DwarfInfo {
            tag: self.tag,
            offset: self.offset,
            name: self.name,
            size: self.size,
            location: self.location,
            type_offset: self.type_offset,
            upper_bound: self.upper_bound,
            children: self.children,
        }
    }
}

impl<OffsetP> DwarfInfoBuilder<(), OffsetP> {
    pub fn tag(self, tag: DwarfTag) -> DwarfInfoBuilder<DwarfTag, OffsetP> {
        DwarfInfoBuilder {
            tag: tag,
            offset: self.offset,
            name: self.name,
            size: self.size,
            location: self.location,
            type_offset: self.type_offset,
            upper_bound: self.upper_bound,
            children: self.children,
        }
    }
}

impl<TagP> DwarfInfoBuilder<TagP, ()> {
    pub fn offset(self, offset: Offset) -> DwarfInfoBuilder<TagP, Offset> {
        DwarfInfoBuilder {
            tag: self.tag,
            offset: offset,
            name: self.name,
            size: self.size,
            location: self.location,
            type_offset: self.type_offset,
            upper_bound: self.upper_bound,
            children: self.children,
        }
    }
}

impl<TagP, OffsetP> DwarfInfoBuilder<TagP, OffsetP> {
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    pub fn type_offset(mut self, type_offset: Offset) -> Self {
        self.type_offset = Some(type_offset);
        self
    }

    pub fn upper_bound(mut self, upper_bound: usize) -> Self {
        self.upper_bound = Some(upper_bound);
        self
    }

    pub fn children(mut self, children: Vec<DwarfInfo>) -> Self {
        self.children = children;
        self
    }
}

pub struct DwarfInfoIterator<'abbrev, 'unit, 'input> {
    entries: gimli::read::EntriesCursor<
        'abbrev,
        'unit,
        gimli::read::EndianSlice<'abbrev, gimli::RunTimeEndian>,
    >,
    encoding: gimli::Encoding,
    dwarf: gimli::read::Dwarf<gimli::read::EndianSlice<'input, gimli::RunTimeEndian>>,
    depth: isize,
}

impl<'abbrev, 'unit, 'input> DwarfInfoIterator<'abbrev, 'unit, 'input> {
    fn current_debug_info_and_next_cursor(&mut self) -> Option<DwarfInfo> {
        if let Some(entry) = self.entries.current() {
            let name = entry
                .attr_value(gimli::DW_AT_name)
                .unwrap()
                .and_then(|value| value.string_value(&self.dwarf.debug_str))
                .map(|r| r.to_string().unwrap())
                .map(String::from);
            let tag = DwarfTag::from(entry.tag());
            let offset = Offset::new(entry.offset().0);
            let size = entry
                .attr_value(gimli::DW_AT_byte_size)
                .unwrap()
                .and_then(|value| value.udata_value())
                .map(|size| size as usize);
            // TODO: always should get location
            // Currently not because handling RequiresFrameBase from Evaluation is needed
            let location = match tag {
                DwarfTag::DW_TAG_variable if self.depth == 0 => entry
                    .attr_value(gimli::DW_AT_location)
                    .unwrap()
                    .map(|location| {
                        let mut eval = location
                            .exprloc_value()
                            .expect("location attribute should be exprloc")
                            .evaluation(self.encoding);
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
            };
            let type_offset = if let Some(gimli::read::AttributeValue::UnitRef(offset)) =
                entry.attr_value(gimli::DW_AT_type).unwrap()
            {
                Some(Offset::new(offset.0))
            } else {
                None
            };

            let upper_bound = if let Some(gimli::read::AttributeValue::Data1(upper_bound)) =
                entry.attr_value(gimli::DW_AT_upper_bound).unwrap()
            {
                Some(upper_bound as usize)
            } else {
                None
            };

            let current_depth = self.depth;
            match self.entries.next_dfs().unwrap() {
                None => Some(DwarfInfo::new(
                    tag,
                    offset,
                    name,
                    size,
                    location,
                    type_offset,
                    upper_bound,
                    Vec::new(),
                )),
                Some((delta_depth, _)) => {
                    self.depth += delta_depth;
                    let mut children = Vec::new();
                    while self.depth > current_depth {
                        if let Some(info) = self.current_debug_info_and_next_cursor() {
                            children.push(info);
                        } else {
                            break;
                        }
                    }
                    Some(DwarfInfo::new(
                        tag,
                        offset,
                        name,
                        size,
                        location,
                        type_offset,
                        upper_bound,
                        children,
                    ))
                }
            }
        } else {
            None
        }
    }
}

impl<'abbrev, 'unit, 'input> Iterator for DwarfInfoIterator<'abbrev, 'unit, 'input> {
    type Item = DwarfInfo;
    fn next(&mut self) -> Option<Self::Item> {
        self.current_debug_info_and_next_cursor()
    }
}

pub fn with_dwarf_info_iterator<Output>(
    path: String,
    consumer: impl for<'abbrev, 'unit, 'input> FnOnce(
        DwarfInfoIterator<'abbrev, 'unit, 'input>,
    ) -> Output,
) -> Output {
    let file = fs::File::open(&path).unwrap();
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
    ) -> gimli::EndianSlice<'b, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(&*section, endian);

    // Create `EndianSlice`s for all of the sections.
    let dwarf = dwarf_cow.borrow(&borrow_section);

    // Iterate over the compilation units.
    let mut iter = dwarf.units();
    let header = iter
        .next()
        .unwrap()
        .expect("ELF binary should contain unit header");
    let unit = dwarf.unit(header).unwrap();
    let depth = 0;
    let mut entries = unit.entries();
    let _ = entries.next_dfs();
    let _ = entries.next_dfs();
    let encoding = unit.encoding();

    consumer(DwarfInfoIterator {
        entries,
        encoding,
        dwarf,
        depth,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn with_dwarf_info_iterator_test() {
        let expected = vec![
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_structure_type)
                .offset(Offset(45))
                .name("hoge")
                .size(16)
                .children(vec![
                    DwarfInfoBuilder::new()
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .offset(Offset(58))
                        .name("hoge")
                        .type_offset(Offset(98))
                        .build(),
                    DwarfInfoBuilder::new()
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .offset(Offset(71))
                        .name("hogehoge")
                        .type_offset(Offset(105))
                        .build(),
                    DwarfInfoBuilder::new()
                        .tag(DwarfTag::DW_TAG_unimplemented)
                        .offset(Offset(84))
                        .name("array")
                        .type_offset(Offset(112))
                        .build(),
                ])
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_base_type)
                .offset(Offset(98))
                .name("int")
                .size(4)
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_base_type)
                .offset(Offset(105))
                .name("char")
                .size(1)
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_array_type)
                .offset(Offset(112))
                .type_offset(Offset(98))
                .children(vec![DwarfInfoBuilder::new()
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .offset(Offset(121))
                    .type_offset(Offset(128))
                    .upper_bound(1)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_base_type)
                .offset(Offset(128))
                .name("long unsigned int")
                .size(8)
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_typedef)
                .offset(Offset(135))
                .name("Hoge")
                .type_offset(Offset(45))
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_array_type)
                .offset(Offset(147))
                .type_offset(Offset(135))
                .children(vec![DwarfInfoBuilder::new()
                    .tag(DwarfTag::DW_TAG_subrange_type)
                    .offset(Offset(156))
                    .type_offset(Offset(128))
                    .upper_bound(2)
                    .build()])
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_variable)
                .offset(Offset(163))
                .name("hoges")
                .location(Location(16480))
                .type_offset(Offset(147))
                .build(),
            DwarfInfoBuilder::new()
                .tag(DwarfTag::DW_TAG_unimplemented)
                .offset(Offset(185))
                .name("main")
                .type_offset(Offset(98))
                .build(),
        ];

        with_dwarf_info_iterator(String::from("examples/simple"), |iter| {
            let got: Vec<DwarfInfo> = iter.collect();
            assert_eq!(expected, got);
        });
    }
}
