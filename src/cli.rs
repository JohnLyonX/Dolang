// CLI argument handling.
use std::env;
use std::path::PathBuf;
use std::process;

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub file: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub body: Option<String>,
}

impl TestConfig {
    pub fn new() -> Self {
        TestConfig {
            file: None,
            method: None,
            path: None,
            body: None,
        }
    }
}

/// Run mode for the interpreter
#[derive(Debug, Clone)]
pub enum RunMode {
    Repl,
    Run(String),                    // Run a single .dol file
    Serve(PathBuf, bool),           // Serve mode: path, show_routertab
    Test(TestConfig),               // Test mode: configuration
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
                // Check for --routertab flag
                let show_routertab = args.iter().any(|arg| arg == "--routertab");

                // serve with optional path
                let path = if args.len() >= 3 && !args[2].starts_with("--") {
                    PathBuf::from(&args[2])
                } else {
                    PathBuf::from(".")
                };
                RunMode::Serve(path, show_routertab)
            }
            "test" => {
                // Parse test command arguments
                let mut config = TestConfig::new();

                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--route" => {
                            // Parse "METHOD /path" - needs at least 3 args: --route METHOD /path
                            if i + 2 < args.len() {
                                config.method = Some(args[i + 1].clone());
                                config.path = Some(args[i + 2].clone());
                                i += 3;
                            } else if i + 1 < args.len() {
                                // Try to parse "METHOD /path" from single argument
                                let route = &args[i + 1];
                                if let Some(space_idx) = route.find(' ') {
                                    config.method = Some(route[..space_idx].to_string());
                                    config.path = Some(route[space_idx + 1..].to_string());
                                    i += 2;
                                } else {
                                    eprintln!("Error: --route requires METHOD and path (e.g., --route GET /hello)");
                                    process::exit(1);
                                }
                            } else {
                                eprintln!("Error: --route requires METHOD and path (e.g., --route GET /hello)");
                                process::exit(1);
                            }
                        }
                        "--body" | "-b" => {
                            if i + 1 < args.len() {
                                config.body = Some(args[i + 1].clone());
                                i += 2;
                            } else {
                                eprintln!("Error: --body requires a value");
                                process::exit(1);
                            }
                        }
                        arg if arg.ends_with(".dol") => {
                            config.file = Some(arg.to_string());
                            i += 1;
                        }
                        "-h" | "--help" => {
                            print_help();
                            process::exit(0);
                        }
                        _ => {
                            eprintln!("Error: Unknown test argument '{}'", args[i]);
                            process::exit(1);
                        }
                    }
                }

                RunMode::Test(config)
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
    println!("  dolang test [options] Run tests (default: main.dol)");
    println!();
    println!("Test Options:");
    println!("  dolang test [file]   Run tests from .dol file (default: main.dol)");
    println!("  dolang test --route METHOD /path  Test a specific route");
    println!("  dolang test --body JSON   Request body for POST/PUT");
    println!();
    println!("Server Options:");
    println!("  -h, --help           Show this help message");
    println!("  --routertab          Show registered routes on server start");
}
