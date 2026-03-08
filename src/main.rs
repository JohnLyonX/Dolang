// Dolang REPL - main program entry point.
// Note: lib.rs declares all core modules (interpreter, parser, etc.)
// This file only declares cli and repl (entry-point specific modules)

mod cli;
mod repl;

fn main() {
    match cli::parse_args() {
        cli::RunMode::Run(filename) => {
            repl::run_file(&filename);
        }
        cli::RunMode::Serve(path) => {
            repl::run_serve(path);
        }
        cli::RunMode::Repl => {
            println!("Dolang REPL v1.7");
            repl::run_repl();
        }
    }
}
