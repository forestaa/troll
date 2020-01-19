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
    StructureType {
        name: String,
        size: usize,
        members: Vec<StructureTypeMemberEntry>,
    },
    UnionType {
        name: String,
        size: usize,
        members: Vec<UnionTypeMemberEntry>,
    },
    ArrayType {
        element_type_ref: TypeEntryId,
        upper_bound: Option<usize>,
    },
    FunctionType {
        argument_type_ref: Vec<TypeEntryId>,
        return_type_ref: TypeEntryId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureTypeMemberEntry {
    pub name: String,
    pub location: usize,
    pub type_ref: TypeEntryId,
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

    pub fn new_structure_type_entry(
        id: TypeEntryId,
        name: String,
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
        name: String,
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
        return_type_ref: TypeEntryId,
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
