use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, DepartmentRecord, DepartmentUpsertPayload, IdPayload, TokenPayload,
};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn list_departments(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<DepartmentRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.list_departments().await {
        Ok(departments) => Ok(ApiResponse::success(departments)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar departamentos: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn create_department(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: DepartmentUpsertPayload,
) -> Result<ApiResponse<DepartmentRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.create_department(&payload.data).await {
        Ok(department) => Ok(ApiResponse::success(department)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao criar departamento: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn update_department(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: IdPayload,
    data: crate::types::DepartmentPayload,
) -> Result<ApiResponse<DepartmentRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.update_department(payload.id, &data).await {
        Ok(department) => Ok(ApiResponse::success(department)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao atualizar departamento: {}",
            e
        ))),
    }
}
