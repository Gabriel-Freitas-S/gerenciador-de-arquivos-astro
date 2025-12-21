use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, DocumentCategoryRecord, DocumentPayload, DocumentRecord, DocumentTypeRecord,
    EmployeeDocumentsPayload, IdPayload, TokenPayload,
};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn list_document_categories(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<DocumentCategoryRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.list_document_categories().await {
        Ok(categories) => Ok(ApiResponse::success(categories)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar categorias: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn list_document_types(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: IdPayload,
) -> Result<ApiResponse<Vec<DocumentTypeRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    // id = 0 means all types, otherwise filter by category
    let category_id = if payload.id == 0 {
        None
    } else {
        Some(payload.id)
    };

    match db.list_document_types(category_id).await {
        Ok(types) => Ok(ApiResponse::success(types)),
        Err(e) => Ok(ApiResponse::error(format!("Erro ao listar tipos: {}", e))),
    }
}

#[tauri::command]
pub async fn create_document(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: DocumentPayload,
) -> Result<ApiResponse<DocumentRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.create_document(&payload, &session.profile.login).await {
        Ok(document) => Ok(ApiResponse::success(document)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao criar documento: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn list_employee_documents(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: EmployeeDocumentsPayload,
) -> Result<ApiResponse<Vec<DocumentRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.get_employee_documents(payload.employee_id).await {
        Ok(documents) => Ok(ApiResponse::success(documents)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar documentos: {}",
            e
        ))),
    }
}
