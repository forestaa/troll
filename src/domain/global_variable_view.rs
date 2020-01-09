use super::global_variable::{Address, GlobalVariable};
use super::type_entry::{StructureTypeMemberEntry, TypeEntryId, TypeEntryKind};
use super::type_entry_repository::TypeEntryRepository;

#[derive(Debug, Clone)]
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

    pub fn map_name(&mut self, f: impl FnOnce(&String) -> String) {
        self.name = f(&self.name);
    }

    pub fn set_type_view(&mut self, type_view: TypeView) {
        self.map_type_view(|_| type_view);
    }

    pub fn map_type_view(&mut self, f: impl FnOnce(TypeView) -> TypeView) {
        self.type_view = f(self.type_view.clone())
    }
}

#[derive(Debug, Clone)]
pub enum TypeView {
    Base {
        name: String,
    },
    TypeDef {
        name: String,
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
            None => unimplemented!(),
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef {
                    name: type_name,
                    type_ref,
                } => {
                    let global_variable = GlobalVariable::new(
                        global_variable.address(),
                        global_variable.name(),
                        type_ref.clone(),
                    );

                    let mut global_variable_view = self.from_global_variable(global_variable);
                    global_variable_view.set_type_view(TypeView::TypeDef {
                        name: type_name.clone(),
                    });
                    global_variable_view
                }
                TypeEntryKind::ConstType { type_ref } => {
                    let global_variable = GlobalVariable::new(
                        global_variable.address(),
                        global_variable.name(),
                        type_ref.clone(),
                    );

                    let mut global_variable_view = self.from_global_variable(global_variable);
                    global_variable_view.map_type_view(|type_view| TypeView::Const {
                        type_view: Box::new(type_view),
                    });
                    global_variable_view
                }
                TypeEntryKind::PointerType { size, type_ref } => match type_ref {
                    None => GlobalVariableView {
                        name: global_variable.name(),
                        address: global_variable.address(),
                        size: *size,
                        type_view: TypeView::VoidPointer,
                        children: Vec::new(),
                    },
                    Some(type_ref) => GlobalVariableView {
                        name: global_variable.name(),
                        address: global_variable.address(),
                        size: *size,
                        type_view: TypeView::Pointer {
                            type_view: Box::new(self.type_view_from_type_entry(type_ref)),
                        },
                        children: Vec::new(),
                    },
                },
                TypeEntryKind::BaseType { name, size } => GlobalVariableView {
                    name: global_variable.name(),
                    address: global_variable.address(),
                    size: *size,
                    type_view: TypeView::Base { name: name.clone() },
                    children: Vec::new(),
                },
                TypeEntryKind::StructureType { name, members } => {
                    let base_address = &mut global_variable.address();
                    let members: Vec<GlobalVariableView> = members
                        .iter()
                        .map(|member| {
                            let mut member_view =
                                self.from_structure_type_member_entry(member, base_address);
                            member_view.map_name(|member_name| {
                                format!("{}.{}", global_variable.name(), member_name)
                            });
                            member_view
                        })
                        .collect();
                    let size = members.iter().map(|member| member.size()).sum();
                    GlobalVariableView {
                        name: global_variable.name(),
                        address: global_variable.address(),
                        size: size,
                        type_view: TypeView::Structure { name: name.clone() },
                        children: members,
                    }
                }
                TypeEntryKind::ArrayType {
                    type_ref,
                    upper_bound,
                } => {
                    let type_view = self.type_view_from_type_entry(type_ref);
                    let mut size = 0;
                    let elements = match upper_bound {
                        None => {
                            let element_view = self.from_global_variable(GlobalVariable::new(
                                global_variable.address(),
                                global_variable.name(),
                                type_ref.clone(),
                            ));
                            size += element_view.size();
                            vec![element_view]
                        }
                        Some(upper_bound) => (0..*upper_bound)
                            .map(|n| {
                                let element_view = self.from_global_variable(GlobalVariable::new(
                                    global_variable.address().map(|mut addr| {
                                        addr.add(size);
                                        addr
                                    }),
                                    format!("{}[{}]", global_variable.name(), n),
                                    type_ref.clone(),
                                ));
                                size += element_view.size();
                                element_view
                            })
                            .collect(),
                    };

                    GlobalVariableView {
                        name: global_variable.name(),
                        address: global_variable.address(),
                        size: size,
                        type_view: TypeView::Array {
                            element_type: Box::new(type_view),
                            upper_bound: *upper_bound,
                        },
                        children: elements,
                    }
                }
            },
        }
    }

    fn from_structure_type_member_entry(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &mut Option<Address>,
    ) -> GlobalVariableView {
        match self.type_entry_repository.find_by_id(&member.type_ref) {
            None => unimplemented!(),
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef {
                    name: type_name,
                    type_ref,
                } => {
                    let member = StructureTypeMemberEntry {
                        name: member.name.clone(),
                        type_ref: type_ref.clone(),
                    };
                    let mut member_view =
                        self.from_structure_type_member_entry(&member, base_address);

                    member_view.set_type_view(TypeView::Base {
                        name: type_name.clone(),
                    });
                    member_view
                }
                TypeEntryKind::ConstType { type_ref } => {
                    let member = StructureTypeMemberEntry {
                        name: member.name.clone(),
                        type_ref: type_ref.clone(),
                    };
                    let mut member_view =
                        self.from_structure_type_member_entry(&member, base_address);

                    member_view.map_type_view(|type_view| TypeView::Const {
                        type_view: Box::new(type_view),
                    });
                    member_view
                }
                TypeEntryKind::PointerType { size, type_ref } => {
                    let address = base_address.clone();
                    if let Some(addr) = base_address {
                        addr.add(*size);
                    }

                    match type_ref {
                        None => GlobalVariableView {
                            name: member.name.clone(),
                            address: address,
                            size: *size,
                            type_view: TypeView::VoidPointer,
                            children: Vec::new(),
                        },
                        Some(type_ref) => GlobalVariableView {
                            name: member.name.clone(),
                            address: address,
                            size: *size,
                            type_view: TypeView::Pointer {
                                type_view: Box::new(self.type_view_from_type_entry(type_ref)),
                            },
                            children: Vec::new(),
                        },
                    }
                }
                TypeEntryKind::BaseType { name, size } => {
                    let address = base_address.clone();
                    if let Some(addr) = base_address {
                        addr.add(*size);
                    }

                    GlobalVariableView {
                        name: member.name.clone(),
                        address: address,
                        size: *size,
                        type_view: TypeView::Base { name: name.clone() },
                        children: Vec::new(),
                    }
                }
                TypeEntryKind::StructureType { name, members } => {
                    let address = base_address.clone();
                    let members: Vec<GlobalVariableView> = members
                        .iter()
                        .map(|member| {
                            let mut member_view =
                                self.from_structure_type_member_entry(member, base_address);
                            member_view
                                .map_name(|member_name| format!("{}.{}", member.name, member_name));
                            member_view
                        })
                        .collect();
                    let size = members.iter().map(|member| member.size()).sum();

                    GlobalVariableView {
                        name: member.name.clone(),
                        address: address,
                        size: size,
                        type_view: TypeView::Base { name: name.clone() },
                        children: members,
                    }
                }
                TypeEntryKind::ArrayType {
                    type_ref,
                    upper_bound,
                } => {
                    let address = base_address.clone();
                    let type_view = self.type_view_from_type_entry(type_ref);
                    let mut size = 0;
                    let elements = match upper_bound {
                        None => {
                            let element_view = self.from_global_variable(GlobalVariable::new(
                                address.clone(),
                                member.name.clone(),
                                type_ref.clone(),
                            ));
                            size += element_view.size();
                            vec![element_view]
                        }
                        Some(upper_bound) => (0..*upper_bound)
                            .map(|n| {
                                let mut address = address.clone();
                                if let Some(ref mut addr) = address {
                                    addr.add(size);
                                }
                                let element_view = self.from_global_variable(GlobalVariable::new(
                                    address,
                                    format!("{}[{}]", member.name.clone(), n),
                                    type_ref.clone(),
                                ));
                                size += element_view.size();
                                element_view
                            })
                            .collect(),
                    };

                    if let Some(addr) = base_address {
                        addr.add(size);
                    }

                    GlobalVariableView {
                        name: member.name.clone(),
                        address: address,
                        size: size,
                        type_view: TypeView::Array {
                            element_type: Box::new(type_view),
                            upper_bound: *upper_bound,
                        },
                        children: elements,
                    }
                }
            },
        }
    }

    fn type_view_from_type_entry(&self, type_entry_id: &TypeEntryId) -> TypeView {
        match self.type_entry_repository.find_by_id(type_entry_id) {
            None => unimplemented!(),
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef { name, .. } => TypeView::TypeDef { name: name.clone() },
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
                    type_ref,
                    upper_bound,
                } => TypeView::Array {
                    element_type: Box::new(self.type_view_from_type_entry(&type_ref)),
                    upper_bound: *upper_bound,
                },
            },
        }
    }
}
