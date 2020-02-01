use crate::domain::global_variable_view::*;
use std::fmt;
use std::fmt::Write;

const ADDRESS_WIDTH: usize = 10;
const SIZE_WIDTH: usize = 5;
const BITFIELD_WIDTH: usize = 7;
const VARIABLE_NAME_WIDTH: usize = 20;

pub struct FromElfStdOut {
    blocks: Vec<FromElfBlock>,
}

impl FromElfStdOut {
    pub fn new(variable_views: Vec<GlobalVariableView>) -> FromElfStdOut {
        let blocks = variable_views
            .into_iter()
            .map(|variable_view| FromElfBlock::from_variable_view(variable_view))
            .collect();
        FromElfStdOut { blocks: blocks }
    }

    pub fn print(&self) {
        for block in &self.blocks {
            block.print();
            println!();
        }
    }
}

struct FromElfBlock {
    lines: Vec<FromElfLine>,
}

impl FromElfBlock {
    fn from_variable_view(variable_view: GlobalVariableView) -> FromElfBlock {
        Self::from_variable_view_with_parent(variable_view, &ParentName::None)
    }

    fn from_variable_view_with_parent(
        variable_view: GlobalVariableView,
        parent_name: &ParentName,
    ) -> FromElfBlock {
        let variable_name = parent_name.with_parent(&variable_view.name);
        let parent_name = parent_name
            .new_parent_from_variable_view(&variable_view.name, &variable_view.type_view);

        let mut lines = vec![FromElfLine {
            address: variable_view.address.map(|addr| addr.clone().into()),
            size: variable_view.size,
            bitfield: OptionalBitField::new(variable_view.bit_offset, variable_view.bit_size),
            variable_name: variable_name,
            variable_type: variable_view.type_view.to_string(),
        }];

        for child in variable_view.children {
            let mut block = Self::from_variable_view_with_parent(child, &parent_name);
            lines.append(&mut block.lines);
        }
        FromElfBlock { lines }
    }

    fn print(&self) {
        println!(
            "{:ADDRESS_WIDTH$} {:SIZE_WIDTH$}{:BITFIELD_WIDTH$} {:VARIABLE_NAME_WIDTH$} {}",
            "address",
            "size",
            "(bit)",
            "variable_name",
            "type",
            ADDRESS_WIDTH = ADDRESS_WIDTH,
            SIZE_WIDTH = SIZE_WIDTH,
            BITFIELD_WIDTH = BITFIELD_WIDTH,
            VARIABLE_NAME_WIDTH = VARIABLE_NAME_WIDTH
        );
        for line in &self.lines {
            println!("{}", line);
        }
    }
}

struct FromElfLine {
    address: Option<usize>,
    size: usize,
    bitfield: OptionalBitField,
    variable_name: String,
    variable_type: String,
}

impl fmt::Display for FromElfLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let address = self.address.unwrap_or(0);
        write!(
            f,
            "{:#0ADDRESS_WIDTH$x} {:#0SIZE_WIDTH$x}{:BITFIELD_WIDTH$} {:VARIABLE_NAME_WIDTH$} {}",
            address,
            self.size,
            self.bitfield,
            self.variable_name,
            self.variable_type,
            ADDRESS_WIDTH = ADDRESS_WIDTH,
            SIZE_WIDTH = SIZE_WIDTH,
            BITFIELD_WIDTH = BITFIELD_WIDTH,
            VARIABLE_NAME_WIDTH = VARIABLE_NAME_WIDTH,
        )
    }
}

enum ParentName {
    None,
    Structure(String),
    Union(String),
    Array(String),
}

impl ParentName {
    fn new_parent_from_variable_view(
        &self,
        variable_view_name: &String,
        type_view: &TypeView,
    ) -> ParentName {
        match type_view {
            TypeView::Structure { .. } => Self::Structure(self.with_parent(variable_view_name)),
            TypeView::Union { .. } => Self::Union(self.with_parent(variable_view_name)),
            TypeView::Array { .. } => Self::Array(self.with_parent(variable_view_name)),
            TypeView::TypeDef { type_view, .. } => {
                self.new_parent_from_variable_view(variable_view_name, type_view)
            }
            TypeView::Volatile { type_view } => {
                self.new_parent_from_variable_view(variable_view_name, type_view)
            }
            TypeView::Const { type_view } => {
                self.new_parent_from_variable_view(variable_view_name, type_view)
            }
            _ => Self::None,
        }
    }

    fn with_parent(&self, child_name: &String) -> String {
        match self {
            Self::None => child_name.clone(),
            Self::Structure(parent_name) => format!("{}.{}", parent_name, child_name),
            Self::Union(parent_name) => format!("{}.{}", parent_name, child_name),
            Self::Array(parent_name) => format!("{}[{}]", parent_name, child_name),
        }
    }
}

struct OptionalBitField(Option<BitField>);
impl OptionalBitField {
    fn new(offset: Option<usize>, size: Option<usize>) -> Self {
        match (offset, size) {
            (Some(offset), Some(size)) => OptionalBitField(Some(BitField { offset, size })),
            _ => OptionalBitField(None),
        }
    }
}

struct BitField {
    offset: usize,
    size: usize,
}

impl fmt::Display for OptionalBitField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(ref bitfield) => format!("({}:{})", bitfield.offset, bitfield.size).fmt(f),
            None => "".fmt(f),
        }
    }
}

impl fmt::Display for TypeView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeView::TypeDef { name, type_view } => match **type_view {
                TypeView::Enum {
                    name: ref enum_name,
                    ref type_view,
                    ref enumerators,
                } => format!(
                    "{} enum {}: {}  values = {}",
                    name,
                    enum_name.as_ref().unwrap_or(&String::from("")),
                    type_view,
                    Enumerators(enumerators)
                )
                .fmt(f),
                _ => format!("{}", name).fmt(f),
            },
            TypeView::Volatile { type_view } => format!("volatile {}", type_view).fmt(f),
            TypeView::Const { type_view } => format!("const {}", type_view).fmt(f),
            TypeView::VoidPointer => format!("void pointer").fmt(f),
            TypeView::Pointer { type_view } => format!("pointer to {}", type_view).fmt(f),
            TypeView::Base { name } => format!("{}", name).fmt(f),
            TypeView::Structure { name } => {
                format!("struct {}", name.as_ref().unwrap_or(&String::from(""))).fmt(f)
            }
            TypeView::Union { name } => {
                format!("union {}", name.as_ref().unwrap_or(&String::from(""))).fmt(f)
            }
            TypeView::Enum {
                name,
                type_view,
                enumerators,
            } => format!(
                "enum {}: {}  values = {}",
                name.as_ref().unwrap_or(&String::from("")),
                type_view,
                Enumerators(enumerators)
            )
            .fmt(f),
            TypeView::Array {
                element_type,
                upper_bound,
            } => match upper_bound {
                None => format!("{}[]", element_type).fmt(f),
                Some(upper_bound) => format!("{}[{}]", element_type, upper_bound).fmt(f),
            },
            TypeView::Function {} => "function".fmt(f),
        }
    }
}

impl fmt::Display for Enumerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format!("{}: {}", self.name, self.value).fmt(f)
    }
}

struct Enumerators<'a>(&'a Vec<Enumerator>);

impl<'a> fmt::Display for Enumerators<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        for enumerator in self.0 {
            let _ = write!(&mut out, "{}, ", enumerator);
        }

        out.fmt(f)
    }
}
