use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{ApiResponse, MovementPayload, MovementRecord, SnapshotSummary, TokenPayload};
use tauri::State;

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
    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };
    if payload.data.action.trim().len() < 3 {
        return Ok(ApiResponse::error("Descreva a movimentação"));
    }
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
