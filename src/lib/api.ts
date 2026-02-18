import { invoke } from "@tauri-apps/api/core";
import type { Task, TaskStatus } from "../types/task";
import type { AppConfig } from "../types/config";

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

  async getTaskStatus(id: number): Promise<TaskStatus> {
    return invoke("get_task_status", { id });
  },

  async stopTask(id: number): Promise<void> {
    return invoke("stop_task", { id });
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
};

export const appApi = {
  async exit(): Promise<void> {
    return invoke("exit");
  }
}

export const configApi = {
  async getConfig(): Promise<AppConfig> {
    return invoke("get_config");
  },

  async saveConfig(config: AppConfig): Promise<void> {
    return invoke("update_config", { config });
  }
};
