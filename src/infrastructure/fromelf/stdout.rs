use crate::domain::global_variable_view::{GlobalVariableView, TypeView};
use std::fmt;

const ADDRESS_WIDTH: usize = 10;
const SIZE_WIDTH: usize = 5;
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
        let variable_name = parent_name.with_parent(variable_view.name().clone());
        let mut lines = vec![FromElfLine {
            address: variable_view.address().map(|addr| addr.clone().into()),
            size: variable_view.size(),
            variable_name: variable_name,
            variable_type: variable_view.type_view().to_string(),
        }];
        let parent_name = parent_name.new_parent_from_variable_view(&variable_view);
        for child in variable_view.children() {
            let mut block = Self::from_variable_view_with_parent(child, &parent_name);
            lines.append(&mut block.lines);
        }
        FromElfBlock { lines }
    }

    fn print(&self) {
        println!(
            "{:ADDRESS_WIDTH$} {:SIZE_WIDTH$} {:VARIABLE_NAME_WIDTH$} {}",
            "address",
            "size",
            "variable_name",
            "type",
            ADDRESS_WIDTH = ADDRESS_WIDTH,
            SIZE_WIDTH = SIZE_WIDTH,
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
    variable_name: String,
    variable_type: String,
}

impl fmt::Display for FromElfLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let address = self.address.unwrap_or(0);
        write!(
            f,
            "{:#0ADDRESS_WIDTH$x} {:#0SIZE_WIDTH$x} {:VARIABLE_NAME_WIDTH$} {}",
            address,
            self.size,
            self.variable_name,
            self.variable_type,
            ADDRESS_WIDTH = ADDRESS_WIDTH,
            SIZE_WIDTH = SIZE_WIDTH,
            VARIABLE_NAME_WIDTH = VARIABLE_NAME_WIDTH,
        )
    }
}

enum ParentName {
    None,
    Structure(String),
    Array(String),
}

impl ParentName {
    fn new_parent_from_variable_view(&self, variable_view: &GlobalVariableView) -> ParentName {
        match variable_view.type_view() {
            TypeView::Structure { .. } => {
                Self::Structure(self.with_parent(variable_view.name().clone()))
            }
            TypeView::Array { .. } => Self::Array(self.with_parent(variable_view.name().clone())),
            TypeView::TypeDef { type_view, .. } => {
                let mut variable_view = variable_view.clone();
                variable_view.set_type_view(*type_view.clone());
                self.new_parent_from_variable_view(&variable_view)
            }
            _ => Self::None,
        }
    }

    fn with_parent(&self, child_name: String) -> String {
        match self {
            Self::None => child_name,
            Self::Structure(parent_name) => format!("{}.{}", parent_name, child_name),
            Self::Array(parent_name) => format!("{}[{}]", parent_name, child_name),
        }
    }
}

impl fmt::Display for TypeView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeView::TypeDef { name, .. } => write!(f, "{}", name),
            TypeView::Const { type_view } => write!(f, "const {}", type_view),
            TypeView::VoidPointer => write!(f, "void pointer"),
            TypeView::Pointer { type_view } => write!(f, "pointer to {}", type_view),
            TypeView::Base { name } => write!(f, "{}", name),
            TypeView::Structure { name } => write!(f, "struct {}", name),
            TypeView::Array {
                element_type,
                upper_bound,
            } => match upper_bound {
                None => write!(f, "{}[]", element_type),
                Some(upper_bound) => write!(f, "{}[{}]", element_type, upper_bound),
            },
        }
    }
}
