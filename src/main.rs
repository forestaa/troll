use object::Object;
use std::collections::HashMap;
use std::{borrow, env, fs};

fn main() {
    for path in env::args().skip(1) {
        let file = fs::File::open(&path).unwrap();
        let mmap = unsafe { memmap::Mmap::map(&file).unwrap() };
        let object = object::File::parse(&*mmap).unwrap();
        let endian = if object.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };
        fuga(&object, endian).unwrap();
    }
}

// put typedef, base_type, structure_type into the map
// variable tag at depth 0 is global variable, so add them to the list
// then, create the map of global variables with member, size, type infomation

#[derive(Debug, Clone)]
enum TypeEntry {
    BaseType {
        name: String,
        size: u64,
    },
    TypeDef {
        name: String,
        type_unit_offset: usize,
    },
    StructureType {
        name: String,
        members: Vec<StructureTypeMember>,
    },
    ArrayType {
        type_unit_offset: usize,
        upper_bound: Option<u8>,
    },
    PointerType {
        type_unit_offset: usize,
        size: u64,
    },
    ConstType {
        type_unit_offset: usize,
    },
}

#[derive(Debug, Clone)]
struct StructureTypeMember {
    name: String,
    type_unit_offset: usize,
}

#[derive(Debug)]
struct GlobalVariable {
    name: String,
    location: Option<u64>,
    type_unit_offset: usize,
}

#[derive(Debug)]
enum VariableView {
    BaseTypeVariable {
        name: String,
        address: Option<u64>,
        size: u64,
        type_name: String,
    },
    PointerTypeVariable {
        name: String,
        address: Option<u64>,
        size: u64,
        type_name: String,
    },
    StructureTypeVariable {
        name: String,
        address: Option<u64>,
        size: Option<u64>,
        type_name: String,
        members: Vec<VariableView>,
    },
    ArrayTypeVariable {
        name: String,
        address: Option<u64>,
        size: Option<u64>,
        elements: Vec<VariableView>,
    },
}

impl VariableView {
    fn from_global_variable(variable: &GlobalVariable, type_map: &TypeEntryMap) -> VariableView {
        match type_map.get(&variable.type_unit_offset) {
            Some(TypeEntry::BaseType { name, size }) => VariableView::BaseTypeVariable {
                name: variable.name.clone(),
                address: variable.location,
                size: *size,
                type_name: name.clone(),
            },
            Some(TypeEntry::PointerType {
                type_unit_offset,
                size,
            }) => VariableView::PointerTypeVariable {
                name: variable.name.clone(),
                address: variable.location,
                size: *size,
                type_name: String::from("* pointer"),
            },
            Some(TypeEntry::ConstType { type_unit_offset }) => VariableView::from_global_variable(
                &GlobalVariable {
                    name: variable.name.clone(),
                    location: variable.location,
                    type_unit_offset: *type_unit_offset,
                },
                type_map,
            ),
            Some(TypeEntry::StructureType { name, members }) => {
                let address = &mut variable.location.clone();
                let members = members
                    .iter()
                    .map(|member| {
                        VariableView::from_structure_type_member(
                            member,
                            type_map,
                            address,
                            variable.name.clone(),
                        )
                    })
                    .collect();
                VariableView::StructureTypeVariable {
                    name: variable.name.clone(),
                    address: variable.location,
                    size: None,
                    type_name: name.clone(),
                    members: members,
                }
            }
            Some(TypeEntry::ArrayType {
                type_unit_offset,
                upper_bound,
            }) => VariableView::ArrayTypeVariable {
                name: variable.name.clone(),
                address: variable.location,
                size: None,
                elements: (0..(upper_bound.unwrap_or(0)))
                    .map(|index| match type_map.get(&type_unit_offset) {
                        Some(TypeEntry::BaseType { name, size }) => {
                            VariableView::BaseTypeVariable {
                                name: format!("{}[{}]", variable.name.clone(), index),
                                address: variable
                                    .location
                                    .map(|address| address + *size * (index as u64)),
                                size: *size,
                                type_name: name.clone(),
                            }
                        }
                        x => {
                            println!("hgehoge: {:?}", x);
                            unimplemented!()
                        }
                    })
                    .collect(),
            },
            x => {
                println!("koko: {:?} {:?}", x, variable.type_unit_offset);
                unimplemented!()
            }
        }
    }

