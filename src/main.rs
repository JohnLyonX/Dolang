// Dolang REPL - main program entry point.
// Note: lib.rs declares all core modules (interpreter, parser, etc.)
// This file only declares cli, repl, server, and test (entry-point specific modules)

extern crate dolang;

mod cli;
mod repl;
mod server;
mod test;

fn main() {
    match cli::parse_args() {
        cli::RunMode::Run(filename) => {
            repl::run_file(&filename);
        }
        cli::RunMode::Serve(path, show_routertab) => {
            server::run_serve(path, show_routertab);
        }
        cli::RunMode::Test(config) => {
            test::run_test(test::TestConfig {
                file: config.file,
                method: config.method,
                path: config.path,
                body: config.body,
            });
        }
        cli::RunMode::Repl => {
            println!("Dolang REPL v1.7");
            repl::run_repl();
        }
    }
}
