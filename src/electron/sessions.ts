import crypto from 'node:crypto';

export interface SessionProfile {
	id: number;
	name: string;
	login: string;
	role: string;
}

export interface ActiveSession {
	token: string;
	profile: SessionProfile;
	issuedAt: number;
}

export class SessionStore {
	private sessions = new Map<string, ActiveSession>();

	create(profile: SessionProfile): ActiveSession {
		const token = crypto.randomUUID();
		const session: ActiveSession = {
			token,
			profile,
			issuedAt: Date.now(),
		};
		this.sessions.set(token, session);
		return session;
	}

	get(token: string | undefined | null): ActiveSession | undefined {
		if (!token) return undefined;
		return this.sessions.get(token);
	}

	require(token: string | undefined | null): ActiveSession {
		const session = this.get(token);
		if (!session) {
			throw new Error('Sessão inválida. Faça login novamente.');
		}
		return session;
	}

	revoke(token: string): void {
		this.sessions.delete(token);
	}

	revokeAll(): void {
		this.sessions.clear();
	}
}
