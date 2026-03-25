use std::sync::Arc;

use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::{NativeFnMap, RuntimeContext};
use rand::Rng;

pub fn register(context: &mut RuntimeContext) {
    let mut exports = NativeFnMap::new();
    exports.insert(
        "sqrt".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.sqrt", args, 0)?;
            Ok(DolangValue::Float(n.sqrt()))
        }),
    );
    exports.insert(
        "pow".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let base = float_arg("math.pow", args, 0)?;
            let exp = float_arg("math.pow", args, 1)?;
            Ok(DolangValue::Float(base.powf(exp)))
        }),
    );
    exports.insert(
        "abs".into(),
        Arc::new(
            |args: &[DolangValue], _ctx: &RuntimeContext| match args.first() {
                Some(DolangValue::Int(n)) => Ok(DolangValue::Int(n.abs())),
                Some(DolangValue::Float(f)) => Ok(DolangValue::Float(f.abs())),
                Some(other) => Err(Error::Interpreter(format!(
                    "math.abs: expected Int or Float, got {}",
                    other.type_name()
                ))),
                None => Err(Error::Interpreter("math.abs: missing first arg".into())),
            },
        ),
    );
    exports.insert(
        "floor".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.floor", args, 0)?;
            Ok(DolangValue::Int(n.floor() as i64))
        }),
    );
    exports.insert(
        "ceil".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.ceil", args, 0)?;
            Ok(DolangValue::Int(n.ceil() as i64))
        }),
    );
    exports.insert(
        "round".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.round", args, 0)?;
            Ok(DolangValue::Int(n.round() as i64))
        }),
    );
    exports.insert(
        "min".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let (left, right) = same_numeric_pair("math.min", args)?;
            Ok(match (left, right) {
                (DolangValue::Int(a), DolangValue::Int(b)) => DolangValue::Int(*a.min(b)),
                (DolangValue::Float(a), DolangValue::Float(b)) => DolangValue::Float(a.min(*b)),
                _ => unreachable!(),
            })
        }),
    );
    exports.insert(
        "max".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let (left, right) = same_numeric_pair("math.max", args)?;
            Ok(match (left, right) {
                (DolangValue::Int(a), DolangValue::Int(b)) => DolangValue::Int(*a.max(b)),
                (DolangValue::Float(a), DolangValue::Float(b)) => DolangValue::Float(a.max(*b)),
                _ => unreachable!(),
            })
        }),
    );
    exports.insert(
        "clamp".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let (value, min, max) = same_numeric_triplet("math.clamp", args)?;
            Ok(match (value, min, max) {
                (DolangValue::Int(v), DolangValue::Int(min), DolangValue::Int(max)) => {
                    DolangValue::Int(*v.clamp(min, max))
                }
                (DolangValue::Float(v), DolangValue::Float(min), DolangValue::Float(max)) => {
                    DolangValue::Float(v.clamp(*min, *max))
                }
                _ => unreachable!(),
            })
        }),
    );
    exports.insert(
        "pi".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            ensure_arity("math.pi", args, 0)?;
            Ok(DolangValue::Float(std::f64::consts::PI))
        }),
    );
    exports.insert(
        "sin".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.sin", args, 0)?;
            Ok(DolangValue::Float(n.sin()))
        }),
    );
    exports.insert(
        "cos".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.cos", args, 0)?;
            Ok(DolangValue::Float(n.cos()))
        }),
    );
    // --- 反三角函数 ---
    exports.insert(
        "tan".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.tan", args, 0)?;
            Ok(DolangValue::Float(n.tan()))
        }),
    );
    exports.insert(
        "asin".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.asin", args, 0)?;
            Ok(DolangValue::Float(n.asin()))
        }),
    );
    exports.insert(
        "acos".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.acos", args, 0)?;
            Ok(DolangValue::Float(n.acos()))
        }),
    );
    exports.insert(
        "atan".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.atan", args, 0)?;
            Ok(DolangValue::Float(n.atan()))
        }),
    );
    exports.insert(
        "atan2".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let y = float_arg("math.atan2", args, 0)?;
            let x = float_arg("math.atan2", args, 1)?;
            Ok(DolangValue::Float(y.atan2(x)))
        }),
    );
    // --- 对数 / 指数 ---
    exports.insert(
        "exp".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.exp", args, 0)?;
            Ok(DolangValue::Float(n.exp()))
        }),
    );
    exports.insert(
        "ln".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.ln", args, 0)?;
            Ok(DolangValue::Float(n.ln()))
        }),
    );
    exports.insert(
        "log".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let x = float_arg("math.log", args, 0)?;
            let base = float_arg("math.log", args, 1)?;
            Ok(DolangValue::Float(x.log(base)))
        }),
    );
    exports.insert(
        "log2".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.log2", args, 0)?;
            Ok(DolangValue::Float(n.log2()))
        }),
    );
    exports.insert(
        "log10".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let n = float_arg("math.log10", args, 0)?;
            Ok(DolangValue::Float(n.log10()))
        }),
    );
    // --- 随机数 ---
    exports.insert(
        "random".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            ensure_arity("math.random", args, 0)?;
            Ok(DolangValue::Float(rand::random::<f64>()))
        }),
    );
    exports.insert(
        "random_int".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            ensure_arity("math.random_int", args, 2)?;
            let min = match &args[0] {
                DolangValue::Int(n) => *n,
                other => {
                    return Err(Error::Interpreter(format!(
                        "math.random_int: expected Int at argument 0, got {}",
                        other.type_name()
                    )));
                }
            };
            let max = match &args[1] {
                DolangValue::Int(n) => *n,
                other => {
                    return Err(Error::Interpreter(format!(
                        "math.random_int: expected Int at argument 1, got {}",
                        other.type_name()
                    )));
                }
            };
            if min > max {
                return Err(Error::Interpreter(format!(
                    "math.random_int: min ({min}) must be <= max ({max})"
                )));
            }
            Ok(DolangValue::Int(rand::thread_rng().gen_range(min..=max)))
        }),
    );
    // --- 其他 ---
    exports.insert(
        "hypot".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let x = float_arg("math.hypot", args, 0)?;
            let y = float_arg("math.hypot", args, 1)?;
            Ok(DolangValue::Float(x.hypot(y)))
        }),
    );
    exports.insert(
        "e".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            ensure_arity("math.e", args, 0)?;
            Ok(DolangValue::Float(std::f64::consts::E))
        }),
    );
    context.register_native_module("std.math", exports);
}

