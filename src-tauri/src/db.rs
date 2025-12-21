use std::collections::HashMap;

use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde_json::Value;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::str::FromStr;

use crate::types::{
    MovementData, MovementRecord, SnapshotSummary, StoragePayload, StorageUnitRecord, UserProfile,
};

const MIGRATIONS: [&str; 39] = [
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
    "CREATE TABLE IF NOT EXISTS departments (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        code TEXT,
        description TEXT,
        is_active INTEGER DEFAULT 1,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS employees (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        full_name TEXT NOT NULL,
        registration TEXT NOT NULL UNIQUE,
        cpf TEXT UNIQUE,
        department_id INTEGER REFERENCES departments(id),
        admission_date TEXT NOT NULL,
        termination_date TEXT,
        status TEXT DEFAULT 'ACTIVE',
        drawer_position_id INTEGER REFERENCES drawer_positions(id),
        notes TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS file_cabinets (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        number TEXT NOT NULL UNIQUE,
        location TEXT,
        num_drawers INTEGER NOT NULL DEFAULT 4,
        description TEXT,
        is_active INTEGER DEFAULT 1,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS drawers (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        file_cabinet_id INTEGER NOT NULL REFERENCES file_cabinets(id),
        number INTEGER NOT NULL,
        capacity INTEGER NOT NULL DEFAULT 30,
        label TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(file_cabinet_id, number)
    )",
    "CREATE TABLE IF NOT EXISTS drawer_positions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        drawer_id INTEGER NOT NULL REFERENCES drawers(id),
        position INTEGER NOT NULL,
        employee_id INTEGER REFERENCES employees(id),
        is_occupied INTEGER DEFAULT 0,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(drawer_id, position)
    )",
    "CREATE TABLE IF NOT EXISTS document_categories (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        code TEXT NOT NULL UNIQUE,
        description TEXT,
        icon TEXT,
        color TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "INSERT OR IGNORE INTO document_categories (name, code, description)
        VALUES
        ('Pessoal', 'PESSOAL', 'Documentos pessoais, contratos, admissão'),
        ('Medicina do Trabalho', 'MEDICINA', 'Exames, ASOs, atestados'),
        ('Segurança do Trabalho', 'SEGURANCA', 'EPIs, treinamentos de segurança'),
        ('Treinamento', 'TREINAMENTO', 'Certificados, capacitações')",
    "CREATE TABLE IF NOT EXISTS document_types (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        category_id INTEGER NOT NULL REFERENCES document_categories(id),
        name TEXT NOT NULL,
        retention_years INTEGER DEFAULT 5,
        is_required INTEGER DEFAULT 0,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(category_id, name)
    )",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'Contrato de Trabalho', 10 FROM document_categories WHERE code = 'PESSOAL'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'RG', 5 FROM document_categories WHERE code = 'PESSOAL'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'CPF', 5 FROM document_categories WHERE code = 'PESSOAL'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'Comprovante de Residência', 2 FROM document_categories WHERE code = 'PESSOAL'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'Certidão de Nascimento/Casamento', 5 FROM document_categories WHERE code = 'PESSOAL'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'ASO Admissional', 20 FROM document_categories WHERE code = 'MEDICINA'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'ASO Periódico', 20 FROM document_categories WHERE code = 'MEDICINA'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'ASO Demissional', 20 FROM document_categories WHERE code = 'MEDICINA'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'Atestado Médico', 5 FROM document_categories WHERE code = 'MEDICINA'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'Ficha de EPI', 5 FROM document_categories WHERE code = 'SEGURANCA'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'Treinamento NR', 5 FROM document_categories WHERE code = 'SEGURANCA'",
    "INSERT OR IGNORE INTO document_types (category_id, name, retention_years)
        SELECT id, 'Certificado de Curso', 5 FROM document_categories WHERE code = 'TREINAMENTO'",
    "CREATE TABLE IF NOT EXISTS documents (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        employee_id INTEGER NOT NULL REFERENCES employees(id),
        category_id INTEGER NOT NULL REFERENCES document_categories(id),
        type_id INTEGER NOT NULL REFERENCES document_types(id),
        description TEXT,
        document_date TEXT,
        filing_date TEXT DEFAULT CURRENT_TIMESTAMP,
        expiration_date TEXT,
        notes TEXT,
        filed_by TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS loans (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        employee_id INTEGER NOT NULL REFERENCES employees(id),
        requester_name TEXT NOT NULL,
        requester_department_id INTEGER REFERENCES departments(id),
        reason TEXT NOT NULL,
        loan_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expected_return_date TEXT NOT NULL,
        actual_return_date TEXT,
        status TEXT DEFAULT 'BORROWED',
        return_notes TEXT,
        loaned_by TEXT NOT NULL,
        returned_by TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS dead_archive_boxes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        box_number TEXT NOT NULL UNIQUE,
        year INTEGER NOT NULL,
        period TEXT,
        letter_range TEXT,
        location TEXT,
        capacity INTEGER DEFAULT 50,
        current_count INTEGER DEFAULT 0,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS dead_archive_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        employee_id INTEGER NOT NULL REFERENCES employees(id),
        box_id INTEGER NOT NULL REFERENCES dead_archive_boxes(id),
        transfer_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        disposal_eligible_date TEXT,
        disposed INTEGER DEFAULT 0,
        disposal_date TEXT,
        disposal_term_number TEXT,
        transferred_by TEXT NOT NULL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS audit_logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER REFERENCES users(id),
        action TEXT NOT NULL,
        entity_type TEXT NOT NULL,
        entity_id INTEGER,
        old_values TEXT,
        new_values TEXT,
        ip_address TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE INDEX IF NOT EXISTS idx_storage_updated_at ON storage_units(updated_at)",
    "CREATE INDEX IF NOT EXISTS idx_movements_created_at ON movements(created_at)",
    "CREATE INDEX IF NOT EXISTS idx_users_login ON users(login)",
    "CREATE INDEX IF NOT EXISTS idx_employees_registration ON employees(registration)",
    "CREATE INDEX IF NOT EXISTS idx_employees_status ON employees(status)",
    "CREATE INDEX IF NOT EXISTS idx_employees_name ON employees(full_name)",
    "CREATE INDEX IF NOT EXISTS idx_documents_employee ON documents(employee_id)",
    "CREATE INDEX IF NOT EXISTS idx_loans_status ON loans(status)",
    "CREATE INDEX IF NOT EXISTS idx_loans_employee ON loans(employee_id)",
    "CREATE INDEX IF NOT EXISTS idx_dead_archive_employee ON dead_archive_items(employee_id)",
    "CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at)",
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

        let options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", path.to_string_lossy()))?
                .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(60))
            .connect_with(options)
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

    // ========================== DEPARTMENTS ==========================

    pub async fn list_departments(&self) -> Result<Vec<crate::types::DepartmentRecord>> {
        let rows = sqlx::query(
            "SELECT id, name, code, description, is_active, created_at, updated_at
             FROM departments ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::DepartmentRecord {
                id: row.get(0),
                name: row.get(1),
                code: row.get(2),
                description: row.get(3),
                is_active: row.get::<i64, _>(4) == 1,
                created_at: row.get(5),
                updated_at: row.get(6),
            });
        }
        Ok(result)
    }

    pub async fn create_department(
        &self,
        payload: &crate::types::DepartmentPayload,
    ) -> Result<crate::types::DepartmentRecord> {
        let now = Utc::now().to_rfc3339();
        let is_active = payload.is_active.unwrap_or(true);

        let result = sqlx::query(
            "INSERT INTO departments (name, code, description, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(payload.name.trim())
        .bind(payload.code.as_deref())
        .bind(payload.description.as_deref())
        .bind(if is_active { 1 } else { 0 })
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_department(id).await
    }

    pub async fn update_department(
        &self,
        id: i64,
        payload: &crate::types::DepartmentPayload,
    ) -> Result<crate::types::DepartmentRecord> {
        let now = Utc::now().to_rfc3339();
        let is_active = payload.is_active.unwrap_or(true);

        sqlx::query(
            "UPDATE departments SET name = ?, code = ?, description = ?, is_active = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(payload.name.trim())
        .bind(payload.code.as_deref())
        .bind(payload.description.as_deref())
        .bind(if is_active { 1 } else { 0 })
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_department(id).await
    }

    pub async fn get_department(&self, id: i64) -> Result<crate::types::DepartmentRecord> {
        let row = sqlx::query(
            "SELECT id, name, code, description, is_active, created_at, updated_at
             FROM departments WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::DepartmentRecord {
            id: row.get(0),
            name: row.get(1),
            code: row.get(2),
            description: row.get(3),
            is_active: row.get::<i64, _>(4) == 1,
            created_at: row.get(5),
            updated_at: row.get(6),
        })
    }

    // ========================== EMPLOYEES ==========================

    pub async fn list_employees(
        &self,
        status: Option<&str>,
        department_id: Option<i64>,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<crate::types::EmployeeRecord>> {
        let offset = (page - 1) * page_size;

        let mut query = String::from(
            "SELECT e.id, e.full_name, e.registration, e.cpf, e.department_id, d.name as department_name,
             e.admission_date, e.termination_date, e.status, e.drawer_position_id, e.notes,
             e.created_at, e.updated_at
             FROM employees e
             LEFT JOIN departments d ON e.department_id = d.id
             WHERE 1=1"
        );

        if status.is_some() {
            query.push_str(" AND e.status = ?");
        }
        if department_id.is_some() {
            query.push_str(" AND e.department_id = ?");
        }
        query.push_str(" ORDER BY e.full_name ASC LIMIT ? OFFSET ?");

        let mut q = sqlx::query(&query);

        if let Some(s) = status {
            q = q.bind(s);
        }
        if let Some(did) = department_id {
            q = q.bind(did);
        }
        q = q.bind(page_size).bind(offset);

        let rows = q.fetch_all(&self.pool).await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::EmployeeRecord {
                id: row.get(0),
                full_name: row.get(1),
                registration: row.get(2),
                cpf: row.get(3),
                department_id: row.get(4),
                department_name: row.get(5),
                admission_date: row.get(6),
                termination_date: row.get(7),
                status: row.get(8),
                drawer_position_id: row.get(9),
                notes: row.get(10),
                created_at: row.get(11),
                updated_at: row.get(12),
            });
        }
        Ok(result)
    }

    pub async fn search_employees(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<crate::types::EmployeeRecord>> {
        let search_pattern = format!("%{}%", query.trim());

        let rows = sqlx::query(
            "SELECT e.id, e.full_name, e.registration, e.cpf, e.department_id, d.name as department_name,
             e.admission_date, e.termination_date, e.status, e.drawer_position_id, e.notes,
             e.created_at, e.updated_at
             FROM employees e
             LEFT JOIN departments d ON e.department_id = d.id
             WHERE e.full_name LIKE ? OR e.registration LIKE ? OR e.cpf LIKE ?
             ORDER BY e.full_name ASC LIMIT ?"
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::EmployeeRecord {
                id: row.get(0),
                full_name: row.get(1),
                registration: row.get(2),
                cpf: row.get(3),
                department_id: row.get(4),
                department_name: row.get(5),
                admission_date: row.get(6),
                termination_date: row.get(7),
                status: row.get(8),
                drawer_position_id: row.get(9),
                notes: row.get(10),
                created_at: row.get(11),
                updated_at: row.get(12),
            });
        }
        Ok(result)
    }

    pub async fn create_employee(
        &self,
        payload: &crate::types::EmployeePayload,
    ) -> Result<crate::types::EmployeeRecord> {
        let now = Utc::now().to_rfc3339();
        let status = payload.status.as_deref().unwrap_or("ACTIVE");

        let result = sqlx::query(
            "INSERT INTO employees (full_name, registration, cpf, department_id, admission_date,
             termination_date, status, drawer_position_id, notes, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(payload.full_name.trim())
        .bind(payload.registration.trim())
        .bind(payload.cpf.as_deref())
        .bind(payload.department_id)
        .bind(&payload.admission_date)
        .bind(payload.termination_date.as_deref())
        .bind(status)
        .bind(payload.drawer_position_id)
        .bind(payload.notes.as_deref())
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_employee_by_id(id).await
    }

    pub async fn update_employee(
        &self,
        id: i64,
        payload: &crate::types::EmployeePayload,
    ) -> Result<crate::types::EmployeeRecord> {
        let now = Utc::now().to_rfc3339();
        let status = payload.status.as_deref().unwrap_or("ACTIVE");

        sqlx::query(
            "UPDATE employees SET full_name = ?, registration = ?, cpf = ?, department_id = ?,
             admission_date = ?, termination_date = ?, status = ?, drawer_position_id = ?,
             notes = ?, updated_at = ? WHERE id = ?",
        )
        .bind(payload.full_name.trim())
        .bind(payload.registration.trim())
        .bind(payload.cpf.as_deref())
        .bind(payload.department_id)
        .bind(&payload.admission_date)
        .bind(payload.termination_date.as_deref())
        .bind(status)
        .bind(payload.drawer_position_id)
        .bind(payload.notes.as_deref())
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_employee_by_id(id).await
    }

    pub async fn terminate_employee(
        &self,
        id: i64,
        termination_date: &str,
    ) -> Result<crate::types::EmployeeRecord> {
        let now = Utc::now().to_rfc3339();

        // Update employee status
        sqlx::query(
            "UPDATE employees SET status = 'TERMINATED', termination_date = ?,
             drawer_position_id = NULL, updated_at = ? WHERE id = ?",
        )
        .bind(termination_date)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        // Free the drawer position if assigned
        sqlx::query(
            "UPDATE drawer_positions SET employee_id = NULL, is_occupied = 0
             WHERE employee_id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_employee_by_id(id).await
    }

    pub async fn get_employee_by_id(&self, id: i64) -> Result<crate::types::EmployeeRecord> {
        let row = sqlx::query(
            "SELECT e.id, e.full_name, e.registration, e.cpf, e.department_id, d.name as department_name,
             e.admission_date, e.termination_date, e.status, e.drawer_position_id, e.notes,
             e.created_at, e.updated_at
             FROM employees e
             LEFT JOIN departments d ON e.department_id = d.id
             WHERE e.id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::EmployeeRecord {
            id: row.get(0),
            full_name: row.get(1),
            registration: row.get(2),
            cpf: row.get(3),
            department_id: row.get(4),
            department_name: row.get(5),
            admission_date: row.get(6),
            termination_date: row.get(7),
            status: row.get(8),
            drawer_position_id: row.get(9),
            notes: row.get(10),
            created_at: row.get(11),
            updated_at: row.get(12),
        })
    }

    pub async fn get_employee_documents(
        &self,
        employee_id: i64,
    ) -> Result<Vec<crate::types::DocumentRecord>> {
        let rows = sqlx::query(
            "SELECT id, employee_id, category_id, type_id, description, document_date,
             filing_date, expiration_date, notes, filed_by, created_at
             FROM documents WHERE employee_id = ? ORDER BY filing_date DESC",
        )
        .bind(employee_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::DocumentRecord {
                id: row.get(0),
                employee_id: row.get(1),
                category_id: row.get(2),
                type_id: row.get(3),
                description: row.get(4),
                document_date: row.get(5),
                filing_date: row.get(6),
                expiration_date: row.get(7),
                notes: row.get(8),
                filed_by: row.get(9),
                created_at: row.get(10),
            });
        }
        Ok(result)
    }

    pub async fn get_employee_active_loans(
        &self,
        employee_id: i64,
    ) -> Result<Vec<crate::types::LoanRecord>> {
        let rows = sqlx::query(
            "SELECT id, employee_id, requester_name, requester_department_id, reason,
             loan_date, expected_return_date, actual_return_date, status, return_notes,
             loaned_by, returned_by, created_at, updated_at
             FROM loans WHERE employee_id = ? AND status = 'BORROWED'
             ORDER BY loan_date DESC",
        )
        .bind(employee_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::LoanRecord {
                id: row.get(0),
                employee_id: row.get(1),
                requester_name: row.get(2),
                requester_department_id: row.get(3),
                reason: row.get(4),
                loan_date: row.get(5),
                expected_return_date: row.get(6),
                actual_return_date: row.get(7),
                status: row.get(8),
                return_notes: row.get(9),
                loaned_by: row.get(10),
                returned_by: row.get(11),
                created_at: row.get(12),
                updated_at: row.get(13),
            });
        }
        Ok(result)
    }

    pub async fn get_employee_drawer_position(
        &self,
        employee_id: i64,
    ) -> Result<Option<crate::types::DrawerPositionRecord>> {
        let row = sqlx::query(
            "SELECT id, drawer_id, position, employee_id, is_occupied, created_at
             FROM drawer_positions WHERE employee_id = ?",
        )
        .bind(employee_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| crate::types::DrawerPositionRecord {
            id: r.get(0),
            drawer_id: r.get(1),
            position: r.get(2),
            employee_id: r.get(3),
            is_occupied: r.get::<i64, _>(4) == 1,
            created_at: r.get(5),
        }))
    }

    // ========================== FILE CABINETS ==========================

    pub async fn create_file_cabinet(
        &self,
        payload: &crate::types::FileCabinetPayload,
    ) -> Result<crate::types::FileCabinetRecord> {
        let now = Utc::now().to_rfc3339();
        let num_drawers = payload.num_drawers.unwrap_or(4);
        let is_active = payload.is_active.unwrap_or(true);

        let result = sqlx::query(
            "INSERT INTO file_cabinets (number, location, num_drawers, description, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(payload.number.trim())
        .bind(payload.location.as_deref())
        .bind(num_drawers)
        .bind(payload.description.as_deref())
        .bind(if is_active { 1 } else { 0 })
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let cabinet_id = result.last_insert_rowid();

        // Automatically create drawers
        for drawer_num in 1..=num_drawers {
            sqlx::query(
                "INSERT INTO drawers (file_cabinet_id, number, capacity, created_at)
                 VALUES (?, ?, 30, ?)",
            )
            .bind(cabinet_id)
            .bind(drawer_num)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        }

        self.get_file_cabinet(cabinet_id).await
    }

    pub async fn get_file_cabinet(&self, id: i64) -> Result<crate::types::FileCabinetRecord> {
        let row = sqlx::query(
            "SELECT id, number, location, num_drawers, description, is_active, created_at, updated_at
             FROM file_cabinets WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::FileCabinetRecord {
            id: row.get(0),
            number: row.get(1),
            location: row.get(2),
            num_drawers: row.get(3),
            description: row.get(4),
            is_active: row.get::<i64, _>(5) == 1,
            created_at: row.get(6),
            updated_at: row.get(7),
        })
    }

    pub async fn create_drawer(
        &self,
        payload: &crate::types::DrawerPayload,
    ) -> Result<crate::types::DrawerRecord> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO drawers (file_cabinet_id, number, capacity, label, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(payload.file_cabinet_id)
        .bind(payload.number)
        .bind(payload.capacity)
        .bind(payload.label.as_deref())
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_drawer(id).await
    }

    pub async fn get_drawer(&self, id: i64) -> Result<crate::types::DrawerRecord> {
        let row = sqlx::query(
            "SELECT id, file_cabinet_id, number, capacity, label, created_at
             FROM drawers WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::DrawerRecord {
            id: row.get(0),
            file_cabinet_id: row.get(1),
            number: row.get(2),
            capacity: row.get(3),
            label: row.get(4),
            created_at: row.get(5),
        })
    }

    pub async fn list_file_cabinets(&self) -> Result<Vec<crate::types::FileCabinetWithOccupancy>> {
        let cabinets = sqlx::query(
            "SELECT id, number, location, num_drawers, description, is_active, created_at, updated_at
             FROM file_cabinets WHERE is_active = 1 ORDER BY number ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();

        for row in cabinets {
            let cabinet = crate::types::FileCabinetRecord {
                id: row.get(0),
                number: row.get(1),
                location: row.get(2),
                num_drawers: row.get(3),
                description: row.get(4),
                is_active: row.get::<i64, _>(5) == 1,
                created_at: row.get(6),
                updated_at: row.get(7),
            };

            let drawers = self.get_drawers_with_occupancy(cabinet.id).await?;

            let total_positions: i64 = drawers.iter().map(|d| d.capacity).sum();
            let occupied_positions: i64 = drawers.iter().map(|d| d.occupied).sum();

            result.push(crate::types::FileCabinetWithOccupancy {
                cabinet,
                drawers,
                total_positions,
                occupied_positions,
            });
        }

        Ok(result)
    }

    async fn get_drawers_with_occupancy(
        &self,
        cabinet_id: i64,
    ) -> Result<Vec<crate::types::DrawerWithOccupancy>> {
        let rows = sqlx::query(
            "SELECT d.id, d.file_cabinet_id, d.number, d.capacity, d.label, d.created_at,
             (SELECT COUNT(*) FROM drawer_positions dp WHERE dp.drawer_id = d.id AND dp.is_occupied = 1) as occupied
             FROM drawers d WHERE d.file_cabinet_id = ? ORDER BY d.number ASC"
        )
        .bind(cabinet_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            let capacity: i64 = row.get(3);
            let occupied: i64 = row.get(6);
            let occupancy_rate = if capacity > 0 {
                (occupied as f32 / capacity as f32) * 100.0
            } else {
                0.0
            };

            result.push(crate::types::DrawerWithOccupancy {
                drawer: crate::types::DrawerRecord {
                    id: row.get(0),
                    file_cabinet_id: row.get(1),
                    number: row.get(2),
                    capacity,
                    label: row.get(4),
                    created_at: row.get(5),
                },
                occupied,
                capacity,
                occupancy_rate,
                critical: occupancy_rate >= 90.0,
            });
        }
        Ok(result)
    }

    pub async fn get_occupation_map(&self) -> Result<crate::types::OccupationMap> {
        let cabinets_with_occ = self.list_file_cabinets().await?;

        let mut nodes = Vec::new();
        let mut total_positions: i64 = 0;
        let mut occupied_positions: i64 = 0;
        let mut warnings: i64 = 0;
        let mut critical: i64 = 0;

        for cab in cabinets_with_occ {
            total_positions += cab.total_positions;
            occupied_positions += cab.occupied_positions;

            let rate = if cab.total_positions > 0 {
                (cab.occupied_positions as f32 / cab.total_positions as f32) * 100.0
            } else {
                0.0
            };

            let status = if rate >= 90.0 {
                critical += 1;
                "CRITICAL"
            } else if rate >= 70.0 {
                warnings += 1;
                "WARNING"
            } else {
                "OK"
            };

            nodes.push(crate::types::CabinetOccupationNode {
                cabinet_id: cab.cabinet.id,
                cabinet_label: cab.cabinet.number.clone(),
                occupancy_rate: rate,
                status: status.to_string(),
                drawers: cab.drawers,
            });
        }

        Ok(crate::types::OccupationMap {
            cabinets: nodes,
            totals: crate::types::OccupationTotals {
                total_positions,
                occupied_positions,
                warnings,
                critical,
            },
        })
    }

    pub async fn assign_employee_position(
        &self,
        employee_id: i64,
        drawer_id: i64,
        position: i64,
    ) -> Result<crate::types::DrawerPositionRecord> {
        let now = Utc::now().to_rfc3339();

        // Check if position exists, if not create it
        let existing =
            sqlx::query("SELECT id FROM drawer_positions WHERE drawer_id = ? AND position = ?")
                .bind(drawer_id)
                .bind(position)
                .fetch_optional(&self.pool)
                .await?;

        let position_id = if let Some(row) = existing {
            let id: i64 = row.get(0);
            // Update existing position
            sqlx::query(
                "UPDATE drawer_positions SET employee_id = ?, is_occupied = 1 WHERE id = ?",
            )
            .bind(employee_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
            id
        } else {
            // Create new position
            let result = sqlx::query(
                "INSERT INTO drawer_positions (drawer_id, position, employee_id, is_occupied, created_at)
                 VALUES (?, ?, ?, 1, ?)"
            )
            .bind(drawer_id)
            .bind(position)
            .bind(employee_id)
            .bind(&now)
            .execute(&self.pool)
            .await?;
            result.last_insert_rowid()
        };

        // Update employee's drawer_position_id
        sqlx::query("UPDATE employees SET drawer_position_id = ? WHERE id = ?")
            .bind(position_id)
            .bind(employee_id)
            .execute(&self.pool)
            .await?;

        self.get_drawer_position(position_id).await
    }

    async fn get_drawer_position(&self, id: i64) -> Result<crate::types::DrawerPositionRecord> {
        let row = sqlx::query(
            "SELECT id, drawer_id, position, employee_id, is_occupied, created_at
             FROM drawer_positions WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::DrawerPositionRecord {
            id: row.get(0),
            drawer_id: row.get(1),
            position: row.get(2),
            employee_id: row.get(3),
            is_occupied: row.get::<i64, _>(4) == 1,
            created_at: row.get(5),
        })
    }

    pub async fn suggest_reorganization(
        &self,
        critical_threshold: i64,
        max_moves: i64,
    ) -> Result<crate::types::ReorganizationPlan> {
        // Find critical drawers (above threshold)
        let critical_drawers = sqlx::query(
            "SELECT d.id, d.file_cabinet_id, d.number, d.capacity, fc.number as cabinet_number,
             (SELECT COUNT(*) FROM drawer_positions dp WHERE dp.drawer_id = d.id AND dp.is_occupied = 1) as occupied
             FROM drawers d
             JOIN file_cabinets fc ON d.file_cabinet_id = fc.id
             HAVING (occupied * 100.0 / d.capacity) >= ?
             ORDER BY (occupied * 1.0 / d.capacity) DESC"
        )
        .bind(critical_threshold)
        .fetch_all(&self.pool)
        .await?;

        // Find drawers with space
        let available_drawers = sqlx::query(
            "SELECT d.id, d.file_cabinet_id, d.number, d.capacity, fc.number as cabinet_number,
             (SELECT COUNT(*) FROM drawer_positions dp WHERE dp.drawer_id = d.id AND dp.is_occupied = 1) as occupied
             FROM drawers d
             JOIN file_cabinets fc ON d.file_cabinet_id = fc.id
             HAVING (occupied * 100.0 / d.capacity) < 70
             ORDER BY (occupied * 1.0 / d.capacity) ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut suggestions = Vec::new();
        let mut moves_count = 0;

        for critical in &critical_drawers {
            if moves_count >= max_moves as usize {
                break;
            }

            let cabinet_number: String = critical.get(4);
            let drawer_number: i64 = critical.get(2);
            let from_drawer = format!("{}-G{}", cabinet_number, drawer_number);

            // Get employees that could be moved
            let employees = sqlx::query(
                "SELECT e.id, e.full_name FROM employees e
                 JOIN drawer_positions dp ON e.drawer_position_id = dp.id
                 WHERE dp.drawer_id = ? LIMIT 3",
            )
            .bind(critical.get::<i64, _>(0))
            .fetch_all(&self.pool)
            .await?;

            for emp in employees {
                if moves_count >= max_moves as usize {
                    break;
                }

                if let Some(target) = available_drawers.get(moves_count % available_drawers.len()) {
                    let target_cabinet: String = target.get(4);
                    let target_drawer: i64 = target.get(2);
                    let to_drawer = format!("{}-G{}", target_cabinet, target_drawer);

                    suggestions.push(crate::types::ReorganizationSuggestion {
                        employee_id: emp.get(0),
                        employee_name: emp.get(1),
                        from_drawer: from_drawer.clone(),
                        to_drawer,
                        reason: "Redistribuição de capacidade".to_string(),
                    });
                    moves_count += 1;
                }
            }
        }

        Ok(crate::types::ReorganizationPlan {
            total_moves: suggestions.len(),
            suggestions,
        })
    }

    // ========================== DOCUMENTS ==========================

    pub async fn list_document_categories(
        &self,
    ) -> Result<Vec<crate::types::DocumentCategoryRecord>> {
        let rows = sqlx::query(
            "SELECT id, name, code, description, icon, color, created_at
             FROM document_categories ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::DocumentCategoryRecord {
                id: row.get(0),
                name: row.get(1),
                code: row.get(2),
                description: row.get(3),
                icon: row.get(4),
                color: row.get(5),
                created_at: row.get(6),
            });
        }
        Ok(result)
    }

    pub async fn list_document_types(
        &self,
        category_id: Option<i64>,
    ) -> Result<Vec<crate::types::DocumentTypeRecord>> {
        let query = if category_id.is_some() {
            "SELECT id, category_id, name, retention_years, is_required, created_at
             FROM document_types WHERE category_id = ? ORDER BY name ASC"
        } else {
            "SELECT id, category_id, name, retention_years, is_required, created_at
             FROM document_types ORDER BY name ASC"
        };

        let mut q = sqlx::query(query);
        if let Some(cid) = category_id {
            q = q.bind(cid);
        }

        let rows = q.fetch_all(&self.pool).await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::DocumentTypeRecord {
                id: row.get(0),
                category_id: row.get(1),
                name: row.get(2),
                retention_years: row.get(3),
                is_required: row.get::<i64, _>(4) == 1,
                created_at: row.get(5),
            });
        }
        Ok(result)
    }

    pub async fn create_document(
        &self,
        payload: &crate::types::DocumentPayload,
        actor: &str,
    ) -> Result<crate::types::DocumentRecord> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO documents (employee_id, category_id, type_id, description, document_date,
             filing_date, expiration_date, notes, filed_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(payload.employee_id)
        .bind(payload.category_id)
        .bind(payload.type_id)
        .bind(payload.description.as_deref())
        .bind(payload.document_date.as_deref())
        .bind(&now)
        .bind(payload.expiration_date.as_deref())
        .bind(payload.notes.as_deref())
        .bind(payload.filed_by.as_deref().unwrap_or(actor))
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_document(id).await
    }

    async fn get_document(&self, id: i64) -> Result<crate::types::DocumentRecord> {
        let row = sqlx::query(
            "SELECT id, employee_id, category_id, type_id, description, document_date,
             filing_date, expiration_date, notes, filed_by, created_at
             FROM documents WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::DocumentRecord {
            id: row.get(0),
            employee_id: row.get(1),
            category_id: row.get(2),
            type_id: row.get(3),
            description: row.get(4),
            document_date: row.get(5),
            filing_date: row.get(6),
            expiration_date: row.get(7),
            notes: row.get(8),
            filed_by: row.get(9),
            created_at: row.get(10),
        })
    }

    // ========================== LOANS ==========================

    pub async fn create_loan(
        &self,
        payload: &crate::types::LoanPayload,
        actor: &str,
    ) -> Result<crate::types::LoanRecord> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO loans (employee_id, requester_name, requester_department_id, reason,
             loan_date, expected_return_date, status, return_notes, loaned_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 'BORROWED', ?, ?, ?, ?)"
        )
        .bind(payload.employee_id)
        .bind(&payload.requester_name)
        .bind(payload.requester_department_id)
        .bind(&payload.reason)
        .bind(&now)
        .bind(&payload.expected_return_date)
        .bind(payload.return_notes.as_deref())
        .bind(actor)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_loan(id).await
    }

    pub async fn return_loan(
        &self,
        loan_id: i64,
        return_date: Option<&str>,
        return_notes: Option<&str>,
        actor: &str,
    ) -> Result<crate::types::LoanRecord> {
        let now = Utc::now().to_rfc3339();
        let actual_date = return_date.unwrap_or(&now);

        sqlx::query(
            "UPDATE loans SET status = 'RETURNED', actual_return_date = ?, return_notes = ?,
             returned_by = ?, updated_at = ? WHERE id = ?",
        )
        .bind(actual_date)
        .bind(return_notes)
        .bind(actor)
        .bind(&now)
        .bind(loan_id)
        .execute(&self.pool)
        .await?;

        self.get_loan(loan_id).await
    }

    async fn get_loan(&self, id: i64) -> Result<crate::types::LoanRecord> {
        let row = sqlx::query(
            "SELECT id, employee_id, requester_name, requester_department_id, reason,
             loan_date, expected_return_date, actual_return_date, status, return_notes,
             loaned_by, returned_by, created_at, updated_at
             FROM loans WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::LoanRecord {
            id: row.get(0),
            employee_id: row.get(1),
            requester_name: row.get(2),
            requester_department_id: row.get(3),
            reason: row.get(4),
            loan_date: row.get(5),
            expected_return_date: row.get(6),
            actual_return_date: row.get(7),
            status: row.get(8),
            return_notes: row.get(9),
            loaned_by: row.get(10),
            returned_by: row.get(11),
            created_at: row.get(12),
            updated_at: row.get(13),
        })
    }

    pub async fn list_loans(&self, status: Option<&str>) -> Result<Vec<crate::types::LoanRecord>> {
        let query = if status.is_some() {
            "SELECT id, employee_id, requester_name, requester_department_id, reason,
             loan_date, expected_return_date, actual_return_date, status, return_notes,
             loaned_by, returned_by, created_at, updated_at
             FROM loans WHERE status = ? ORDER BY loan_date DESC"
        } else {
            "SELECT id, employee_id, requester_name, requester_department_id, reason,
             loan_date, expected_return_date, actual_return_date, status, return_notes,
             loaned_by, returned_by, created_at, updated_at
             FROM loans ORDER BY loan_date DESC"
        };

        let mut q = sqlx::query(query);
        if let Some(s) = status {
            q = q.bind(s);
        }

        let rows = q.fetch_all(&self.pool).await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::LoanRecord {
                id: row.get(0),
                employee_id: row.get(1),
                requester_name: row.get(2),
                requester_department_id: row.get(3),
                reason: row.get(4),
                loan_date: row.get(5),
                expected_return_date: row.get(6),
                actual_return_date: row.get(7),
                status: row.get(8),
                return_notes: row.get(9),
                loaned_by: row.get(10),
                returned_by: row.get(11),
                created_at: row.get(12),
                updated_at: row.get(13),
            });
        }
        Ok(result)
    }

    pub async fn get_overdue_loans(&self) -> Result<Vec<crate::types::LoanWithEmployee>> {
        let rows = sqlx::query(
            "SELECT l.id, l.employee_id, l.requester_name, l.requester_department_id, l.reason,
             l.loan_date, l.expected_return_date, l.actual_return_date, l.status, l.return_notes,
             l.loaned_by, l.returned_by, l.created_at, l.updated_at,
             e.id, e.full_name, e.registration, e.cpf, e.department_id, d.name,
             e.admission_date, e.termination_date, e.status, e.drawer_position_id, e.notes,
             e.created_at, e.updated_at
             FROM loans l
             JOIN employees e ON l.employee_id = e.id
             LEFT JOIN departments d ON e.department_id = d.id
             WHERE l.status = 'BORROWED' AND l.expected_return_date < DATE('now')
             ORDER BY l.expected_return_date ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::LoanWithEmployee {
                loan: crate::types::LoanRecord {
                    id: row.get(0),
                    employee_id: row.get(1),
                    requester_name: row.get(2),
                    requester_department_id: row.get(3),
                    reason: row.get(4),
                    loan_date: row.get(5),
                    expected_return_date: row.get(6),
                    actual_return_date: row.get(7),
                    status: row.get(8),
                    return_notes: row.get(9),
                    loaned_by: row.get(10),
                    returned_by: row.get(11),
                    created_at: row.get(12),
                    updated_at: row.get(13),
                },
                employee: crate::types::EmployeeRecord {
                    id: row.get(14),
                    full_name: row.get(15),
                    registration: row.get(16),
                    cpf: row.get(17),
                    department_id: row.get(18),
                    department_name: row.get(19),
                    admission_date: row.get(20),
                    termination_date: row.get(21),
                    status: row.get(22),
                    drawer_position_id: row.get(23),
                    notes: row.get(24),
                    created_at: row.get(25),
                    updated_at: row.get(26),
                },
            });
        }
        Ok(result)
    }

    // ========================== DEAD ARCHIVE ==========================

    pub async fn create_archive_box(
        &self,
        payload: &crate::types::ArchiveBoxPayload,
    ) -> Result<crate::types::ArchiveBoxRecord> {
        let now = Utc::now().to_rfc3339();
        let capacity = payload.capacity.unwrap_or(50);

        let result = sqlx::query(
            "INSERT INTO dead_archive_boxes (box_number, year, period, letter_range, location, capacity, current_count, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 0, ?)"
        )
        .bind(payload.box_number.trim())
        .bind(payload.year)
        .bind(payload.period.as_deref())
        .bind(payload.letter_range.as_deref())
        .bind(payload.location.as_deref())
        .bind(capacity)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_archive_box(id).await
    }

    async fn get_archive_box(&self, id: i64) -> Result<crate::types::ArchiveBoxRecord> {
        let row = sqlx::query(
            "SELECT id, box_number, year, period, letter_range, location, capacity, current_count, created_at
             FROM dead_archive_boxes WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::ArchiveBoxRecord {
            id: row.get(0),
            box_number: row.get(1),
            year: row.get(2),
            period: row.get(3),
            letter_range: row.get(4),
            location: row.get(5),
            capacity: row.get(6),
            current_count: row.get(7),
            created_at: row.get(8),
        })
    }

    pub async fn list_archive_boxes(&self) -> Result<Vec<crate::types::ArchiveBoxRecord>> {
        let rows = sqlx::query(
            "SELECT id, box_number, year, period, letter_range, location, capacity, current_count, created_at
             FROM dead_archive_boxes ORDER BY year DESC, box_number ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::ArchiveBoxRecord {
                id: row.get(0),
                box_number: row.get(1),
                year: row.get(2),
                period: row.get(3),
                letter_range: row.get(4),
                location: row.get(5),
                capacity: row.get(6),
                current_count: row.get(7),
                created_at: row.get(8),
            });
        }
        Ok(result)
    }

    pub async fn transfer_to_archive(
        &self,
        employee_id: i64,
        box_id: i64,
        disposal_eligible_date: Option<&str>,
        actor: &str,
    ) -> Result<crate::types::ArchiveItemRecord> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO dead_archive_items (employee_id, box_id, transfer_date, disposal_eligible_date, transferred_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(employee_id)
        .bind(box_id)
        .bind(&now)
        .bind(disposal_eligible_date)
        .bind(actor)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        // Update box count
        sqlx::query("UPDATE dead_archive_boxes SET current_count = current_count + 1 WHERE id = ?")
            .bind(box_id)
            .execute(&self.pool)
            .await?;

        let id = result.last_insert_rowid();
        self.get_archive_item(id).await
    }

    async fn get_archive_item(&self, id: i64) -> Result<crate::types::ArchiveItemRecord> {
        let row = sqlx::query(
            "SELECT id, employee_id, box_id, transfer_date, disposal_eligible_date, disposed, disposal_date, disposal_term_number, transferred_by, created_at
             FROM dead_archive_items WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::types::ArchiveItemRecord {
            id: row.get(0),
            employee_id: row.get(1),
            box_id: row.get(2),
            transfer_date: row.get(3),
            disposal_eligible_date: row.get(4),
            disposed: row.get::<i64, _>(5) == 1,
            disposal_date: row.get(6),
            disposal_term_number: row.get(7),
            transferred_by: row.get(8),
            created_at: row.get(9),
        })
    }

    pub async fn get_disposal_candidates(&self) -> Result<Vec<crate::types::DisposalCandidate>> {
        let rows = sqlx::query(
            "SELECT dai.id, dai.employee_id, dai.box_id, dai.transfer_date, dai.disposal_eligible_date,
             dai.disposed, dai.disposal_date, dai.disposal_term_number, dai.transferred_by, dai.created_at,
             e.id, e.full_name, e.registration, e.cpf, e.department_id, d.name,
             e.admission_date, e.termination_date, e.status, e.drawer_position_id, e.notes,
             e.created_at, e.updated_at
             FROM dead_archive_items dai
             JOIN employees e ON dai.employee_id = e.id
             LEFT JOIN departments d ON e.department_id = d.id
             WHERE dai.disposed = 0 AND dai.disposal_eligible_date <= DATE('now')
             ORDER BY dai.disposal_eligible_date ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(crate::types::DisposalCandidate {
                archive_item: crate::types::ArchiveItemRecord {
                    id: row.get(0),
                    employee_id: row.get(1),
                    box_id: row.get(2),
                    transfer_date: row.get(3),
                    disposal_eligible_date: row.get(4),
                    disposed: row.get::<i64, _>(5) == 1,
                    disposal_date: row.get(6),
                    disposal_term_number: row.get(7),
                    transferred_by: row.get(8),
                    created_at: row.get(9),
                },
                employee: crate::types::EmployeeRecord {
                    id: row.get(10),
                    full_name: row.get(11),
                    registration: row.get(12),
                    cpf: row.get(13),
                    department_id: row.get(14),
                    department_name: row.get(15),
                    admission_date: row.get(16),
                    termination_date: row.get(17),
                    status: row.get(18),
                    drawer_position_id: row.get(19),
                    notes: row.get(20),
                    created_at: row.get(21),
                    updated_at: row.get(22),
                },
            });
        }
        Ok(result)
    }

    pub async fn register_disposal(
        &self,
        item_ids: &[i64],
        term_number: Option<&str>,
    ) -> Result<crate::types::DisposalTerm> {
        let now = Utc::now().to_rfc3339();
        let term = term_number
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("TERMO-{}", now.replace(":", "-")));

        let mut items = Vec::new();
        for id in item_ids {
            sqlx::query(
                "UPDATE dead_archive_items SET disposed = 1, disposal_date = ?, disposal_term_number = ?
                 WHERE id = ?"
            )
            .bind(&now)
            .bind(&term)
            .bind(id)
            .execute(&self.pool)
            .await?;

            items.push(self.get_archive_item(*id).await?);
        }

        Ok(crate::types::DisposalTerm {
            term_number: term,
            generated_at: now,
            items,
            generated_by: "Sistema".to_string(),
        })
    }

    // ========================== REPORTS ==========================

    pub async fn get_dashboard_stats(&self) -> Result<crate::types::DashboardStats> {
        let now = Utc::now().to_rfc3339();

        // Active employees
        let active_row = sqlx::query("SELECT COUNT(*) FROM employees WHERE status = 'ACTIVE'")
            .fetch_one(&self.pool)
            .await?;
        let active_employees: i64 = active_row.get(0);

        // Terminated employees
        let terminated_row =
            sqlx::query("SELECT COUNT(*) FROM employees WHERE status = 'TERMINATED'")
                .fetch_one(&self.pool)
                .await?;
        let terminated_employees: i64 = terminated_row.get(0);

        // Open loans
        let open_row = sqlx::query("SELECT COUNT(*) FROM loans WHERE status = 'BORROWED'")
            .fetch_one(&self.pool)
            .await?;
        let open_loans: i64 = open_row.get(0);

        // Overdue loans
        let overdue_row = sqlx::query("SELECT COUNT(*) FROM loans WHERE status = 'BORROWED' AND expected_return_date < DATE('now')")
            .fetch_one(&self.pool)
            .await?;
        let overdue_loans: i64 = overdue_row.get(0);

        // Archive boxes
        let boxes_row = sqlx::query("SELECT COUNT(*) FROM dead_archive_boxes")
            .fetch_one(&self.pool)
            .await?;
        let archive_boxes: i64 = boxes_row.get(0);

        // Critical cabinets
        let occupation_map = self.get_occupation_map().await?;
        let critical_cabinets: Vec<crate::types::CabinetOccupationNode> = occupation_map
            .cabinets
            .into_iter()
            .filter(|c| c.status == "CRITICAL")
            .collect();

        Ok(crate::types::DashboardStats {
            active_employees,
            terminated_employees,
            open_loans,
            overdue_loans,
            critical_cabinets,
            archive_boxes,
            last_sync: now,
        })
    }

    pub async fn get_movements_report(&self, limit: i64) -> Result<crate::types::MovementsReport> {
        let total_row = sqlx::query("SELECT COUNT(*) FROM movements")
            .fetch_one(&self.pool)
            .await?;
        let total_movements: i64 = total_row.get(0);

        let by_action_rows =
            sqlx::query("SELECT action, COUNT(*) as count FROM movements GROUP BY action")
                .fetch_all(&self.pool)
                .await?;

        let mut by_action = std::collections::HashMap::new();
        for row in by_action_rows {
            let action: String = row.get(0);
            let count: i64 = row.get(1);
            by_action.insert(action, count);
        }

        let latest = self.list_movements(limit).await?;

        Ok(crate::types::MovementsReport {
            total_movements,
            by_action,
            latest,
        })
    }

    pub async fn get_loans_report(&self) -> Result<crate::types::LoansReport> {
        let total_row = sqlx::query("SELECT COUNT(*) FROM loans")
            .fetch_one(&self.pool)
            .await?;
        let total_loans: i64 = total_row.get(0);

        let open_row = sqlx::query("SELECT COUNT(*) FROM loans WHERE status = 'BORROWED'")
            .fetch_one(&self.pool)
            .await?;
        let open_loans: i64 = open_row.get(0);

        let returned_today_row = sqlx::query("SELECT COUNT(*) FROM loans WHERE status = 'RETURNED' AND DATE(actual_return_date) = DATE('now')")
            .fetch_one(&self.pool)
            .await?;
        let returned_today: i64 = returned_today_row.get(0);

        let overdue_loans = self.get_overdue_loans().await?;

        Ok(crate::types::LoansReport {
            total_loans,
            open_loans,
            overdue_loans,
            returned_today,
        })
    }

    // ========================== LABELS ==========================

    pub async fn generate_folder_label(&self, employee_id: i64) -> Result<crate::types::LabelData> {
        let emp = self.get_employee_by_id(employee_id).await?;
        let now = Utc::now().to_rfc3339();

        let mut details = std::collections::HashMap::new();
        details.insert("Matrícula".to_string(), emp.registration.clone());
        if let Some(dept) = &emp.department_name {
            details.insert("Departamento".to_string(), dept.clone());
        }
        details.insert("Admissão".to_string(), emp.admission_date.clone());

        // Get drawer position info
        if let Some(pos_id) = emp.drawer_position_id {
            if let Ok(pos) = self.get_drawer_position(pos_id).await {
                if let Ok(drawer) = self.get_drawer(pos.drawer_id).await {
                    if let Ok(cab) = self.get_file_cabinet(drawer.file_cabinet_id).await {
                        details.insert(
                            "Localização".to_string(),
                            format!("{}-G{}-P{}", cab.number, drawer.number, pos.position),
                        );
                    }
                }
            }
        }

        Ok(crate::types::LabelData {
            title: emp.full_name,
            subtitle: Some(emp.registration),
            details,
            generated_at: now,
        })
    }

    pub async fn generate_envelope_label(
        &self,
        employee_id: i64,
        category: &str,
    ) -> Result<crate::types::LabelData> {
        let emp = self.get_employee_by_id(employee_id).await?;
        let now = Utc::now().to_rfc3339();

        let mut details = std::collections::HashMap::new();
        details.insert("Matrícula".to_string(), emp.registration.clone());
        details.insert("Categoria".to_string(), category.to_string());

        Ok(crate::types::LabelData {
            title: emp.full_name,
            subtitle: Some(category.to_string()),
            details,
            generated_at: now,
        })
    }

    pub async fn generate_box_label(&self, box_id: i64) -> Result<crate::types::LabelData> {
        let archive_box = self.get_archive_box(box_id).await?;
        let now = Utc::now().to_rfc3339();

        let mut details = std::collections::HashMap::new();
        details.insert("Ano".to_string(), archive_box.year.to_string());
        if let Some(period) = &archive_box.period {
            details.insert("Período".to_string(), period.clone());
        }
        if let Some(range) = &archive_box.letter_range {
            details.insert("Faixa".to_string(), range.clone());
        }
        if let Some(loc) = &archive_box.location {
            details.insert("Local".to_string(), loc.clone());
        }
        details.insert(
            "Capacidade".to_string(),
            format!("{}/{}", archive_box.current_count, archive_box.capacity),
        );

        Ok(crate::types::LabelData {
            title: format!("Caixa {}", archive_box.box_number),
            subtitle: Some(format!("Arquivo Morto {}", archive_box.year)),
            details,
            generated_at: now,
        })
    }
}
