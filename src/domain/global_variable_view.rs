use super::global_variable::{Address, GlobalVariable};
use super::type_entry::{StructureTypeMemberEntry, TypeEntryId, TypeEntryKind};
use super::type_entry_repository::TypeEntryRepository;
use log::warn;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVariableView {
    name: String,
    address: Option<Address>,
    size: usize,
    type_view: TypeView,
    children: Vec<GlobalVariableView>,
}

impl GlobalVariableView {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    pub fn type_view(&self) -> &TypeView {
        &self.type_view
    }

    pub fn children(self) -> Vec<Self> {
        self.children
    }

    pub fn set_type_view(&mut self, type_view: TypeView) {
        self.map_type_view(|_| type_view);
    }

    pub fn map_type_view(&mut self, f: impl FnOnce(TypeView) -> TypeView) {
        self.type_view = f(self.type_view.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeView {
    Base {
        name: String,
    },
    TypeDef {
        name: String,
        type_view: Box<TypeView>,
    },
    Const {
        type_view: Box<TypeView>,
    },
    VoidPointer,
    Pointer {
        type_view: Box<TypeView>,
    },
    Structure {
        name: String,
    },
    Array {
        element_type: Box<TypeView>,
        upper_bound: Option<usize>,
    },
}

pub struct GlobalVariableViewFactory<'repo> {
    type_entry_repository: &'repo TypeEntryRepository,
}

impl<'repo> GlobalVariableViewFactory<'repo> {
    pub fn new(type_entry_repository: &TypeEntryRepository) -> GlobalVariableViewFactory {
        GlobalVariableViewFactory {
            type_entry_repository,
        }
    }

    pub fn from_global_variable(
        &self,
        global_variable: GlobalVariable,
    ) -> Option<GlobalVariableView> {
        match self
            .type_entry_repository
            .find_by_id(global_variable.type_ref())
        {
            None => {
                warn!(
                    "global variable refers unknown offset: variable: {}, refered offset {:?}",
                    global_variable.name(),
                    global_variable.type_ref()
                );
                None
            }
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef {
                    name: type_name,
                    type_ref,
                } => self.from_global_variable_typedef(
                    global_variable,
                    type_name.clone(),
                    type_ref.clone(),
                ),
                TypeEntryKind::ConstType { type_ref } => {
                    self.from_global_variable_const_type(global_variable, type_ref.clone())
                }
                TypeEntryKind::PointerType { size, type_ref } => {
                    self.from_global_variable_pointer_type(global_variable, *size, type_ref)
                }
                TypeEntryKind::BaseType {
                    name: type_name,
                    size,
                } => Some(self.from_global_variable_base_type(
                    global_variable,
                    type_name.clone(),
                    *size,
                )),
                TypeEntryKind::StructureType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.from_global_variable_structure_type(
                    global_variable,
                    type_name.clone(),
                    *size,
                    members,
                )),
                TypeEntryKind::ArrayType {
                    element_type_ref,
                    upper_bound,
                } => self.from_global_variable_array_type(
                    global_variable,
                    element_type_ref,
                    *upper_bound,
                ),
            },
        }
    }

    fn from_global_variable_typedef(
        &self,
        global_variable: GlobalVariable,
        type_name: String,
        type_ref: TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let global_variable =
            GlobalVariable::new(global_variable.address(), global_variable.name(), type_ref);

        let mut global_variable_view = self.from_global_variable(global_variable)?;
        global_variable_view.map_type_view(|type_view| TypeView::TypeDef {
            name: type_name,
            type_view: Box::new(type_view),
        });
        Some(global_variable_view)
    }

    fn from_global_variable_const_type(
        &self,
        global_variable: GlobalVariable,
        type_ref: TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let global_variable =
            GlobalVariable::new(global_variable.address(), global_variable.name(), type_ref);

        let mut global_variable_view = self.from_global_variable(global_variable)?;
        global_variable_view.map_type_view(|type_view| TypeView::Const {
            type_view: Box::new(type_view),
        });
        Some(global_variable_view)
    }

