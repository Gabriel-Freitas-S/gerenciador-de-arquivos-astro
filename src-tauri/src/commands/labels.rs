use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{ApiResponse, LabelData, LabelRequestPayload};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn generate_folder_label(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: LabelRequestPayload,
) -> Result<ApiResponse<LabelData>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.generate_folder_label(payload.entity_id).await {
        Ok(label) => Ok(ApiResponse::success(label)),
        Err(e) => Ok(ApiResponse::error(format!("Erro ao gerar etiqueta: {}", e))),
    }
}

#[tauri::command]
pub async fn generate_envelope_label(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: LabelRequestPayload,
) -> Result<ApiResponse<LabelData>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    // Use format as category, default to "Pessoal"
    let category = payload.format.as_deref().unwrap_or("Pessoal");

    match db
        .generate_envelope_label(payload.entity_id, category)
        .await
    {
        Ok(label) => Ok(ApiResponse::success(label)),
        Err(e) => Ok(ApiResponse::error(format!("Erro ao gerar etiqueta: {}", e))),
    }
}

#[tauri::command]
pub async fn generate_box_label(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: LabelRequestPayload,
) -> Result<ApiResponse<LabelData>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.generate_box_label(payload.entity_id).await {
        Ok(label) => Ok(ApiResponse::success(label)),
        Err(e) => Ok(ApiResponse::error(format!("Erro ao gerar etiqueta: {}", e))),
    }
}
