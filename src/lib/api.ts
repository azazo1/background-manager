import { invoke } from "@tauri-apps/api/core";
import type { Task } from "../types/task";

export const taskApi = {
  async listTasks(): Promise<Task[]> {
    return invoke("list_tasks");
  },

  async getTask(id: number): Promise<Task | null> {
    return invoke("get_task", { id });
  },

  async saveTask(task: Task): Promise<void> {
    return invoke("save_task", { task });
  },

  async removeTask(id: number): Promise<void> {
    return invoke("remove_task", { id });
  },

  async switchTask(id: number, enable: boolean): Promise<void> {
    return invoke("switch_task", { id, enable });
  },

  async manualRunTask(id: number): Promise<void> {
    return invoke("manually_run_task", { id });
  },

  async isTaskRunning(id: number): Promise<boolean> {
    return invoke("is_task_running", { id });
  },

  async reconnectDb(): Promise<void> {
    return invoke("reconnect_db");
  },

  async pickFile(): Promise<string | null> {
    return invoke("pick_file");
  },

  async isProgramRunnable(path: string): Promise<boolean> {
    return invoke("is_program_runnable", { path });
  },

  async exit(): Promise<void> {
    return invoke("exit");
  }
};
