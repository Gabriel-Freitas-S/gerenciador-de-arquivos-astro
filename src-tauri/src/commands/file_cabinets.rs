use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, DrawerAssignmentPayload, DrawerCreatePayload, DrawerPositionRecord, DrawerRecord,
    FileCabinetCreatePayload, FileCabinetRecord, FileCabinetWithOccupancy, OccupationMap,
    ReorganizationPlan, ReorganizationRequestPayload, TokenPayload,
};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn create_file_cabinet(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: FileCabinetCreatePayload,
) -> Result<ApiResponse<FileCabinetRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.create_file_cabinet(&payload.data).await {
        Ok(cabinet) => Ok(ApiResponse::success(cabinet)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao criar gaveteiro: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn create_drawer(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: DrawerCreatePayload,
) -> Result<ApiResponse<DrawerRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.create_drawer(&payload.data).await {
        Ok(drawer) => Ok(ApiResponse::success(drawer)),
        Err(e) => Ok(ApiResponse::error(format!("Erro ao criar gaveta: {}", e))),
    }
}

#[tauri::command]
pub async fn list_file_cabinets(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<FileCabinetWithOccupancy>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.list_file_cabinets().await {
        Ok(cabinets) => Ok(ApiResponse::success(cabinets)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar gaveteiros: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_occupation_map(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<OccupationMap>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.get_occupation_map().await {
        Ok(map) => Ok(ApiResponse::success(map)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao obter mapa de ocupação: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn assign_employee_position(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: DrawerAssignmentPayload,
) -> Result<ApiResponse<DrawerPositionRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db
        .assign_employee_position(payload.employee_id, payload.drawer_id, payload.position)
        .await
    {
        Ok(position) => Ok(ApiResponse::success(position)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao atribuir posição: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn suggest_reorganization(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: ReorganizationRequestPayload,
) -> Result<ApiResponse<ReorganizationPlan>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    let threshold = payload.critical_threshold.unwrap_or(90);
    let max_moves = payload.max_moves.unwrap_or(10);

    match db.suggest_reorganization(threshold, max_moves).await {
        Ok(plan) => Ok(ApiResponse::success(plan)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao sugerir reorganização: {}",
            e
        ))),
    }
}
