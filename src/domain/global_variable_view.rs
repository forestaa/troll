use super::global_variable::{Address, GlobalVariable};
use super::type_entry::{StructureTypeMemberEntry, TypeEntryId, TypeEntryKind};
use super::type_entry_repository::TypeEntryRepository;
use log::error;

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

    pub fn from_global_variable(&self, global_variable: GlobalVariable) -> GlobalVariableView {
        match self
            .type_entry_repository
            .find_by_id(global_variable.type_ref())
        {
            None => {
                error!(
                    "global variable refers unknown offset: variable: {}, refered offset {:?}",
                    global_variable.name(),
                    global_variable.type_ref()
                );
                unimplemented!()
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
                TypeEntryKind::BaseType { name, size } => {
                    self.from_global_variable_base_type(global_variable, name.clone(), *size)
                }
                TypeEntryKind::StructureType {
                    name,
                    size,
                    members,
                } => self.from_global_variable_structure_type(
                    global_variable,
                    name.clone(),
                    *size,
                    members,
                ),
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
    ) -> GlobalVariableView {
        let global_variable =
            GlobalVariable::new(global_variable.address(), global_variable.name(), type_ref);

        let mut global_variable_view = self.from_global_variable(global_variable);
        global_variable_view.map_type_view(|type_view| TypeView::TypeDef {
            name: type_name,
            type_view: Box::new(type_view),
        });
        global_variable_view
    }

    fn from_global_variable_const_type(
        &self,
        global_variable: GlobalVariable,
        type_ref: TypeEntryId,
    ) -> GlobalVariableView {
        let global_variable =
            GlobalVariable::new(global_variable.address(), global_variable.name(), type_ref);

        let mut global_variable_view = self.from_global_variable(global_variable);
        global_variable_view.map_type_view(|type_view| TypeView::Const {
            type_view: Box::new(type_view),
        });
        global_variable_view
    }

    fn from_global_variable_pointer_type(
        &self,
        global_variable: GlobalVariable,
        size: usize,
        type_ref: &Option<TypeEntryId>,
    ) -> GlobalVariableView {
        match type_ref {
            None => GlobalVariableView {
                name: global_variable.name(),
                address: global_variable.address(),
                size: size,
                type_view: TypeView::VoidPointer,
                children: Vec::new(),
            },
            Some(type_ref) => GlobalVariableView {
                name: global_variable.name(),
                address: global_variable.address(),
                size: size,
                type_view: TypeView::Pointer {
                    type_view: Box::new(self.type_view_from_type_entry(type_ref)),
                },
                children: Vec::new(),
            },
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
            .map(|member| self.from_structure_type_member_entry(member, &base_address))
            .collect();

        GlobalVariableView {
            name: global_variable.name(),
            address: global_variable.address(),
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
    ) -> GlobalVariableView {
        let type_view = self.type_view_from_type_entry(element_type_ref);
        let address = global_variable.address();
        let (elements, size) = self.array_elements(
            global_variable.name(),
            &address,
            upper_bound,
            element_type_ref.clone(),
        );

        GlobalVariableView {
            name: global_variable.name(),
            address: address,
            size: size,
            type_view: TypeView::Array {
                element_type: Box::new(type_view),
                upper_bound: upper_bound,
            },
            children: elements,
        }
    }

    fn from_structure_type_member_entry(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
    ) -> GlobalVariableView {
        match self.type_entry_repository.find_by_id(&member.type_ref) {
            None => {
                error!(
                    "structure member refers unknown offset: member: {}, refered offset: {:?}",
                    member.name, member.type_ref
                );
                unimplemented!()
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
                TypeEntryKind::BaseType { name, size } => self
                    .from_structure_type_member_entry_base_type(
                        member,
                        base_address,
                        name.clone(),
                        *size,
                    ),
                TypeEntryKind::StructureType {
                    name,
                    size,
                    members,
                } => self.from_structure_type_member_entry_structure_type(
                    member,
                    base_address,
                    name.clone(),
                    *size,
                    members,
                ),
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
    ) -> GlobalVariableView {
        let member = StructureTypeMemberEntry {
            name: member.name.clone(),
            location: member.location,
            type_ref: type_ref,
        };
        let mut member_view = self.from_structure_type_member_entry(&member, base_address);

        member_view.map_type_view(|type_view| TypeView::TypeDef {
            name: type_name,
            type_view: Box::new(type_view),
        });
        member_view
    }

    fn from_structure_type_member_entry_const_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_ref: TypeEntryId,
    ) -> GlobalVariableView {
        let member = StructureTypeMemberEntry {
            name: member.name.clone(),
            location: member.location,
            type_ref: type_ref,
        };
        let mut member_view = self.from_structure_type_member_entry(&member, base_address);

        member_view.map_type_view(|type_view| TypeView::Const {
            type_view: Box::new(type_view),
        });
        member_view
    }

    fn from_structure_type_member_entry_pointer_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_ref: Option<&TypeEntryId>,
        size: usize,
    ) -> GlobalVariableView {
        let mut address = base_address.clone();
        if let Some(ref mut addr) = address {
            addr.add(member.location);
        }

        match type_ref {
            None => GlobalVariableView {
                name: member.name.clone(),
                address: address,
                size: size,
                type_view: TypeView::VoidPointer,
                children: Vec::new(),
            },
            Some(type_ref) => GlobalVariableView {
                name: member.name.clone(),
                address: address,
                size: size,
                type_view: TypeView::Pointer {
                    type_view: Box::new(self.type_view_from_type_entry(type_ref)),
                },
                children: Vec::new(),
            },
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
            .map(|member| self.from_structure_type_member_entry(member, &address))
            .collect();

        GlobalVariableView {
            name: member.name.clone(),
            address: address.clone(),
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
    ) -> GlobalVariableView {
        let mut address = base_address.clone();
        if let Some(ref mut addr) = address {
            addr.add(member.location);
        }

        let type_view = self.type_view_from_type_entry(element_type_ref);
        let (elements, size) = self.array_elements(
            member.name.clone(),
            &address,
            upper_bound,
            element_type_ref.clone(),
        );

        GlobalVariableView {
            name: member.name.clone(),
            address: address,
            size: size,
            type_view: TypeView::Array {
                element_type: Box::new(type_view),
                upper_bound: upper_bound,
            },
            children: elements,
        }
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
                let element_view = self.from_global_variable(GlobalVariable::new(
                    address.clone(),
                    name,
                    element_type_ref,
                ));
                let size = element_view.size();
                (vec![element_view], size)
            }
            Some(upper_bound) => {
                let mut size = 0;
                let elements = (0..=upper_bound)
                    .map(|n| {
                        let mut address = address.clone();
                        if let Some(ref mut addr) = address {
                            addr.add(size);
                        }
                        let element_view = self.from_global_variable(GlobalVariable::new(
                            address,
                            n.to_string(),
                            element_type_ref.clone(),
                        ));
                        size += element_view.size();
                        element_view
                    })
                    .collect();
                (elements, size)
            }
        }
    }

    fn type_view_from_type_entry(&self, type_entry_id: &TypeEntryId) -> TypeView {
        match self.type_entry_repository.find_by_id(type_entry_id) {
            None => {
                error!(
                    "something refers unknown offset: refered offset: {:?}",
                    type_entry_id
                );
                unimplemented!()
            }
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef { name, type_ref } => TypeView::TypeDef {
                    name: name.clone(),
                    type_view: Box::new(self.type_view_from_type_entry(type_ref)),
                },
                TypeEntryKind::ConstType { type_ref } => TypeView::Const {
                    type_view: Box::new(self.type_view_from_type_entry(&type_ref)),
                },
                TypeEntryKind::PointerType { type_ref, .. } => match type_ref {
                    None => TypeView::VoidPointer,
                    Some(type_ref) => TypeView::Pointer {
                        type_view: Box::new(self.type_view_from_type_entry(&type_ref)),
                    },
                },
                TypeEntryKind::BaseType { name, .. } => TypeView::Base { name: name.clone() },
                TypeEntryKind::StructureType { name, .. } => {
                    TypeView::Structure { name: name.clone() }
                }
                TypeEntryKind::ArrayType {
                    element_type_ref,
                    upper_bound,
                } => TypeView::Array {
                    element_type: Box::new(self.type_view_from_type_entry(&element_type_ref)),
                    upper_bound: *upper_bound,
                },
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

    #[test]
    fn from_global_variable_test() {
        init();

        let mut type_entry_repository = TypeEntryRepository::new();
        let defined_types = vec![
            TypeEntry::new_structure_type_entry(
                TypeEntryId::new(Offset::new(45)),
                String::from("hoge"),
                16,
                vec![
                    StructureTypeMemberEntry {
                        name: String::from("hoge"),
                        location: 0,
                        type_ref: TypeEntryId::new(Offset::new(98)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("hogehoge"),
                        location: 4,
                        type_ref: TypeEntryId::new(Offset::new(105)),
                    },
                    StructureTypeMemberEntry {
                        name: String::from("array"),
                        location: 8,
                        type_ref: TypeEntryId::new(Offset::new(112)),
                    },
                ],
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(98)),
                String::from("int"),
                4,
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(105)),
                String::from("char"),
                1,
            ),
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(112)),
                TypeEntryId::new(Offset::new(98)),
                Some(1),
            ),
            TypeEntry::new_base_type_entry(
                TypeEntryId::new(Offset::new(128)),
                String::from("long unsigned int"),
                8,
            ),
            TypeEntry::new_typedef_entry(
                TypeEntryId::new(Offset::new(135)),
                String::from("Hoge"),
                TypeEntryId::new(Offset::new(45)),
            ),
            TypeEntry::new_array_type_entry(
                TypeEntryId::new(Offset::new(147)),
                TypeEntryId::new(Offset::new(135)),
                Some(2),
            ),
        ];
        for defined_type in defined_types {
            type_entry_repository.save(defined_type);
        }
        let factory = GlobalVariableViewFactory::new(&type_entry_repository);

        let global_variable = GlobalVariable::new(
            Some(Address::new(Location::new(16480))),
            String::from("hoges"),
            TypeEntryId::new(Offset::new(147)),
        );

        let expected_view = GlobalVariableView {
            name: String::from("hoges"),
            address: Some(Address::new(Location::new(16480))),
            size: 48,
            type_view: TypeView::Array {
                element_type: Box::new(TypeView::TypeDef {
                    name: String::from("Hoge"),
                    type_view: Box::new(TypeView::Structure {
                        name: String::from("hoge"),
                    }),
                }),
                upper_bound: Some(2),
            },
            children: vec![
                GlobalVariableView {
                    name: String::from("0"),
                    address: Some(Address::new(Location::new(16480))),
                    size: 16,
                    type_view: TypeView::TypeDef {
                        name: String::from("Hoge"),
                        type_view: Box::new(TypeView::Structure {
                            name: String::from("hoge"),
                        }),
                    },
                    children: vec![
                        GlobalVariableView {
                            name: String::from("hoge"),
                            address: Some(Address::new(Location::new(16480))),
                            size: 4,
                            type_view: TypeView::Base {
                                name: String::from("int"),
                            },
                            children: vec![],
                        },
                        GlobalVariableView {
                            name: String::from("hogehoge"),
                            address: Some(Address::new(Location::new(16484))),
                            size: 1,
                            type_view: TypeView::Base {
                                name: String::from("char"),
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
                    ],
                },
                GlobalVariableView {
                    name: String::from("1"),
                    address: Some(Address::new(Location::new(16496))),
                    size: 16,
                    type_view: TypeView::TypeDef {
                        name: String::from("Hoge"),
                        type_view: Box::new(TypeView::Structure {
                            name: String::from("hoge"),
                        }),
                    },
                    children: vec![
                        GlobalVariableView {
                            name: String::from("hoge"),
                            address: Some(Address::new(Location::new(16496))),
                            size: 4,
                            type_view: TypeView::Base {
                                name: String::from("int"),
                            },
                            children: vec![],
                        },
                        GlobalVariableView {
                            name: String::from("hogehoge"),
                            address: Some(Address::new(Location::new(16500))),
                            size: 1,
                            type_view: TypeView::Base {
                                name: String::from("char"),
                            },
                            children: vec![],
                        },
                        GlobalVariableView {
                            name: String::from("array"),
                            address: Some(Address::new(Location::new(16504))),
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
                                    address: Some(Address::new(Location::new(16504))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                                GlobalVariableView {
                                    name: String::from("1"),
                                    address: Some(Address::new(Location::new(16508))),
                                    size: 4,
                                    type_view: TypeView::Base {
                                        name: String::from("int"),
                                    },
                                    children: vec![],
                                },
                            ],
                        },
                    ],
                },
                GlobalVariableView {
                    name: String::from("2"),
                    address: Some(Address::new(Location::new(16512))),
                    size: 16,
                    type_view: TypeView::TypeDef {
                        name: String::from("Hoge"),
                        type_view: Box::new(TypeView::Structure {
                            name: String::from("hoge"),
                        }),
                    },
                    children: vec![
                        GlobalVariableView {
                            name: String::from("hoge"),
                            address: Some(Address::new(Location::new(16512))),
                            size: 4,
                            type_view: TypeView::Base {
                                name: String::from("int"),
                            },
                            children: vec![],
                        },
                        GlobalVariableView {
                            name: String::from("hogehoge"),
                            address: Some(Address::new(Location::new(16516))),
                            size: 1,
                            type_view: TypeView::Base {
                                name: String::from("char"),
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
                    ],
                },
            ],
        };

        let got_view = factory.from_global_variable(global_variable);
        assert_eq!(expected_view, got_view);
    }
}
