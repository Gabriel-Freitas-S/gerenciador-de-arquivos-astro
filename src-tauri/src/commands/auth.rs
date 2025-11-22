use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{ApiResponse, CredentialsPayload, LoginResult, TokenPayload};
use tauri::State;
use validator::Validate;

#[tauri::command]
pub async fn auth_login(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: CredentialsPayload,
) -> Result<ApiResponse<LoginResult>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    println!("Tentativa de login: {}", payload.login);
    match db.verify_login(&payload.login, &payload.password).await {
        Ok(Some(profile)) => {
            let session = sessions.create(profile.clone());
            match db.snapshot().await {
                Ok(snapshot) => Ok(ApiResponse::success(LoginResult {
                    token: session.token,
                    profile,
                    snapshot,
                })),
                Err(error) => Ok(ApiResponse::error(error.to_string())),
            }
        }
        Ok(None) => Ok(ApiResponse::error("Credenciais inválidas")),
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}

#[tauri::command]
pub async fn auth_session(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<LoginResult>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    match sessions.require(&payload.token) {
        Ok(session) => match db.snapshot().await {
            Ok(snapshot) => Ok(ApiResponse::success(LoginResult {
                token: session.token,
                profile: session.profile,
                snapshot,
            })),
            Err(error) => Ok(ApiResponse::error(error.to_string())),
        },
        Err(message) => Ok(ApiResponse::error(message)),
    }
}

#[tauri::command]
pub async fn auth_logout(
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<()>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    sessions.revoke(&payload.token);
    Ok(ApiResponse::success(()))
}
