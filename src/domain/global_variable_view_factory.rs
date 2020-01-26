use super::global_variable::*;
use super::global_variable_view::*;
use super::type_entry::*;
use super::type_entry_repository::TypeEntryRepository;
use super::variable_declaration_repository::VariableDeclarationRepository;
use log::warn;

pub struct GlobalVariableViewFactory<'type_repo, 'dec_repo> {
    type_entry_repository: &'type_repo TypeEntryRepository,
    variable_declaration_repository: &'dec_repo VariableDeclarationRepository,
}

impl<'type_repo, 'dec_repo> GlobalVariableViewFactory<'type_repo, 'dec_repo> {
    pub fn new(
        type_entry_repository: &'type_repo TypeEntryRepository,
        variable_declaration_repository: &'dec_repo VariableDeclarationRepository,
    ) -> Self {
        Self {
            type_entry_repository,
            variable_declaration_repository,
        }
    }

    pub fn from_global_variable(
        &self,
        global_variable: GlobalVariable,
    ) -> Option<GlobalVariableView> {
        match global_variable {
            GlobalVariable::HasSpec { address, spec } => {
                self.from_global_variable_with_spec(address, spec)
            }
            GlobalVariable::NoSpec {
                address,
                name,
                type_ref,
            } => self.from_global_variable_no_spec(address, name, type_ref),
        }
    }

    fn from_global_variable_with_spec(
        &self,
        address: Option<Address>,
        spec: VariableDeclarationEntryId,
    ) -> Option<GlobalVariableView> {
        match self.variable_declaration_repository.find_by_id(&spec) {
            None => {
                let offset: usize = spec.clone().into();
                warn!(
                    "global variable refers unknown specification: refered specification offset: {:#x}",
                    offset
                );
                None
            }
            Some(variable_dec) => self.from_global_variable_no_spec(
                address,
                variable_dec.name.clone(),
                variable_dec.type_ref.clone(),
            ),
        }
    }

