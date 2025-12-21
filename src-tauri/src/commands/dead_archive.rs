use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, ArchiveBoxCreatePayload, ArchiveBoxRecord, ArchiveItemRecord,
    ArchiveTransferPayload, DisposalCandidate, DisposalRegisterPayload, DisposalTerm, TokenPayload,
};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn create_archive_box(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: ArchiveBoxCreatePayload,
) -> Result<ApiResponse<ArchiveBoxRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.create_archive_box(&payload.data).await {
        Ok(archive_box) => Ok(ApiResponse::success(archive_box)),
        Err(e) => Ok(ApiResponse::error(format!("Erro ao criar caixa: {}", e))),
    }
}

#[tauri::command]
pub async fn list_archive_boxes(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<ArchiveBoxRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.list_archive_boxes().await {
        Ok(boxes) => Ok(ApiResponse::success(boxes)),
        Err(e) => Ok(ApiResponse::error(format!("Erro ao listar caixas: {}", e))),
    }
}

#[tauri::command]
pub async fn transfer_to_archive(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: ArchiveTransferPayload,
) -> Result<ApiResponse<ArchiveItemRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db
        .transfer_to_archive(
            payload.employee_id,
            payload.box_id,
            payload.disposal_eligible_date.as_deref(),
            &session.profile.login,
        )
        .await
    {
        Ok(item) => Ok(ApiResponse::success(item)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao transferir para arquivo: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_disposal_candidates(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<DisposalCandidate>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.get_disposal_candidates().await {
        Ok(candidates) => Ok(ApiResponse::success(candidates)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar candidatos: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn register_disposal(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: DisposalRegisterPayload,
) -> Result<ApiResponse<DisposalTerm>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db
        .register_disposal(&payload.item_ids, payload.term_number.as_deref())
        .await
    {
        Ok(term) => Ok(ApiResponse::success(term)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao registrar descarte: {}",
            e
        ))),
    }
}
