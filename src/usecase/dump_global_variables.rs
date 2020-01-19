use crate::domain::global_variable_view::{GlobalVariableView, GlobalVariableViewFactory};
use crate::domain::global_variables_extractor::GlobalVariablesExtractor;
use crate::domain::type_entry_repository::TypeEntryRepository;
use crate::library::dwarf;

pub struct DumpGlobalVariablesUsecase {
    type_entry_repository: TypeEntryRepository,
}

impl DumpGlobalVariablesUsecase {
    pub fn new() -> DumpGlobalVariablesUsecase {
        DumpGlobalVariablesUsecase {
            type_entry_repository: TypeEntryRepository::new(),
        }
    }

    pub fn dump_global_variables(&mut self, elf_path: String) -> Vec<GlobalVariableView> {
        let iter = dwarf::DwarfInfoIntoIterator::new(elf_path).into_iter();

        let mut global_variables_extractor =
            GlobalVariablesExtractor::new(&mut self.type_entry_repository);
        let global_variables = global_variables_extractor.extract(iter);

        let global_variable_view_factory =
            GlobalVariableViewFactory::new(&self.type_entry_repository);
        global_variables
            .into_iter()
            .flat_map(|variable| global_variable_view_factory.from_global_variable(variable))
            .collect()
    }
}
