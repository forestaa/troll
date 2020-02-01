use std::marker::PhantomData;
use std::ops::Deref;

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
    VolatileType {
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
    pub value: isize,
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

    pub fn new_volatile_type_entry(id: TypeEntryId, type_ref: TypeEntryId) -> TypeEntry {
        let kind = TypeEntryKind::VolatileType { type_ref };
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

#[derive(Debug, Clone, PartialEq)]
pub struct Structure;
#[derive(Debug, Clone, PartialEq)]
pub struct Union;
#[derive(Debug, Clone, PartialEq)]
pub struct MemberEntry<T> {
    pub name: String,
    pub location: usize,
    pub type_ref: TypeEntryId,
    pub bit_size: Option<usize>,
    pub bit_offset: Option<usize>,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureTypeMemberEntry(MemberEntry<Structure>);
#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeMemberEntry(MemberEntry<Union>);

impl StructureTypeMemberEntry {
    pub fn new(
        name: String,
        location: usize,
        type_ref: TypeEntryId,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
    ) -> Self {
        Self(
            MemberEntryBuilder::new_structure()
                .name(name)
                .location(location)
                .type_ref(type_ref)
                .bit_size(bit_size)
                .bit_offset(bit_offset)
                .build(),
        )
    }
}

impl From<MemberEntry<Structure>> for StructureTypeMemberEntry {
    fn from(entry: MemberEntry<Structure>) -> Self {
        Self(entry)
    }
}

impl Into<MemberEntry<Structure>> for StructureTypeMemberEntry {
    fn into(self) -> MemberEntry<Structure> {
        self.0
    }
}

impl Deref for StructureTypeMemberEntry {
    type Target = MemberEntry<Structure>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnionTypeMemberEntry {
    pub fn new(
        name: String,
        type_ref: TypeEntryId,
        bit_size: Option<usize>,
        bit_offset: Option<usize>,
    ) -> Self {
        Self(
            MemberEntryBuilder::new_union()
                .name(name)
                .type_ref(type_ref)
                .bit_size(bit_size)
                .bit_offset(bit_offset)
                .build(),
        )
    }
}

impl From<MemberEntry<Union>> for UnionTypeMemberEntry {
    fn from(entry: MemberEntry<Union>) -> Self {
        Self(entry)
    }
}

impl Into<MemberEntry<Union>> for UnionTypeMemberEntry {
    fn into(self) -> MemberEntry<Union> {
        self.0
    }
}

impl Deref for UnionTypeMemberEntry {
    type Target = MemberEntry<Union>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MemberEntryBuilder<NameP, LocationP, TypeRefP, T> {
    name: NameP,
    location: LocationP,
    type_ref: TypeRefP,
    bit_size: Option<usize>,
    bit_offset: Option<usize>,
    _phantom: PhantomData<T>,
}

impl MemberEntryBuilder<(), (), (), Structure> {
    pub fn new_structure() -> Self {
        Self {
            name: (),
            location: (),
            type_ref: (),
            bit_size: None,
            bit_offset: None,
            _phantom: PhantomData,
        }
    }
}

impl MemberEntryBuilder<(), usize, (), Union> {
    pub fn new_union() -> Self {
        Self {
            name: (),
            location: 0,
            type_ref: (),
            bit_size: None,
            bit_offset: None,
            _phantom: PhantomData,
        }
    }
}

impl<T> MemberEntryBuilder<String, usize, TypeEntryId, T> {
    pub fn build(self) -> MemberEntry<T> {
        MemberEntry {
            name: self.name,
            location: self.location,
            type_ref: self.type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            _phantom: PhantomData,
        }
    }
}

impl<LocationP, TypeRefP, T> MemberEntryBuilder<(), LocationP, TypeRefP, T> {
    pub fn name<S: Into<String>>(
        self,
        name: S,
    ) -> MemberEntryBuilder<String, LocationP, TypeRefP, T> {
        MemberEntryBuilder {
            name: name.into(),
            location: self.location,
            type_ref: self.type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            _phantom: PhantomData,
        }
    }
}

impl<NameP, TypeRefP> MemberEntryBuilder<NameP, (), TypeRefP, Structure> {
    pub fn location(
        self,
        location: usize,
    ) -> MemberEntryBuilder<NameP, usize, TypeRefP, Structure> {
        MemberEntryBuilder {
            name: self.name,
            location: location,
            type_ref: self.type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            _phantom: PhantomData,
        }
    }
}

impl<NameP, LocationP, T> MemberEntryBuilder<NameP, LocationP, (), T> {
    pub fn type_ref(
        self,
        type_ref: TypeEntryId,
    ) -> MemberEntryBuilder<NameP, LocationP, TypeEntryId, T> {
        MemberEntryBuilder {
            name: self.name,
            location: self.location,
            type_ref: type_ref,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            _phantom: PhantomData,
        }
    }
}

impl<NameP, LocationP, TypeRefP, T> MemberEntryBuilder<NameP, LocationP, TypeRefP, T> {
    pub fn bit_size(mut self, size: Option<usize>) -> Self {
        self.bit_size = size;
        self
    }

    pub fn bit_offset(mut self, offset: Option<usize>) -> Self {
        self.bit_offset = offset;
        self
    }
}