    fn from_global_variable_no_spec(
        &self,
        address: Option<Address>,
        variable_name: String,
        type_ref: TypeEntryId,
    ) -> Option<GlobalVariableView> {
        match self.type_entry_repository.find_by_id(&type_ref) {
            None => {
                let offset: usize = type_ref.clone().into();
                warn!(
                    "global variable refers unknown offset: variable: {}, refered offset {:#x}",
                    variable_name, offset
                );
                None
            }
            Some(type_entry) => match &type_entry.kind {
                TypeEntryKind::TypeDef {
                    name: type_name,
                    type_ref,
                } => self.from_global_variable_typedef(
                    address,
                    variable_name,
                    type_name.clone(),
                    type_ref.clone(),
                ),
                TypeEntryKind::ConstType { type_ref } => {
                    self.from_global_variable_const_type(address, variable_name, type_ref.clone())
                }
                TypeEntryKind::PointerType { size, type_ref } => {
                    self.from_global_variable_pointer_type(address, variable_name, *size, type_ref)
                }
                TypeEntryKind::BaseType {
                    name: type_name,
                    size,
                } => Some(self.from_global_variable_base_type(
                    address,
                    variable_name,
                    type_name.clone(),
                    *size,
                )),
                TypeEntryKind::EnumType {
                    name: type_name,
                    type_ref,
                    enumerators,
                } => self.from_global_variable_enum_type(
                    address,
                    variable_name,
                    type_name.clone(),
                    type_ref.clone(),
                    enumerators,
                ),
                TypeEntryKind::StructureType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.from_global_variable_structure_type(
                    address,
                    variable_name,
                    type_name.clone(),
                    *size,
                    members,
                )),
                TypeEntryKind::UnionType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.from_global_variable_union_type(
                    address,
                    variable_name,
                    type_name.clone(),
                    *size,
                    members,
                )),
                TypeEntryKind::ArrayType {
                    element_type_ref,
                    upper_bound,
                } => self.from_global_variable_array_type(
                    address,
                    variable_name,
                    element_type_ref,
                    *upper_bound,
                ),
                TypeEntryKind::FunctionType { .. } => {
                    let offset: usize = type_ref.clone().into();
                    warn!(
                        "global variable should not refer subroutine_type: variable: {}, refered offset {:#x}",
                        variable_name,
                       offset
                    );
                    None
                }
            },
        }
    }

    fn from_global_variable_typedef(
        &self,
        address: Option<Address>,
        variable_name: String,
        type_name: String,
        type_ref: TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let mut global_variable_view =
            self.from_global_variable_no_spec(address, variable_name, type_ref)?;
        global_variable_view
            .map_type_view(|type_view| TypeView::new_typedef_type_view(type_name, type_view));
        Some(global_variable_view)
    }

    fn from_global_variable_const_type(
        &self,
        address: Option<Address>,
        variable_name: String,
        type_ref: TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let mut global_variable_view =
            self.from_global_variable_no_spec(address, variable_name, type_ref)?;
        global_variable_view.map_type_view(|type_view| TypeView::new_const_type_view(type_view));
        Some(global_variable_view)
    }

    fn from_global_variable_pointer_type(
        &self,
        address: Option<Address>,
        variable_name: String,
        size: usize,
        type_ref: &Option<TypeEntryId>,
    ) -> Option<GlobalVariableView> {
        match type_ref {
            None => Some(GlobalVariableView::new(
                variable_name,
                address,
                size,
                TypeView::new_void_pointer_type_view(),
                Vec::new(),
            )),
            Some(type_ref) => {
                let type_view = self.type_view_from_type_entry(type_ref)?;
                Some(GlobalVariableView::new(
                    variable_name,
                    address,
                    size,
                    TypeView::new_pointer_type_view(type_view),
                    Vec::new(),
                ))
            }
        }
    }

    fn from_global_variable_base_type(
        &self,
        address: Option<Address>,
        variable_name: String,
        type_name: String,
        size: usize,
    ) -> GlobalVariableView {
        GlobalVariableView::new(
            variable_name,
            address,
            size,
            TypeView::new_base_type_view(type_name),
            Vec::new(),
        )
    }

    fn from_global_variable_enum_type(
        &self,
        address: Option<Address>,
        variable_name: String,
        type_name: Option<String>,
        type_ref: TypeEntryId,
        enumerators: &Vec<EnumeratorEntry>,
    ) -> Option<GlobalVariableView> {
        let mut global_variable_view =
            self.from_global_variable_no_spec(address, variable_name, type_ref)?;

        let enumerators = enumerators.iter().map(Enumerator::from).collect();
        global_variable_view.map_type_view(|type_view| {
            TypeView::new_enum_type_view(type_name, type_view, enumerators)
        });

        Some(global_variable_view)
    }

    fn from_global_variable_structure_type(
        &self,
        address: Option<Address>,
        variable_name: String,
        type_name: Option<String>,
        size: usize,
        members: &Vec<StructureTypeMemberEntry>,
    ) -> GlobalVariableView {
        let members: Vec<GlobalVariableView> = members
            .iter()
            .flat_map(|member| self.from_structure_type_member_entry(member, &address))
            .collect();

        GlobalVariableView::new(
            variable_name,
            address,
            size,
            TypeView::new_structure_type_view(type_name),
            members,
        )
    }

    fn from_global_variable_union_type(
        &self,
        address: Option<Address>,
        variable_name: String,
        type_name: Option<String>,
        size: usize,
        members: &Vec<UnionTypeMemberEntry>,
    ) -> GlobalVariableView {
        let members: Vec<GlobalVariableView> = members
            .iter()
            .flat_map(|member| {
                self.from_global_variable_no_spec(
                    address.clone(),
                    member.name.clone(),
                    member.type_ref.clone(),
                )
            })
            .collect();

        GlobalVariableView::new(
            variable_name,
            address,
            size,
            TypeView::new_union_type_view(type_name),
            members,
        )
    }

    fn from_global_variable_array_type(
        &self,
        address: Option<Address>,
        variable_name: String,
        element_type_ref: &TypeEntryId,
        upper_bound: Option<usize>,
    ) -> Option<GlobalVariableView> {
        let type_view = self.type_view_from_type_entry(element_type_ref)?;
        let (elements, size) = self.array_elements(&address, upper_bound, element_type_ref.clone());

        Some(GlobalVariableView::new(
            variable_name,
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
                TypeEntryKind::EnumType {
                    name: type_name,
                    type_ref,
                    enumerators,
                } => self.from_structure_type_member_entry_enum_type(
                    member,
                    base_address,
                    type_name.clone(),
                    type_ref.clone(),
                    enumerators,
                ),
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
        let member = StructureTypeMemberEntry::new(
            member.name.clone(),
            member.location,
            type_ref,
            None,
            None,
        );
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
        let member = StructureTypeMemberEntry::new(
            member.name.clone(),
            member.location,
            type_ref,
            None,
            None,
        );
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

    fn from_structure_type_member_entry_enum_type(
        &self,
        member: &StructureTypeMemberEntry,
        base_address: &Option<Address>,
        type_name: Option<String>,
        type_ref: TypeEntryId,
        enumerators: &Vec<EnumeratorEntry>,
    ) -> Option<GlobalVariableView> {
        let member = StructureTypeMemberEntry::new(
            member.name.clone(),
            member.location,
            type_ref,
            None,
            None,
        );
        let mut member_view = self.from_structure_type_member_entry(&member, base_address)?;

        let enumerators = enumerators.iter().map(Enumerator::from).collect();
        member_view.map_type_view(|type_view| {
            TypeView::new_enum_type_view(type_name, type_view, enumerators)
        });

        Some(member_view)
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
                self.from_global_variable(GlobalVariable::new_variable(
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
        let (elements, size) = self.array_elements(&address, upper_bound, element_type_ref.clone());

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
        address: &Option<Address>,
        upper_bound: Option<usize>,
        element_type_ref: TypeEntryId,
    ) -> (Vec<GlobalVariableView>, usize) {
        match upper_bound {
            None => {
                let mut elements = vec![];
                let mut size = 0;
                if let Some(element_view) = self.from_global_variable_no_spec(
                    address.clone(),
                    0.to_string(),
                    element_type_ref,
                ) {
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
                        let element_view =
                            self.from_global_variable(GlobalVariable::new_variable(
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
                TypeEntryKind::EnumType {
                    name,
                    type_ref,
                    enumerators,
                } => {
                    let type_view = self.type_view_from_type_entry(type_ref)?;
                    let enumerators = enumerators.iter().map(Enumerator::from).collect();
                    Some(TypeView::new_enum_type_view(
                        name.clone(),
                        type_view,
                        enumerators,
                    ))
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