    fn from_global_variable_pointer_type(
        &self,
        global_variable: GlobalVariable,
        size: usize,
        type_ref: &Option<TypeEntryId>,
    ) -> Option<GlobalVariableView> {
        match type_ref {
            None => Some(GlobalVariableView {
                name: global_variable.name(),
                address: global_variable.address(),
                size: size,
                type_view: TypeView::VoidPointer,
                children: Vec::new(),
            }),
            Some(type_ref) => {
                let type_view = self.type_view_from_type_entry(type_ref)?;
                Some(GlobalVariableView {
                    name: global_variable.name(),
                    address: global_variable.address(),
                    size: size,
                    type_view: TypeView::Pointer {
                        type_view: Box::new(type_view),
                    },
                    children: Vec::new(),
                })
            }
        }
    }

    fn from_global_variable_base_type(
        &self,
        global_variable: GlobalVariable,
        type_name: String,
        size: usize,
    ) -> GlobalVariableView {
        GlobalVariableView {
            name: global_variable.name(),
            address: global_variable.address(),
            size: size,
            type_view: TypeView::Base { name: type_name },
            children: Vec::new(),
        }
    }

    fn from_global_variable_structure_type(
        &self,
        global_variable: GlobalVariable,
        type_name: String,
        size: usize,
        members: &Vec<StructureTypeMemberEntry>,
    ) -> GlobalVariableView {
        let base_address = global_variable.address();
        let members: Vec<GlobalVariableView> = members
            .iter()
            .flat_map(|member| self.from_structure_type_member_entry(member, &base_address))
            .collect();

        GlobalVariableView {
            name: global_variable.name(),
            address: base_address,
            size: size,
            type_view: TypeView::Structure { name: type_name },
            children: members,
        }
    }

    fn from_global_variable_array_type(
        &self,
        global_variable: GlobalVariable,
        element_type_ref: &TypeEntryId,
        upper_bound: Option<usize>,
    ) -> Option<GlobalVariableView> {
        let type_view = self.type_view_from_type_entry(element_type_ref)?;
        let address = global_variable.address();
        let (elements, size) = self.array_elements(
            global_variable.name(),
            &address,
            upper_bound,
            element_type_ref.clone(),
        );

        Some(GlobalVariableView {
            name: global_variable.name(),
            address: address,
            size: size,
            type_view: TypeView::Array {
                element_type: Box::new(type_view),
                upper_bound: upper_bound,
            },
            children: elements,
        })
    }

