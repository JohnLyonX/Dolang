// CLI argument handling.
use std::env;
use std::process;

/// Parse CLI arguments and return the filename to run, or None for REPL mode.
pub fn parse_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        if args[1] == "run" && args.len() >= 3 {
            let filename = &args[2];
            if filename.ends_with(".dol") {
                return Some(filename.clone());
            }
            eprintln!("Error: Only .dol files can be executed");
            process::exit(1);
        }
        if args[1] == "--help" || args[1] == "-h" {
            print_help();
            process::exit(0);
        }
        eprintln!(
            "Error: Unknown command '{}'. Use 'dolang run
  <file.dol>' to run a file.",
            args[1]
        );
        process::exit(1);
    } else {
        None
    }
}

pub fn print_help() {
    println!("Dolang - A simple programming language");
    println!();
    println!("Usage:");
    println!("  dolang               Start REPL mode");
    println!("  dolang run <file>    Run a .dol file");
    println!();
    println!("Options:");
    println!("  -h, --help           Show this help message");
}
