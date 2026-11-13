//! Accès SQLite et migrations versionnées (PRAGMA user_version).
//! Aucun token n'est stocké ici : ils vont dans le trousseau système.

use crate::error::{LauncherError, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

const MIGRATIONS: &[(i32, &str)] = &[(1, include_str!("migrations/001_init.sql"))];

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::from_connection(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Exécute une opération avec la connexion verrouillée.
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| LauncherError::Invalid("verrou de la base de données corrompu".into()))?;
        f(&guard)
    }
}

fn migrate(conn: &Connection) -> Result<()> {
    let current: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for (version, sql) in MIGRATIONS {
        if *version > current {
            conn.execute_batch(sql)?;
            conn.pragma_update(None, "user_version", version)?;
            tracing::info!("migration de base de données appliquée : v{version}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_applied_once() {
        let db = Database::open_in_memory().unwrap();
        db.with(|c| {
            let version: i32 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
            assert_eq!(version, 1);
            // Une seconde migration sur la même base ne doit rien casser.
            migrate(c)?;
            Ok(())
        })
        .unwrap();
    }
}
