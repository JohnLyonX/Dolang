// File 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::{RuntimeContext, intrinsics::ids};

/// File 方法分发（不可变）
pub fn call(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    let (path, mode) = match receiver {
        DolangValue::File { path, mode } => (path, mode.as_deref()),
        _ => return Err(Error::Interpreter("expected File".to_string())),
    };

    let path_arg = [DolangValue::Str(path.clone())];

    match method {
        "exists" => context.call_intrinsic(ids::FS_EXISTS, &path_arg),
        "size" => context.call_intrinsic(ids::FS_SIZE, &path_arg),
        "is_dir" => context.call_intrinsic(ids::FS_IS_DIR, &path_arg),
        "read" => {
            if mode == Some("LINES") {
                context.call_intrinsic(ids::FS_READ_LINES, &path_arg)
            } else {
                context.call_intrinsic(ids::FS_READ_TEXT, &path_arg)
            }
        }
        "read_lines" => context.call_intrinsic(ids::FS_READ_LINES, &path_arg),
        "type" => Ok(DolangValue::Str("File".to_string())),
        "content" => {
            let content_str = match args.first() {
                Some(DolangValue::Str(s)) => s.clone(),
                Some(v) => {
                    return Err(Error::Interpreter(format!(
                        "content() requires String argument, got {}",
                        v.type_name()
                    )));
                }
                None => {
                    return Err(Error::Interpreter(
                        "content() requires String argument".to_string(),
                    ));
                }
            };

            match mode {
                Some("A") | Some("a") | Some("append") => context.call_intrinsic(
                    ids::FS_APPEND_TEXT,
                    &[
                        DolangValue::Str(path.clone()),
                        DolangValue::Str(content_str),
                    ],
                ),
                _ => context.call_intrinsic(
                    ids::FS_WRITE_TEXT,
                    &[
                        DolangValue::Str(path.clone()),
                        DolangValue::Str(content_str),
                    ],
                ),
            }
        }
        _ => Err(Error::Interpreter(format!(
            "file has no method '{}'",
            method
        ))),
    }
}
