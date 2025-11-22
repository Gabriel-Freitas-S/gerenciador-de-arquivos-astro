import { archiveApi } from './archive-api.js';
import type {
	MovementRecord,
	SnapshotSummary,
	StorageUnitRecord,
	StorageUnitType,
	LoginResult,
} from '../types/archive.js';

const STORAGE_KEY = 'archive::token';

const typeLabels: Record<StorageUnitType, string> = {
	PASTA: 'Pasta suspensa',
	ENVELOPE: 'Envelope',
	GAVETEIRO: 'Gaveteiro',
	CAIXA: 'Caixa/Arquivo',
};

type UserProfile = LoginResult['profile'];

type AppState = {
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

const dom = {
	loginForm: document.querySelector<HTMLFormElement>('#login-form'),
	logoutButton: document.querySelector<HTMLButtonElement>('#logout-button'),
	identifierInput: document.querySelector<HTMLInputElement>('#login-identifier'),
	passwordInput: document.querySelector<HTMLInputElement>('#login-password'),
	welcomeBox: document.querySelector<HTMLDivElement>('#welcome-box'),
	authInfo: document.querySelector<HTMLDivElement>('#auth-info'),
	cards: document.querySelectorAll<HTMLDivElement>('[data-card]'),
	storageTable: document.querySelector<HTMLTableSectionElement>('#storage-table-body'),
	movementList: document.querySelector<HTMLUListElement>('#movement-list'),
	storageForm: document.querySelector<HTMLFormElement>('#storage-form'),
	movementForm: document.querySelector<HTMLFormElement>('#movement-form'),
	movementSelects: document.querySelectorAll<HTMLSelectElement>('[data-storage-select]'),
	authError: document.querySelector<HTMLSpanElement>('#auth-error'),
	toast: document.querySelector<HTMLDivElement>('#app-toast'),
	guardSections: document.querySelectorAll<HTMLElement>('[data-guarded]'),
	loginSection: document.querySelector<HTMLElement>('[data-section="login"]'),
	appSection: document.querySelector<HTMLElement>('[data-section="app"]'),
};

init();

function init() {
	bindEvents();
	setGuardState(!state.token);
	setSectionVisibility(Boolean(state.token));
	if (state.token) {
		void resumeSession();
	}
}

function bindEvents() {
	dom.loginForm?.addEventListener('submit', async (event) => {
		event.preventDefault();
		await handleLogin();
	});

	dom.logoutButton?.addEventListener('click', async () => {
		if (!state.token) return;
		await archiveApi.logout(state.token);
		state.token = null;
		state.snapshot = null;
		state.profile = null;
		state.storage = [];
		state.movements = [];
		localStorage.removeItem(STORAGE_KEY);
		updateAuthUI(null);
		setGuardState(true);
		setSectionVisibility(false);
		renderStorage();
		renderMovements();
		renderSummary();
	});

	dom.storageForm?.addEventListener('submit', async (event) => {
		event.preventDefault();
		await handleStorageCreation();
	});

	dom.movementForm?.addEventListener('submit', async (event) => {
		event.preventDefault();
		await handleMovement();
	});
}

async function resumeSession() {
	if (!state.token) return;
	const response = await archiveApi.session(state.token);
	if (!response.success || !response.data) {
		localStorage.removeItem(STORAGE_KEY);
		state.token = null;
		state.profile = null;
		updateAuthUI(null);
		setGuardState(true);
		setSectionVisibility(false);
		return;
	}
	state.snapshot = response.data.snapshot;
	state.profile = response.data.profile;
	updateAuthUI(response.data.profile);
	await Promise.all([refreshStorage(), refreshMovements()]);
	renderSummary();
	setGuardState(false);
	setSectionVisibility(true);
}

async function handleLogin() {
	if (!dom.identifierInput || !dom.passwordInput) return;
	const login = dom.identifierInput.value.trim();
	const password = dom.passwordInput.value;
	if (!login || !password) {
		setAuthError('Informe usuário e senha.');
		return;
	}
	setAuthError('');
	try {
		const response = await archiveApi.login({ login, password });
		if (!response.success || !response.data) {
			setAuthError(response.error ?? 'Falha ao autenticar.');
			return;
		}
		state.token = response.data.token;
		state.snapshot = response.data.snapshot;
		state.profile = response.data.profile;
		if (state.token) {
			localStorage.setItem(STORAGE_KEY, state.token);
		}
		updateAuthUI(response.data.profile);
		await Promise.all([refreshStorage(), refreshMovements()]);
		renderSummary();
		setGuardState(false);
		setSectionVisibility(true);
		showToast('Sessão iniciada');
	} catch (error) {
		console.error('Login error:', error);
		setAuthError('Erro de conexão com o servidor. Verifique se o aplicativo está rodando corretamente.');
	}
}

async function refreshStorage() {
	if (!state.token) return;
	const response = await archiveApi.storage.list(state.token);
	if (response.success && response.data) {
		state.storage = response.data;
		renderStorage();
		updateStorageOptions();
	}
}

async function refreshMovements() {
	if (!state.token) return;
	const response = await archiveApi.movements.list(state.token);
	if (response.success && response.data) {
		state.movements = response.data;
		renderMovements();
	}
}

async function handleStorageCreation() {
	if (!state.token || !dom.storageForm) return;
	const formData = new FormData(dom.storageForm);
	const payload = {
		label: String(formData.get('label') ?? '').trim(),
		type: (formData.get('type') as StorageUnitType) ?? 'PASTA',
		section: String(formData.get('section') ?? '').trim() || undefined,
		capacity: Number(formData.get('capacity') ?? 0) || 0,
	};
	if (!payload.label) {
		showToast('Informe um identificador para a unidade', true);
		return;
	}
	const response = await archiveApi.storage.create(state.token, payload);
	if (!response.success || !response.data) {
		showToast(response.error ?? 'Erro ao registrar unidade', true);
		return;
	}
	state.snapshot = response.data.snapshot;
	state.storage.unshift(response.data.unit);
	updateStorageOptions();
	renderStorage();
	renderSummary();
	dom.storageForm.reset();
	showToast('Unidade criada');
}

async function handleMovement() {
	if (!state.token || !dom.movementForm) return;
	const formData = new FormData(dom.movementForm);
	const payload = {
		action: String(formData.get('action') ?? '').trim(),
		reference: formData.get('reference')?.toString().trim() || undefined,
		item_label: formData.get('item_label')?.toString().trim() || undefined,
		from_unit: formData.get('from_unit')?.toString().trim() || undefined,
		to_unit: formData.get('to_unit')?.toString().trim() || undefined,
		note: formData.get('note')?.toString().trim() || undefined,
	};
	if (!payload.action) {
		showToast('Descreva a movimentação realizada', true);
		return;
	}
	const response = await archiveApi.movements.record(state.token, payload);
	if (!response.success || !response.data) {
		showToast(response.error ?? 'Erro ao registrar movimentação', true);
		return;
	}
	state.snapshot = response.data.snapshot;
	state.movements.unshift(response.data.movement);
	state.movements = state.movements.slice(0, 25);
	renderMovements();
	renderSummary();
	dom.movementForm.reset();
	showToast('Movimentação registrada');
}

function updateAuthUI(profile: UserProfile | null) {
	document.body.classList.toggle('is-auth', Boolean(profile));
	if (!profile) {
		if (dom.welcomeBox) dom.welcomeBox.textContent = 'Faça login para acompanhar seu arquivo físico.';
		return;
	}
	if (dom.welcomeBox) {
		dom.welcomeBox.textContent = `Olá, ${profile.name}`;
	}
	if (dom.authInfo) {
		dom.authInfo.textContent = profile.login;
	}
}

function setGuardState(locked: boolean) {
	dom.guardSections.forEach((section) => {
		if (locked) {
			section.dataset.locked = 'true';
		} else {
			delete section.dataset.locked;
		}
	});
	setFormDisabled(dom.storageForm, locked);
	setFormDisabled(dom.movementForm, locked);
}

function setFormDisabled(form: HTMLFormElement | null, disabled: boolean) {
	if (!form) return;
	Array.from(form.elements).forEach((element) => {
		if (
			element instanceof HTMLInputElement ||
			element instanceof HTMLSelectElement ||
			element instanceof HTMLTextAreaElement ||
			element instanceof HTMLButtonElement
		) {
			element.disabled = disabled;
		}
	});
}

function setSectionVisibility(isAuthenticated: boolean) {
	if (dom.loginSection) {
		dom.loginSection.dataset.hidden = isAuthenticated ? 'true' : 'false';
	}
	if (dom.appSection) {
		dom.appSection.dataset.hidden = isAuthenticated ? 'false' : 'true';
	}
}

function renderSummary() {
	if (!state.snapshot) {
		dom.cards.forEach((card) => {
			const label = card.querySelector('strong');
			if (label) {
				label.textContent = '--';
			}
		});
		return;
	}
	const totalCard = document.querySelector<HTMLDivElement>('[data-card="total"] strong');
	const todayCard = document.querySelector<HTMLDivElement>('[data-card="today"] strong');
	const unitCard = document.querySelector<HTMLDivElement>('[data-card="units"] strong');
	const lastCard = document.querySelector<HTMLDivElement>('[data-card="last"] strong');
	if (totalCard) totalCard.textContent = String(state.snapshot.totalUnits);
	if (todayCard) todayCard.textContent = String(state.snapshot.movementsToday);
	if (unitCard)
		unitCard.textContent = `${state.snapshot.unitsByType.PASTA + state.snapshot.unitsByType.ENVELOPE} pastas/envelopes`;
	if (lastCard)
		lastCard.textContent = state.snapshot.lastMovement?.action ?? 'Sem registros';
}

function renderStorage() {
	if (!dom.storageTable) return;
	dom.storageTable.innerHTML = '';
	if (state.storage.length === 0) {
		dom.storageTable.innerHTML =
			'<tr><td class="px-4 py-3 text-center text-white/60" colspan="5">Nenhuma unidade cadastrada ainda.</td></tr>';
		return;
	}
	for (const unit of state.storage) {
		const row = document.createElement('tr');
		row.className = 'border-t border-white/5';
		row.innerHTML = `
			<td class="px-4 py-3">${unit.label}</td>
			<td class="px-4 py-3">${typeLabels[unit.type]}</td>
			<td class="px-4 py-3">${unit.section ?? '—'}</td>
			<td class="px-4 py-3">${unit.occupancy}/${unit.capacity}</td>
			<td class="px-4 py-3">${new Date(unit.updated_at).toLocaleString('pt-BR')}</td>
		`;
		dom.storageTable.appendChild(row);
	}
}

function renderMovements() {
	if (!dom.movementList) return;
	dom.movementList.innerHTML = '';
	if (state.movements.length === 0) {
		dom.movementList.innerHTML =
			'<li class="rounded-2xl border border-white/10 bg-white/5 p-4 text-center text-sm text-white/70">Registre a primeira movimentação do dia.</li>';
		return;
	}
	for (const movement of state.movements) {
		const item = document.createElement('li');
		item.className =
			'rounded-2xl border border-white/10 bg-white/5 p-4 text-white/80 shadow-inner shadow-white/5';
		item.innerHTML = `
			<strong class="text-base font-semibold text-white">${movement.action}</strong>
			<span class="mt-1 block text-xs text-white/60">${movement.reference ?? 'Sem ref.'} · ${new Date(movement.created_at).toLocaleString('pt-BR')}</span>
			<p class="mt-2 text-sm text-white/80">${movement.note ?? 'Sem observações.'}</p>
		`;
		dom.movementList.appendChild(item);
	}
}

function updateStorageOptions() {
	const options = state.storage.map((unit) => `<option value="${unit.label}">${unit.label}</option>`).join('');
	dom.movementSelects.forEach((select) => {
		select.innerHTML = '<option value="">—</option>' + options;
	});
}

function setAuthError(message: string) {
	if (dom.authError) {
		dom.authError.textContent = message;
		dom.authError.hidden = !message;
	}
}

function showToast(message: string, isError = false) {
	if (!dom.toast) return;
	dom.toast.textContent = message;
	dom.toast.dataset.type = isError ? 'error' : 'info';
	dom.toast.classList.add('visible');
	setTimeout(() => dom.toast?.classList.remove('visible'), 2600);
}
