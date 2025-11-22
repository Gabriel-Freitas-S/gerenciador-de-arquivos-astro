/// <reference types="astro/client" />

import type { MovementRecord, SnapshotSummary, StorageUnitRecord } from './electron/types';

declare global {
	interface ArchiveLoginResponse {
		success: boolean;
		error?: string;
		data?: {
			token: string;
			profile: {
				id: number;
				name: string;
				login: string;
				role: string;
			};
			snapshot: SnapshotSummary;
		};
	}

	interface ArchiveListResponse<T> {
		success: boolean;
		error?: string;
		data?: T;
	}

	interface Window {
		archive: {
			login: (credentials: { login: string; password: string }) => Promise<ArchiveLoginResponse>;
			session: (token: string) => Promise<ArchiveLoginResponse>;
			logout: (token: string) => Promise<{ success: boolean }>;
			storage: {
				list: (token: string) => Promise<ArchiveListResponse<StorageUnitRecord[]>>;
				create: (
					token: string,
					data: {
						label: string;
						type: 'PASTA' | 'ENVELOPE' | 'GAVETEIRO' | 'CAIXA';
						section?: string;
						capacity?: number;
						metadata?: Record<string, unknown> | null;
					}
				) => Promise<ArchiveListResponse<{ unit: StorageUnitRecord; snapshot: SnapshotSummary }>>;
			};
			movements: {
				list: (token: string) => Promise<ArchiveListResponse<MovementRecord[]>>;
				record: (
					token: string,
					data: {
						action: string;
						reference?: string;
						item_label?: string;
						from_unit?: string;
						to_unit?: string;
						note?: string;
					}
				) => Promise<ArchiveListResponse<{ movement: MovementRecord; snapshot: SnapshotSummary }>>;
			};
		};
	}
}

export {};
