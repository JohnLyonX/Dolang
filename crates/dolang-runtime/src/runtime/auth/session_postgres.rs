use indexmap::IndexMap;
use postgres::{Client, NoTls};

use crate::error::Error;
use crate::interpreter::{DolangValue, value::value_to_json};

use super::{SessionRecord, SessionStore};

pub struct PostgresSessionStore {
    client: Client,
    table: String,
}

impl PostgresSessionStore {
    pub fn connect(url: &str, table: impl Into<String>) -> Result<Self, Error> {
        let client = Client::connect(url, NoTls).map_err(|err| {
            Error::Interpreter(format!("postgres auth session store error: {err}"))
        })?;
        Ok(Self {
            client,
            table: table.into(),
        })
    }

    pub fn ensure_schema(&mut self) -> Result<(), Error> {
        self.client
            .batch_execute(&format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    session_id TEXT PRIMARY KEY,
                    subject TEXT NOT NULL,
                    roles_json TEXT NOT NULL,
                    permissions_json TEXT NOT NULL,
                    claims_json TEXT NOT NULL,
                    expires_at BIGINT NOT NULL,
                    idle_timeout_at BIGINT
                )",
                self.table
            ))
            .map_err(postgres_error)?;
        Ok(())
    }
}

impl SessionStore for PostgresSessionStore {
    fn get(&mut self, session_id: &str) -> Result<Option<SessionRecord>, Error> {
        let row = self
            .client
            .query_opt(
                &format!(
                    "SELECT session_id, subject, roles_json, permissions_json, claims_json, expires_at, idle_timeout_at FROM {} WHERE session_id = $1",
                    self.table
                ),
                &[&session_id],
            )
            .map_err(postgres_error)?;

        Ok(row.map(|row| SessionRecord {
            session_id: row.get(0),
            subject: row.get(1),
            roles: decode_string_list(row.get::<_, String>(2).as_str()),
            permissions: decode_string_list(row.get::<_, String>(3).as_str()),
            claims: decode_claims(row.get::<_, String>(4).as_str()),
            expires_at: row.get(5),
            idle_timeout_at: row.get(6),
        }))
    }

    fn create(&mut self, session: SessionRecord) -> Result<(), Error> {
        self.client
            .execute(
                &format!(
                    "INSERT INTO {} (session_id, subject, roles_json, permissions_json, claims_json, expires_at, idle_timeout_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    self.table
                ),
                &[
                    &session.session_id,
                    &session.subject,
                    &encode_string_list(&session.roles),
                    &encode_string_list(&session.permissions),
                    &encode_claims(&session.claims),
                    &session.expires_at,
                    &session.idle_timeout_at,
                ],
            )
            .map_err(postgres_error)?;
        Ok(())
    }

    fn update(&mut self, session: SessionRecord) -> Result<(), Error> {
        self.client
            .execute(
                &format!(
                    "UPDATE {} SET subject = $2, roles_json = $3, permissions_json = $4, claims_json = $5, expires_at = $6, idle_timeout_at = $7 WHERE session_id = $1",
                    self.table
                ),
                &[
                    &session.session_id,
                    &session.subject,
                    &encode_string_list(&session.roles),
                    &encode_string_list(&session.permissions),
                    &encode_claims(&session.claims),
                    &session.expires_at,
                    &session.idle_timeout_at,
                ],
            )
            .map_err(postgres_error)?;
        Ok(())
    }

    fn rotate(&mut self, old_session_id: &str, session: SessionRecord) -> Result<(), Error> {
        self.delete(old_session_id)?;
        self.create(session)
    }

    fn delete(&mut self, session_id: &str) -> Result<(), Error> {
        self.client
            .execute(
                &format!("DELETE FROM {} WHERE session_id = $1", self.table),
                &[&session_id],
            )
            .map_err(postgres_error)?;
        Ok(())
    }
}

fn postgres_error(err: postgres::Error) -> Error {
    Error::Interpreter(format!("postgres auth session store error: {err}"))
}

fn encode_string_list(values: &[String]) -> String {
    serde_json::to_string(values).expect("string list should serialize")
}

fn decode_string_list(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn encode_claims(claims: &IndexMap<String, DolangValue>) -> String {
    let value = value_to_json(&DolangValue::Map(claims.clone()));
    serde_json::to_string(&value).expect("claims should serialize")
}

fn decode_claims(raw: &str) -> IndexMap<String, DolangValue> {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::Object(map)) => map
            .into_iter()
            .map(|(key, value)| (key, json_value_to_dolang(value)))
            .collect(),
        _ => IndexMap::new(),
    }
}

fn json_value_to_dolang(value: serde_json::Value) -> DolangValue {
    match value {
        serde_json::Value::Null => DolangValue::Null,
        serde_json::Value::Bool(b) => DolangValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(int) = n.as_i64() {
                DolangValue::Int(int)
            } else if let Some(float) = n.as_f64() {
                DolangValue::Float(float)
            } else {
                DolangValue::Null
            }
        }
        serde_json::Value::String(s) => DolangValue::Str(s),
        serde_json::Value::Array(values) => {
            DolangValue::List(values.into_iter().map(json_value_to_dolang).collect())
        }
        serde_json::Value::Object(entries) => DolangValue::Map(
            entries
                .into_iter()
                .map(|(key, value)| (key, json_value_to_dolang(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::{PostgresSessionStore, SessionRecord, SessionStore};

    fn sample_session(session_id: &str) -> SessionRecord {
        SessionRecord {
            session_id: session_id.to_string(),
            subject: "user_1".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec!["post:create".to_string()],
            claims: IndexMap::new(),
            expires_at: 1_800_000_000,
            idle_timeout_at: Some(1_800_000_600),
        }
    }

    #[test]
    fn postgres_store_round_trips_session_record_when_url_present() {
        let Some(url) = std::env::var("DOLANG_TEST_POSTGRES_URL").ok() else {
            return;
        };

        let table = format!("auth_sessions_plan_{}", std::process::id());
        let mut store = PostgresSessionStore::connect(&url, &table).unwrap();
        store.ensure_schema().unwrap();

        let session = sample_session("sess_pg");
        store.create(session.clone()).unwrap();

        assert_eq!(store.get("sess_pg").unwrap(), Some(session));
        store.delete("sess_pg").unwrap();
    }
}
