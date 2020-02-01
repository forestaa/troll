use super::global_variable::Address;
use super::type_entry::EnumeratorEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVariableView {
    pub name: String,
    pub address: Option<Address>,
    pub size: usize,
    pub bit_size: Option<usize>,
    pub bit_offset: Option<usize>,
    pub type_view: TypeView,
    pub children: Vec<GlobalVariableView>,
}

impl GlobalVariableView {
    pub fn map_type_view(&mut self, f: impl FnOnce(TypeView) -> TypeView) {
        self.type_view = f(self.type_view.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeView {
    TypeDef {
        name: String,
        type_view: Box<TypeView>,
    },
    Volatile {
        type_view: Box<TypeView>,
    },
    Const {
        type_view: Box<TypeView>,
    },
    VoidPointer,
    Pointer {
        type_view: Box<TypeView>,
    },
    Base {
        name: String,
    },
    Structure {
        name: Option<String>,
    },
    Union {
        name: Option<String>,
    },
    Array {
        element_type: Box<TypeView>,
        upper_bound: Option<usize>,
    },
    Enum {
        name: Option<String>,
        type_view: Box<TypeView>,
        enumerators: Vec<Enumerator>,
    },
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enumerator {
    pub name: String,
    pub value: usize,
}

impl From<&EnumeratorEntry> for Enumerator {
    fn from(entry: &EnumeratorEntry) -> Self {
        Enumerator {
            name: entry.name.clone(),
            value: entry.value,
        }
    }
}

impl TypeView {
    pub fn new_typedef_type_view<S: Into<String>>(name: S, type_view: Self) -> Self {
        Self::TypeDef {
            name: name.into(),
            type_view: Box::new(type_view),
        }
    }

    pub fn new_volatile_type_view(type_view: Self) -> Self {
        Self::Volatile {
            type_view: Box::new(type_view),
        }
    }

    pub fn new_const_type_view(type_view: Self) -> Self {
        Self::Const {
            type_view: Box::new(type_view),
        }
    }

    pub fn new_void_pointer_type_view() -> Self {
        Self::VoidPointer
    }

    pub fn new_pointer_type_view(type_view: Self) -> Self {
        Self::Pointer {
            type_view: Box::new(type_view),
        }
    }

    pub fn new_base_type_view<S: Into<String>>(name: S) -> Self {
        Self::Base { name: name.into() }
    }

    pub fn new_structure_type_view<S: Into<String>>(name: Option<S>) -> Self {
        Self::Structure {
            name: name.map(S::into),
        }
    }

    pub fn new_union_type_view<S: Into<String>>(name: Option<S>) -> Self {
        Self::Union {
            name: name.map(S::into),
        }
    }

    pub fn new_array_type_view(element_type: Self, upper_bound: Option<usize>) -> Self {
        Self::Array {
            element_type: Box::new(element_type),
            upper_bound,
        }
    }

    pub fn new_enum_type_view<S: Into<String>>(
        name: Option<S>,
        type_view: Self,
        enumerators: Vec<Enumerator>,
    ) -> Self {
        Self::Enum {
            name: name.map(S::into),
            type_view: Box::new(type_view),
            enumerators,
        }
    }

    pub fn new_function_type_view() -> Self {
        Self::Function
    }
}

pub struct GlobalVariableViewBuilder<NameP, AddressP, SizeP, TypeViewP> {
    name: NameP,
    address: AddressP,
    size: SizeP,
    bit_size: Option<usize>,
    bit_offset: Option<usize>,
    type_view: TypeViewP,
    children: Vec<GlobalVariableView>,
}

impl GlobalVariableViewBuilder<(), (), (), ()> {
    pub fn new() -> Self {
        GlobalVariableViewBuilder {
            name: (),
            address: (),
            size: (),
            bit_size: None,
            bit_offset: None,
            type_view: (),
            children: Vec::new(),
        }
    }
}

impl GlobalVariableViewBuilder<String, Option<Address>, usize, TypeView> {
    pub fn build(self) -> GlobalVariableView {
        GlobalVariableView {
            name: self.name,
            address: self.address,
            size: self.size,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            type_view: self.type_view,
            children: self.children,
        }
    }
}

impl<AddressP, SizeP, TypeViewP> GlobalVariableViewBuilder<(), AddressP, SizeP, TypeViewP> {
    pub fn name<S: Into<String>>(
        self,
        name: S,
    ) -> GlobalVariableViewBuilder<String, AddressP, SizeP, TypeViewP> {
        GlobalVariableViewBuilder {
            name: name.into(),
            address: self.address,
            size: self.size,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            type_view: self.type_view,
            children: self.children,
        }
    }
}

impl<NameP, SizeP, TypeViewP> GlobalVariableViewBuilder<NameP, (), SizeP, TypeViewP> {
    pub fn address(
        self,
        address: Option<Address>,
    ) -> GlobalVariableViewBuilder<NameP, Option<Address>, SizeP, TypeViewP> {
        GlobalVariableViewBuilder {
            name: self.name,
            address: address,
            size: self.size,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            type_view: self.type_view,
            children: self.children,
        }
    }
}

impl<NameP, AddressP, TypeViewP> GlobalVariableViewBuilder<NameP, AddressP, (), TypeViewP> {
    pub fn size(self, size: usize) -> GlobalVariableViewBuilder<NameP, AddressP, usize, TypeViewP> {
        GlobalVariableViewBuilder {
            name: self.name,
            address: self.address,
            size: size,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            type_view: self.type_view,
            children: self.children,
        }
    }
}

impl<NameP, AddressP, SizeP> GlobalVariableViewBuilder<NameP, AddressP, SizeP, ()> {
    pub fn type_view(
        self,
        type_view: TypeView,
    ) -> GlobalVariableViewBuilder<NameP, AddressP, SizeP, TypeView> {
        GlobalVariableViewBuilder {
            name: self.name,
            address: self.address,
            size: self.size,
            bit_size: self.bit_size,
            bit_offset: self.bit_offset,
            type_view: type_view,
            children: self.children,
        }
    }
}

impl<NameP, AddressP, SizeP, TypeViewP>
    GlobalVariableViewBuilder<NameP, AddressP, SizeP, TypeViewP>
{
    pub fn bit_size(mut self, size: Option<usize>) -> Self {
        self.bit_size = size;
        self
    }

    pub fn bit_offset(mut self, offset: Option<usize>) -> Self {
        self.bit_offset = offset;
        self
    }

    pub fn children(mut self, children: Vec<GlobalVariableView>) -> Self {
        self.children = children;
        self
    }
}
