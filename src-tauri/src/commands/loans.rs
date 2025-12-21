use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, LoanPayload, LoanRecord, LoanReturnPayload, LoanWithEmployee, TokenPayload,
};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn create_loan(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: LoanPayload,
) -> Result<ApiResponse<LoanRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db.create_loan(&payload, &session.profile.login).await {
        Ok(loan) => Ok(ApiResponse::success(loan)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao criar empréstimo: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn return_loan(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: LoanReturnPayload,
) -> Result<ApiResponse<LoanRecord>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };

    match db
        .return_loan(
            payload.loan_id,
            payload.actual_return_date.as_deref(),
            payload.return_notes.as_deref(),
            &session.profile.login,
        )
        .await
    {
        Ok(loan) => Ok(ApiResponse::success(loan)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao devolver empréstimo: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn list_loans(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<LoanRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.list_loans(None).await {
        Ok(loans) => Ok(ApiResponse::success(loans)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar empréstimos: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_pending_loans(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<LoanRecord>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.list_loans(Some("BORROWED")).await {
        Ok(loans) => Ok(ApiResponse::success(loans)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar empréstimos pendentes: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_overdue_loans(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<LoanWithEmployee>>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.get_overdue_loans().await {
        Ok(loans) => Ok(ApiResponse::success(loans)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao listar empréstimos atrasados: {}",
            e
        ))),
    }
}
