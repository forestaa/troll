mod domain;
mod infrastructure;
mod library;
mod usecase;

use std::env;

use infrastructure::fromelf::stdout::FromElfStdOut;
use usecase::DumpGlobalVariablesUsecase;

fn main() {
    for path in env::args().skip(1) {
        dump_global_variables(path);
    }
}

fn dump_global_variables(filepath: String) {
    let mut usecase = DumpGlobalVariablesUsecase::new();
    let global_variables = usecase.dump_global_variables(filepath);
    FromElfStdOut::new(global_variables).print();
}
