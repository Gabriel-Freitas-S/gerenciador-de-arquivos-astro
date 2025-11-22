import fs from 'node:fs/promises';
import path from 'node:path';
import sqlite3pkg from '@journeyapps/sqlcipher';
import type { Database as SqliteDatabase } from '@journeyapps/sqlcipher';
import bcrypt from 'bcryptjs';
import type {
	StoragePayload,
	MovementPayload,
	StorageUnitRecord,
	MovementRecord,
	SnapshotSummary,
	StorageUnitType,
} from './types.js';

const sqlite3 = (sqlite3pkg as typeof sqlite3pkg & { default?: typeof sqlite3pkg }).default ?? sqlite3pkg;

const migrations = [
	`CREATE TABLE IF NOT EXISTS users (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL,
		login TEXT NOT NULL UNIQUE,
		password_hash TEXT NOT NULL,
		role TEXT NOT NULL DEFAULT 'admin',
		created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
	)` ,
	`CREATE TABLE IF NOT EXISTS storage_units (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		label TEXT NOT NULL,
		type TEXT NOT NULL,
		section TEXT,
		capacity INTEGER DEFAULT 0,
		occupancy INTEGER DEFAULT 0,
		metadata TEXT,
		created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
		updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
	)`,
	`CREATE TABLE IF NOT EXISTS movements (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		reference TEXT,
		item_label TEXT,
		from_unit TEXT,
		to_unit TEXT,
		action TEXT NOT NULL,
		note TEXT,
		actor TEXT NOT NULL,
		created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
	)`
];

type StorageUnitRow = Omit<StorageUnitRecord, 'metadata'> & { metadata: string | null };

type RunResult = { changes?: number; lastID?: number };

export class ArchiveDatabase {
	private db: SqliteDatabase | null = null;
	private filePath: string | null = null;
	private key: string | null = null;