    fn from_structure_type_member_entry(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
    ) -> Option<GlobalVariableView> {
        match self.type_entry_repository.find_by_id(&member.type_ref) {
            None => {
                let offset: usize = member.type_ref.clone().into();
                warn!(
                    "structure member refers unknown offset: member: {}, refered offset: {:#x}",
                    member.name, offset
                );
                None
            }
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef {
                    name: type_name,
                    type_ref,
                } => self.from_structure_type_member_entry_typedef(
                    member,
                    base_address,
                    type_name.clone(),
                    type_ref.clone(),
                ),
                TypeEntryKind::ConstType { type_ref } => self
                    .from_structure_type_member_entry_const_type(
                        member,
                        base_address,
                        type_ref.clone(),
                    ),
                TypeEntryKind::PointerType { size, type_ref } => self
                    .from_structure_type_member_entry_pointer_type(
                        member,
                        base_address,
                        type_ref.as_ref(),
                        *size,
                    ),
                TypeEntryKind::BaseType {
                    name: type_name,
                    size,
                } => Some(self.from_structure_type_member_entry_base_type(
                    member,
                    base_address,
                    type_name.clone(),
                    *size,
                )),
                TypeEntryKind::StructureType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.from_structure_type_member_entry_structure_type(
                    member,
                    base_address,
                    type_name.clone(),
                    *size,
                    members,
                )),
                TypeEntryKind::ArrayType {
                    element_type_ref,
                    upper_bound,
                } => self.from_structure_type_member_entry_array_type(
                    member,
                    base_address,
                    element_type_ref,
                    *upper_bound,
                ),
            },
        }
    }

    fn from_structure_type_member_entry_typedef(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_name: String,
        type_ref: TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let member = StructureTypeMemberEntry {
            name: member.name.clone(),
            location: member.location,
            type_ref: type_ref,
        };
        let mut member_view = self.from_structure_type_member_entry(&member, base_address)?;

        member_view.map_type_view(|type_view| TypeView::TypeDef {
            name: type_name,
            type_view: Box::new(type_view),
        });
        Some(member_view)
    }

    fn from_structure_type_member_entry_const_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_ref: TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let member = StructureTypeMemberEntry {
            name: member.name.clone(),
            location: member.location,
            type_ref: type_ref,
        };
        let mut member_view = self.from_structure_type_member_entry(&member, base_address)?;

        member_view.map_type_view(|type_view| TypeView::Const {
            type_view: Box::new(type_view),
        });
        Some(member_view)
    }

    fn from_structure_type_member_entry_pointer_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_ref: Option<&TypeEntryId>,
        size: usize,
    ) -> Option<GlobalVariableView> {
        let mut address = base_address.clone();
        if let Some(ref mut addr) = address {
            addr.add(member.location);
        }

        match type_ref {
            None => Some(GlobalVariableView {
                name: member.name.clone(),
                address: address,
                size: size,
                type_view: TypeView::VoidPointer,
                children: Vec::new(),
            }),
            Some(type_ref) => {
                let type_view = self.type_view_from_type_entry(type_ref)?;
                Some(GlobalVariableView {
                    name: member.name.clone(),
                    address: address,
                    size: size,
                    type_view: TypeView::Pointer {
                        type_view: Box::new(type_view),
                    },
                    children: Vec::new(),
                })
            }
        }
    }

    fn from_structure_type_member_entry_base_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_name: String,
        size: usize,
    ) -> GlobalVariableView {
        let mut address = base_address.clone();
        if let Some(ref mut addr) = address {
            addr.add(member.location);
        }

        GlobalVariableView {
            name: member.name.clone(),
            address: address,
            size: size,
            type_view: TypeView::Base { name: type_name },
            children: Vec::new(),
        }
    }

    fn from_structure_type_member_entry_structure_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_name: String,
        size: usize,
        members: &Vec<StructureTypeMemberEntry>,
    ) -> GlobalVariableView {
        let mut address = base_address.clone();
        if let Some(ref mut addr) = address {
            addr.add(member.location);
        }
        let members: Vec<GlobalVariableView> = members
            .iter()
            .flat_map(|member| self.from_structure_type_member_entry(member, &address))
            .collect();

        GlobalVariableView {
            name: member.name.clone(),
            address: address,
            size: size,
            type_view: TypeView::Structure { name: type_name },
            children: members,
        }
    }

    fn from_structure_type_member_entry_array_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        element_type_ref: &TypeEntryId,
        upper_bound: Option<usize>,
    ) -> Option<GlobalVariableView> {
        let mut address = base_address.clone();
        if let Some(ref mut addr) = address {
            addr.add(member.location);
        }

        let type_view = self.type_view_from_type_entry(element_type_ref)?;
        let (elements, size) = self.array_elements(
            member.name.clone(),
            &address,
            upper_bound,
            element_type_ref.clone(),
        );

        Some(GlobalVariableView {
            name: member.name.clone(),
            address: address,
            size: size,
            type_view: TypeView::Array {
                element_type: Box::new(type_view),
                upper_bound: upper_bound,
            },
            children: elements,
        })
    }

    fn array_elements(
        &self,
        name: String,
        address: &Option<Address>,
        upper_bound: Option<usize>,
        element_type_ref: TypeEntryId,
    ) -> (Vec<GlobalVariableView>, usize) {
        match upper_bound {
            None => {
                let mut elements = vec![];
                let mut size = 0;
                if let Some(element_view) = self.from_global_variable(GlobalVariable::new(
                    address.clone(),
                    name,
                    element_type_ref,
                )) {
                    size += element_view.size();
                    elements.push(element_view);
                }
                (elements, size)
            }
            Some(upper_bound) => {
                let mut size = 0;
                let elements = (0..=upper_bound)
                    .flat_map(|n| {
                        let mut address = address.clone();
                        if let Some(ref mut addr) = address {
                            addr.add(size);
                        }
                        let element_view = self.from_global_variable(GlobalVariable::new(
                            address,
                            n.to_string(),
                            element_type_ref.clone(),
                        ))?;
                        size += element_view.size();
                        Some(element_view)
                    })
                    .collect();
                (elements, size)
            }
        }
    }

    fn type_view_from_type_entry(&self, type_entry_id: &TypeEntryId) -> Option<TypeView> {
        match self.type_entry_repository.find_by_id(type_entry_id) {
            None => {
                let offset: usize = type_entry_id.clone().into();
                warn!(
                    "something refers unknown offset: refered offset: {:#x}",
                    offset
                );
                None
            }
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef { name, type_ref } => {
                    let type_view = self.type_view_from_type_entry(type_ref)?;
                    Some(TypeView::TypeDef {
                        name: name.clone(),
                        type_view: Box::new(type_view),
                    })
                }
                TypeEntryKind::ConstType { type_ref } => {
                    let type_view = self.type_view_from_type_entry(type_ref)?;
                    Some(TypeView::Const {
                        type_view: Box::new(type_view),
                    })
                }
                TypeEntryKind::PointerType { type_ref, .. } => match type_ref {
                    None => Some(TypeView::VoidPointer),
                    Some(type_ref) => {
                        let type_view = self.type_view_from_type_entry(type_ref)?;
                        Some(TypeView::Pointer {
                            type_view: Box::new(type_view),
                        })
                    }
                },
                TypeEntryKind::BaseType { name, .. } => Some(TypeView::Base { name: name.clone() }),
                TypeEntryKind::StructureType { name, .. } => {
                    Some(TypeView::Structure { name: name.clone() })
                }
                TypeEntryKind::ArrayType {
                    element_type_ref,
                    upper_bound,
                } => {
                    let type_view = self.type_view_from_type_entry(element_type_ref)?;
                    Some(TypeView::Array {
                        element_type: Box::new(type_view),
                        upper_bound: *upper_bound,
                    })
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::type_entry::TypeEntry;
    use crate::library::dwarf::{Location, Offset};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn from_global_variable_test(
        defined_types: Vec<TypeEntry>,
        global_variable: GlobalVariable,
        expected_view: GlobalVariableView,
    ) {
        init();

        let mut type_entry_repository = TypeEntryRepository::new();
        for defined_type in defined_types {
            type_entry_repository.save(defined_type);
        }
        let factory = GlobalVariableViewFactory::new(&type_entry_repository);

        let got_view = factory.from_global_variable(global_variable);
        assert_eq!(Some(expected_view), got_view);
    }

    #[test]
    fn from_global_variable_const() {
        let defined_types = vec![
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

        let global_variable = GlobalVariable::new(
            Some(Address::new(Location::new(8196))),
            String::from("c"),
            TypeEntryId::new(Offset::new(72)),
        );

        let expected_view = GlobalVariableView {
            name: String::from("c"),
            address: Some(Address::new(Location::new(8196))),
            size: 4,
            type_view: TypeView::Const {
                type_view: Box::new(TypeView::Base {
                    name: String::from("int"),
                }),
            },
            children: vec![],
        };

        from_global_variable_test(defined_types, global_variable, expected_view);
    }

    #[test]
    fn from_global_variable_pointer() {
        let defined_types = vec![
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

        let global_variable = GlobalVariable::new(
            Some(Address::new(Location::new(16432))),
            String::from("p"),
            TypeEntryId::new(Offset::new(65)),
        );

        let expected_view = GlobalVariableView {
            name: String::from("p"),
            address: Some(Address::new(Location::new(16432))),
            size: 8,
            type_view: TypeView::Pointer {
                type_view: Box::new(TypeView::Base {
                    name: String::from("int"),
                }),
            },
            children: vec![],
        };

        from_global_variable_test(defined_types, global_variable, expected_view);
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
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(114)),
                String::from("int"),
                4,
            ),
        ];

        let global_variable = GlobalVariable::new(
            Some(Address::new(Location::new(16428))),
            String::from("a"),
            TypeEntryId::new(Offset::new(45)),
        );

        let expected_view = GlobalVariableView {
            name: String::from("a"),
            address: Some(Address::new(Location::new(16428))),
            size: 4,
            type_view: TypeView::TypeDef {
                name: String::from("uint8"),
                type_view: Box::new(TypeView::Base {
                    name: String::from("unsigned int"),
                }),
            },
            children: vec![],
        };

        from_global_variable_test(defined_types, global_variable, expected_view);
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
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(68)),
                String::from("int"),
                4,
            ),
        ];

        let global_variable = GlobalVariable::new(
            Some(Address::new(Location::new(16432))),
            String::from("hoges"),
            TypeEntryId::new(Offset::new(45)),
        );

        let expected_view = GlobalVariableView {
            name: String::from("hoges"),
            address: Some(Address::new(Location::new(16432))),
            size: 12,
            type_view: TypeView::Array {
                element_type: Box::new(TypeView::Base {
                    name: String::from("int"),
                }),
                upper_bound: Some(2),
            },
            children: vec![
                GlobalVariableView {
                    name: String::from("0"),
                    address: Some(Address::new(Location::new(16432))),
                    size: 4,
                    type_view: TypeView::Base {
                        name: String::from("int"),
                    },
                    children: vec![],
                },
                GlobalVariableView {
                    name: String::from("1"),
                    address: Some(Address::new(Location::new(16436))),
                    size: 4,
                    type_view: TypeView::Base {
                        name: String::from("int"),
                    },
                    children: vec![],
                },
                GlobalVariableView {
                    name: String::from("2"),
                    address: Some(Address::new(Location::new(16440))),
                    size: 4,
                    type_view: TypeView::Base {
                        name: String::from("int"),
                    },
                    children: vec![],
                },
            ],
        };

        from_global_variable_test(defined_types, global_variable, expected_view);
    }

    #[test]
    fn from_global_variable_structure() {
        let defined_types = vec![
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

        let global_variable = GlobalVariable::new(
            Some(Address::new(Location::new(16432))),
            String::from("hoge"),
            TypeEntryId::new(Offset::new(45)),
        );

        let expected_view = GlobalVariableView {
            name: String::from("hoge"),
            address: Some(Address::new(Location::new(16432))),
            size: 8,
            type_view: TypeView::Structure {
                name: String::from("hoge"),
            },
            children: vec![
                GlobalVariableView {
                    name: String::from("hoge"),
                    address: Some(Address::new(Location::new(16432))),
                    size: 4,
                    type_view: TypeView::Base {
                        name: String::from("int"),
                    },
                    children: vec![],
                },
                GlobalVariableView {
                    name: String::from("fuga"),
                    address: Some(Address::new(Location::new(16436))),
                    size: 1,
                    type_view: TypeView::Base {
                        name: String::from("char"),
                    },
                    children: vec![],
                },
                GlobalVariableView {
                    name: String::from("pohe"),
                    address: Some(Address::new(Location::new(16436))),
                    size: 4,
                    type_view: TypeView::Base {
                        name: String::from("unsigned int"),
                    },
                    children: vec![],
                },
            ],
        };

        from_global_variable_test(defined_types, global_variable, expected_view);
    }

    #[test]
    fn from_global_variable_complex_structure() {
        let defined_types = vec![
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

        let global_variable = GlobalVariable::new(
            Some(Address::new(Location::new(16480))),
            String::from("hoge"),
            TypeEntryId::new(Offset::new(184)),
        );

        let expected_view = GlobalVariableView {
            name: String::from("hoge"),
            address: Some(Address::new(Location::new(16480))),
            size: 96,
            type_view: TypeView::Array {
                element_type: Box::new(TypeView::Structure {
                    name: String::from("hoge"),
                }),
                upper_bound: Some(2),
            },
            children: vec![
                GlobalVariableView {
                    name: String::from("0"),
                    address: Some(Address::new(Location::new(16480))),
                    size: 32,
                    type_view: TypeView::Structure {
                        name: String::from("hoge"),
                    },
                    children: vec![
                        GlobalVariableView {
                            name: String::from("hoge"),
                            address: Some(Address::new(Location::new(16480))),
                            size: 8,
                            type_view: TypeView::Pointer {
                                type_view: Box::new(TypeView::Base {
                                    name: String::from("int"),
                                }),
                            },
                            children: vec![],
                        },
                        GlobalVariableView {
                            name: String::from("array"),
                            address: Some(Address::new(Location::new(16488))),
                            size: 8,
                            type_view: TypeView::Array {
                                element_type: Box::new(TypeView::Base {
                                    name: String::from("int"),
                                }),
                                upper_bound: Some(1),
                            },
                            children: vec![
                                GlobalVariableView {
                                    name: String::from("0"),
                                    address: Some(Address::new(Location::new(16488))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                                GlobalVariableView {
                                    name: String::from("1"),
                                    address: Some(Address::new(Location::new(16492))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                            ],
                        },
                        GlobalVariableView {
                            name: String::from("student"),
                            address: Some(Address::new(Location::new(16496))),
                            size: 16,
                            type_view: TypeView::Structure {
                                name: String::from("student"),
                            },
                            children: vec![GlobalVariableView {
                                name: String::from("name"),
                                address: Some(Address::new(Location::new(16496))),
                                size: 16,
                                type_view: TypeView::Array {
                                    element_type: Box::new(TypeView::Base {
                                        name: String::from("char"),
                                    }),
                                    upper_bound: Some(15),
                                },
                                children: vec![
                                    GlobalVariableView {
                                        name: String::from("0"),
                                        address: Some(Address::new(Location::new(16496))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("1"),
                                        address: Some(Address::new(Location::new(16497))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("2"),
                                        address: Some(Address::new(Location::new(16498))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("3"),
                                        address: Some(Address::new(Location::new(16499))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("4"),
                                        address: Some(Address::new(Location::new(16500))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("5"),
                                        address: Some(Address::new(Location::new(16501))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("6"),
                                        address: Some(Address::new(Location::new(16502))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("7"),
                                        address: Some(Address::new(Location::new(16503))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("8"),
                                        address: Some(Address::new(Location::new(16504))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("9"),
                                        address: Some(Address::new(Location::new(16505))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("10"),
                                        address: Some(Address::new(Location::new(16506))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("11"),
                                        address: Some(Address::new(Location::new(16507))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("12"),
                                        address: Some(Address::new(Location::new(16508))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("13"),
                                        address: Some(Address::new(Location::new(16509))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("14"),
                                        address: Some(Address::new(Location::new(16510))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("15"),
                                        address: Some(Address::new(Location::new(16511))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                ],
                            }],
                        },
                    ],
                },
                GlobalVariableView {
                    name: String::from("1"),
                    address: Some(Address::new(Location::new(16512))),
                    size: 32,
                    type_view: TypeView::Structure {
                        name: String::from("hoge"),
                    },
                    children: vec![
                        GlobalVariableView {
                            name: String::from("hoge"),
                            address: Some(Address::new(Location::new(16512))),
                            size: 8,
                            type_view: TypeView::Pointer {
                                type_view: Box::new(TypeView::Base {
                                    name: String::from("int"),
                                }),
                            },
                            children: vec![],
                        },
                        GlobalVariableView {
                            name: String::from("array"),
                            address: Some(Address::new(Location::new(16520))),
                            size: 8,
                            type_view: TypeView::Array {
                                element_type: Box::new(TypeView::Base {
                                    name: String::from("int"),
                                }),
                                upper_bound: Some(1),
                            },
                            children: vec![
                                GlobalVariableView {
                                    name: String::from("0"),
                                    address: Some(Address::new(Location::new(16520))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                                GlobalVariableView {
                                    name: String::from("1"),
                                    address: Some(Address::new(Location::new(16524))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                            ],
                        },
                        GlobalVariableView {
                            name: String::from("student"),
                            address: Some(Address::new(Location::new(16528))),
                            size: 16,
                            type_view: TypeView::Structure {
                                name: String::from("student"),
                            },
                            children: vec![GlobalVariableView {
                                name: String::from("name"),
                                address: Some(Address::new(Location::new(16528))),
                                size: 16,
                                type_view: TypeView::Array {
                                    element_type: Box::new(TypeView::Base {
                                        name: String::from("char"),
                                    }),
                                    upper_bound: Some(15),
                                },
                                children: vec![
                                    GlobalVariableView {
                                        name: String::from("0"),
                                        address: Some(Address::new(Location::new(16528))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("1"),
                                        address: Some(Address::new(Location::new(16529))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("2"),
                                        address: Some(Address::new(Location::new(16530))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("3"),
                                        address: Some(Address::new(Location::new(16531))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("4"),
                                        address: Some(Address::new(Location::new(16532))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("5"),
                                        address: Some(Address::new(Location::new(16533))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("6"),
                                        address: Some(Address::new(Location::new(16534))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("7"),
                                        address: Some(Address::new(Location::new(16535))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("8"),
                                        address: Some(Address::new(Location::new(16536))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("9"),
                                        address: Some(Address::new(Location::new(16537))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("10"),
                                        address: Some(Address::new(Location::new(16538))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("11"),
                                        address: Some(Address::new(Location::new(16539))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("12"),
                                        address: Some(Address::new(Location::new(16540))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("13"),
                                        address: Some(Address::new(Location::new(16541))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("14"),
                                        address: Some(Address::new(Location::new(16542))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("15"),
                                        address: Some(Address::new(Location::new(16543))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                ],
                            }],
                        },
                    ],
                },
                GlobalVariableView {
                    name: String::from("2"),
                    address: Some(Address::new(Location::new(16544))),
                    size: 32,
                    type_view: TypeView::Structure {
                        name: String::from("hoge"),
                    },
                    children: vec![
                        GlobalVariableView {
                            name: String::from("hoge"),
                            address: Some(Address::new(Location::new(16544))),
                            size: 8,
                            type_view: TypeView::Pointer {
                                type_view: Box::new(TypeView::Base {
                                    name: String::from("int"),
                                }),
                            },
                            children: vec![],
                        },
                        GlobalVariableView {
                            name: String::from("array"),
                            address: Some(Address::new(Location::new(16552))),
                            size: 8,
                            type_view: TypeView::Array {
                                element_type: Box::new(TypeView::Base {
                                    name: String::from("int"),
                                }),
                                upper_bound: Some(1),
                            },
                            children: vec![
                                GlobalVariableView {
                                    name: String::from("0"),
                                    address: Some(Address::new(Location::new(16552))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                                GlobalVariableView {
                                    name: String::from("1"),
                                    address: Some(Address::new(Location::new(16556))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                            ],
                        },
                        GlobalVariableView {
                            name: String::from("student"),
                            address: Some(Address::new(Location::new(16560))),
                            size: 16,
                            type_view: TypeView::Structure {
                                name: String::from("student"),
                            },
                            children: vec![GlobalVariableView {
                                name: String::from("name"),
                                address: Some(Address::new(Location::new(16560))),
                                size: 16,
                                type_view: TypeView::Array {
                                    element_type: Box::new(TypeView::Base {
                                        name: String::from("char"),
                                    }),
                                    upper_bound: Some(15),
                                },
                                children: vec![
                                    GlobalVariableView {
                                        name: String::from("0"),
                                        address: Some(Address::new(Location::new(16560))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("1"),
                                        address: Some(Address::new(Location::new(16561))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("2"),
                                        address: Some(Address::new(Location::new(16562))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("3"),
                                        address: Some(Address::new(Location::new(16563))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("4"),
                                        address: Some(Address::new(Location::new(16564))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("5"),
                                        address: Some(Address::new(Location::new(16565))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("6"),
                                        address: Some(Address::new(Location::new(16566))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("7"),
                                        address: Some(Address::new(Location::new(16567))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("8"),
                                        address: Some(Address::new(Location::new(16568))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("9"),
                                        address: Some(Address::new(Location::new(16569))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("10"),
                                        address: Some(Address::new(Location::new(16570))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("11"),
                                        address: Some(Address::new(Location::new(16571))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("12"),
                                        address: Some(Address::new(Location::new(16572))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("13"),
                                        address: Some(Address::new(Location::new(16573))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("14"),
                                        address: Some(Address::new(Location::new(16574))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                    GlobalVariableView {
                                        name: String::from("15"),
                                        address: Some(Address::new(Location::new(16575))),
                                        size: 1,
                                        type_view: TypeView::Base {
                                            name: String::from("char"),
                                        },
                                        children: vec![],
                                    },
                                ],
                            }],
                        },
                    ],
                },
            ],
        };

        from_global_variable_test(defined_types, global_variable, expected_view);
    }
}
