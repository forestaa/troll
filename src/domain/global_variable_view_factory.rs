use super::global_variable::{Address, GlobalVariable};
use super::global_variable_view::{GlobalVariableView, TypeView};
use super::type_entry::*;
use super::type_entry_repository::TypeEntryRepository;
use log::warn;

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
                let offset: usize = global_variable.type_ref().clone().into();
                warn!(
                    "global variable refers unknown offset: variable: {}, refered offset {:#x}",
                    global_variable.name(),
                    offset
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
                TypeEntryKind::UnionType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.from_global_variable_union_type(
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
                TypeEntryKind::FunctionType { .. } => {
                    let offset: usize = global_variable.type_ref().clone().into();
                    warn!(
                        "global variable should not refer subroutine_type: variable: {}, refered offset {:#x}",
                        global_variable.name(),
                       offset
                    );
                    None
                }
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
        global_variable_view
            .map_type_view(|type_view| TypeView::new_typedef_type_view(type_name, type_view));
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
        global_variable_view.map_type_view(|type_view| TypeView::new_const_type_view(type_view));
        Some(global_variable_view)
    }

    fn from_global_variable_pointer_type(
        &self,
        global_variable: GlobalVariable,
        size: usize,
        type_ref: &Option<TypeEntryId>,
    ) -> Option<GlobalVariableView> {
        match type_ref {
            None => Some(GlobalVariableView::new(
                global_variable.name(),
                global_variable.address(),
                size,
                TypeView::new_void_pointer_type_view(),
                Vec::new(),
            )),
            Some(type_ref) => {
                let type_view = self.type_view_from_type_entry(type_ref)?;
                Some(GlobalVariableView::new(
                    global_variable.name(),
                    global_variable.address(),
                    size,
                    TypeView::new_pointer_type_view(type_view),
                    Vec::new(),
                ))
            }
        }
    }

    fn from_global_variable_base_type(
        &self,
        global_variable: GlobalVariable,
        type_name: String,
        size: usize,
    ) -> GlobalVariableView {
        GlobalVariableView::new(
            global_variable.name(),
            global_variable.address(),
            size,
            TypeView::new_base_type_view(type_name),
            Vec::new(),
        )
    }

    fn from_global_variable_structure_type(
        &self,
        global_variable: GlobalVariable,
        type_name: Option<String>,
        size: usize,
        members: &Vec<StructureTypeMemberEntry>,
    ) -> GlobalVariableView {
        let base_address = global_variable.address();
        let members: Vec<GlobalVariableView> = members
            .iter()
            .flat_map(|member| self.from_structure_type_member_entry(member, &base_address))
            .collect();

        GlobalVariableView::new(
            global_variable.name(),
            base_address,
            size,
            TypeView::new_structure_type_view(type_name),
            members,
        )
    }

    fn from_global_variable_union_type(
        &self,
        global_variable: GlobalVariable,
        type_name: Option<String>,
        size: usize,
        members: &Vec<UnionTypeMemberEntry>,
    ) -> GlobalVariableView {
        let members: Vec<GlobalVariableView> = members
            .iter()
            .flat_map(|member| {
                self.from_global_variable(GlobalVariable::new(
                    global_variable.address(),
                    member.name.clone(),
                    member.type_ref.clone(),
                ))
            })
            .collect();

        GlobalVariableView::new(
            global_variable.name(),
            global_variable.address(),
            size,
            TypeView::new_union_type_view(type_name),
            members,
        )
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

        Some(GlobalVariableView::new(
            global_variable.name(),
            address,
            size,
            TypeView::new_array_type_view(type_view, upper_bound),
            elements,
        ))
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
                TypeEntryKind::UnionType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.from_structure_type_member_entry_union_type(
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
                TypeEntryKind::FunctionType { .. } => {
                    let offset: usize = member.type_ref.clone().into();
                    warn!(
                        "structure member should not refer subroutine_type: member: {}, refered offset {:#x}",
                        member.name,
                        offset
                    );
                    None
                }
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

        member_view
            .map_type_view(|type_view| TypeView::new_typedef_type_view(type_name, type_view));
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

        member_view.map_type_view(|type_view| TypeView::new_const_type_view(type_view));
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
            None => Some(GlobalVariableView::new(
                member.name.clone(),
                address,
                size,
                TypeView::new_void_pointer_type_view(),
                Vec::new(),
            )),
            Some(type_ref) => {
                let type_view = self.type_view_from_type_entry(type_ref)?;
                Some(GlobalVariableView::new(
                    member.name.clone(),
                    address,
                    size,
                    TypeView::new_pointer_type_view(type_view),
                    Vec::new(),
                ))
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

        GlobalVariableView::new(
            member.name.clone(),
            address,
            size,
            TypeView::new_base_type_view(type_name),
            Vec::new(),
        )
    }

    fn from_structure_type_member_entry_structure_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_name: Option<String>,
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

        GlobalVariableView::new(
            member.name.clone(),
            address,
            size,
            TypeView::new_structure_type_view(type_name),
            members,
        )
    }

    fn from_structure_type_member_entry_union_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_name: Option<String>,
        size: usize,
        members: &Vec<UnionTypeMemberEntry>,
    ) -> GlobalVariableView {
        let mut address = base_address.clone();
        if let Some(ref mut addr) = address {
            addr.add(member.location);
        }
        let members: Vec<GlobalVariableView> = members
            .iter()
            .flat_map(|member| {
                self.from_global_variable(GlobalVariable::new(
                    address.clone(),
                    member.name.clone(),
                    member.type_ref.clone(),
                ))
            })
            .collect();

        GlobalVariableView::new(
            member.name.clone(),
            address,
            size,
            TypeView::new_structure_type_view(type_name),
            members,
        )
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

        Some(GlobalVariableView::new(
            member.name.clone(),
            address,
            size,
            TypeView::new_array_type_view(type_view, upper_bound),
            elements,
        ))
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
                    Some(TypeView::new_typedef_type_view(name.clone(), type_view))
                }
                TypeEntryKind::ConstType { type_ref } => {
                    let type_view = self.type_view_from_type_entry(type_ref)?;
                    Some(TypeView::new_const_type_view(type_view))
                }
                TypeEntryKind::PointerType { type_ref, .. } => match type_ref {
                    None => Some(TypeView::new_void_pointer_type_view()),
                    Some(type_ref) => {
                        let type_view = self.type_view_from_type_entry(type_ref)?;
                        Some(TypeView::new_pointer_type_view(type_view))
                    }
                },
                TypeEntryKind::BaseType { name, .. } => {
                    Some(TypeView::new_base_type_view(name.clone()))
                }
                TypeEntryKind::StructureType { name, .. } => {
                    Some(TypeView::new_structure_type_view(name.clone()))
                }
                TypeEntryKind::UnionType { name, .. } => {
                    Some(TypeView::new_union_type_view(name.clone()))
                }
                TypeEntryKind::ArrayType {
                    element_type_ref,
                    upper_bound,
                } => {
                    let type_view = self.type_view_from_type_entry(element_type_ref)?;
                    Some(TypeView::new_array_type_view(type_view, *upper_bound))
                }
                TypeEntryKind::FunctionType { .. } => Some(TypeView::new_function_type_view()),
            },
        }
    }
}
