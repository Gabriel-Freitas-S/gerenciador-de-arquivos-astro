use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, MovementData, SnapshotSummary, StorageCreatePayload, StorageUnitRecord,
    TokenPayload,
};
use tauri::State;
use validator::Validate;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StorageCreateResponse {
    pub unit: StorageUnitRecord,
    pub snapshot: SnapshotSummary,
}

#[tauri::command]
pub async fn storage_list(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<StorageUnitRecord>>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }
    match db.list_storage_units().await {
        Ok(units) => Ok(ApiResponse::success(units)),
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}

#[tauri::command]
pub async fn storage_create(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: StorageCreatePayload,
) -> Result<ApiResponse<StorageCreateResponse>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };
    match db.create_storage_unit(&payload.data).await {
        Ok(unit) => {
            let movement = MovementData {
                action: "Cadastro de unidade".into(),
                reference: payload.data.section.clone(),
                item_label: Some(unit.label.clone()),
                from_unit: None,
                to_unit: payload.data.section.clone(),
                note: Some(format!("Unidade {} criada", unit.label)),
            };
            let _ = db.record_movement(&session.profile.name, &movement).await;
            match db.snapshot().await {
                Ok(snapshot) => Ok(ApiResponse::success(StorageCreateResponse {
                    unit,
                    snapshot,
                })),
                Err(error) => Ok(ApiResponse::error(error.to_string())),
            }
        }
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}
