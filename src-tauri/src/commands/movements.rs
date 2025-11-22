use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{ApiResponse, MovementPayload, MovementRecord, SnapshotSummary, TokenPayload};
use tauri::State;
use validator::Validate;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MovementRecordResponse {
    pub movement: MovementRecord,
    pub snapshot: SnapshotSummary,
}

#[tauri::command]
pub async fn movements_list(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<MovementRecord>>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }
    match db.list_movements(25).await {
        Ok(records) => Ok(ApiResponse::success(records)),
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}

#[tauri::command]
pub async fn movements_record(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: MovementPayload,
) -> Result<ApiResponse<MovementRecordResponse>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };
    match db
        .record_movement(&session.profile.name, &payload.data)
        .await
    {
        Ok(movement) => match db.snapshot().await {
            Ok(snapshot) => Ok(ApiResponse::success(MovementRecordResponse {
                movement,
                snapshot,
            })),
            Err(error) => Ok(ApiResponse::error(error.to_string())),
        },
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}
