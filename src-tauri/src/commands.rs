use crate::task::Task;

#[tauri::command]
pub(crate) async fn list_tasks() -> Vec<Task> {}

#[tauri::command]
pub(crate) async fn task_enable() -> Result<(), String> {}
