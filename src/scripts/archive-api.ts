import { invoke } from '@tauri-apps/api/core';
import type {
    ApiResponse,
    LoginResult,
    MovementPayload,
    MovementRecord,
    SnapshotSummary,
    StoragePayload,
    StorageUnitRecord,
} from '../types/archive.js';

export const archiveApi = {
    login(payload: { login: string; password: string }) {
        return invoke<ApiResponse<LoginResult>>('auth_login', payload);
    },
    session(token: string) {
        return invoke<ApiResponse<LoginResult>>('auth_session', { token });
    },
    logout(token: string) {
        return invoke<ApiResponse<null>>('auth_logout', { token });
    },
    storage: {
        list(token: string) {
            return invoke<ApiResponse<StorageUnitRecord[]>>('storage_list', { token });
        },
        create(token: string, data: StoragePayload) {
            return invoke<ApiResponse<{ unit: StorageUnitRecord; snapshot: SnapshotSummary }>>('storage_create', {
                token,
                data,
            });
        },
    },
    movements: {
        list(token: string) {
            return invoke<ApiResponse<MovementRecord[]>>('movements_list', { token });
        },
        record(token: string, data: MovementPayload) {
            return invoke<ApiResponse<{ movement: MovementRecord; snapshot: SnapshotSummary }>>(
                'movements_record',
                {
                    token,
                    data,
                }
            );
        },
    },
};
