use std::collections::HashMap;

use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde_json::Value;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

use crate::types::{
    MovementData, MovementRecord, SnapshotSummary, StoragePayload, StorageUnitRecord, UserProfile,
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
    pool: SqlitePool,
}

impl ArchiveDatabase {
    pub async fn connect(path: std::path::PathBuf) -> Result<Self> {
        // Create the database file if it doesn't exist
        if !path.exists() {
            std::fs::File::create(&path)?;
        }

        let db_url = format!("sqlite://{}", path.to_string_lossy());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let db = Self { pool };
        db.apply_migrations().await?;
        Ok(db)
    }

    async fn apply_migrations(&self) -> Result<()> {
        for ddl in MIGRATIONS {
            sqlx::query(ddl).execute(&self.pool).await?;
        }
        self.ensure_login_column().await?;
        Ok(())
    }

    pub async fn ensure_default_admin(&self, login: &str, password: &str) -> Result<()> {
        let normalized = login.trim().to_lowercase();
        if normalized.is_empty() {
            return Ok(());
        }

        println!("Verificando admin padrão: {}", normalized);
        let existing = sqlx::query(
            "SELECT id FROM users WHERE LOWER(login) = ? OR LOWER(login) LIKE ? LIMIT 1",
        )
        .bind(&normalized)
        .bind(format!("{}@%", normalized))
        .fetch_optional(&self.pool)
        .await?;

        let hash = hash(password, DEFAULT_COST)?;

        if let Some(row) = existing {
            let id: i64 = row.get(0);
            sqlx::query("UPDATE users SET login = ?, password_hash = ? WHERE id = ?")
                .bind(&normalized)
                .bind(hash)
                .bind(id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query(
                "INSERT INTO users (name, login, password_hash, role) VALUES (?, ?, ?, 'admin')",
            )
            .bind("Administrador")
            .bind(&normalized)
            .bind(hash)
            .execute(&self.pool)
            .await?;
        }

        // If the configured admin is NOT "admin", remove the old default "admin" user if it exists
        if normalized != "admin" {
            sqlx::query("DELETE FROM users WHERE login = 'admin'")
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn verify_login(&self, login: &str, password: &str) -> Result<Option<UserProfile>> {
        let normalized = login.trim().to_lowercase();
        if normalized.is_empty() {
            return Ok(None);
        }

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
            let query_str = if term.contains('%') {
                "SELECT id, name, login, password_hash, role FROM users WHERE LOWER(login) LIKE ? LIMIT 1"
            } else {
                "SELECT id, name, login, password_hash, role FROM users WHERE LOWER(login) = ? LIMIT 1"
            };

            let record = sqlx::query(query_str)
                .bind(term)
                .fetch_optional(&self.pool)
                .await?;

            if let Some(row) = record {
                let password_hash: String = row.get(3);
                if verify(password, &password_hash)? {
                    return Ok(Some(UserProfile {
                        id: row.get(0),
                        name: row.get(1),
                        login: row.get(2),
                        role: row.get(4),
                    }));
                }
            }
        }
        Ok(None)
    }

    pub async fn list_storage_units(&self) -> Result<Vec<StorageUnitRecord>> {
        let rows = sqlx::query(
            "SELECT id, label, type, section, capacity, occupancy, metadata, created_at, updated_at
             FROM storage_units ORDER BY datetime(updated_at) DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            let metadata_str: Option<String> = row.get(6);
            let parsed = metadata_str.and_then(|json| serde_json::from_str::<Value>(&json).ok());

            result.push(StorageUnitRecord {
                id: row.get(0),
                label: row.get(1),
                r#type: row.get(2),
                section: row.get(3),
                capacity: row.get(4),
                occupancy: row.get(5),
                metadata: parsed,
                created_at: row.get(7),
                updated_at: row.get(8),
            });
        }
        Ok(result)
    }

    pub async fn create_storage_unit(&self, payload: &StoragePayload) -> Result<StorageUnitRecord> {
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

        let result = sqlx::query(
            "INSERT INTO storage_units (label, type, section, capacity, occupancy, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, 0, ?, ?, ?)",
        )
        .bind(payload.label.trim())
        .bind(payload.r#type.trim().to_uppercase())
        .bind(section)
        .bind(payload.capacity.unwrap_or(0))
        .bind(metadata)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_storage_unit(id).await
    }

    async fn get_storage_unit(&self, id: i64) -> Result<StorageUnitRecord> {
        let row = sqlx::query(
            "SELECT id, label, type, section, capacity, occupancy, metadata, created_at, updated_at FROM storage_units WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let metadata_str: Option<String> = row.get(6);
        let parsed = metadata_str.and_then(|json| serde_json::from_str::<Value>(&json).ok());

        Ok(StorageUnitRecord {
            id: row.get(0),
            label: row.get(1),
            r#type: row.get(2),
            section: row.get(3),
            capacity: row.get(4),
            occupancy: row.get(5),
            metadata: parsed,
            created_at: row.get(7),
            updated_at: row.get(8),
        })
    }

    pub async fn list_movements(&self, limit: i64) -> Result<Vec<MovementRecord>> {
        let rows = sqlx::query(
            "SELECT id, reference, item_label, from_unit, to_unit, action, note, actor, created_at
             FROM movements ORDER BY datetime(created_at) DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(MovementRecord {
                id: row.get(0),
                reference: row.get(1),
                item_label: row.get(2),
                from_unit: row.get(3),
                to_unit: row.get(4),
                action: row.get(5),
                note: row.get(6),
                actor: row.get(7),
                created_at: row.get(8),
            });
        }
        Ok(result)
    }

    pub async fn record_movement(
        &self,
        actor: &str,
        payload: &MovementData,
    ) -> Result<MovementRecord> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO movements (reference, item_label, from_unit, to_unit, action, note, actor, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(payload.reference.as_deref())
        .bind(payload.item_label.as_deref())
        .bind(payload.from_unit.as_deref())
        .bind(payload.to_unit.as_deref())
        .bind(payload.action.trim())
        .bind(payload.note.as_deref())
        .bind(actor)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_movement(id).await
    }

    async fn get_movement(&self, id: i64) -> Result<MovementRecord> {
        let row = sqlx::query(
            "SELECT id, reference, item_label, from_unit, to_unit, action, note, actor, created_at FROM movements WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(MovementRecord {
            id: row.get(0),
            reference: row.get(1),
            item_label: row.get(2),
            from_unit: row.get(3),
            to_unit: row.get(4),
            action: row.get(5),
            note: row.get(6),
            actor: row.get(7),
            created_at: row.get(8),
        })
    }

    pub async fn snapshot(&self) -> Result<SnapshotSummary> {
        let counters =
            sqlx::query("SELECT type, COUNT(1) as total FROM storage_units GROUP BY type")
                .fetch_all(&self.pool)
                .await?;

        let mut units_by_type: HashMap<String, i64> = HashMap::from([
            ("PASTA".into(), 0),
            ("ENVELOPE".into(), 0),
            ("GAVETEIRO".into(), 0),
            ("CAIXA".into(), 0),
        ]);
        let mut total_units = 0;

        for row in counters {
            let kind: String = row.get(0);
            let total: i64 = row.get(1);
            let key = kind.to_uppercase();
            units_by_type.insert(key, total);
            total_units += total;
        }

        let movements_today_row =
            sqlx::query("SELECT COUNT(1) FROM movements WHERE DATE(created_at) = DATE('now')")
                .fetch_one(&self.pool)
                .await?;
        let movements_today: i64 = movements_today_row.get(0);

        let last_movement = sqlx::query(
            "SELECT id, reference, item_label, from_unit, to_unit, action, note, actor, created_at
             FROM movements ORDER BY datetime(created_at) DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| MovementRecord {
            id: row.get(0),
            reference: row.get(1),
            item_label: row.get(2),
            from_unit: row.get(3),
            to_unit: row.get(4),
            action: row.get(5),
            note: row.get(6),
            actor: row.get(7),
            created_at: row.get(8),
        });

        Ok(SnapshotSummary {
            total_units,
            units_by_type,
            movements_today,
            last_movement,
        })
    }

    async fn ensure_login_column(&self) -> Result<()> {
        let rows = sqlx::query("PRAGMA table_info(users)")
            .fetch_all(&self.pool)
            .await?;

        let mut has_login = false;
        let mut has_email = false;

        for row in rows {
            let name: String = row.get(1);
            if name == "login" {
                has_login = true;
            }
            if name == "email" {
                has_email = true;
            }
        }

        if !has_login && has_email {
            sqlx::query("ALTER TABLE users RENAME COLUMN email TO login")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}