fn ensure_arity(id: &str, args: &[DolangValue], expected: usize) -> Result<(), Error> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(Error::Interpreter(format!(
            "{id}: expected {expected} argument(s), got {}",
            args.len()
        )))
    }
}

fn float_arg(id: &str, args: &[DolangValue], index: usize) -> Result<f64, Error> {
    let Some(value) = args.get(index) else {
        return Err(Error::Interpreter(format!(
            "{id}: expected at least {} argument(s), got {}",
            index + 1,
            args.len()
        )));
    };

    match value {
        DolangValue::Int(n) => Ok(*n as f64),
        DolangValue::Float(f) => Ok(*f),
        other => Err(Error::Interpreter(format!(
            "{id}: expected numeric at argument {index}, got {}",
            other.type_name()
        ))),
    }
}

fn same_numeric_pair<'a>(
    id: &str,
    args: &'a [DolangValue],
) -> Result<(&'a DolangValue, &'a DolangValue), Error> {
    ensure_arity(id, args, 2)?;
    let left = &args[0];
    let right = &args[1];
    match (left, right) {
        (DolangValue::Int(_), DolangValue::Int(_))
        | (DolangValue::Float(_), DolangValue::Float(_)) => Ok((left, right)),
        _ => Err(Error::Interpreter(format!(
            "{id}: arguments must both be Int or both be Float, got {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn same_numeric_triplet<'a>(
    id: &str,
    args: &'a [DolangValue],
) -> Result<(&'a DolangValue, &'a DolangValue, &'a DolangValue), Error> {
    ensure_arity(id, args, 3)?;
    let value = &args[0];
    let min = &args[1];
    let max = &args[2];
    match (value, min, max) {
        (DolangValue::Int(_), DolangValue::Int(_), DolangValue::Int(_))
        | (DolangValue::Float(_), DolangValue::Float(_), DolangValue::Float(_)) => {
            Ok((value, min, max))
        }
        _ => Err(Error::Interpreter(format!(
            "{id}: arguments must all be Int or all be Float, got {}, {}, {}",
            value.type_name(),
            min.type_name(),
            max.type_name()
        ))),
    }
}
