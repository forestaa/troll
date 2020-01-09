pub mod fromelf {
    pub mod stdout {
        use crate::domain::global_variable_view::{GlobalVariableView, TypeView};
        use std::fmt;

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
                let mut lines = vec![FromElfLine {
                    address: variable_view.address().map(|addr| addr.clone().into()),
                    size: variable_view.size(),
                    variable_name: variable_view.name().clone(),
                    variable_type: variable_view.type_view().to_string(),
                }];
                for child in variable_view.children() {
                    let mut block = Self::from_variable_view(child);
                    lines.append(&mut block.lines);
                }
                FromElfBlock { lines }
            }

            fn print(&self) {
                println!(
                    "{:10} {:05} {:20} {}",
                    "address", "size", "variable_name", "type"
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
                    "{:#010x} {:#05x} {:20} {}",
                    address, self.size, self.variable_name, self.variable_type
                )
            }
        }

        impl fmt::Display for TypeView {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    TypeView::TypeDef { name } => write!(f, "{}", name),
                    TypeView::Const { type_view } => write!(f, "const {}", type_view),
                    TypeView::VoidPointer => write!(f, "void pointer"),
                    TypeView::Pointer { type_view } => write!(f, "pointer of {}", type_view),
                    TypeView::Base { name } => write!(f, "{}", name),
                    TypeView::Structure { name } => write!(f, "struct {}", name),
                    TypeView::Array {
                        element_type,
                        upper_bound,
                    } => match upper_bound {
                        None => write!(f, "array[] of {}", element_type),
                        Some(upper_bound) => {
                            write!(f, "array[{}] of {}", upper_bound, element_type)
                        }
                    },
                }
            }
        }
    }
}
