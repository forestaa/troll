use super::global_variable::Address;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVariableView {
    name: String,
    address: Option<Address>,
    size: usize,
    type_view: TypeView,
    children: Vec<GlobalVariableView>,
}

impl GlobalVariableView {
    pub fn new(
        name: String,
        address: Option<Address>,
        size: usize,
        type_view: TypeView,
        children: Vec<GlobalVariableView>,
    ) -> Self {
        GlobalVariableView {
            name,
            address,
            size,
            type_view,
            children,
        }
    }

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
    Base {
        name: String,
    },
    Structure {
        name: String,
    },
    Union {
        name: String,
    },
    Array {
        element_type: Box<TypeView>,
        upper_bound: Option<usize>,
    },
    Function,
}

impl TypeView {
    pub fn new_typedef_type_view<S: Into<String>>(name: S, type_view: Self) -> Self {
        Self::TypeDef {
            name: name.into(),
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

    pub fn new_structure_type_view<S: Into<String>>(name: S) -> Self {
        Self::Structure { name: name.into() }
    }

    pub fn new_union_type_view<S: Into<String>>(name: S) -> Self {
        Self::Union { name: name.into() }
    }

    pub fn new_array_type_view(element_type: Self, upper_bound: Option<usize>) -> Self {
        Self::Array {
            element_type: Box::new(element_type),
            upper_bound,
        }
    }

    pub fn new_function_type_view() -> Self {
        Self::Function
    }
}
