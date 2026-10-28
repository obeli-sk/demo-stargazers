//! Schema migrations driven by refinery's async runner.
//! Each migration and its history row are applied in one non-interactive transaction.

use crate::split::split_statements;
use crate::turso::request::Stmt;
use crate::turso::{TursoClient, TursoValue};
use refinery::Migration;
use refinery_core::traits::r#async::{AsyncMigrate, AsyncQuery, AsyncTransaction};
use std::io;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

mod embedded {
    refinery::embed_migrations!("../migrations");
}

struct Connection(TursoClient);

impl AsyncTransaction for Connection {
    type Error = io::Error;

    async fn execute<'a, T: Iterator<Item = &'a str> + Send + 'a>(
        &'a mut self,
        queries: T,
    ) -> Result<usize, Self::Error> {
        let mut statements = Vec::new();
        let mut count = 0;
        for query in queries {
            count += 1;
            let split = split_statements(query).map_err(io::Error::other)?;
            statements.extend(split.into_iter().map(Stmt::new));
        }
        self.0
            .transaction(statements)
            .await
            .map_err(io::Error::other)?;
        Ok(count)
    }
}

impl AsyncQuery<Vec<Migration>> for Connection {
    async fn query<'a>(&'a mut self, query: &'a str) -> Result<Vec<Migration>, Self::Error> {
        let results = self
            .0
            .transaction(vec![Stmt::new(query)])
            .await
            .map_err(io::Error::other)?;
        let rows = results.into_iter().flat_map(|result| result.rows);
        rows.map(|row| parse_applied(row.0).map_err(io::Error::other))
            .collect()
    }
}

impl AsyncMigrate for Connection {}

// Parses a `version, name, applied_on, checksum` row of the history table.
fn parse_applied(row: Vec<TursoValue>) -> Result<Migration, String> {
    let Ok([version, name, applied_on, checksum]) = <[TursoValue; 4]>::try_from(row) else {
        return Err("unexpected column count, expected 4".to_string());
    };
    let text = |value: TursoValue| match value {
        TursoValue::Text { value } | TursoValue::Integer { value } => Ok(value),
        TursoValue::Null => Err("unexpected null in the history table".to_string()),
    };
    let parse_err = |err: &dyn std::fmt::Display| format!("invalid history row: {err}");
    Ok(Migration::applied(
        text(version)?.parse().map_err(|err| parse_err(&err))?,
        text(name)?,
        OffsetDateTime::parse(&text(applied_on)?, &Rfc3339).map_err(|err| parse_err(&err))?,
        text(checksum)?.parse().map_err(|err| parse_err(&err))?,
    ))
}

/// The version of the latest embedded migration.
pub fn latest_version() -> Option<u32> {
    let runner = embedded::migrations::runner();
    // Versions are parsed from digits, so they are never negative.
    let versions = runner.get_migrations().iter();
    versions.map(|m| m.version().cast_unsigned()).max()
}

/// Applies pending migrations and returns their versions.
pub async fn migrate(latest_version: u32) -> Result<Vec<u32>, anyhow::Error> {
    let latest_known = self::latest_version();
    if latest_known != Some(latest_version) {
        anyhow::bail!("latest migration is {latest_known:?}, expected {latest_version}");
    }
    let report = embedded::migrations::runner()
        .run_async(&mut Connection(TursoClient::new()?))
        .await?;
    Ok(report
        .applied_migrations()
        .iter()
        .map(|migration| migration.version().cast_unsigned())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::parse_applied;
    use crate::turso::TursoValue;
    use refinery::Migration;

    #[test]
    fn checksum_should_match_the_js_port() {
        // Same vector as the JS `checksum matches refinery` test.
        let migration = Migration::unapplied("V7__vector.sql", "SELECT 'ž';").unwrap();
        assert_eq!(16225368461553448942, migration.checksum());
    }

    #[test]
    fn parse_applied_should_read_a_history_row() {
        let text = |value: &str| TursoValue::Text {
            value: value.to_string(),
        };
        let migration = parse_applied(vec![
            TursoValue::Integer {
                value: "2".to_string(),
            },
            text("users_updated_at_index"),
            text("2026-10-28T10:00:00.123456789Z"),
            text("15242138593942272401"),
        ])
        .unwrap();
        assert_eq!(2, migration.version());
        assert_eq!("users_updated_at_index", migration.name());
        assert_eq!(15242138593942272401, migration.checksum());
    }
}
