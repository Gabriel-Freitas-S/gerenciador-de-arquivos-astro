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
    login(credentials: { login: string; password: string }) {
        return invoke<ApiResponse<LoginResult>>('auth_login', { payload: credentials });
    },
    session(token: string) {
        return invoke<ApiResponse<LoginResult>>('auth_session', { payload: { token } });
    },
    logout(token: string) {
        return invoke<ApiResponse<null>>('auth_logout', { payload: { token } });
    },
    storage: {
        list(token: string) {
            return invoke<ApiResponse<StorageUnitRecord[]>>('storage_list', { payload: { token } });
        },
        create(token: string, data: StoragePayload) {
            return invoke<ApiResponse<{ unit: StorageUnitRecord; snapshot: SnapshotSummary }>>('storage_create', {
                payload: {
                    token,
                    data,
                }
            });
        },
    },
    movements: {
        list(token: string) {
            return invoke<ApiResponse<MovementRecord[]>>('movements_list', { payload: { token } });
        },
        record(token: string, data: MovementPayload) {
            return invoke<ApiResponse<{ movement: MovementRecord; snapshot: SnapshotSummary }>>(
                'movements_record',
                {
                    payload: {
                        token,
                        data,
                    }
                }
            );
        },
    },
};
