// SQL Connection 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::{RuntimeContext, intrinsics::ids};

pub fn call(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    let connection = match receiver {
        DolangValue::Connection { .. } => receiver,
        _ => return Err(Error::Interpreter("expected Connection".to_string())),
    };

    match method {
        "query" => call_query(connection, args, context),
        "execute" => call_execute(connection, args, context),
        "close" => call_close(connection, args, context),
        _ => Err(Error::Interpreter(format!(
            "Connection has no method '{}'",
            method
        ))),
    }
}

fn call_query(
    connection: &DolangValue,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    let (sql, params) = sql_and_params("Connection.query", args)?;
    context.call_intrinsic(
        ids::SQL_QUERY,
        &[
            connection.clone(),
            DolangValue::Str(sql),
            DolangValue::List(params),
        ],
    )
}

fn call_execute(
    connection: &DolangValue,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    let (sql, params) = sql_and_params("Connection.execute", args)?;
    context.call_intrinsic(
        ids::SQL_EXECUTE,
        &[
            connection.clone(),
            DolangValue::Str(sql),
            DolangValue::List(params),
        ],
    )
}

fn call_close(
    connection: &DolangValue,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    if !args.is_empty() {
        return Err(Error::Interpreter(format!(
            "Connection.close() requires 0 arguments, got {}",
            args.len()
        )));
    }

    context.call_intrinsic(ids::SQL_CLOSE, &[connection.clone()])
}

fn sql_and_params(method: &str, args: &[DolangValue]) -> Result<(String, Vec<DolangValue>), Error> {
    if args.len() != 2 {
        return Err(Error::Interpreter(format!(
            "{}() requires 2 arguments (sql, params), got {}",
            method,
            args.len()
        )));
    }

    let sql = match &args[0] {
        DolangValue::Str(sql) => sql.clone(),
        other => {
            return Err(Error::Interpreter(format!(
                "{}() expects String at argument 0, got {}",
                method,
                other.type_name()
            )));
        }
    };

    let params = match &args[1] {
        DolangValue::List(params) => params.clone(),
        other => {
            return Err(Error::Interpreter(format!(
                "{}() expects List at argument 1, got {}",
                method,
                other.type_name()
            )));
        }
    };

    Ok((sql, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{RuntimeContext, RuntimeMode};
    use std::path::PathBuf;

    fn test_context() -> RuntimeContext {
        RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."))
    }

    fn test_connection() -> (RuntimeContext, DolangValue) {
        let context = test_context();
        let conn = context
            .call_intrinsic(
                ids::SQL_SQLITE_CONNECT,
                &[DolangValue::Str(":memory:".to_string())],
            )
            .expect("sqlite connection should open");
        (context, conn)
    }

    #[test]
    fn query_rejects_wrong_argument_shapes() {
        let (context, conn) = test_connection();

        let error = call(
            &conn,
            "query",
            &[DolangValue::Str("SELECT 1".to_string())],
            &context,
        )
        .expect_err("missing params list should fail");
        assert!(
            error
                .to_string()
                .contains("Connection.query() requires 2 arguments")
        );

        let error = call(
            &conn,
            "query",
            &[
                DolangValue::Int(1),
                DolangValue::List(vec![DolangValue::Int(1)]),
            ],
            &context,
        )
        .expect_err("non-string sql should fail");
        assert!(
            error
                .to_string()
                .contains("Connection.query() expects String at argument 0")
        );

        let error = call(
            &conn,
            "execute",
            &[
                DolangValue::Str("SELECT 1".to_string()),
                DolangValue::Str("not a list".to_string()),
            ],
            &context,
        )
        .expect_err("non-list params should fail");
        assert!(
            error
                .to_string()
                .contains("Connection.execute() expects List at argument 1")
        );
    }

    #[test]
    fn close_rejects_extra_arguments() {
        let (context, conn) = test_connection();

        let error = call(
            &conn,
            "close",
            &[DolangValue::Str("unexpected".to_string())],
            &context,
        )
        .expect_err("close should reject arguments");

        assert!(
            error
                .to_string()
                .contains("Connection.close() requires 0 arguments")
        );
    }
}
