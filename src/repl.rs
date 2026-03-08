// REPL - interactive interpreter loop.
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::process;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};

use dolang::interpreter::{exec, set_current_file, FnEnv, DolangValue};
use dolang::interpreter::env::ValueType;
use dolang::parser;
use dolang::syntax;

const COLOR_RED: &str = "\x1b[31m";
const COLOR_RESET: &str = "\x1b[0m";

fn err_red(msg: &str) {
    println!("{}{}{}", COLOR_RED, msg, COLOR_RESET);
}

// ─── Raw mode for terminal input ───────────────────────────────────────────

fn refresh_line(prompt: &str, buf: &[char], cursor: usize) {
    print!("\r\x1b[K");
    print!("{}", prompt);
    print!("{}", buf.iter().collect::<String>());
    if let Some(moved) = buf.len().checked_sub(cursor) {
        if moved > 0 {
            print!("\x1b[{}D", moved);
        }
    }
    io::stdout().flush().ok();
}

pub fn read_line(prompt: &str, history: &mut Vec<String>) -> Option<String> {
    if !atty::is(atty::Stream::Stdin) {
        return read_line_simple(prompt, history);
    }

    // Try to enable raw mode
    let raw_mode_was_enabled = is_raw_mode_enabled().unwrap_or(false);
    if !raw_mode_was_enabled {
        if let Err(_) = enable_raw_mode() {
            return read_line_simple(prompt, history);
        }
    }

    let result = read_line_raw(prompt, history);

    // Only disable if we enabled it
    if !raw_mode_was_enabled {
        let _ = disable_raw_mode();
    }

    result
}

fn read_line_simple(prompt: &str, history: &mut Vec<String>) -> Option<String> {
    print!("{}", prompt);
    io::stdout().flush().ok();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).ok()? == 0 {
        return None;
    }

    let line = line.trim_end_matches(|c| c == '\n' || c == '\r').to_string();
    if !line.trim().is_empty() {
        history.push(line.clone());
    }
    Some(line)
}

fn read_line_raw(prompt: &str, history: &mut Vec<String>) -> Option<String> {
    print!("\r{}", prompt);
    io::stdout().flush().ok();

    let mut buf: Vec<char> = Vec::new();
    let mut cursor = 0;
    let mut hist_idx = history.len();
    let mut saved_line = String::new();

    loop {
        // Check for keyboard events
        if event::poll(std::time::Duration::from_millis(100)).ok() != Some(true) {
            continue;
        }

        let evt = match event::read() {
            Ok(e) => e,
            Err(_) => continue,
        };

        match evt {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Enter => {
                        print!("\r\n");
                        io::stdout().flush().ok();
                        let line: String = buf.iter().collect();
                        if !line.trim().is_empty() {
                            history.push(line.clone());
                        }
                        return Some(line);
                    }

                    KeyCode::Backspace => {
                        if cursor > 0 {
                            buf.remove(cursor - 1);
                            cursor -= 1;
                            refresh_line(prompt, &buf, cursor);
                        }
                    }

                    KeyCode::Left => {
                        if cursor > 0 {
                            cursor -= 1;
                            print!("\x1b[D");
                            io::stdout().flush().ok();
                        }
                    }

                    KeyCode::Right => {
                        if cursor < buf.len() {
                            cursor += 1;
                            print!("\x1b[C");
                            io::stdout().flush().ok();
                        }
                    }

                    KeyCode::Up => {
                        if hist_idx == history.len() {
                            saved_line = buf.iter().collect();
                        }
                        if hist_idx > 0 {
                            hist_idx -= 1;
                            buf = history[hist_idx].chars().collect();
                            cursor = buf.len();
                            refresh_line(prompt, &buf, cursor);
                        }
                    }

                    KeyCode::Down => {
                        if hist_idx < history.len() {
                            hist_idx += 1;
                            let new_line = if hist_idx == history.len() {
                                saved_line.clone()
                            } else {
                                history[hist_idx].clone()
                            };
                            buf = new_line.chars().collect();
                            cursor = buf.len();
                            refresh_line(prompt, &buf, cursor);
                        }
                    }

                    KeyCode::Home => {
                        // Ctrl+A: move to start
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            cursor = 0;
                            refresh_line(prompt, &buf, cursor);
                        }
                    }

                    KeyCode::End => {
                        // Ctrl+E: move to end
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            cursor = buf.len();
                            refresh_line(prompt, &buf, cursor);
                        }
                    }

                    KeyCode::Char(c) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl+C: cancel current line
                            if c == 'c' {
                                print!("\r\n");
                                io::stdout().flush().ok();
                                return Some(String::new());
                            }
                            // Ctrl+D: EOF if buffer is empty
                            if c == 'd' && buf.is_empty() {
                                return None;
                            }
                        } else {
                            buf.insert(cursor, c);
                            cursor += 1;
                            refresh_line(prompt, &buf, cursor);
                        }
                    }

                    KeyCode::Delete => {
                        if cursor < buf.len() {
                            buf.remove(cursor);
                            refresh_line(prompt, &buf, cursor);
                        }
                    }

                    _ => {}
                }
            }

            Event::Resize(_, _) => {
                refresh_line(prompt, &buf, cursor);
            }

            _ => {}
        }
    }
}

// ─── REPL main loop ────────────────────────────────────────────────────────

