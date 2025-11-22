import fs from 'node:fs';
import path from 'node:path';
import url from 'node:url';
import { app, BrowserWindow, ipcMain, shell } from 'electron';
import dotenv from 'dotenv';
import { z } from 'zod';
import { ArchiveDatabase } from './database.js';
import { SessionStore } from './sessions.js';
import type { MovementPayload, StoragePayload, StorageUnitType } from './types.js';

dotenv.config();

const db = new ArchiveDatabase();
const sessions = new SessionStore();

const storageTypes = ['PASTA', 'ENVELOPE', 'GAVETEIRO', 'CAIXA'] as const satisfies readonly StorageUnitType[];
const storageTypeSchema = z.enum(storageTypes);

const resolveAssetPath = (file: string): string => {
	const baseDir = app.isPackaged ? process.resourcesPath : app.getAppPath();
	const distCandidate = path.join(baseDir, 'dist', file);
	if (fs.existsSync(distCandidate)) {
		return distCandidate;
	}
	const publicCandidate = path.join(baseDir, 'public', file);
	if (fs.existsSync(publicCandidate)) {
		return publicCandidate;
	}
	return path.join(process.cwd(), 'public', file);
};

const loginSchema = z.object({
	login: z.string().min(3, 'Informe o usuário'),
	password: z.string().min(4, 'Senha inválida'),
});

const tokenSchema = z.object({ token: z.string().min(10) });

const storageSchema = z.object({
	label: z.string().min(2, 'Informe um identificador'),
	type: storageTypeSchema,
	section: z.string().min(2).optional(),
	capacity: z.number().int().min(0).default(0),
	metadata: z.record(z.unknown()).optional(),
});

const movementSchema = z.object({
	action: z.string().min(3, 'Descreva a movimentação'),
	reference: z.string().optional(),
	item_label: z.string().optional(),
	from_unit: z.string().optional(),
	to_unit: z.string().optional(),
	note: z.string().optional(),
});

const formatIssues = (error: z.ZodError) => error.issues.map((issue) => issue.message).join(', ');

const isMac = process.platform === 'darwin';

async function bootstrap(): Promise<void> {
	const key = process.env.ARCHIVE_DB_KEY;
	if (!key) {
		throw new Error('Configure a variável ARCHIVE_DB_KEY no arquivo .env');
	}
	await db.connect(path.join(app.getPath('userData'), 'archive.sqlite'), key);
	await db.migrate();
	const defaultLogin = process.env.ARCHIVE_DEFAULT_ADMIN_LOGIN ?? 'admin';
	await db.ensureDefaultAdmin(defaultLogin, process.env.ARCHIVE_DEFAULT_ADMIN_PASSWORD ?? 'admin');
}

async function createWindow(): Promise<BrowserWindow> {
	const preloadFile = url.fileURLToPath(new URL(/* @vite-ignore */ 'preload.mjs', import.meta.url));
	const iconPath = resolveAssetPath('app-icon.png');
	const win = new BrowserWindow({
		width: 1280,
		height: 900,
		minWidth: 1100,
		minHeight: 720,
		title: 'Arquivo Inteligente',
		backgroundColor: '#0f1117',
		icon: iconPath,
		webPreferences: {
			preload: preloadFile,
		},
	});

	if (process.env.VITE_DEV_SERVER_URL) {
		await win.loadURL(process.env.VITE_DEV_SERVER_URL);
		win.webContents.openDevTools({ mode: 'detach' });
	} else {
		await win.loadFile(path.join(app.getAppPath(), 'dist', 'index.html'));
	}

	win.webContents.setWindowOpenHandler(({ url: target }) => {
		void shell.openExternal(target);
		return { action: 'deny' };
	});

	return win;
}

function registerIpc(): void {
	ipcMain.handle('auth:login', async (_event, payload: unknown) => {
		try {
			const parsed = loginSchema.safeParse(payload);
			if (!parsed.success) {
				return { success: false, error: formatIssues(parsed.error) };
			}
			const user = await db.verifyLogin(parsed.data.login, parsed.data.password);
			if (!user) {
				return { success: false, error: 'Credenciais inválidas' };
			}
			const session = sessions.create(user);
			const snapshot = await db.snapshot();
			return { success: true, data: { token: session.token, profile: session.profile, snapshot } };
		} catch (error) {
			return { success: false, error: (error as Error).message };
		}
	});

	ipcMain.handle('auth:session', async (_event, payload: unknown) => {
		try {
			const parsed = tokenSchema.safeParse(payload);
			if (!parsed.success) {
				return { success: false, error: 'Sessão inválida' };
			}
			const session = sessions.require(parsed.data.token);
			const snapshot = await db.snapshot();
			return { success: true, data: { token: session.token, profile: session.profile, snapshot } };
		} catch (error) {
			return { success: false, error: (error as Error).message };
		}
	});

	ipcMain.handle('auth:logout', (_event, payload: unknown) => {
		const parsed = tokenSchema.safeParse(payload);
		if (parsed.success) {
			sessions.revoke(parsed.data.token);
		}
		return { success: true };
	});

	ipcMain.handle('storage:list', async (_event, payload: unknown) => {
		try {
			const parsed = tokenSchema.safeParse(payload);
			if (!parsed.success) {
				return { success: false, error: 'Sessão inválida' };
			}
			sessions.require(parsed.data.token);
			const units = await db.listStorageUnits();
			return { success: true, data: units };
		} catch (error) {
			return { success: false, error: (error as Error).message };
		}
	});

	ipcMain.handle('storage:create', async (_event, payload: unknown) => {
		try {
			const parsed = z
				.object({ token: z.string().min(10), data: storageSchema })
				.safeParse(payload);
			if (!parsed.success) {
				return { success: false, error: formatIssues(parsed.error) };
			}
			const session = sessions.require(parsed.data.token);
			const unit = await db.createStorageUnit(parsed.data.data as StoragePayload);
			await db.recordMovement({
				action: 'Cadastro de unidade',
				actor: session.profile.name,
				note: `Unidade ${parsed.data.data.label} criada`,
			});
			const snapshot = await db.snapshot();
			return { success: true, data: { unit, snapshot } };
		} catch (error) {
			return { success: false, error: (error as Error).message };
		}
	});

	ipcMain.handle('movements:list', async (_event, payload: unknown) => {
		try {
			const parsed = tokenSchema.safeParse(payload);
			if (!parsed.success) {
				return { success: false, error: 'Sessão inválida' };
			}
			sessions.require(parsed.data.token);
			const movements = await db.listMovements();
			return { success: true, data: movements };
		} catch (error) {
			return { success: false, error: (error as Error).message };
		}
	});

	ipcMain.handle('movements:record', async (_event, payload: unknown) => {
		try {
			const parsed = z
				.object({ token: z.string().min(10), data: movementSchema })
				.safeParse(payload);
			if (!parsed.success) {
				return { success: false, error: formatIssues(parsed.error) };
			}
			const session = sessions.require(parsed.data.token);
			const movement = await db.recordMovement({
				...(parsed.data.data as MovementPayload),
				actor: session.profile.name,
			});
			const snapshot = await db.snapshot();
			return { success: true, data: { movement, snapshot } };
		} catch (error) {
			return { success: false, error: (error as Error).message };
		}
	});
}

app.whenReady().then(async () => {
	await bootstrap();
	registerIpc();
	await createWindow();

	app.on('activate', async () => {
		if (BrowserWindow.getAllWindows().length === 0) {
			await createWindow();
		}
	});
});

app.on('window-all-closed', () => {
	if (!isMac) {
		app.quit();
	}
});
