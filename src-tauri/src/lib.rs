use crate::app_state::AppState;

mod app_state;
mod commands;
mod config;
mod error;
mod log;
mod schedule;
mod task;
mod utils;

use error::{Error, ErrorKind, Result};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    use commands::*;

    let _guard = log::init().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::build().await.unwrap())
        .invoke_handler(tauri::generate_handler![
            list_tasks,
            get_task,
            save_task,
            remove_task,
            manually_run_task,
            switch_task,
            reconnect_db,
            is_task_running,
            pick_file,
            is_program_runnable
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