    fn from_structure_type_member(
        member: &StructureTypeMember,
        type_map: &TypeEntryMap,
        variable_address: &mut Option<u64>,
        variable_name: String,
    ) -> VariableView {
        match type_map.get(&member.type_unit_offset) {
            Some(TypeEntry::BaseType { name, size }) => {
                let address = variable_address.map(|address| address + size);
                *variable_address = address;
                VariableView::BaseTypeVariable {
                    name: format!("{}.{}", variable_name, member.name.clone()),
                    address: address,
                    size: *size,
                    type_name: name.clone(),
                }
            }
            // Some(TypeEntry::StructureType{})
            x => {
                println!("here: {:?}", x);
                unimplemented!()
            }
        }
    }
}

fn print_variable_view(variable_view: VariableView) {
    match variable_view {
        VariableView::BaseTypeVariable {
            name,
            address,
            size,
            type_name,
        } => println!(
            "{}   {}   {}   {}",
            name,
            address.unwrap_or(0),
            size,
            type_name
        ),
        VariableView::PointerTypeVariable {
            name,
            address,
            size,
            type_name,
        } => println!(
            "{}   {}   {}   {}",
            name,
            address.unwrap_or(0),
            size,
            type_name
        ),
        VariableView::StructureTypeVariable {
            name,
            address,
            size,
            type_name,
            members,
        } => {
            println!(
                "{}   {}   {}   {}",
                name,
                address.unwrap_or(0),
                size.unwrap_or(0),
                type_name
            );
            for member in members {
                print_variable_view(member);
            }
        }
        VariableView::ArrayTypeVariable {
            name,
            address,
            size,
            elements,
        } => {
            println!(
                "{}   {}   {}   {}",
                name,
                address.unwrap_or(0),
                size.unwrap_or(0),
                "array"
            );
            for element in elements {
                print_variable_view(element);
            }
        }
    }
}

fn print_variable_view_vec(variable_views: Vec<VariableView>) {
    for variable_view in variable_views {
        print_variable_view(variable_view);
    }
}

#[derive(Debug)]
struct TypeEntryMap {
    map: HashMap<usize, TypeEntry>,
}

impl TypeEntryMap {
    fn new() -> TypeEntryMap {
        TypeEntryMap {
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, key: usize, value: TypeEntry) {
        self.map.insert(key, value);
    }

    fn get(&self, key: &usize) -> Option<&TypeEntry> {
        let mut entry = self.map.get(key);
        while let Some(TypeEntry::TypeDef {
            name: _,
            type_unit_offset: offset,
        }) = entry
        {
            entry = self.map.get(&offset);
        }
        entry
    }
}

#[derive(Debug)]
struct WithChildren<T> {
    value: T,
    children: Vec<WithChildren<T>>,
}

impl<T> WithChildren<T> {
    fn new(value: T) -> WithChildren<T> {
        WithChildren {
            value: value,
            children: Vec::new(),
        }
    }
}

fn fuga(object: &object::File, endian: gimli::RunTimeEndian) -> Result<(), gimli::Error> {
    // Load a section and return as `Cow<[u8]>`.
    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
        Ok(object
            .section_data_by_name(id.name())
            .unwrap_or(borrow::Cow::Borrowed(&[][..])))
    };
    // Load a supplementary section. We don't have a supplementary object file,
    // so always return an empty slice.
    let load_section_sup = |_| Ok(borrow::Cow::Borrowed(&[][..]));

