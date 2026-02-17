use crate::app_state::AppState;

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
use tauri::WindowEvent;
use tracing::{info, warn};

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
            tray::focus_window(app);
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
            is_program_runnable
        ])
        .on_window_event(|window, evt| {
            if window.label() != "main" {
                return;
            }
            match evt {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    if let Err(e) = window.hide() {
                        warn!("failed to hide window: {e}.");
                        window.minimize().ok();
                    };

                    // 窗口隐藏后, 从 Dock 移除图标
                    #[cfg(target_os = "macos")]
                    {
                        use objc2::MainThreadMarker;
                        use objc2_app_kit::NSApplication;
                        use objc2_app_kit::NSApplicationActivationPolicy;

                        // 获取当前应用实例
                        let app =
                            NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
                        // 设置为 Accessory 模式（即从 Dock 移除，但在托盘可见）
                        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                    }
                }
                WindowEvent::Focused(true) => {
                    // 窗口获取焦点, 从 Dock 恢复图标
                    #[cfg(target_os = "macos")]
                    {
                        use objc2::MainThreadMarker;
                        use objc2_app_kit::NSApplication;
                        use objc2_app_kit::NSApplicationActivationPolicy;

                        let app =
                            NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
                        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
