import { useState } from "react";
import { format } from "date-fns";
import { ChevronRight, Play, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Task } from "@/types/task";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";

interface TaskListProps {
  tasks: Task[];
  onEdit: (task: Task) => void;
  onDelete: (id: number) => void;
  onRun: (id: number) => void;
  onToggleEnabled: (id: number, enabled: boolean) => void;
  isRunning?: Record<number, boolean>;
  runnablePrograms?: Record<number, boolean>;
}

export function TaskList({
  tasks,
  onEdit,
  onDelete,
  onRun,
  onToggleEnabled,
  isRunning = {},
  runnablePrograms = {},
}: TaskListProps) {
  const { t } = useTranslation();
  const [runningTasks, setRunningTasks] = useState<Set<number>>(new Set());
  const [deleteTargetId, setDeleteTargetId] = useState<number | null>(null);

  const getTriggerLabel = (task: Task): string => {
    if (typeof task.trigger === "object" && task.trigger !== null) {
      if ("tag" in task.trigger) {
        switch (task.trigger.tag) {
          case "Routine":
            const ms = (task.trigger as any).content;
            return t("task.every", { time: (ms / 1000).toFixed(1) });
          case "Instant":
            return t("task.onceAt", {
              time: new Date((task.trigger as any).content).toLocaleString()
            });
          case "Startup":
            return t("trigger.startup");
          case "KeepAlive":
            return t("trigger.keepAlive");
          case "UntilSucceed":
            return t("trigger.untilSucceed");
          case "Manual":
            return t("trigger.manual");
          default:
            return "Unknown";
        }
      }
    }
    return "Unknown";
  };

  const formatLastRun = (timestamp?: string): string => {
    if (!timestamp) return "-";
    try {
      const date = new Date(timestamp);
      return format(date, "MMM dd, HH:mm:ss");
    } catch {
      return "-";
    }
  };

  const handleDelete = async (id: number | undefined) => {
    if (id) {
      setDeleteTargetId(id);
    }
  };

  const handleConfirmDelete = () => {
    if (deleteTargetId !== null) {
      onDelete(deleteTargetId);
      setDeleteTargetId(null);
    }
  };

  const handleRun = async (id: number | undefined) => {
    if (id) {
      setRunningTasks((prev) => new Set([...prev, id]));
      try {
        await onRun(id);
        setTimeout(() => {
          setRunningTasks((prev) => {
            const next = new Set(prev);
            next.delete(id);
            return next;
          });
        }, 1000);
      } catch {
        setRunningTasks((prev) => {
          const next = new Set(prev);
          next.delete(id);
          return next;
        });
      }
    }
  };

  if (tasks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-center">
        <p className="text-gray-500 text-lg">{t("task.emptyTitle")}</p>
        <p className="text-gray-400 text-sm">{t("task.emptySubtitle")}</p>
      </div>
    );
  }

  return (
    <>
      <div className="space-y-2">
        {tasks.map((task) => {
          const running = isRunning[task.id!] || runningTasks.has(task.id!);
          const programRunnable = runnablePrograms[task.id!] !== false;
          return (
            <div
              key={task.id}
              className={cn(
                "flex items-center gap-3 p-4 bg-white border rounded-lg transition-all hover:shadow-sm",
                programRunnable
                  ? "border-slate-200 hover:border-slate-300"
                  : "border-yellow-400 hover:border-yellow-500"
              )}
              title={programRunnable ? undefined : t("task.programNotRunnable")}
            >
              {/* Main task info */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="font-semibold text-slate-900 truncate">{task.name || t("task.defaultNameDisplay")}</h3>
                  {running && (
                    <span className="px-2 py-1 text-xs bg-green-100 text-green-800 rounded-full">
                      {t("task.running")}
                    </span>
                  )}
                </div>
                <p className="text-xs text-slate-600 truncate mb-1">{task.program}</p>
                <div className="flex flex-wrap gap-2 text-xs text-slate-500">
                  <span>{getTriggerLabel(task)}</span>
                  {task.last_exit_code || task.last_exit_code === 0 ? (
                    <span
                      className={cn(
                        "px-1.5 py-0.5 rounded",
                        task.last_exit_code === 0
                          ? "bg-green-50 text-green-700"
                          : "bg-red-50 text-red-700"
                      )}
                    >
                      {t("task.exit")} {task.last_exit_code}
                    </span>
                  ) : null}
                  {task.last_run_at && (
                    <span className="text-slate-400">
                      {t("task.last", { time: formatLastRun(task.last_run_at) })}
                    </span>
                  )}
                </div>
              </div>

              {/* Controls */}
              <div className="flex items-center gap-2">
                {/* Toggle switch */}
                <Switch
                  checked={task.enabled}
                  onCheckedChange={(checked) =>
                    task.id && onToggleEnabled(task.id, checked)
                  }
                  className="data-[state=unchecked]:bg-slate-200"
                />

                {/* Manual run button */}
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => handleRun(task.id)}
                  disabled={running || !task.enabled}
                  title={t("button.run")}
                >
                  <Play className="h-4 w-4" />
                </Button>

                {/* Edit button */}
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => onEdit(task)}
                  title={t("button.edit")}
                >
                  <ChevronRight className="h-4 w-4" />
                </Button>

                {/* Delete button */}
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => handleDelete(task.id)}
                  className="text-red-600 hover:text-red-700 hover:bg-red-50"
                  title={t("button.delete")}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </div>
          );
        })}
      </div>

      <Dialog open={deleteTargetId !== null} onOpenChange={(open) => !open && setDeleteTargetId(null)}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>{t("button.delete")}</DialogTitle>
            <DialogDescription>{t("error.confirmDelete")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setDeleteTargetId(null)}>
              {t("button.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
            >
              {t("button.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
