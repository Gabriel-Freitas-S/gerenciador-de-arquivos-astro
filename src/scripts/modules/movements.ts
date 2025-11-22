import { archiveApi } from '../archive-api.js';
import { dom, showToast, renderSummary } from './ui.js';
import type { AppState } from '../app.js';

export async function refreshMovements(state: AppState) {
    if (!state.token) return;
    const response = await archiveApi.movements.list(state.token);
    if (response.success && response.data) {
        state.movements = response.data;
        renderMovements(state);
    }
}

export async function handleMovement(state: AppState) {
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
    renderMovements(state);
    renderSummary(state.snapshot);
    dom.movementForm.reset();
    showToast('Movimentação registrada');
}

function renderMovements(state: AppState) {
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
