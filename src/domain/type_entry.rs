use super::entity::Entity;
use crate::library::dwarf;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeEntryId(dwarf::Offset);
impl TypeEntryId {
    pub fn new(offset: dwarf::Offset) -> TypeEntryId {
        TypeEntryId(offset)
    }
}

impl Into<dwarf::Offset> for TypeEntryId {
    fn into(self) -> dwarf::Offset {
        self.0
    }
}

impl Into<usize> for TypeEntryId {
    fn into(self) -> usize {
        let offset: dwarf::Offset = self.into();
        offset.into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeEntryKind {
    TypeDef {
        name: String,
        type_ref: TypeEntryId,
    },
    ConstType {
        type_ref: TypeEntryId,
    },
    PointerType {
        size: usize,
        type_ref: Option<TypeEntryId>,
    },
    BaseType {
        name: String,
        size: usize,
    },
    EnumType {
        name: Option<String>,
        type_ref: TypeEntryId,
        enumerators: Vec<EnumeratorEntry>,
    },
    StructureType {
        name: Option<String>,
        size: usize,
        members: Vec<StructureTypeMemberEntry>,
    },
    UnionType {
        name: Option<String>,
        size: usize,
        members: Vec<UnionTypeMemberEntry>,
    },
    ArrayType {
        element_type_ref: TypeEntryId,
        upper_bound: Option<usize>,
    },
    FunctionType {
        argument_type_ref: Vec<TypeEntryId>,
        return_type_ref: Option<TypeEntryId>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumeratorEntry {
    pub name: String,
    pub value: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureTypeMemberEntry {
    pub name: String,
    pub location: usize,
    pub type_ref: TypeEntryId,
    pub bit_size: Option<usize>,
    pub bit_offset: Option<usize>,
}

impl StructureTypeMemberEntry {
    pub fn new(
        name: String,
        location: usize,
        type_ref: TypeEntryId,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
    ) -> StructureTypeMemberEntry {
        StructureTypeMemberEntry {
            name,
            location,
            type_ref,
            bit_size,
            bit_offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeMemberEntry {
    pub name: String,
    pub type_ref: TypeEntryId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeEntry {
    id: TypeEntryId,
    pub kind: TypeEntryKind,
}

impl TypeEntry {
    pub fn new_typedef_entry(id: TypeEntryId, name: String, type_ref: TypeEntryId) -> TypeEntry {
        let kind = TypeEntryKind::TypeDef { name, type_ref };
        TypeEntry { id, kind }
    }

    pub fn new_const_type_entry(id: TypeEntryId, type_ref: TypeEntryId) -> TypeEntry {
        let kind = TypeEntryKind::ConstType { type_ref };
        TypeEntry { id, kind }
    }

    pub fn new_pointer_type_entry(
        id: TypeEntryId,
        size: usize,
        type_ref: Option<TypeEntryId>,
    ) -> TypeEntry {
        let kind = TypeEntryKind::PointerType { size, type_ref };
        TypeEntry { id, kind }
    }

    pub fn new_base_type_entry(id: TypeEntryId, name: String, size: usize) -> TypeEntry {
        let kind = TypeEntryKind::BaseType { name, size };
        TypeEntry { id, kind }
    }

    pub fn new_enum_type_entry(
        id: TypeEntryId,
        name: Option<String>,
        type_ref: TypeEntryId,
        enumerators: Vec<EnumeratorEntry>,
    ) -> TypeEntry {
        let kind = TypeEntryKind::EnumType {
            name,
            type_ref,
            enumerators,
        };
        TypeEntry { id, kind }
    }

    pub fn new_structure_type_entry(
        id: TypeEntryId,
        name: Option<String>,
        size: usize,
        members: Vec<StructureTypeMemberEntry>,
    ) -> TypeEntry {
        let kind = TypeEntryKind::StructureType {
            name,
            size,
            members,
        };
        TypeEntry { id, kind }
    }

    pub fn new_union_type_entry(
        id: TypeEntryId,
        name: Option<String>,
        size: usize,
        members: Vec<UnionTypeMemberEntry>,
    ) -> TypeEntry {
        let kind = TypeEntryKind::UnionType {
            name,
            size,
            members,
        };
        TypeEntry { id, kind }
    }

    pub fn new_array_type_entry(
        id: TypeEntryId,
        element_type_ref: TypeEntryId,
        upper_bound: Option<usize>,
    ) -> TypeEntry {
        let kind = TypeEntryKind::ArrayType {
            element_type_ref,
            upper_bound,
        };
        TypeEntry { id, kind }
    }

    pub fn new_function_type_entry(
        id: TypeEntryId,
        argument_type_ref: Vec<TypeEntryId>,
        return_type_ref: Option<TypeEntryId>,
    ) -> TypeEntry {
        let kind = TypeEntryKind::FunctionType {
            argument_type_ref,
            return_type_ref,
        };
        TypeEntry { id, kind }
    }

    pub fn id(&self) -> TypeEntryId {
        self.id.clone()
    }
}

impl Entity for TypeEntry {
    type Id = TypeEntryId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

pub struct StructureTypeMemberEntryBuilder<NameP, LocationP, TypeRefP> {
    name: NameP,
    location: LocationP,
    type_ref: TypeRefP,
    bit_size: Option<usize>,
    bit_offset: Option<usize>,
}

impl StructureTypeMemberEntryBuilder<(), (), ()> {
    pub fn new() -> Self {
        StructureTypeMemberEntryBuilder {
            name: (),
            location: (),
            type_ref: (),
            bit_size: None,
            bit_offset: None,
        }
    }
}

impl StructureTypeMemberEntryBuilder<String, usize, TypeEntryId> {
    pub fn build(self) -> StructureTypeMemberEntry {
        StructureTypeMemberEntry {
            name: self.name,
            location: self.location,
            type_ref: self.type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
        }
    }
}

impl<LocationP, TypeRefP> StructureTypeMemberEntryBuilder<(), LocationP, TypeRefP> {
    pub fn name<S: Into<String>>(
        self,
        name: S,
    ) -> StructureTypeMemberEntryBuilder<String, LocationP, TypeRefP> {
        StructureTypeMemberEntryBuilder {
            name: name.into(),
            location: self.location,
            type_ref: self.type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
        }
    }
}

impl<NameP, TypeRefP> StructureTypeMemberEntryBuilder<NameP, (), TypeRefP> {
    pub fn location(
        self,
        location: usize,
    ) -> StructureTypeMemberEntryBuilder<NameP, usize, TypeRefP> {
        StructureTypeMemberEntryBuilder {
            name: self.name,
            location: location,
            type_ref: self.type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
        }
    }
}

impl<NameP, LocationP> StructureTypeMemberEntryBuilder<NameP, LocationP, ()> {
    pub fn type_ref(
        self,
        type_ref: TypeEntryId,
    ) -> StructureTypeMemberEntryBuilder<NameP, LocationP, TypeEntryId> {
        StructureTypeMemberEntryBuilder {
            name: self.name,
            location: self.location,
            type_ref: type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
        }
    }
}

impl<NameP, LocationP, TypeRefP> StructureTypeMemberEntryBuilder<NameP, LocationP, TypeRefP> {
    pub fn bit_size(
        mut self,
        size: usize,
    ) -> StructureTypeMemberEntryBuilder<NameP, LocationP, TypeRefP> {
        self.bit_size = Some(size);
        self
    }

    pub fn bit_offset(
        mut self,
        offset: usize,
    ) -> StructureTypeMemberEntryBuilder<NameP, LocationP, TypeRefP> {
        self.bit_offset = Some(offset);
        self
    }
}
