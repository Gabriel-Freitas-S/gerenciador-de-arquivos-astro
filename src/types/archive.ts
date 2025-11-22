export type StorageUnitType = 'PASTA' | 'ENVELOPE' | 'GAVETEIRO' | 'CAIXA';

export interface StorageUnitRecord {
    id: number;
    label: string;
    type: StorageUnitType;
    section: string | null;
    capacity: number;
    occupancy: number;
    metadata: Record<string, unknown> | null;
    created_at: string;
    updated_at: string;
}

export interface MovementRecord {
    id: number;
    reference: string | null;
    item_label: string | null;
    from_unit: string | null;
    to_unit: string | null;
    action: string;
    note: string | null;
    actor: string;
    created_at: string;
}

export interface SnapshotSummary {
    totalUnits: number;
    unitsByType: Record<StorageUnitType, number>;
    movementsToday: number;
    lastMovement: MovementRecord | null;
}

export interface LoginResult {
    token: string;
    profile: {
        id: number;
        name: string;
        login: string;
        role: string;
    };
    snapshot: SnapshotSummary;
}

export interface StoragePayload {
    label: string;
    type: StorageUnitType;
    section?: string;
    capacity?: number;
    metadata?: Record<string, unknown> | null;
}

export interface MovementPayload {
    reference?: string;
    item_label?: string;
    from_unit?: string;
    to_unit?: string;
    action: string;
    note?: string;
}

export interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
}
