import { useState, useEffect } from "react";
import { AlertCircle, Globe, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import "./App.css";
import { TaskList } from "./components/TaskList";
import { TaskEditDialog } from "./components/TaskEditDialog";
import { Button } from "./components/ui/button";
import { useTaskList, useTaskActions } from "./lib/hooks";
import { taskApi } from "./lib/api";
import type { Task } from "./types/task";

function App() {
  const { t, i18n } = useTranslation();
  const { tasks, loading, error, fetchTasks, setTasks } = useTaskList();
  const { saveTask, removeTask, switchTask, manuallyRunTask } = useTaskActions();
  const [selectedTask, setSelectedTask] = useState<Task | undefined>(undefined);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [taskRunStatus, setTaskRunStatus] = useState<Record<number, boolean>>({});
  const [runnableProgramStatus, setRunnableProgramStatus] = useState<Record<number, boolean>>({});

  // Update task statuses
  const updateTaskStatuses = async (taskList: Task[] = tasks) => {
    const statuses: Record<number, boolean> = {};
    const runnableStatus: Record<number, boolean> = {};
    for (const task of taskList) {
      if (task.id) {
        try {
          const isRunning = await taskApi.isTaskRunning(task.id);
          statuses[task.id] = isRunning;
        } catch {
          statuses[task.id] = false;
        }

        // Check if program is runnable
        if (task.program) {
          try {
            const isRunnable = await taskApi.isProgramRunnable(task.program);
            runnableStatus[task.id] = isRunnable;
          } catch {
            runnableStatus[task.id] = false;
          }
        } else {
          runnableStatus[task.id] = false;
        }
      }
    }
    setTaskRunStatus(statuses);
    setRunnableProgramStatus(runnableStatus);
  };

  // Auto-refresh task statuses every second
  useEffect(() => {
    updateTaskStatuses();
    const interval = setInterval(async () => {
      updateTaskStatuses();
      fetchTasks();
    }, 2000);
    return () => clearInterval(interval);
  }, [tasks]);

  const handleEditTask = (task: Task) => {
    setSelectedTask(task);
    setDialogOpen(true);
  };

  const handleCreateTask = () => {
    setSelectedTask(undefined);
    setDialogOpen(true);
  };

  const handleSaveTask = async (task: Task) => {
    try {
      await saveTask(task);
      setDialogOpen(false);
      setSelectedTask(undefined);
      await fetchTasks();
      toast.success(t("toast.saveSuccess"));
    } catch (err) {
      console.error("Failed to save task:", err);
      toast.error(t("toast.saveFailed"), {
        description: err instanceof Error ? err.message : t("toast.unknownError"),
      });
    }
  };

  const handleDeleteTask = async (id: number) => {
    try {
      await removeTask(id);
      await fetchTasks();
    } catch (err) {
      console.error("Failed to delete task:", err);
    }
  };

  const handleRunTask = async (id: number) => {
    try {
      await manuallyRunTask(id);
    } catch (err) {
      console.error("Failed to run task:", err);
    }
  };

  const handleToggleTask = async (id: number, enabled: boolean) => {
    try {
      await switchTask(id, enabled);
      setTasks(
        tasks.map((t) =>
          t.id === id ? { ...t, enabled } : t
        )
      );
    } catch (err) {
      console.error("Failed to toggle task:", err);
    }
  };

  const handleLanguageChange = (lng: string) => {
    i18n.changeLanguage(lng);
    localStorage.setItem("language", lng);
  };

  const handleRefresh = async () => {
    try {
      await fetchTasks();
      await updateTaskStatuses();
      toast.success(t("toast.refreshSuccess"));
    } catch (err) {
      console.error("Failed to refresh tasks:", err);
      toast.error(t("toast.refreshFailed"));
    }
  };

  const handleLanguageToggle = () => {
    const nextLanguage = i18n.language === "en" ? "zh" : "en";
    handleLanguageChange(nextLanguage);
  };

  return (
    <div className="min-h-screen bg-linear-to-br from-slate-50 to-slate-100">
      {/* Header */}
      <header className="bg-white border-b border-slate-200">
        <div className="max-w-6xl mx-auto px-6 py-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-slate-900">
                {t("header.title")}
              </h1>
              <p className="text-sm text-slate-600 mt-1">
                {t("header.subtitle")}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                onClick={handleRefresh}
                className="text-xs"
                title={t("button.refresh")}
              >
                <RefreshCw className="h-3 w-3" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleLanguageToggle}
                className="text-xs"
              >
                <Globe className="h-3 w-3 mr-1" />
                {i18n.language === "en" ? "EN" : "ZH"}
              </Button>
              <Button onClick={handleCreateTask} size="lg">
                {t("button.newTask")}
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-6xl mx-auto px-6 py-8">
        {/* Error Banner */}
        {error && (
          <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg flex items-start gap-3">
            <AlertCircle className="h-5 w-5 text-red-600 mt-0.5 shrink-0" />
            <div>
              <h3 className="font-semibold text-red-900">{t("error.title")}</h3>
              <p className="text-sm text-red-700 mt-1">{error}</p>
              <Button
                size="sm"
                variant="outline"
                onClick={() => fetchTasks()}
                className="mt-2"
              >
                {t("button.retry")}
              </Button>
            </div>
          </div>
        )}

        {/* Loading State */}
        {loading ? (
          <div className="flex items-center justify-center py-16">
            <div className="space-y-2 text-center">
              <div className="h-8 w-8 border-4 border-slate-200 border-t-slate-900 rounded-full animate-spin mx-auto" />
              <p className="text-slate-600">{t("status.loading")}</p>
            </div>
          </div>
        ) : (
          <TaskList
            tasks={tasks}
            onEdit={handleEditTask}
            onDelete={handleDeleteTask}
            onRun={handleRunTask}
            onToggleEnabled={handleToggleTask}
            isRunning={taskRunStatus}
            runnablePrograms={runnableProgramStatus}
          />
        )}
      </main>

      {/* Task Edit Dialog */}
      <TaskEditDialog
        open={dialogOpen}
        task={selectedTask}
        onOpenChange={setDialogOpen}
        onSave={handleSaveTask}
      />
    </div>
  );
}

export default App;
