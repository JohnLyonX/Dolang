use postgres::{Client, NoTls};

use crate::error::Error;

use super::{RefreshTokenRecord, RefreshTokenStore};

pub struct RefreshTokenStorePostgres {
    client: Client,
    table: String,
}

impl RefreshTokenStorePostgres {
    pub fn connect(url: &str, table: impl Into<String>) -> Result<Self, Error> {
        let client = Client::connect(url, NoTls).map_err(|err| {
            Error::Interpreter(format!("postgres auth refresh token store error: {err}"))
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
                    token_id TEXT PRIMARY KEY,
                    subject TEXT NOT NULL,
                    expires_at BIGINT NOT NULL,
                    revoked_at BIGINT
                )",
                self.table
            ))
            .map_err(postgres_error)?;
        Ok(())
    }
}

impl RefreshTokenStore for RefreshTokenStorePostgres {
    fn get(&mut self, token_id: &str) -> Result<Option<RefreshTokenRecord>, Error> {
        let row = self
            .client
            .query_opt(
                &format!(
                    "SELECT token_id, subject, expires_at, revoked_at FROM {} WHERE token_id = $1",
                    self.table
                ),
                &[&token_id],
            )
            .map_err(postgres_error)?;

        Ok(row.map(|row| RefreshTokenRecord {
            token_id: row.get(0),
            subject: row.get(1),
            expires_at: row.get(2),
            revoked_at: row.get(3),
        }))
    }

    fn create(&mut self, record: RefreshTokenRecord) -> Result<(), Error> {
        self.client
            .execute(
                &format!(
                    "INSERT INTO {} (token_id, subject, expires_at, revoked_at) VALUES ($1, $2, $3, $4)",
                    self.table
                ),
                &[&record.token_id, &record.subject, &record.expires_at, &record.revoked_at],
            )
            .map_err(postgres_error)?;
        Ok(())
    }

    fn revoke(&mut self, token_id: &str) -> Result<(), Error> {
        self.client
            .execute(
                &format!(
                    "UPDATE {} SET revoked_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE token_id = $1 AND revoked_at IS NULL",
                    self.table
                ),
                &[&token_id],
            )
            .map_err(postgres_error)?;
        Ok(())
    }

    fn revoke_subject(&mut self, subject: &str) -> Result<usize, Error> {
        let updated = self
            .client
            .execute(
                &format!(
                    "UPDATE {} SET revoked_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE subject = $1 AND revoked_at IS NULL",
                    self.table
                ),
                &[&subject],
            )
            .map_err(postgres_error)?;
        Ok(updated as usize)
    }
}

fn postgres_error(err: postgres::Error) -> Error {
    Error::Interpreter(format!("postgres auth refresh token store error: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{RefreshTokenRecord, RefreshTokenStore, RefreshTokenStorePostgres};

    #[test]
    fn postgres_refresh_store_round_trips_and_revokes_when_url_present() {
        let Some(url) = std::env::var("DOLANG_TEST_POSTGRES_URL").ok() else {
            return;
        };

        let table = format!("auth_refresh_tokens_{}", std::process::id());
        let mut store = RefreshTokenStorePostgres::connect(&url, &table).unwrap();
        store.ensure_schema().unwrap();

        let record = RefreshTokenRecord {
            token_id: "rjti_pg".to_string(),
            subject: "user_1".to_string(),
            expires_at: 1_800_000_000,
            revoked_at: None,
        };

        store.create(record.clone()).unwrap();
        assert_eq!(store.get("rjti_pg").unwrap(), Some(record));

        store.revoke("rjti_pg").unwrap();
        assert!(
            store
                .get("rjti_pg")
                .unwrap()
                .expect("record should exist")
                .revoked_at
                .is_some()
        );
        store.revoke_subject("user_1").unwrap();
    }
}
