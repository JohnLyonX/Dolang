use crate::ast::{BinaryExpr, Expr, IndexAccess, UnaryExpr};
use crate::diagnostics::codes;
use crate::error::Error;
use crate::runtime::RuntimeContext;
use crate::token::Type;

use super::super::env::{Env, FnEnv};
use super::super::value::DolangValue;
use super::eval_expr;

pub fn eval_index_access(
    e: &Expr,
    idx: &IndexAccess,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let obj_val = eval_expr(&idx.object, env, fns, context, w, false)?;
    let idx_val = eval_expr(&idx.index, env, fns, context, w, false)?;

    match obj_val {
        DolangValue::List(list) => {
            let idx = match idx_val {
                DolangValue::Int(i) => i as usize,
                DolangValue::Float(f) if f.fract() == 0.0 => f as usize,
                _ => {
                    return Err(super::runtime_error(
                        e,
                        codes::RUNTIME_INVALID_EXPRESSION,
                        "invalid list index: must be a non-negative integer",
                    ));
                }
            };
            if idx >= list.len() {
                return Err(super::runtime_error(
                    e,
                    codes::RUNTIME_INVALID_EXPRESSION,
                    format!("list index {idx} out of bounds (length: {})", list.len()),
                ));
            }
            Ok(list[idx].clone())
        }
        DolangValue::Map(map) | DolangValue::Json(map) => {
            let key = idx_val.to_string();
            match map.get(&key) {
                Some(val) => Ok(val.clone()),
                None => Err(super::runtime_error(
                    e,
                    codes::RUNTIME_INVALID_EXPRESSION,
                    format!("map key '{key}' not found"),
                )),
            }
        }
        _ => Err(super::runtime_error(
            e,
            codes::RUNTIME_INVALID_EXPRESSION,
            "index access is only supported on List, Map, and Json values",
        )),
    }
}

pub fn eval_unary(
    e: &Expr,
    u: &UnaryExpr,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
    as_identifier: bool,
) -> Result<DolangValue, Error> {
    let right = eval_expr(&u.right, env, fns, context, w, as_identifier)?;
    match u.op {
        Type::Not => Ok(DolangValue::Bool(!right.is_truthy())),
        Type::Minus => match right {
            DolangValue::Int(n) => Ok(DolangValue::Int(-n)),
            DolangValue::Float(f) => Ok(DolangValue::Float(-f)),
            _ => Err(super::runtime_error(
                e,
                codes::RUNTIME_INVALID_EXPRESSION,
                "unary '-' only supports Int and Float values",
            )),
        },
        _ => Err(super::runtime_error(
            e,
            codes::RUNTIME_INVALID_EXPRESSION,
            "unsupported unary operator",
        )),
    }
}

