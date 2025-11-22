use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{ApiResponse, CredentialsPayload, LoginResult, TokenPayload};
use tauri::State;

#[tauri::command]
pub async fn auth_login(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: CredentialsPayload,
) -> Result<ApiResponse<LoginResult>, String> {
    if payload.login.trim().len() < 3 {
        return Ok(ApiResponse::error("Informe o usuário"));
    }
    if payload.password.trim().len() < 4 {
        return Ok(ApiResponse::error("Senha inválida"));
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
    sessions.revoke(&payload.token);
    Ok(ApiResponse::success(()))
}
