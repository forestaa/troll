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
            } => self.variable_view_from_type_ref(name, address, None, None, &type_ref),
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
            Some(variable_dec) => self.variable_view_from_type_ref(
                variable_dec.name.clone(),
                address,
                None,
                None,
                &variable_dec.type_ref,
            ),
        }
    }

    fn variable_view_from_type_ref(
        &self,
        variable_name: String,
        address: Option<Address>,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_ref: &TypeEntryId,
    ) -> Option<GlobalVariableView> {
        match self.type_entry_repository.find_by_id(type_ref) {
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
                } => self.typedef_variable_view(
                    variable_name,
                    address,
                    bit_size,
                    bit_offset,
                    type_name.clone(),
                    type_ref,
                ),
                TypeEntryKind::VolatileType { type_ref } => self.volatile_type_variable_view(
                    variable_name,
                    address,
                    bit_size,
                    bit_offset,
                    type_ref,
                ),
                TypeEntryKind::ConstType { type_ref } => self.const_type_variable_view(
                    variable_name,
                    address,
                    bit_size,
                    bit_offset,
                    type_ref,
                ),
                TypeEntryKind::PointerType { size, type_ref } => self.pointer_type_variable_view(
                    variable_name,
                    address,
                    *size,
                    bit_size,
                    bit_offset,
                    type_ref.as_ref(),
                ),
                TypeEntryKind::BaseType {
                    name: type_name,
                    size,
                } => Some(Self::base_type_variable_view(
                    variable_name,
                    address,
                    *size,
                    bit_size,
                    bit_offset,
                    type_name.clone(),
                )),
                TypeEntryKind::EnumType {
                    name: type_name,
                    type_ref,
                    enumerators,
                } => self.enum_type_variable_view(
                    variable_name,
                    address,
                    bit_size,
                    bit_offset,
                    type_name.clone(),
                    type_ref,
                    enumerators,
                ),
                TypeEntryKind::StructureType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.structure_type_variable_view(
                    variable_name,
                    address,
                    *size,
                    bit_size,
                    bit_offset,
                    type_name.clone(),
                    members,
                )),
                TypeEntryKind::UnionType {
                    name: type_name,
                    size,
                    members,
                } => Some(self.union_type_variable_view(
                    variable_name,
                    address,
                    *size,
                    bit_size,
                    bit_offset,
                    type_name.clone(),
                    members,
                )),
                TypeEntryKind::ArrayType {
                    element_type_ref,
                    upper_bound,
                } => self.array_type_variable_view(
                    variable_name,
                    address,
                    bit_size,
                    bit_offset,
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

    fn typedef_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_name: String,
        type_ref: &TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let mut global_variable_view = self.variable_view_from_type_ref(
            variable_name,
            address,
            bit_size,
            bit_offset,
            type_ref,
        )?;
        global_variable_view
            .map_type_view(|type_view| TypeView::new_typedef_type_view(type_name, type_view));
        Some(global_variable_view)
    }

    fn volatile_type_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_ref: &TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let mut global_variable_view = self.variable_view_from_type_ref(
            variable_name,
            address,
            bit_size,
            bit_offset,
            type_ref,
        )?;
        global_variable_view.map_type_view(|type_view| TypeView::new_volatile_type_view(type_view));
        Some(global_variable_view)
    }

    fn const_type_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_ref: &TypeEntryId,
    ) -> Option<GlobalVariableView> {
        let mut global_variable_view = self.variable_view_from_type_ref(
            variable_name,
            address,
            bit_size,
            bit_offset,
            type_ref,
        )?;
        global_variable_view.map_type_view(|type_view| TypeView::new_const_type_view(type_view));
        Some(global_variable_view)
    }

    fn pointer_type_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        size: usize,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_ref: Option<&TypeEntryId>,
    ) -> Option<GlobalVariableView> {
        match type_ref {
            None => Some(
                GlobalVariableViewBuilder::new()
                    .name(variable_name)
                    .address(address)
                    .size(size)
                    .bit_size(bit_size)
                    .bit_offset(bit_offset)
                    .type_view(TypeView::new_void_pointer_type_view())
                    .build(),
            ),
            Some(type_ref) => {
                let type_view = self.type_view_from_type_entry(type_ref)?;
                Some(
                    GlobalVariableViewBuilder::new()
                        .name(variable_name)
                        .address(address)
                        .size(size)
                        .bit_size(bit_size)
                        .bit_offset(bit_offset)
                        .type_view(TypeView::new_pointer_type_view(type_view))
                        .build(),
                )
            }
        }
    }

    fn base_type_variable_view(
        variable_name: String,
        address: Option<Address>,
        size: usize,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_name: String,
    ) -> GlobalVariableView {
        GlobalVariableViewBuilder::new()
            .name(variable_name)
            .address(address)
            .size(size)
            .bit_size(bit_size)
            .bit_offset(bit_offset)
            .type_view(TypeView::new_base_type_view(type_name))
            .build()
    }

    fn enum_type_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_name: Option<String>,
        type_ref: &TypeEntryId,
        enumerators: &Vec<EnumeratorEntry>,
    ) -> Option<GlobalVariableView> {
        let mut global_variable_view = self.variable_view_from_type_ref(
            variable_name,
            address,
            bit_size,
            bit_offset,
            type_ref,
        )?;

        let enumerators = enumerators.iter().map(Enumerator::from).collect();
        global_variable_view.map_type_view(|type_view| {
            TypeView::new_enum_type_view(type_name, type_view, enumerators)
        });

        Some(global_variable_view)
    }

    fn structure_type_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        size: usize,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_name: Option<String>,
        members: &Vec<StructureTypeMemberEntry>,
    ) -> GlobalVariableView {
        let members: Vec<MemberEntry<Structure>> =
            members.iter().map(|member| member.clone().into()).collect();
        let children = self.members_variable_view(address.as_ref(), members);

        GlobalVariableViewBuilder::new()
            .name(variable_name)
            .address(address)
            .size(size)
            .bit_size(bit_size)
            .bit_offset(bit_offset)
            .type_view(TypeView::new_structure_type_view(type_name))
            .children(children)
            .build()
    }

    fn union_type_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        size: usize,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        type_name: Option<String>,
        members: &Vec<UnionTypeMemberEntry>,
    ) -> GlobalVariableView {
        let members: Vec<MemberEntry<Union>> =
            members.iter().map(|member| member.clone().into()).collect();
        let children = self.members_variable_view(address.as_ref(), members);

        GlobalVariableViewBuilder::new()
            .name(variable_name)
            .address(address)
            .size(size)
            .bit_size(bit_size)
            .bit_offset(bit_offset)
            .type_view(TypeView::new_union_type_view(type_name))
            .children(children)
            .build()
    }

    fn members_variable_view<T>(
        &self,
        base_address: Option<&Address>,
        members: Vec<MemberEntry<T>>,
    ) -> Vec<GlobalVariableView> {
        members
            .iter()
            .flat_map(|member| {
                let address = base_address.map(|addr| {
                    let mut addr = addr.clone();
                    addr.add(member.location);
                    addr
                });
                self.variable_view_from_type_ref(
                    member.name.clone(),
                    address,
                    member.bit_size,
                    member.bit_offset,
                    &member.type_ref,
                )
            })
            .collect()
    }

    fn array_type_variable_view(
        &self,
        variable_name: String,
        address: Option<Address>,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
        element_type_ref: &TypeEntryId,
        upper_bound: Option<usize>,
    ) -> Option<GlobalVariableView> {
        let type_view = self.type_view_from_type_entry(element_type_ref)?;
        let (elements, size) =
            self.array_elements_(&address, upper_bound, element_type_ref.clone());

        Some(
            GlobalVariableViewBuilder::new()
                .name(variable_name)
                .address(address)
                .size(size)
                .bit_size(bit_size)
                .bit_offset(bit_offset)
                .type_view(TypeView::new_array_type_view(type_view, upper_bound))
                .children(elements)
                .build(),
        )
    }

    fn array_elements_(
        &self,
        address: &Option<Address>,
        upper_bound: Option<usize>,
        element_type_ref: TypeEntryId,
    ) -> (Vec<GlobalVariableView>, usize) {
        match upper_bound {
            None => {
                let mut elements = vec![];
                let mut size = 0;
                //TODO: What happens if use array as a member with bit field?
                if let Some(element_view) = self.variable_view_from_type_ref(
                    0.to_string(),
                    address.clone(),
                    None,
                    None,
                    &element_type_ref,
                ) {
                    size += element_view.size;
                    elements.push(element_view);
                }
                (elements, size)
            }
            Some(upper_bound) => {
                let mut size = 0;
                let elements = (0..=upper_bound)
                    .flat_map(|n| {
                        let address = address.clone().map(|mut addr| {
                            addr.add(size);
                            addr
                        });
                        //TODO: What happens if use array as a member with bit field?
                        let element_view = self.variable_view_from_type_ref(
                            n.to_string(),
                            address,
                            None,
                            None,
                            &element_type_ref,
                        )?;
                        size += element_view.size;
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
                TypeEntryKind::VolatileType { type_ref } => {
                    let type_view = self.type_view_from_type_entry(type_ref)?;
                    Some(TypeView::new_volatile_type_view(type_view))
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
