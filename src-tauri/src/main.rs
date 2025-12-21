#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod sessions;
mod types;

use std::fs;

use anyhow::Context;
use commands::auth::LoginRateLimiter;
use db::ArchiveDatabase;
use sessions::SessionStore;
use tauri::Manager;

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
            app.manage(LoginRateLimiter::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::auth_login,
            commands::auth::auth_session,
            commands::auth::auth_logout,
            commands::storage::storage_list,
            commands::storage::storage_create,
            commands::movements::movements_list,
            commands::movements::movements_record
        ])
        .plugin(tauri_plugin_sql::Builder::default().build())
        .run(tauri::generate_context!())?;
    Ok(())
}