pub fn eval_binary(
    e: &Expr,
    b: &BinaryExpr,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
    as_identifier: bool,
) -> Result<DolangValue, Error> {
    let left = eval_expr(&b.left, env, fns, context, w, as_identifier)?;
    let right = eval_expr(&b.right, env, fns, context, w, as_identifier)?;

    match b.op {
        Type::Plus => match (&left, &right) {
            (DolangValue::Int(l), DolangValue::Int(r)) => Ok(DolangValue::Int(l + r)),
            (DolangValue::Int(l), DolangValue::Float(r)) => Ok(DolangValue::Float(*l as f64 + r)),
            (DolangValue::Float(l), DolangValue::Int(r)) => Ok(DolangValue::Float(l + *r as f64)),
            (DolangValue::Float(l), DolangValue::Float(r)) => Ok(DolangValue::Float(l + r)),
            (DolangValue::Str(l), DolangValue::Str(r)) => Ok(DolangValue::Str(l.clone() + r)),
            (DolangValue::Str(l), DolangValue::Int(r)) => Ok(DolangValue::Str(format!("{l}{r}"))),
            (DolangValue::Int(l), DolangValue::Str(r)) => Ok(DolangValue::Str(format!("{l}{r}"))),
            (DolangValue::Str(l), DolangValue::Float(r)) => Ok(DolangValue::Str(format!("{l}{r}"))),
            (DolangValue::Float(l), DolangValue::Str(r)) => Ok(DolangValue::Str(format!("{l}{r}"))),
            (DolangValue::List(l), DolangValue::List(r)) => {
                let mut new_list = l.clone();
                new_list.extend(r.clone());
                Ok(DolangValue::List(new_list))
            }
            _ => Err(super::runtime_error(
                e,
                codes::RUNTIME_INVALID_EXPRESSION,
                format!("operator '+' does not support {} and {}", left, right),
            )),
        },
        Type::Minus => match (&left, &right) {
            (DolangValue::Int(l), DolangValue::Int(r)) => Ok(DolangValue::Int(l - r)),
            (DolangValue::Int(l), DolangValue::Float(r)) => Ok(DolangValue::Float(*l as f64 - r)),
            (DolangValue::Float(l), DolangValue::Int(r)) => Ok(DolangValue::Float(l - *r as f64)),
            (DolangValue::Float(l), DolangValue::Float(r)) => Ok(DolangValue::Float(l - r)),
            _ => Err(super::runtime_error(
                e,
                codes::RUNTIME_INVALID_EXPRESSION,
                format!("operator '-' does not support {} and {}", left, right),
            )),
        },
        Type::Mul => match (&left, &right) {
            (DolangValue::Int(l), DolangValue::Int(r)) => Ok(DolangValue::Int(l * r)),
            (DolangValue::Int(l), DolangValue::Float(r)) => Ok(DolangValue::Float(*l as f64 * r)),
            (DolangValue::Float(l), DolangValue::Int(r)) => Ok(DolangValue::Float(l * *r as f64)),
            (DolangValue::Float(l), DolangValue::Float(r)) => Ok(DolangValue::Float(l * r)),
            (DolangValue::Str(s), DolangValue::Int(n)) => {
                if *n >= 0 {
                    Ok(DolangValue::Str(s.repeat(*n as usize)))
                } else {
                    Err(super::runtime_error(
                        e,
                        codes::RUNTIME_INVALID_EXPRESSION,
                        "string repetition requires a non-negative integer",
                    ))
                }
            }
            (DolangValue::List(l), DolangValue::Int(n)) => {
                if *n >= 0 {
                    let mut new_list = Vec::new();
                    for _ in 0..*n as usize {
                        new_list.extend(l.clone());
                    }
                    Ok(DolangValue::List(new_list))
                } else {
                    Err(super::runtime_error(
                        e,
                        codes::RUNTIME_INVALID_EXPRESSION,
                        "list repetition requires a non-negative integer",
                    ))
                }
            }
            _ => Err(super::runtime_error(
                e,
                codes::RUNTIME_INVALID_EXPRESSION,
                format!("operator '*' does not support {} and {}", left, right),
            )),
        },
        Type::Div => numeric_division(e, &left, &right),
        Type::Mod => numeric_modulo(e, &left, &right),
        Type::Eq => Ok(DolangValue::Bool(left == right)),
        Type::Ne => Ok(DolangValue::Bool(left != right)),
        Type::Gt => compare_values(e, &left, &right, ">", |l, r| l > r, |l, r| l > r),
        Type::Lt => compare_values(e, &left, &right, "<", |l, r| l < r, |l, r| l < r),
        Type::Gte => compare_values(e, &left, &right, ">=", |l, r| l >= r, |l, r| l >= r),
        Type::Lte => compare_values(e, &left, &right, "<=", |l, r| l <= r, |l, r| l <= r),
        Type::And => Ok(DolangValue::Bool(left.is_truthy() && right.is_truthy())),
        Type::Or => Ok(DolangValue::Bool(left.is_truthy() || right.is_truthy())),
        _ => Err(super::runtime_error(
            e,
            codes::RUNTIME_INVALID_EXPRESSION,
            "unsupported binary operator",
        )),
    }
}

