import type {
	MovementRecord,
	SnapshotSummary,
	StorageUnitRecord,
	LoginResult,
} from '../types/archive.js';
import { dom, setGuardState, setSectionVisibility } from './modules/ui.js';
import { handleLogin, handleLogout, resumeSession } from './modules/auth.js';
import { handleStorageCreation } from './modules/storage.js';
import { handleMovement } from './modules/movements.js';

const STORAGE_KEY = 'archive::token';

export type UserProfile = LoginResult['profile'];

export type AppState = {
	token: string | null;
	profile: UserProfile | null;
	storage: StorageUnitRecord[];
	movements: MovementRecord[];
	snapshot: SnapshotSummary | null;
};

const state: AppState = {
	token: localStorage.getItem(STORAGE_KEY),
	profile: null,
	storage: [],
	movements: [],
	snapshot: null,
};

init();

function init() {
	bindEvents();
	setGuardState(!state.token);
	setSectionVisibility(Boolean(state.token));
	if (state.token) {
		void resumeSession(state);
	}
}

function bindEvents() {
	dom.loginForm?.addEventListener('submit', async (event) => {
		event.preventDefault();
		await handleLogin(state);
	});

	dom.logoutButton?.addEventListener('click', async () => {
		await handleLogout(state);
	});

	dom.storageForm?.addEventListener('submit', async (event) => {
		event.preventDefault();
		await handleStorageCreation(state);
	});

	dom.movementForm?.addEventListener('submit', async (event) => {
		event.preventDefault();
		await handleMovement(state);
	});
}
