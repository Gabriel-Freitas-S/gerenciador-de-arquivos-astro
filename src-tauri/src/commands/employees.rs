use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, EmployeeCreatePayload, EmployeeDetail, EmployeeFilterPayload, EmployeeRecord,
    EmployeeUpdatePayload, IdPayload, SearchPayload, TerminationPayload, TerminationResult,
};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn create_employee(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: EmployeeCreatePayload,
) -> Result<ApiResponse<EmployeeRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.create_employee(&payload.data).await {
        Ok(employee) => Ok(ApiResponse::success(employee)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao criar funcionário: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn update_employee(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: EmployeeUpdatePayload,
) -> Result<ApiResponse<EmployeeRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.update_employee(payload.employee_id, &payload.data).await {
        Ok(employee) => Ok(ApiResponse::success(employee)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao atualizar funcionário: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn terminate_employee(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TerminationPayload,
) -> Result<ApiResponse<TerminationResult>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let _session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    // Terminate employee
    let employee = match db
        .terminate_employee(payload.employee_id, &payload.termination_date)
        .await
    {
        Ok(emp) => emp,
        Err(e) => {
            return Ok(ApiResponse::error(format!(
                "Erro ao demitir funcionário: {}",
                e
            )))
        }
    };

    // TODO: Transfer to archive if box_id provided
    // TODO: Generate label data

    Ok(ApiResponse::success(TerminationResult {
        employee,
        archive_item: None,
        label: None,
    }))
}

#[tauri::command]
pub async fn list_employees(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: EmployeeFilterPayload,
) -> Result<ApiResponse<Vec<EmployeeRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    let page = payload.page.unwrap_or(1);
    let page_size = payload.page_size.unwrap_or(50);

    match db
        .list_employees(
            payload.status.as_deref(),
            payload.department_id,
            page,
            page_size,
        )
        .await
    {
        Ok(employees) => Ok(ApiResponse::success(employees)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar funcionários: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn search_employees(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: SearchPayload,
) -> Result<ApiResponse<Vec<EmployeeRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    let limit = payload.limit.unwrap_or(20);

    match db.search_employees(&payload.query, limit).await {
        Ok(employees) => Ok(ApiResponse::success(employees)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao buscar funcionários: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_employee(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: IdPayload,
) -> Result<ApiResponse<EmployeeDetail>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    // Get basic employee info
    let basic = match db.get_employee_by_id(payload.id).await {
        Ok(emp) => emp,
        Err(e) => {
            return Ok(ApiResponse::error(format!(
                "Funcionário não encontrado: {}",
                e
            )))
        }
    };

    // Get related data
    let documents = db
        .get_employee_documents(payload.id)
        .await
        .unwrap_or_default();
    let active_loans = db
        .get_employee_active_loans(payload.id)
        .await
        .unwrap_or_default();
    let drawer_position = db
        .get_employee_drawer_position(payload.id)
        .await
        .ok()
        .flatten();

    Ok(ApiResponse::success(EmployeeDetail {
        basic,
        documents,
        active_loans,
        drawer_position,
    }))
}
