use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{
    ApiResponse, DashboardStats, FileExportResult, LoansReport, MovementsReport, TokenPayload,
};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn get_dashboard_stats(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<DashboardStats>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.get_dashboard_stats().await {
        Ok(stats) => Ok(ApiResponse::success(stats)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao obter estatísticas: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_movements_report(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<MovementsReport>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.get_movements_report(100).await {
        Ok(report) => Ok(ApiResponse::success(report)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao gerar relatório: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_loans_report(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<LoansReport>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    match db.get_loans_report().await {
        Ok(report) => Ok(ApiResponse::success(report)),
        Err(e) => Ok(ApiResponse::error(format!(
            "Erro ao gerar relatório: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn export_to_excel(
    _db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<FileExportResult>, String> {
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }

    // TODO: Implement Excel export using a library like rust_xlsxwriter
    Ok(ApiResponse::error(
        "Exportação para Excel ainda não implementada",
    ))
}
