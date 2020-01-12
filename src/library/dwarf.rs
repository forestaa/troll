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
        with_dwarf_info_iterator(String::from("examples/simple"), |iter| {
            let mut expecteds = vec![
                DwarfInfo::new(
                    DwarfTag::DW_TAG_structure_type,
                    Offset(45),
                    Some(String::from("hoge")),
                    Some(16),
                    None,
                    None,
                    None,
                    vec![
                        DwarfInfo::new(
                            DwarfTag::DW_TAG_unimplemented,
                            Offset(58),
                            Some(String::from("hoge")),
                            None,
                            None,
                            Some(Offset(98)),
                            None,
                            vec![],
                        ),
                        DwarfInfo::new(
                            DwarfTag::DW_TAG_unimplemented,
                            Offset(71),
                            Some(String::from("hogehoge")),
                            None,
                            None,
                            Some(Offset(105)),
                            None,
                            vec![],
                        ),
                        DwarfInfo::new(
                            DwarfTag::DW_TAG_unimplemented,
                            Offset(84),
                            Some(String::from("array")),
                            None,
                            None,
                            Some(Offset(112)),
                            None,
                            vec![],
                        ),
                    ],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_base_type,
                    Offset(98),
                    Some(String::from("int")),
                    Some(4),
                    None,
                    None,
                    None,
                    vec![],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_base_type,
                    Offset(105),
                    Some(String::from("char")),
                    Some(1),
                    None,
                    None,
                    None,
                    vec![],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_array_type,
                    Offset(112),
                    None,
                    None,
                    None,
                    Some(Offset(98)),
                    None,
                    vec![DwarfInfo::new(
                        DwarfTag::DW_TAG_subrange_type,
                        Offset(121),
                        None,
                        None,
                        None,
                        Some(Offset(128)),
                        Some(1),
                        vec![],
                    )],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_base_type,
                    Offset(128),
                    Some(String::from("long unsigned int")),
                    Some(8),
                    None,
                    None,
                    None,
                    vec![],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_typedef,
                    Offset(135),
                    Some(String::from("Hoge")),
                    None,
                    None,
                    Some(Offset(45)),
                    None,
                    vec![],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_array_type,
                    Offset(147),
                    None,
                    None,
                    None,
                    Some(Offset(135)),
                    None,
                    vec![DwarfInfo::new(
                        DwarfTag::DW_TAG_subrange_type,
                        Offset(156),
                        None,
                        None,
                        None,
                        Some(Offset(128)),
                        Some(2),
                        vec![],
                    )],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_variable,
                    Offset(163),
                    Some(String::from("hoges")),
                    None,
                    Some(Location(16480)),
                    Some(Offset(147)),
                    None,
                    vec![],
                ),
                DwarfInfo::new(
                    DwarfTag::DW_TAG_unimplemented,
                    Offset(185),
                    Some(String::from("main")),
                    None,
                    None,
                    Some(Offset(98)),
                    None,
                    vec![],
                ),
            ];
            expecteds.reverse();

            for info in iter {
                if let Some(expected) = expecteds.pop() {
                    assert_eq!(info, expected);
                }
            }
        });
    }
}
