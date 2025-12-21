use crate::db::ArchiveDatabase;
use crate::sessions::SessionStore;
use crate::types::{ApiResponse, CredentialsPayload, LoginResult, TokenPayload};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::State;
use validator::Validate;

pub struct LoginRateLimiter {
    // Stores (attempts, first_attempt_time) for a given login/IP equivalent
    // Since we don't have IP easily in desktop app, we limit by login username
    attempts: Mutex<HashMap<String, (u32, Instant)>>,
}

impl Default for LoginRateLimiter {
    fn default() -> Self {
        Self {
            attempts: Mutex::new(HashMap::new()),
        }
    }
}

impl LoginRateLimiter {
    pub fn check(&self, login: &str) -> Result<(), String> {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();
        let entry = attempts.entry(login.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) > Duration::from_secs(60) {
            // Reset after 1 minute
            *entry = (1, now);
        } else {
            entry.0 += 1;
            if entry.0 > 5 {
                return Err("Muitas tentativas de login. Tente novamente em 1 minuto.".to_string());
            }
        }
        Ok(())
    }
}

#[tauri::command]
pub async fn auth_login(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    limiter: State<'_, LoginRateLimiter>,
    payload: CredentialsPayload,
) -> Result<ApiResponse<LoginResult>, String> {
    // Validate input
    if let Err(e) = payload.validate() {
        return Ok(ApiResponse::error(format!("Dados inválidos: {}", e)));
    }

    if let Err(msg) = limiter.check(&payload.login) {
        return Ok(ApiResponse::error(msg));
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
