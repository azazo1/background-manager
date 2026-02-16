use tauri::State;
use tracing::error;

use crate::{app_state::AppState, task::Task};

#[tauri::command]
pub(crate) async fn list_tasks(app_state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    todo!()
}

#[tauri::command]
pub(crate) async fn get_task(app_state: State<'_, AppState>, id: i64) -> Result<Task, String> {
    todo!()
}

/// 添加或者修改一个 task
/// - `task` 中的 id 为 None 的时候, 添加新的 Task.
/// - `task` 中的 id 为 Some 的时候, 修改已有 Task 的内容, 如果指定 id 的 task 不存在, 那么返回错误.
#[tauri::command]
pub(crate) async fn save_task(app_state: State<'_, AppState>, task: Task) -> Result<i64, String> {
    todo!()
}

#[tauri::command]
pub(crate) async fn remove_task(app_state: State<'_, AppState>) -> Result<(), String> {
    todo!()
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
        error!("{e:?}");
        format!("{e}")
    })
}
