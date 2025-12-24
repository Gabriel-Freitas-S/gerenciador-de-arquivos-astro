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
            commands::movements::movements_record,
            commands::employees::create_employee,
            commands::employees::update_employee,
            commands::employees::terminate_employee,
            commands::employees::list_employees,
            commands::employees::search_employees,
            commands::employees::get_employee,
            commands::departments::list_departments,
            commands::departments::create_department,
            commands::departments::update_department,
            commands::file_cabinets::create_file_cabinet,
            commands::file_cabinets::create_drawer,
            commands::file_cabinets::list_file_cabinets,
            commands::file_cabinets::get_occupation_map,
            commands::file_cabinets::assign_employee_position,
            commands::file_cabinets::suggest_reorganization,
            commands::documents::list_document_categories,
            commands::documents::list_document_types,
            commands::documents::create_document,
            commands::documents::list_employee_documents,
            commands::loans::create_loan,
            commands::loans::return_loan,
            commands::loans::list_loans,
            commands::loans::get_pending_loans,
            commands::loans::get_overdue_loans,
            commands::dead_archive::create_archive_box,
            commands::dead_archive::list_archive_boxes,
            commands::dead_archive::transfer_to_archive,
            commands::dead_archive::get_disposal_candidates,
            commands::dead_archive::register_disposal,
            commands::reports::get_dashboard_stats,
            commands::reports::get_movements_report,
            commands::reports::get_loans_report,
            commands::reports::export_to_excel,
            commands::labels::generate_folder_label,
            commands::labels::generate_envelope_label,
            commands::labels::generate_box_label
        ])
        .plugin(tauri_plugin_sql::Builder::default().build())
        .run(tauri::generate_context!())?;
    Ok(())
}
