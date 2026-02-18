use crate::{
    app_state::AppState,
    utils::{toggle_dock_icon, toggle_window},
};

mod app_state;
mod commands;
mod config;
mod error;
mod log;
mod schedule;
mod task;
mod tray;
mod utils;

use error::{Error, ErrorKind, Result};
use tauri::{Manager, WindowEvent};
use tracing::info;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    use commands::*;

    let _guard = log::init().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            info!(target: "second instance", "argv: {argv:?}, cwd: {cwd:?}");
            // 当第二个实例尝试启动时, 聚焦主窗口
            toggle_window(app, true);
        }))
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
            is_program_runnable,
            get_config,
            update_config,
            exit
        ])
        .setup(|app| {
            tray::init_tray(app)?;
            let handle = app.handle().clone();
            toggle_dock_icon(false);
            tokio::spawn(async move {
                let app_state = handle.state::<AppState>();
                let config = app_state.get_config().await;
                if !config.quiet_launch() {
                    toggle_window(&handle, true);
                }
            });
            Ok(())
        })
        .on_window_event(|window, evt| {
            if window.label() != "main" {
                return;
            }
            match evt {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    toggle_window(window.app_handle(), false);
                }
                WindowEvent::Focused(true) => {
                    // 窗口获取焦点, 从 Dock 恢复图标
                    toggle_dock_icon(true);
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
