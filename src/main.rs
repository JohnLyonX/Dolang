// Dolang REPL - main program entry point.
// Note: lib.rs declares all core modules (interpreter, parser, etc.)
// This file only declares cli and repl (entry-point specific modules)

mod cli;
mod repl;

fn main() {
    if let Some(filename) = cli::parse_args() {
        repl::run_file(&filename);
    } else {
        println!("Dolang REPL v1.7");
        repl::run_repl();
    }
}