    // Load all of the sections.
    let dwarf_cow = gimli::Dwarf::load(&load_section, &load_section_sup)?;

    // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
    let borrow_section: &dyn for<'a> Fn(
        &'a borrow::Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(&*section, endian);

    // Create `EndianSlice`s for all of the sections.
    let dwarf = dwarf_cow.borrow(&borrow_section);

    // Iterate over the compilation units.
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        println!("Unit at <.debug_info+0x{:x}>", header.offset().0);
        let unit = dwarf.unit(header)?;

        // Iterate over the Debugging Information Entries (DIEs) in the unit.
        let mut with_children_entries: Vec<WithChildren<_>> = Vec::new();
        let mut depth = 0;
        let mut entries = unit.entries();
        while let Some((delta_depth, entry)) = entries.next_dfs()? {
            depth += delta_depth;

            let mut current = &mut with_children_entries;
            let mut depth_clone = depth;
            while depth_clone > 1 {
                current = &mut current.last_mut().unwrap().children;
                depth_clone -= 1;
            }
            current.push(WithChildren::new(entry.clone()));
            // current.push(WithChildren::new(entry));
        }

        let mut type_map = TypeEntryMap::new();
        let mut global_variables = Vec::new();
        for entry in with_children_entries {
            println!(
                "<{}><{:x}> {}",
                depth,
                entry.value.offset().0,
                entry.value.tag()
            );

            match entry.value.tag() {
                gimli::DW_TAG_base_type => {
                    let name = String::from(
                        entry
                            .value
                            .attr_value(gimli::DW_AT_name)?
                            .unwrap()
                            .string_value(&dwarf.debug_str)
                            .unwrap()
                            .to_string()?,
                    );
                    let size = entry
                        .value
                        .attr_value(gimli::DW_AT_byte_size)?
                        .unwrap()
                        .udata_value()
                        .unwrap();
                    type_map.insert(
                        entry.value.offset().0,
                        TypeEntry::BaseType {
                            name: name,
                            size: size,
                        },
                    );
                }
                gimli::DW_TAG_pointer_type => {
                    let size = entry
                        .value
                        .attr_value(gimli::DW_AT_byte_size)?
                        .unwrap()
                        .udata_value()
                        .unwrap();
                    if let Some(gimli::read::AttributeValue::UnitRef(offset)) =
                        entry.value.attr_value(gimli::DW_AT_type)?
                    {
                        type_map.insert(
                            entry.value.offset().0,
                            TypeEntry::PointerType {
                                type_unit_offset: offset.0,
                                size: size,
                            },
                        );
                    }
                }
                gimli::DW_TAG_const_type => {
                    if let Some(gimli::read::AttributeValue::UnitRef(offset)) =
                        entry.value.attr_value(gimli::DW_AT_type)?
                    {
                        type_map.insert(
                            entry.value.offset().0,
                            TypeEntry::ConstType {
                                type_unit_offset: offset.0,
                            },
                        );
                    }
                }
                gimli::DW_TAG_typedef => {
                    let name = String::from(
                        entry
                            .value
                            .attr_value(gimli::DW_AT_name)?
                            .unwrap()
                            .string_value(&dwarf.debug_str)
                            .unwrap()
                            .to_string()?,
                    );
                    if let Some(gimli::read::AttributeValue::UnitRef(offset)) =
                        entry.value.attr_value(gimli::DW_AT_type)?
                    {
                        type_map.insert(
                            entry.value.offset().0,
                            TypeEntry::TypeDef {
                                name: name,
                                type_unit_offset: offset.0,
                            },
                        );
                    }
                }
                gimli::DW_TAG_array_type => {
                    if let Some(gimli::read::AttributeValue::UnitRef(offset)) =
                        entry.value.attr_value(gimli::DW_AT_type)?
                    {
                        if let Some(gimli::read::AttributeValue::Data1(upper_bound)) =
                            entry.children.iter().find_map(|entry| {
                                entry.value.attr_value(gimli::DW_AT_upper_bound).unwrap()
                            })
                        {
                            type_map.insert(
                                entry.value.offset().0,
                                TypeEntry::ArrayType {
                                    type_unit_offset: offset.0,
                                    upper_bound: Some(upper_bound),
                                },
                            );
                        } else {
                            type_map.insert(
                                entry.value.offset().0,
                                TypeEntry::ArrayType {
                                    type_unit_offset: offset.0,
                                    upper_bound: None,
                                },
                            );
                        }
                    }
                }
                gimli::DW_TAG_structure_type => {
                    let name = String::from(
                        entry
                            .value
                            .attr_value(gimli::DW_AT_name)?
                            .unwrap()
                            .string_value(&dwarf.debug_str)
                            .unwrap()
                            .to_string()?,
                    );
                    let members = entry
                        .children
                        .iter()
                        .filter_map(|entry| match entry.value.tag() {
                            gimli::DW_TAG_member => {
                                let name = String::from(
                                    entry
                                        .value
                                        .attr_value(gimli::DW_AT_name)
                                        .unwrap()
                                        .unwrap()
                                        .string_value(&dwarf.debug_str)
                                        .unwrap()
                                        .to_string()
                                        .unwrap(),
                                );
                                let type_unit_offset =
                                    if let gimli::read::AttributeValue::UnitRef(offset) =
                                        entry.value.attr_value(gimli::DW_AT_type).unwrap().unwrap()
                                    {
                                        offset.0
                                    } else {
                                        0
                                    };
                                Some(StructureTypeMember {
                                    name: name,
                                    type_unit_offset: type_unit_offset,
                                })
                            }
                            _ => None,
                        })
                        .collect();
                    type_map.insert(
                        entry.value.offset().0,
                        TypeEntry::StructureType {
                            name: name,
                            members: members,
                        },
                    );
                }
                gimli::DW_TAG_variable => {
                    let name = String::from(
                        entry
                            .value
                            .attr_value(gimli::DW_AT_name)?
                            .unwrap()
                            .string_value(&dwarf.debug_str)
                            .unwrap()
                            .to_string()?,
                    );
                    let location = entry
                        .value
                        .attr_value(gimli::DW_AT_location)?
                        .map(|location| {
                            let mut eval = location
                                .exprloc_value()
                                .unwrap()
                                .evaluation(unit.encoding());
                            let mut result = eval.evaluate().unwrap();
                            while result != gimli::EvaluationResult::Complete {
                                println!("test: expression result: {:?}", result);
                                match result {
                                    gimli::EvaluationResult::RequiresRelocatedAddress(address) => {
                                        result =
                                            eval.resume_with_relocated_address(address).unwrap()
                                    }
                                    x => {
                                        println!("variable: {:?}", x);
                                        unimplemented!()
                                    }
                                }
                            }
                            let result = eval.result();
                            println!("test: expression result: finish {:?}", result);
                            if let gimli::Location::Address { address } = result[0].location {
                                address
                            } else {
                                println!("result: {:?}", result);
                                unimplemented!()
                            }
                        });
                    let type_unit_offset = if let gimli::read::AttributeValue::UnitRef(offset) =
                        entry.value.attr_value(gimli::DW_AT_type)?.unwrap()
                    {
                        offset.0
                    } else {
                        0
                    };
                    global_variables.push(GlobalVariable {
                        name: name,
                        location: location,
                        type_unit_offset: type_unit_offset,
                    });
                }
                _ => (),
            }
        }
        println!("finish: {:?}, {:?}", type_map, global_variables);

        let global_variable_views: Vec<VariableView> = global_variables
            .iter()
            .map(|variable| VariableView::from_global_variable(variable, &type_map))
            .collect();
        println!("view debug");
        print_variable_view_vec(global_variable_views);
    }

    Ok(())
}