pub fn run_repl() {
    let mut env: HashMap<String, DolangValue> = HashMap::new();
    let mut type_env: HashMap<String, ValueType> = HashMap::new();
    let mut const_env: HashMap<String, bool> = HashMap::new();
    let mut fns: FnEnv = HashMap::new();
    let mut history: Vec<String> = Vec::new();

    loop {
        let line = match read_line(">> ", &mut history) {
            Some(l) => l,
            None => break,
        };

        let first = line.trim().to_string();
        if first.is_empty() {
            continue;
        }

        // Check for exit/quit commands
        if first == "exit"
        {
            break;
        }

        // Multi-line input collection (bracket-aware)
        let mut buf = first;
        let mut is_multiline = false;
        loop {
            let depth: i64 = buf.chars().fold(0, |acc, c| match c {
                '{' => acc + 1,
                '}' => acc - 1,
                _ => acc,
            });
            if depth <= 0 {
                break;
            }
            is_multiline = true;
            match read_line(".. ", &mut history) {
                Some(next) => {
                    // In multiline mode, handle single-line comments specially
                    // Remove // comments but keep the newline
                    let processed = if let Some(pos) = next.find("//") {
                        format!("{}\n", &next[..pos])
                    } else {
                        next
                    };
                    buf.push('\n');
                    let trimmed = processed.trim_start().trim_end();
                    buf.push_str(trimmed);
                }
                None => break,
            }
        }
        let full = buf;
        let line = full.trim();

        if syntax::contains_raw_english(line) {
            err_red("[ERROR] illegal identifier: raw english is not allowed");
            continue;
        }

        if let (false, bad) = syntax::validate_dollar_literals(line) {
            err_red(&format!("[ERROR] invalid $...$ literal: {}", bad));
            continue;
        }

        // Strip comments before checking for semicolon (only in single-line mode)
        // In multiline mode (function body), we need to keep comments to preserve brace matching
        let line_without_comments;
        let is_empty_after_strip;
        if is_multiline {
            // In multiline mode, don't strip comments but still trim
            line_without_comments = line.trim().to_string();
            // Check if the line (after trimming) is effectively empty (comment-only)
            is_empty_after_strip = line_without_comments.is_empty() ||
                line_without_comments.chars().all(|c| c.is_whitespace() ||
                    (c == '/' && line_without_comments.contains("//")) ||
                    (c == '/' && line_without_comments.contains("/*")));
        } else {
            // In single-line mode, strip comments
            match syntax::syntax::strip_comments(line) {
                Ok(stripped) => {
                    line_without_comments = stripped.trim().to_string();
                    is_empty_after_strip = line_without_comments.is_empty();
                }
                Err(e) => {
                    err_red(&format!("[ERROR] {}", e));
                    continue;
                }
            }
        }

        // Skip empty lines (e.g., comment-only lines)
        if is_empty_after_strip && !is_multiline {
            continue;
        }

        let is_block_stmt = line_without_comments.ends_with('}');
        // Check if this might be a multiline statement (starts with $fn, $if, $while, etc.)
        let is_likely_multiline = line_without_comments.starts_with("$fn ")
            || line_without_comments.starts_with("$if ")
            || line_without_comments.starts_with("$while ")
            || line_without_comments.starts_with("$for ")
            || line_without_comments.starts_with("$loop ");

        // In multiline mode or likely multiline statements, we don't check for semicolon
        if !is_multiline && !is_likely_multiline && !is_block_stmt && !line_without_comments.ends_with(';') {
            err_red("[ERROR] missing ';'");
            continue;
        }

        let src = if is_multiline {
            // In multiline mode, preserve the full input
            line_without_comments.as_str()
        } else if is_block_stmt {
            line_without_comments.as_str()
        } else {
            line_without_comments.trim_end_matches(';')
        };

        let statements = match parser::parse(src) {
            Ok(stmts) => stmts,
            Err(e) => {
                err_red(&format!("[ERROR] {}", e));
                continue;
            }
        };

        for stmt in statements {
            let (cont, exec_err) = exec(&stmt, &mut env, &mut fns, &mut type_env, &mut const_env);
            if let Err(e) = exec_err {
                err_red(&format!("[ERROR] {}", e));
                break;
            }
            if !cont {
                process::exit(0);
            }
        }
    }
}

// ─── File execution ────────────────────────────────────────────────────────

pub fn run_file(filename: &str) {
    // Set current file for $main() scope checking
    let file_path = std::path::Path::new(filename);
    let file_name = file_path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| filename.to_string());
    set_current_file(Some(file_name.clone()));

    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => {
            err_red(&format!("[ERROR] cannot read file '{}': {}", filename, e));
            process::exit(1);
        }
    };

    // Parse the file - validation is handled by the parser
    let statements = match parser::parse(&content) {
        Ok(stmts) => stmts,
        Err(e) => {
            err_red(&format!("[ERROR] {}", e));
            process::exit(1);
        }
    };

    let mut env: HashMap<String, DolangValue> = HashMap::new();
    let mut type_env: HashMap<String, ValueType> = HashMap::new();
    let mut const_env: HashMap<String, bool> = HashMap::new();
    let mut fns: FnEnv = HashMap::new();

    for stmt in statements {
        let (cont, exec_err) = exec(&stmt, &mut env, &mut fns, &mut type_env, &mut const_env);
        if let Err(e) = exec_err {
            err_red(&format!("[ERROR] {}", e));
            process::exit(1);
        }
        if !cont {
            process::exit(0);
        }
    }

    // Clear current file after execution
    set_current_file(None);
}
