//! Gestion des instances (création, liste, suppression).
//! La création de dossiers de jeu et la duplication arriveront en v0.1 suivante.

use crate::error::{LauncherError, Result};
use crate::modules::database::Database;
use crate::modules::security;
use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

pub const DEFAULT_RAM_MB: i64 = 4096;
const COLUMNS: &str = "id, name, mc_version, loader, ram_mb, created_at";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: i64,
    pub name: String,
    pub mc_version: String,
    pub loader: String,
    pub ram_mb: i64,
    pub created_at: String,
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<Instance> {
    Ok(Instance {
        id: row.get(0)?,
        name: row.get(1)?,
        mc_version: row.get(2)?,
        loader: row.get(3)?,
        ram_mb: row.get(4)?,
        created_at: row.get(5)?,
    })
}

pub fn list(db: &Database) -> Result<Vec<Instance>> {
    db.with(|c| {
        let mut stmt = c.prepare(&format!("SELECT {COLUMNS} FROM instances ORDER BY name COLLATE NOCASE"))?;
        let rows = stmt.query_map([], from_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

pub fn create(db: &Database, name: &str, mc_version: &str) -> Result<Instance> {
    security::validate_instance_name(name)?;
    security::validate_mc_version(mc_version)?;
    let name = name.trim();

    db.with(|c| {
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM instances WHERE name = ?1)",
            [name],
            |r| r.get(0),
        )?;
        if exists {
            return Err(LauncherError::Invalid("Une instance porte déjà ce nom.".into()));
        }
        c.execute(
            "INSERT INTO instances (name, mc_version, ram_mb) VALUES (?1, ?2, ?3)",
            params![name, mc_version, DEFAULT_RAM_MB],
        )?;
        let id = c.last_insert_rowid();
        Ok(c.query_row(&format!("SELECT {COLUMNS} FROM instances WHERE id = ?1"), [id], from_row)?)
    })
}

pub fn get(db: &Database, id: i64) -> Result<Instance> {
    db.with(|c| {
        let found = c
            .query_row(&format!("SELECT {COLUMNS} FROM instances WHERE id = ?1"), [id], from_row)
            .optional()?;
        found.ok_or_else(|| LauncherError::Invalid("Instance introuvable.".into()))
    })
}

pub fn delete(db: &Database, id: i64) -> Result<()> {
    db.with(|c| {
        let removed = c.execute("DELETE FROM instances WHERE id = ?1", [id])?;
        if removed == 0 {
            return Err(LauncherError::Invalid("Instance introuvable.".into()));
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list_delete_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        let created = create(&db, "  Survie PvP  ", "1.21.1").unwrap();
        assert_eq!(created.name, "Survie PvP");
        assert_eq!(created.ram_mb, DEFAULT_RAM_MB);
        assert_eq!(list(&db).unwrap().len(), 1);

        assert!(create(&db, "Survie PvP", "1.21.1").is_err(), "doublon refusé");

        delete(&db, created.id).unwrap();
        assert!(list(&db).unwrap().is_empty());
        assert!(delete(&db, created.id).is_err(), "suppression d'un id absent refusée");
    }

    #[test]
    fn rejects_path_traversal_in_name() {
        let db = Database::open_in_memory().unwrap();
        assert!(create(&db, "../evil", "1.21.1").is_err());
        assert!(list(&db).unwrap().is_empty());
    }
}
