import { archiveApi } from '../archive-api.js';
import { dom, setAuthError, updateAuthUI, setGuardState, setSectionVisibility, showToast, renderSummary } from './ui.js';
import type { AppState } from '../app.js';
import { refreshStorage } from './storage.js';
import { refreshMovements } from './movements.js';

const STORAGE_KEY = 'archive_token';

export async function handleLogin(state: AppState) {
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
        await Promise.all([refreshStorage(state), refreshMovements(state)]);
        renderSummary(state.snapshot);
        setGuardState(false);
        setSectionVisibility(true);
        showToast('Sessão iniciada');
    } catch (error) {
        console.error('Login error:', error);
        setAuthError('Erro de conexão com o servidor. Verifique se o aplicativo está rodando corretamente.');
    }
}

export async function handleLogout(state: AppState) {
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
    // Clear UI
    if (dom.storageTable) dom.storageTable.innerHTML = '';
    if (dom.movementList) dom.movementList.innerHTML = '';
    renderSummary(null);
}

export async function resumeSession(state: AppState) {
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
    await Promise.all([refreshStorage(state), refreshMovements(state)]);
    renderSummary(state.snapshot);
    setGuardState(false);
    setSectionVisibility(true);
}