	async connect(rootPath: string, key: string): Promise<void> {
		this.key = key;
		await fs.mkdir(path.dirname(rootPath), { recursive: true });
		await new Promise<void>((resolve, reject) => {
			const db = new sqlite3.Database(rootPath, (error) => {
				if (error) {
					reject(error);
					return;
				}
				this.db = db;
				this.filePath = rootPath;
				resolve();
			});
		});

		const escapedKey = key.replace(/'/g, "''");
		await this.exec(`PRAGMA key = '${escapedKey}';`);
		await this.exec('PRAGMA journal_mode = WAL;');
	}

	async migrate(): Promise<void> {
		for (const ddl of migrations) {
			await this.run(ddl);
		}
		await this.ensureLoginColumn();
	}

	async ensureDefaultAdmin(login: string, password: string): Promise<void> {
		const normalized = login.trim().toLowerCase();
		const existing = await this.get<{ id: number }>(
			'SELECT id FROM users WHERE LOWER(login) = ? OR LOWER(login) LIKE ? LIMIT 1',
			[normalized, `${normalized}@%`]
		);
		const hash = await bcrypt.hash(password, 10);
		if (!existing) {
			await this.run('INSERT INTO users (name, login, password_hash, role) VALUES (?, ?, ?, ?)', [
				'Administrador',
				normalized,
				hash,
				'admin',
			]);
			return;
		}
		await this.run('UPDATE users SET login = ?, password_hash = ? WHERE id = ?', [normalized, hash, existing.id]);
	}

	async verifyLogin(login: string, password: string): Promise<{ id: number; name: string; login: string; role: string } | null> {
		const normalized = login.trim().toLowerCase();
		if (!normalized) {
			return null;
		}
		const base = normalized.includes('@') ? normalized.split('@')[0] : normalized;
		let user = await this.get<{ id: number; name: string; login: string; password_hash: string; role: string }>(
			'SELECT id, name, login, password_hash, role FROM users WHERE LOWER(login) = ? LIMIT 1',
			[normalized]
		);
		if (!user && base !== normalized) {
			user = await this.get<{ id: number; name: string; login: string; password_hash: string; role: string }>(
				'SELECT id, name, login, password_hash, role FROM users WHERE LOWER(login) = ? LIMIT 1',
				[base]
			);
		}
		if (!user && !normalized.includes('@')) {
			user = await this.get<{ id: number; name: string; login: string; password_hash: string; role: string }>(
				'SELECT id, name, login, password_hash, role FROM users WHERE LOWER(login) LIKE ? LIMIT 1',
				[`${base}@%`]
			);
		}
		if (!user) {
			return null;
		}
		const valid = await bcrypt.compare(password, user.password_hash);
		if (!valid) {
			return null;
		}
		return { id: user.id, name: user.name, login: user.login, role: user.role };
	}

	async listStorageUnits(): Promise<StorageUnitRecord[]> {
		const rows = await this.all<StorageUnitRow>('SELECT * FROM storage_units ORDER BY updated_at DESC');
		return rows.map((row) => ({
			...row,
			metadata: row.metadata ? JSON.parse(row.metadata) : null,
		}));
	}

	async createStorageUnit(payload: StoragePayload): Promise<StorageUnitRecord> {
		const now = new Date().toISOString();
		const metadata = payload.metadata ? JSON.stringify(payload.metadata) : null;
		const runResult = await this.run(
			`INSERT INTO storage_units (label, type, section, capacity, occupancy, metadata, created_at, updated_at)
			 VALUES (?, ?, ?, ?, 0, ?, ?, ?)`,
			[
				payload.label.trim(),
				payload.type,
				payload.section ?? null,
				payload.capacity ?? 0,
				metadata,
				now,
				now,
			]
		);
		const id = runResult.lastID as number;
		const record = await this.get<StorageUnitRow>('SELECT * FROM storage_units WHERE id = ?', [id]);
		if (!record) {
			throw new Error('Falha ao criar unidade de guarda.');
		}
		return {
			...record,
			metadata: record.metadata ? JSON.parse(record.metadata) : null,
		};
	}

	async listMovements(limit = 25): Promise<MovementRecord[]> {
		return this.all<MovementRecord>(
			'SELECT * FROM movements ORDER BY datetime(created_at) DESC LIMIT ?',
			[limit]
		);
	}

	async recordMovement(payload: MovementPayload & { actor: string }): Promise<MovementRecord> {
		const now = new Date().toISOString();
		const runResult = await this.run(
			`INSERT INTO movements (reference, item_label, from_unit, to_unit, action, note, actor, created_at)
			 VALUES (?, ?, ?, ?, ?, ?, ?, ?)` ,
			[
				payload.reference ?? null,
				payload.item_label ?? null,
				payload.from_unit ?? null,
				payload.to_unit ?? null,
				payload.action,
				payload.note ?? null,
				payload.actor,
				now,
			]
		);
		const id = runResult.lastID as number;
		const record = await this.get<MovementRecord>('SELECT * FROM movements WHERE id = ?', [id]);
		if (!record) {
			throw new Error('Falha ao registrar movimentação.');
		}
		return record;
	}

	async snapshot(): Promise<SnapshotSummary> {
		const counters = await this.all<{ type: string; total: number }>(
			'SELECT type, COUNT(1) as total FROM storage_units GROUP BY type'
		);
		const template: Record<StorageUnitType, number> = {
			PASTA: 0,
			ENVELOPE: 0,
			GAVETEIRO: 0,
			CAIXA: 0,
		};
		for (const row of counters) {
			const kind = row.type as StorageUnitType;
			if (kind in template) {
				template[kind] = row.total;
			}
		}
		const totalUnits = Object.values(template).reduce((acc, value) => acc + value, 0);
		const today = await this.get<{ total: number }>(
			"SELECT COUNT(1) as total FROM movements WHERE DATE(created_at) = DATE('now')"
		);
		const lastMovement = await this.get<MovementRecord>(
			'SELECT * FROM movements ORDER BY datetime(created_at) DESC LIMIT 1'
		);
		return {
			totalUnits,
			unitsByType: template,
			movementsToday: today?.total ?? 0,
			lastMovement: lastMovement ?? null,
		};
	}

	private async ensureLoginColumn(): Promise<void> {
		const columns = await this.all<{ name: string }>('PRAGMA table_info(users)');
		const hasLogin = columns.some((column) => column.name === 'login');
		const hasEmail = columns.some((column) => column.name === 'email');
		if (!hasLogin && hasEmail) {
			await this.run('ALTER TABLE users RENAME COLUMN email TO login;');
		}
	}

	private run(sql: string, params: unknown[] = []): Promise<RunResult> {
		return new Promise((resolve, reject) => {
			this.assertDb().run(sql, params, function (this: RunResult, error: Error | null) {
				if (error) {
					reject(error);
					return;
				}
				resolve({ changes: this.changes, lastID: this.lastID });
			});
		});
	}

	private exec(sql: string): Promise<void> {
		return new Promise((resolve, reject) => {
			this.assertDb().exec(sql, (error: Error | null) => {
				if (error) {
					reject(error);
					return;
				}
				resolve();
			});
		});
	}

	private get<T>(sql: string, params: unknown[] = []): Promise<T | undefined> {
		return new Promise((resolve, reject) => {
			this.assertDb().get(sql, params, (error: Error | null, row: T) => {
				if (error) {
					reject(error);
					return;
				}
				resolve(row);
			});
		});
	}

	private all<T>(sql: string, params: unknown[] = []): Promise<T[]> {
		return new Promise((resolve, reject) => {
			this.assertDb().all(sql, params, (error: Error | null, rows: T[]) => {
				if (error) {
					reject(error);
					return;
				}
				resolve(rows);
			});
		});
	}

	private assertDb(): SqliteDatabase {
		if (!this.db) {
			throw new Error('Banco de dados não inicializado.');
		}
		return this.db;
	}
}
