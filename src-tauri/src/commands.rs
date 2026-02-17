use tauri::State;
use tracing::{error, warn};

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
pub(crate) async fn manual_run_task(app_state: State<'_, AppState>, id: i64) -> Result<(), String> {
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
    app_state.reconnect_db().await.map_err(|e| {
        error!(target: "command", "{e:?}");
        format!("{e}")
    })
}
