use std::collections::HashMap;

use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::types::{
    MovementData,
    MovementRecord,
    SnapshotSummary,
    StoragePayload,
    StorageUnitRecord,
    UserProfile,
};

const MIGRATIONS: [&str; 3] = [
    "CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        login TEXT NOT NULL UNIQUE,
        password_hash TEXT NOT NULL,
        role TEXT NOT NULL DEFAULT 'admin',
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS storage_units (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        label TEXT NOT NULL,
        type TEXT NOT NULL,
        section TEXT,
        capacity INTEGER DEFAULT 0,
        occupancy INTEGER DEFAULT 0,
        metadata TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS movements (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        reference TEXT,
        item_label TEXT,
        from_unit TEXT,
        to_unit TEXT,
        action TEXT NOT NULL,
        note TEXT,
        actor TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
];

pub struct ArchiveDatabase {
    conn: Mutex<Connection>,
}

impl ArchiveDatabase {
    pub fn connect(path: std::path::PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.apply_migrations()?;
        Ok(db)
    }

    fn apply_migrations(&self) -> Result<()> {
        let conn = self.conn.lock();
        for ddl in MIGRATIONS {
            conn.execute_batch(ddl)?;
        }
        drop(conn);
        self.ensure_login_column()?;
        Ok(())
    }

    pub fn ensure_default_admin(&self, login: &str, password: &str) -> Result<()> {
        let normalized = login.trim().to_lowercase();
        if normalized.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock();
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM users WHERE LOWER(login) = ?1 OR LOWER(login) LIKE ?2 LIMIT 1",
                params![normalized, format!("{}@%", normalized)],
                |row| row.get(0),
            )
            .optional()?;
        let hash = hash(password, DEFAULT_COST)?;
        if let Some(id) = existing {
            conn.execute(
                "UPDATE users SET login = ?1, password_hash = ?2 WHERE id = ?3",
                params![normalized, hash, id],
            )?;
        } else {
            conn.execute(
                "INSERT INTO users (name, login, password_hash, role) VALUES (?1, ?2, ?3, 'admin')",
                params!["Administrador", normalized, hash],
            )?;
        }
        Ok(())
    }

    pub fn verify_login(&self, login: &str, password: &str) -> Result<Option<UserProfile>> {
        let normalized = login.trim().to_lowercase();
        if normalized.is_empty() {
            return Ok(None);
        }
        let conn = self.conn.lock();
        let mut search_terms = vec![normalized.clone()];
        if let Some(base) = normalized.split('@').next() {
            if base != normalized {
                search_terms.push(base.to_string());
            }
        }
        if !normalized.contains('@') {
            search_terms.push(format!("{}@%", normalized));
        }
        for term in search_terms {
            let query = if term.contains('%') {
                "SELECT id, name, login, password_hash, role FROM users WHERE LOWER(login) LIKE ?1 LIMIT 1"
            } else {
                "SELECT id, name, login, password_hash, role FROM users WHERE LOWER(login) = ?1 LIMIT 1"
            };
            if let Some(record) = conn
                .query_row(query, params![term], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                })
                .optional()? 
            {
                if verify(password, &record.3)? {
                    return Ok(Some(UserProfile {
                        id: record.0,
                        name: record.1,
                        login: record.2,
                        role: record.4,
                    }));
                }
            }
        }
        Ok(None)
    }

    pub fn list_storage_units(&self) -> Result<Vec<StorageUnitRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, label, type, section, capacity, occupancy, metadata, created_at, updated_at
             FROM storage_units ORDER BY datetime(updated_at) DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let metadata: Option<String> = row.get(6)?;
            let parsed = metadata.and_then(|json| serde_json::from_str::<Value>(&json).ok());
            Ok(StorageUnitRecord {
                id: row.get(0)?,
                label: row.get(1)?,
                r#type: row.get(2)?,
                section: row.get(3)?,
                capacity: row.get(4)?,
                occupancy: row.get(5)?,
                metadata: parsed,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn create_storage_unit(&self, payload: &StoragePayload) -> Result<StorageUnitRecord> {
        let now = Utc::now().to_rfc3339();
        let metadata = payload
            .metadata
            .as_ref()
            .map(|value| serde_json::to_string(value))
            .transpose()?;
        let section = payload
            .section
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO storage_units (label, type, section, capacity, occupancy, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?6)",
            params![
                payload.label.trim(),
                payload.r#type.trim().to_uppercase(),
                section,
                payload.capacity.unwrap_or(0),
                metadata,
                now,
            ],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_storage_unit(id)
    }

    fn get_storage_unit(&self, id: i64) -> Result<StorageUnitRecord> {
        let conn = self.conn.lock();
        Ok(conn.query_row(
            "SELECT id, label, type, section, capacity, occupancy, metadata, created_at, updated_at FROM storage_units WHERE id = ?1",
            params![id],
            |row| {
                let metadata: Option<String> = row.get(6)?;
                let parsed = metadata.and_then(|json| serde_json::from_str::<Value>(&json).ok());
                Ok(StorageUnitRecord {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    r#type: row.get(2)?,
                    section: row.get(3)?,
                    capacity: row.get(4)?,
                    occupancy: row.get(5)?,
                    metadata: parsed,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )?)
    }

    pub fn list_movements(&self, limit: i64) -> Result<Vec<MovementRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, reference, item_label, from_unit, to_unit, action, note, actor, created_at
             FROM movements ORDER BY datetime(created_at) DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(MovementRecord {
                id: row.get(0)?,
                reference: row.get(1)?,
                item_label: row.get(2)?,
                from_unit: row.get(3)?,
                to_unit: row.get(4)?,
                action: row.get(5)?,
                note: row.get(6)?,
                actor: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn record_movement(&self, actor: &str, payload: &MovementData) -> Result<MovementRecord> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO movements (reference, item_label, from_unit, to_unit, action, note, actor, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                payload.reference.as_deref(),
                payload.item_label.as_deref(),
                payload.from_unit.as_deref(),
                payload.to_unit.as_deref(),
                payload.action.trim(),
                payload.note.as_deref(),
                actor,
                now,
            ],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_movement(id)
    }

    fn get_movement(&self, id: i64) -> Result<MovementRecord> {
        let conn = self.conn.lock();
        Ok(conn.query_row(
            "SELECT id, reference, item_label, from_unit, to_unit, action, note, actor, created_at FROM movements WHERE id = ?1",
            params![id],
            |row| {
                Ok(MovementRecord {
                    id: row.get(0)?,
                    reference: row.get(1)?,
                    item_label: row.get(2)?,
                    from_unit: row.get(3)?,
                    to_unit: row.get(4)?,
                    action: row.get(5)?,
                    note: row.get(6)?,
                    actor: row.get(7)?,
                    created_at: row.get(8)?,
                })
            },
        )?)
    }

    pub fn snapshot(&self) -> Result<SnapshotSummary> {
        let conn = self.conn.lock();
        let mut counters_stmt = conn.prepare("SELECT type, COUNT(1) as total FROM storage_units GROUP BY type")?;
        let counters = counters_stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut units_by_type: HashMap<String, i64> = HashMap::from([
            ("PASTA".into(), 0),
            ("ENVELOPE".into(), 0),
            ("GAVETEIRO".into(), 0),
            ("CAIXA".into(), 0),
        ]);
        let mut total_units = 0;
        for row in counters {
            let (kind, total) = row?;
            let key = kind.to_uppercase();
            units_by_type.insert(key, total);
            total_units += total;
        }
        let movements_today: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM movements WHERE DATE(created_at) = DATE('now')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let last_movement = conn
            .query_row(
                "SELECT id, reference, item_label, from_unit, to_unit, action, note, actor, created_at
                 FROM movements ORDER BY datetime(created_at) DESC LIMIT 1",
                [],
                |row| {
                    Ok(MovementRecord {
                        id: row.get(0)?,
                        reference: row.get(1)?,
                        item_label: row.get(2)?,
                        from_unit: row.get(3)?,
                        to_unit: row.get(4)?,
                        action: row.get(5)?,
                        note: row.get(6)?,
                        actor: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                },
            )
            .optional()?;
        Ok(SnapshotSummary {
            total_units,
            units_by_type,
            movements_today,
            last_movement,
        })
    }

    fn ensure_login_column(&self) -> Result<()> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("PRAGMA table_info(users)")?;
        let rows = stmt.query_map([], |row| Ok(row.get::<_, String>(1)?))?;
        let mut has_login = false;
        let mut has_email = false;
        for row in rows {
            let name = row?;
            if name == "login" {
                has_login = true;
            }
            if name == "email" {
                has_email = true;
            }
        }
        if !has_login && has_email {
            conn.execute("ALTER TABLE users RENAME COLUMN email TO login", [])?;
        }
        Ok(())
    }
}
