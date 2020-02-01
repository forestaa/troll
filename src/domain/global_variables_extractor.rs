use log::warn;

use super::entry_factory::*;
use super::global_variable::*;
use super::type_entry_repository::TypeEntryRepository;
use super::variable_declaration_entry_repository::VariableDeclarationEntryRepository;
use crate::library::dwarf::DwarfInfo;

pub struct GlobalVariablesExtractor<'type_repo, 'dec_repo> {
    type_entry_repository: &'type_repo mut TypeEntryRepository,
    variable_declaration_repository: &'dec_repo mut VariableDeclarationEntryRepository,
}

impl<'type_repo, 'dec_repo> GlobalVariablesExtractor<'type_repo, 'dec_repo> {
    pub fn new(
        type_entry_repository: &'type_repo mut TypeEntryRepository,
        variable_declaration_repository: &'dec_repo mut VariableDeclarationEntryRepository,
    ) -> Self {
        Self {
            type_entry_repository,
            variable_declaration_repository,
        }
    }

    pub fn extract(&mut self, infos: impl Iterator<Item = DwarfInfo>) -> Vec<GlobalVariable> {
        let mut global_variables = Vec::new();
        for info in infos {
            match EntryFactory::from_dwarf_info(&info) {
                Ok(FromDwarfInfoOutput::GlobalVariable(global_variable)) => {
                    global_variables.push(global_variable)
                }
                Ok(FromDwarfInfoOutput::TypeEntry {
                    entry,
                    children_warnings,
                }) => {
                    for warnings in children_warnings {
                        Self::warning_no_expected_attribute(warnings, &info);
                    }
                    self.type_entry_repository.save(entry)
                }
                Ok(FromDwarfInfoOutput::VariableDeclarationEntry(entry)) => {
                    self.variable_declaration_repository.save(entry)
                }
                _ => (),
            }
        }
        global_variables
    }

    fn warning_no_expected_attribute(message: String, entry: &DwarfInfo) {
        let offset: usize = entry.offset().into();
        warn!("Skip this entry: {}: offset = {:#x}", message, offset);
    }
}
