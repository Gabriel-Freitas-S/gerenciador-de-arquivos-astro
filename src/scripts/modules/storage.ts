import { archiveApi } from '../archive-api.js';
import { dom, showToast, renderSummary } from './ui.js';
import type { AppState } from '../app.js';
import type { StorageUnitType } from '../../types/archive.js';

export const typeLabels: Record<StorageUnitType, string> = {
    PASTA: 'Pasta suspensa',
    ENVELOPE: 'Envelope',
    GAVETEIRO: 'Gaveteiro',
    CAIXA: 'Caixa/Arquivo',
};

export async function refreshStorage(state: AppState) {
    if (!state.token) return;
    const response = await archiveApi.storage.list(state.token);
    if (response.success && response.data) {
        state.storage = response.data;
        renderStorage(state);
        updateStorageOptions(state);
    }
}

export async function handleStorageCreation(state: AppState) {
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
    updateStorageOptions(state);
    renderStorage(state);
    renderSummary(state.snapshot);
    dom.storageForm.reset();
    showToast('Unidade criada');
}

function renderStorage(state: AppState) {
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

function updateStorageOptions(state: AppState) {
    const options = state.storage.map((unit) => `<option value="${unit.label}">${unit.label}</option>`).join('');
    dom.movementSelects.forEach((select) => {
        select.innerHTML = '<option value="">—</option>' + options;
    });
}