fn numeric_division(
    e: &Expr,
    left: &DolangValue,
    right: &DolangValue,
) -> Result<DolangValue, Error> {
    match (left, right) {
        (DolangValue::Int(l), DolangValue::Int(r)) => {
            if *r != 0 {
                if *l % *r == 0 {
                    Ok(DolangValue::Int(l / r))
                } else {
                    Ok(DolangValue::Float(*l as f64 / *r as f64))
                }
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_DIVISION_BY_ZERO,
                    "division by zero",
                ))
            }
        }
        (DolangValue::Int(l), DolangValue::Float(r)) => {
            if *r != 0.0 {
                Ok(DolangValue::Float(*l as f64 / r))
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_DIVISION_BY_ZERO,
                    "division by zero",
                ))
            }
        }
        (DolangValue::Float(l), DolangValue::Int(r)) => {
            if *r != 0 {
                Ok(DolangValue::Float(l / *r as f64))
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_DIVISION_BY_ZERO,
                    "division by zero",
                ))
            }
        }
        (DolangValue::Float(l), DolangValue::Float(r)) => {
            if *r != 0.0 {
                Ok(DolangValue::Float(l / r))
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_DIVISION_BY_ZERO,
                    "division by zero",
                ))
            }
        }
        _ => Err(super::runtime_error(
            e,
            codes::RUNTIME_INVALID_EXPRESSION,
            format!("operator '/' does not support {} and {}", left, right),
        )),
    }
}

fn numeric_modulo(e: &Expr, left: &DolangValue, right: &DolangValue) -> Result<DolangValue, Error> {
    match (left, right) {
        (DolangValue::Int(l), DolangValue::Int(r)) => {
            if *r != 0 {
                Ok(DolangValue::Int(l % r))
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_MODULO_BY_ZERO,
                    "modulo by zero",
                ))
            }
        }
        (DolangValue::Int(l), DolangValue::Float(r)) => {
            if *r != 0.0 {
                Ok(DolangValue::Float(*l as f64 % r))
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_MODULO_BY_ZERO,
                    "modulo by zero",
                ))
            }
        }
        (DolangValue::Float(l), DolangValue::Int(r)) => {
            if *r != 0 {
                Ok(DolangValue::Float(l % *r as f64))
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_MODULO_BY_ZERO,
                    "modulo by zero",
                ))
            }
        }
        (DolangValue::Float(l), DolangValue::Float(r)) => {
            if *r != 0.0 {
                Ok(DolangValue::Float(l % r))
            } else {
                Err(super::runtime_error(
                    e,
                    codes::RUNTIME_MODULO_BY_ZERO,
                    "modulo by zero",
                ))
            }
        }
        _ => Err(super::runtime_error(
            e,
            codes::RUNTIME_INVALID_EXPRESSION,
            format!("operator '%' does not support {} and {}", left, right),
        )),
    }
}

fn compare_values<FNum, FStr>(
    e: &Expr,
    left: &DolangValue,
    right: &DolangValue,
    op: &str,
    cmp_num: FNum,
    cmp_str: FStr,
) -> Result<DolangValue, Error>
where
    FNum: Fn(f64, f64) -> bool,
    FStr: Fn(&str, &str) -> bool,
{
    match (left, right) {
        (DolangValue::Int(l), DolangValue::Int(r)) => {
            Ok(DolangValue::Bool(cmp_num(*l as f64, *r as f64)))
        }
        (DolangValue::Float(l), DolangValue::Float(r)) => Ok(DolangValue::Bool(cmp_num(*l, *r))),
        (DolangValue::Int(l), DolangValue::Float(r)) => {
            Ok(DolangValue::Bool(cmp_num(*l as f64, *r)))
        }
        (DolangValue::Float(l), DolangValue::Int(r)) => {
            Ok(DolangValue::Bool(cmp_num(*l, *r as f64)))
        }
        (DolangValue::Str(l), DolangValue::Str(r)) => Ok(DolangValue::Bool(cmp_str(l, r))),
        _ => Err(super::runtime_error(
            e,
            codes::RUNTIME_INVALID_EXPRESSION,
            format!("operator '{op}' does not support {} and {}", left, right),
        )),
    }
}
