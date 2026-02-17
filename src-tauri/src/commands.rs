use std::path::Path;

use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tokio::fs;

use crate::{
    app_state::AppState,
    task::{Task, TaskDAO},
};

#[tauri::command]
pub(crate) async fn list_tasks(app_state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    app_state
        .db()
        .await
        .list_tasks()
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn get_task(
    app_state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Task>, String> {
    app_state
        .db()
        .await
        .get_task(id)
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn save_task(app_state: State<'_, AppState>, task: Task) -> Result<(), String> {
    app_state
        .scheduler()
        .save_task(task)
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn remove_task(app_state: State<'_, AppState>, id: i64) -> Result<(), String> {
    app_state
        .scheduler()
        .remove_task(id)
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn manually_run_task(
    app_state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    app_state
        .scheduler()
        .manually_run_task(id)
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn switch_task(
    app_state: State<'_, AppState>,
    id: i64,
    enable: bool,
) -> Result<(), String> {
    app_state
        .scheduler()
        .switch_task(id, enable)
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn reconnect_db(app_state: State<'_, AppState>) -> Result<(), String> {
    app_state.reconnect_db().await.map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn is_task_running(
    app_state: State<'_, AppState>,
    id: i64,
) -> Result<bool, String> {
    app_state
        .scheduler()
        .is_running(id)
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub(crate) async fn pick_file(window: tauri::Window) -> Result<Option<String>, String> {
    let file = window.dialog().file().blocking_pick_file();
    Ok(file
        .as_ref()
        .and_then(|f| f.as_path())
        .and_then(|f| f.to_str())
        .map(str::to_string))
}

#[tauri::command]
pub(crate) async fn is_program_runnable(path: &Path) -> Result<bool, String> {
    match fs::metadata(path).await {
        Ok(md) => {
            #[cfg(unix)]
            {
                use std::os::unix::prelude::MetadataExt;

                // .app 目录或者可执行文件.
                if md.mode() & 0o111 == 0 {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}
