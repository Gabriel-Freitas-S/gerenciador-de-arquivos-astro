import type { UserProfile, SnapshotSummary } from '../../types/archive.js';

export const dom = {
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

export function showToast(message: string, isError = false) {
    if (!dom.toast) return;
    dom.toast.textContent = message;
    dom.toast.dataset.type = isError ? 'error' : 'info';
    dom.toast.classList.add('visible');
    setTimeout(() => dom.toast?.classList.remove('visible'), 2600);
}

export function setAuthError(message: string) {
    if (dom.authError) {
        dom.authError.textContent = message;
        dom.authError.hidden = !message;
    }
}

export function updateAuthUI(profile: UserProfile | null) {
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

export function setGuardState(locked: boolean) {
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

export function setSectionVisibility(isAuthenticated: boolean) {
    // Toggle is-auth class on body for CSS rules
    document.body.classList.toggle('is-auth', isAuthenticated);

    if (dom.loginSection) {
        (dom.loginSection as HTMLElement).style.display = isAuthenticated ? 'none' : 'flex';
    }
    if (dom.appSection) {
        (dom.appSection as HTMLElement).style.display = isAuthenticated ? 'flex' : 'none';
    }

    // Scroll to top
    window.scrollTo(0, 0);
}

export function renderSummary(snapshot: SnapshotSummary | null) {
    if (!snapshot) {
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
    if (totalCard) totalCard.textContent = String(snapshot.totalUnits);
    if (todayCard) todayCard.textContent = String(snapshot.movementsToday);
    if (unitCard)
        unitCard.textContent = `${snapshot.unitsByType.PASTA + snapshot.unitsByType.ENVELOPE} pastas/envelopes`;
    if (lastCard)
        lastCard.textContent = snapshot.lastMovement?.action ?? 'Sem registros';
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
