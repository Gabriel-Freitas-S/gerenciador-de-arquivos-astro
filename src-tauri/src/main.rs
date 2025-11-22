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
    ApiResponse,
    CredentialsPayload,
    LoginResult,
    MovementPayload,
    SnapshotSummary,
    StorageCreatePayload,
    TokenPayload,
};

fn main() -> anyhow::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            dotenvy::dotenv().ok();
            let default_login = std::env::var("ARCHIVE_DEFAULT_ADMIN_LOGIN").unwrap_or_else(|_| "admin".into());
            let default_password =
                std::env::var("ARCHIVE_DEFAULT_ADMIN_PASSWORD").unwrap_or_else(|_| "admin".into());
            let data_dir = app
                .handle()
                .path()
                .app_data_dir()
                .context("Não foi possível localizar a pasta de dados do aplicativo")?;
            fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("archive.sqlite");
            let db = ArchiveDatabase::connect(db_path)?;
            db.ensure_default_admin(&default_login, &default_password)?;
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
        .run(tauri::generate_context!())?;
    Ok(())
}

#[tauri::command]
fn auth_login(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: CredentialsPayload,
) -> ApiResponse<LoginResult> {
    if payload.login.trim().len() < 3 {
        return ApiResponse::error("Informe o usuário");
    }
    if payload.password.trim().len() < 4 {
        return ApiResponse::error("Senha inválida");
    }
    match db.verify_login(&payload.login, &payload.password) {
        Ok(Some(profile)) => {
            let session = sessions.create(profile.clone());
            match db.snapshot() {
                Ok(snapshot) => ApiResponse::success(LoginResult {
                    token: session.token,
                    profile,
                    snapshot,
                }),
                Err(error) => ApiResponse::error(error.to_string()),
            }
        }
        Ok(None) => ApiResponse::error("Credenciais inválidas"),
        Err(error) => ApiResponse::error(error.to_string()),
    }
}

#[tauri::command]
fn auth_session(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> ApiResponse<LoginResult> {
    match sessions.require(&payload.token) {
        Ok(session) => match db.snapshot() {
            Ok(snapshot) => ApiResponse::success(LoginResult {
                token: session.token,
                profile: session.profile,
                snapshot,
            }),
            Err(error) => ApiResponse::error(error.to_string()),
        },
        Err(message) => ApiResponse::error(message),
    }
}

#[tauri::command]
fn auth_logout(sessions: State<'_, SessionStore>, payload: TokenPayload) -> ApiResponse<()> {
    sessions.revoke(&payload.token);
    ApiResponse::success(())
}

#[tauri::command]
fn storage_list(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> ApiResponse<Vec<types::StorageUnitRecord>> {
    if let Err(message) = sessions.require(&payload.token) {
        return ApiResponse::error(message);
    }
    match db.list_storage_units() {
        Ok(units) => ApiResponse::success(units),
        Err(error) => ApiResponse::error(error.to_string()),
    }
}

#[tauri::command]
fn storage_create(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: StorageCreatePayload,
) -> ApiResponse<StorageCreateResponse> {
    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return ApiResponse::error(message),
    };
    if payload.data.label.trim().len() < 2 {
        return ApiResponse::error("Informe um identificador");
    }
    if payload.data.r#type.trim().is_empty() {
        return ApiResponse::error("Informe o tipo da unidade");
    }
    match db.create_storage_unit(&payload.data) {
        Ok(unit) => {
            let movement = types::MovementData {
                action: "Cadastro de unidade".into(),
                reference: payload.data.section.clone(),
                item_label: Some(unit.label.clone()),
                from_unit: None,
                to_unit: payload.data.section.clone(),
                note: Some(format!("Unidade {} criada", unit.label)),
            };
            let _ = db.record_movement(&session.profile.name, &movement);
            match db.snapshot() {
                Ok(snapshot) => ApiResponse::success(StorageCreateResponse { unit, snapshot }),
                Err(error) => ApiResponse::error(error.to_string()),
            }
        }
        Err(error) => ApiResponse::error(error.to_string()),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StorageCreateResponse {
    unit: types::StorageUnitRecord,
    snapshot: SnapshotSummary,
}

#[tauri::command]
fn movements_list(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: TokenPayload,
) -> ApiResponse<Vec<types::MovementRecord>> {
    if let Err(message) = sessions.require(&payload.token) {
        return ApiResponse::error(message);
    }
    match db.list_movements(25) {
        Ok(records) => ApiResponse::success(records),
        Err(error) => ApiResponse::error(error.to_string()),
    }
}

#[tauri::command]
fn movements_record(
    db: State<'_, ArchiveDatabase>,
    sessions: State<'_, SessionStore>,
    payload: MovementPayload,
) -> ApiResponse<MovementRecordResponse> {
    let session = match sessions.require(&payload.token) {
        Ok(session) => session,
        Err(message) => return ApiResponse::error(message),
    };
    if payload.data.action.trim().len() < 3 {
        return ApiResponse::error("Descreva a movimentação");
    }
    match db.record_movement(&session.profile.name, &payload.data) {
        Ok(movement) => match db.snapshot() {
            Ok(snapshot) => ApiResponse::success(MovementRecordResponse { movement, snapshot }),
            Err(error) => ApiResponse::error(error.to_string()),
        },
        Err(error) => ApiResponse::error(error.to_string()),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MovementRecordResponse {
    movement: types::MovementRecord,
    snapshot: SnapshotSummary,
}
