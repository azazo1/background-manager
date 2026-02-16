use tauri::State;
use tracing::{error, warn};

use crate::{
    app_state::AppState,
    task::{Task, TaskDAO},
};

#[tauri::command]
pub(crate) async fn list_tasks(app_state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    todo!()
}

#[tauri::command]
pub(crate) async fn get_task(app_state: State<'_, AppState>, id: i64) -> Result<Task, String> {
    todo!()
}

#[tauri::command]
pub(crate) async fn save_task(app_state: State<'_, AppState>, task: Task) -> Result<i64, String> {
    todo!()
}

#[tauri::command]
pub(crate) async fn remove_task(app_state: State<'_, AppState>) -> Result<(), String> {
    todo!()
}

#[tauri::command]
pub(crate) async fn manual_run_task(app_state: State<'_, AppState>, id: i64) -> Result<(), String> {
    if let Err(e) = app_state
        .db()
        .await
        .update_task_run_at(id, chrono::Local::now().fixed_offset())
        .await
    {
        warn!(target: "command", "{e:?}");
    }

    // todo 使用 schedule 模块执行 task, 上面的 update_task_run_at 放在内部.
    Ok(())
}

#[tauri::command]
pub(crate) async fn switch_task(
    app_state: State<'_, AppState>,
    enable: bool,
) -> Result<(), String> {
    todo!()
}

#[tauri::command]
pub(crate) async fn reconnect_db(app_state: State<'_, AppState>) -> Result<(), String> {
    app_state.reconnect_db().await.map_err(|e| {
        error!(target: "command", "{e:?}");
        format!("{e}")
    })
}
