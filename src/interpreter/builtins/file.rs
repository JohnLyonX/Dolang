// File 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;

/// File 方法分发（不可变）
pub fn call(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    let (path, mode) = match receiver {
        DolangValue::File { path, mode } => (path, mode.as_deref()),
        _ => return Err(Error::Interpreter("expected File".to_string())),
    };

    match method {
        "exists" => {
            use std::fs;
            let exists = fs::metadata(path).is_ok();
            Ok(DolangValue::Bool(exists))
        }
        "size" => {
            use std::fs;
            match fs::metadata(path) {
                Ok(metadata) => Ok(DolangValue::Int(metadata.len() as i64)),
                Err(_) => Ok(DolangValue::Int(0)),
            }
        }
        "is_dir" => {
            use std::fs;
            match fs::metadata(path) {
                Ok(metadata) => Ok(DolangValue::Bool(metadata.is_dir())),
                Err(_) => Ok(DolangValue::Bool(false)),
            }
        }
        "read" => {
            use std::fs;
            // If mode is "LINES", return List instead
            if mode == Some("LINES") {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        let lines: Vec<DolangValue> = content
                            .lines()
                            .map(|s| DolangValue::Str(s.to_string()))
                            .collect();
                        Ok(DolangValue::List(lines))
                    }
                    Err(e) => Err(Error::Interpreter(format!("cannot read file: {}", e))),
                }
            } else {
                match fs::read_to_string(path) {
                    Ok(content) => Ok(DolangValue::Str(content)),
                    Err(e) => Err(Error::Interpreter(format!("cannot read file: {}", e))),
                }
            }
        }
        "read_lines" => {
            use std::fs;
            match fs::read_to_string(path) {
                Ok(content) => {
                    let lines: Vec<DolangValue> = content
                        .lines()
                        .map(|s| DolangValue::Str(s.to_string()))
                        .collect();
                    Ok(DolangValue::List(lines))
                }
                Err(e) => Err(Error::Interpreter(format!("cannot read file: {}", e))),
            }
        }
        "type" => Ok(DolangValue::Str("File".to_string())),
        "content" => {
            use std::fs::{self, OpenOptions};
            use std::io::Write;

            // Get content string from args
            let content_str = match args.first() {
                Some(DolangValue::Str(s)) => s.clone(),
                Some(v) => return Err(Error::Interpreter(format!(
                    "content() requires String argument, got {}",
                    v.type_name()
                ))),
                None => return Err(Error::Interpreter("content() requires String argument".to_string())),
            };

            // Handle based on mode
            match mode {
                Some("A") | Some("a") | Some("append") => {
                    // Append mode
                    match OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                    {
                        Ok(mut file) => {
                            match file.write_all(content_str.as_bytes()) {
                                Ok(_) => Ok(DolangValue::Null),
                                Err(e) => Err(Error::Interpreter(format!(
                                    "write error: {}",
                                    e
                                ))),
                            }
                        }
                        Err(e) => Err(Error::Interpreter(format!(
                            "cannot open file: {}",
                            e
                        ))),
                    }
                }
                _ => {
                    // Write mode (default)
                    match fs::write(path, &content_str) {
                        Ok(_) => Ok(DolangValue::Null),
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                Err(Error::Interpreter(format!(
                                    "directory not found for path: {}",
                                    path
                                )))
                            } else {
                                Err(Error::Interpreter(format!(
                                    "write error: {}",
                                    e
                                )))
                            }
                        }
                    }
                }
            }
        }
        _ => {
            Err(Error::Interpreter(format!("file has no method '{}'", method)))
        }
    }
}
