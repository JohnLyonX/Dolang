use crate::ast::FnDeclStmt;
use crate::error::Error;
use crate::runtime::{ProgramState, RuntimeContext};

use super::super::env::{FnEnv, ValueType};
use super::super::value::DolangValue;
use super::{Flow, exec_block};

pub(super) fn handle_fn_decl(stmt: &FnDeclStmt, state: &mut ProgramState) -> Flow {
    if state.fns.contains_key(&stmt.name) {
        return Flow::Err(Error::Interpreter(format!(
            "function '{}' is already defined",
            stmt.name
        )));
    }
    state.fns.insert(stmt.name.clone(), stmt.clone());
    Flow::Normal
}

pub fn call_fn(
    fn_def: &FnDeclStmt,
    args: &[DolangValue],
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<Option<DolangValue>, Error> {
    let min_params = fn_def.params.len();
    let has_variadic = fn_def.variadic_param.is_some();
    if has_variadic && args.len() < min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects at least {} arguments, got {}",
            fn_def.name,
            min_params,
            args.len()
        )));
    }
    if !has_variadic && args.len() != min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects {} arguments, got {}",
            fn_def.name,
            min_params,
            args.len()
        )));
    }

    let mut local_state = ProgramState::new();

    for (param, arg) in fn_def.params.iter().zip(args.iter()) {
        local_state.env.insert(param.clone(), arg.clone());
    }

    if let Some(var_param) = &fn_def.variadic_param {
        let extra_args: Vec<DolangValue> = args[min_params..].to_vec();
        local_state
            .env
            .insert(var_param.clone(), DolangValue::List(extra_args));
    }

    local_state.fns = fns.clone();

    let flow = exec_block(&fn_def.body, &mut local_state, context, w);
    *fns = local_state.fns.clone();
    match flow {
        Flow::Return(val) => {
            validate_return_type(fn_def, val.as_ref())?;
            Ok(val)
        }
        Flow::Normal => Ok(None),
        Flow::Err(err) => Err(err),
        Flow::Exit => Ok(None),
        Flow::Break | Flow::Continue => Err(Error::Interpreter(
            "break/continue used outside of loop".to_string(),
        )),
    }
}

fn validate_return_type(fn_def: &FnDeclStmt, value: Option<&DolangValue>) -> Result<(), Error> {
    let Some(expected_type) = &fn_def.return_type else {
        return Ok(());
    };

    let Some(value) = value else {
        return Err(Error::Interpreter(format!(
            "function '{}' expects return type '{}' but returned nothing",
            fn_def.name, expected_type
        )));
    };

    let actual_type = match value {
        DolangValue::Int(_) => ValueType::Int,
        DolangValue::Float(_) => ValueType::Float,
        DolangValue::Str(_) => ValueType::String,
        DolangValue::Bool(_) => ValueType::Bool,
        DolangValue::List(_) => ValueType::List,
        DolangValue::Map(_) => ValueType::Map,
        DolangValue::Function { .. } => ValueType::Dynamic,
        DolangValue::File { .. } => ValueType::File,
        DolangValue::Json(_) => ValueType::Json,
        DolangValue::Html(_) => ValueType::Dynamic,
        DolangValue::Response { .. } => ValueType::Response,
        DolangValue::ModuleProxy { .. } => ValueType::Dynamic,
        DolangValue::Null => ValueType::Dynamic,
    };

    let (expected, expected_str) = match expected_type.to_lowercase().as_str() {
        "int" | "integer" => (ValueType::Int, "Int"),
        "float" => (ValueType::Float, "Float"),
        "string" => (ValueType::String, "String"),
        "bool" | "boolean" => (ValueType::Bool, "Bool"),
        "json" => (ValueType::Json, "Json"),
        _ => {
            return Err(Error::Interpreter(format!(
                "unknown return type '{}' for function '{}'",
                expected_type, fn_def.name
            )));
        }
    };

    if actual_type != expected {
        let actual_str = match actual_type {
            ValueType::Dynamic => "Dynamic",
            ValueType::Int => "Int",
            ValueType::Float => "Float",
            ValueType::String => "String",
            ValueType::Bool => "Bool",
            ValueType::List => "List",
            ValueType::Map => "Map",
            ValueType::File => "File",
            ValueType::Json => "Json",
            ValueType::Response => "Response",
        };
        return Err(Error::Interpreter(format!(
            "function '{}' expects return type '{}' but got '{}'",
            fn_def.name, expected_str, actual_str
        )));
    }

    Ok(())
}
