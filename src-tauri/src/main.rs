#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod sessions;
mod types;

use std::fs;

use anyhow::Context;
use db::ArchiveDatabase;
use sessions::SessionStore;
use tauri::{Manager, State};
use types::{
    ApiResponse, CredentialsPayload, LoginResult, MovementPayload, SnapshotSummary,
    StorageCreatePayload, TokenPayload,
};

fn main() -> anyhow::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            dotenvy::dotenv().ok();

            let data_dir = app
                .handle()
                .path()
                .app_data_dir()
                .context("Não foi possível localizar a pasta de dados do aplicativo")?;
            fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("archive.sqlite");

            // Initialize DB asynchronously
            let db = tauri::async_runtime::block_on(async {
                let db = ArchiveDatabase::connect(db_path).await?;

                let default_login = std::env::var("ARCHIVE_DEFAULT_ADMIN_LOGIN").ok();
                let default_password = std::env::var("ARCHIVE_DEFAULT_ADMIN_PASSWORD").ok();

                if let (Some(login), Some(password)) = (default_login, default_password) {
                    if !login.trim().is_empty() && !password.trim().is_empty() {
                        db.ensure_default_admin(&login, &password).await?;
                    }
                }

                Ok::<_, anyhow::Error>(db)
            })?;

            app.manage(db);
            app.manage(SessionStore::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_login,
            auth_session,
            auth_logout,
            storage_list,
            storage_create,
            movements_list,
            movements_record
        ])
        .plugin(tauri_plugin_sql::Builder::default().build())
        .run(tauri::generate_context!())?;
    Ok(())
}

#[tauri::command]
async fn auth_login(
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
async fn auth_session(
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
async fn auth_logout(
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<()>, String> {
    sessions.revoke(&payload.token);
    Ok(ApiResponse::success(()))
}

#[tauri::command]
async fn storage_list(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<types::StorageUnitRecord>>, String> {
    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }
    match db.list_storage_units().await {
        Ok(units) => Ok(ApiResponse::success(units)),
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}

#[tauri::command]
async fn storage_create(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: StorageCreatePayload,
) -> Result<ApiResponse<StorageCreateResponse>, String> {
    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };
    if payload.data.label.trim().len() < 2 {
        return Ok(ApiResponse::error("Informe um identificador"));
    }
    if payload.data.r#type.trim().is_empty() {
        return Ok(ApiResponse::error("Informe o tipo da unidade"));
    }
    match db.create_storage_unit(&payload.data).await {
        Ok(unit) => {
            let movement = types::MovementData {
                action: "Cadastro de unidade".into(),
                reference: payload.data.section.clone(),
                item_label: Some(unit.label.clone()),
                from_unit: None,
                to_unit: payload.data.section.clone(),
                note: Some(format!("Unidade {} criada", unit.label)),
            };
            let _ = db.record_movement(&session.profile.name, &movement).await;
            match db.snapshot().await {
                Ok(snapshot) => Ok(ApiResponse::success(StorageCreateResponse {
                    unit,
                    snapshot,
                })),
                Err(error) => Ok(ApiResponse::error(error.to_string())),
            }
        }
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StorageCreateResponse {
    unit: types::StorageUnitRecord,
    snapshot: SnapshotSummary,
}

#[tauri::command]
async fn movements_list(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> Result<ApiResponse<Vec<types::MovementRecord>>, String> {
    if let Err(message) = sessions.require(&payload.token) {
        return Ok(ApiResponse::error(message));
    }
    match db.list_movements(25).await {
        Ok(records) => Ok(ApiResponse::success(records)),
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}

#[tauri::command]
async fn movements_record(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: MovementPayload,
) -> Result<ApiResponse<MovementRecordResponse>, String> {
    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return Ok(ApiResponse::error(message)),
    };
    if payload.data.action.trim().len() < 3 {
        return Ok(ApiResponse::error("Descreva a movimentação"));
    }
    match db
        .record_movement(&session.profile.name, &payload.data)
        .await
    {
        Ok(movement) => match db.snapshot().await {
            Ok(snapshot) => Ok(ApiResponse::success(MovementRecordResponse {
                movement,
                snapshot,
            })),
            Err(error) => Ok(ApiResponse::error(error.to_string())),
        },
        Err(error) => Ok(ApiResponse::error(error.to_string())),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MovementRecordResponse {
    movement: types::MovementRecord,
    snapshot: SnapshotSummary,
}
