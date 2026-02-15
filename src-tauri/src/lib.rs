use crate::{
    app_state::AppState,
    task::{Task, Trigger},
};

mod app_state;
mod commands;
mod config;
mod entity;
mod error;
mod log;
mod task;
mod utils;

use error::{Error, ErrorKind, Result};
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    use commands::*;

    let _guard = log::init().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::build().await.unwrap())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                state
                    .save_task(
                        Task::builder()
                            .name("program1name")
                            .program("program1")
                            .trigger(Trigger::Startup)
                            .build(),
                    )
                    .await
                    .unwrap();
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_tasks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
