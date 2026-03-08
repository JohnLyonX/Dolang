// CLI argument handling.
use std::env;
use std::path::PathBuf;
use std::process;

/// Run mode for the interpreter
#[derive(Debug, Clone)]
pub enum RunMode {
    Repl,
    Run(String),       // Run a single .dol file
    Serve(PathBuf),   // Serve mode: path to main.dol or directory
}

/// Parse CLI arguments and return the run mode.
pub fn parse_args() -> RunMode {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        match args[1].as_str() {
            "run" if args.len() >= 3 => {
                let filename = &args[2];
                if filename.ends_with(".dol") {
                    RunMode::Run(filename.clone())
                } else {
                    eprintln!("Error: Only .dol files can be executed");
                    process::exit(1);
                }
            }
            "serve" => {
                // serve with optional path
                let path = if args.len() >= 3 {
                    PathBuf::from(&args[2])
                } else {
                    PathBuf::from(".")
                };
                RunMode::Serve(path)
            }
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            _ => {
                eprintln!(
                    "Error: Unknown command '{}'. Use 'dolang run <file>' to run a file, or 'dolang serve' to start server mode.",
                    args[1]
                );
                process::exit(1);
            }
        }
    } else {
        RunMode::Repl
    }
}

pub fn print_help() {
    println!("Dolang - A simple programming language");
    println!();
    println!("Usage:");
    println!("  dolang               Start REPL mode");
    println!("  dolang run <file>    Run a .dol file");
    println!("  dolang serve [path]  Start server mode (default: current directory)");
    println!();
    println!("Options:");
    println!("  -h, --help           Show this help message");
}
