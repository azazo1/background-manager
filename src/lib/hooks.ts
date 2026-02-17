import { useEffect, useState, useCallback } from "react";
import type { Task } from "../types/task";
import { taskApi } from "./api";

export function useTaskList() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTasks = useCallback(async () => {
    try {
      setError(null);
      const data = await taskApi.listTasks();
      setTasks(data || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load tasks");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  return { tasks, loading, error, fetchTasks, setTasks };
}

export function useTaskActions() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const saveTask = useCallback(async (task: Task) => {
    try {
      setLoading(true);
      setError(null);
      await taskApi.saveTask(task);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save task");
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const removeTask = useCallback(async (id: number) => {
    try {
      setLoading(true);
      setError(null);
      await taskApi.removeTask(id);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to remove task");
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const switchTask = useCallback(async (id: number, enabled: boolean) => {
    try {
      setError(null);
      await taskApi.switchTask(id, enabled);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to switch task");
      throw err;
    }
  }, []);

  const manuallyRunTask = useCallback(async (id: number) => {
    try {
      setError(null);
      await taskApi.manualRunTask(id);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to run task");
      throw err;
    }
  }, []);

  return { saveTask, removeTask, switchTask, manuallyRunTask, loading, error };
}
