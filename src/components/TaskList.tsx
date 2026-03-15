import { useState } from "react";
import { format } from "date-fns";
import { ChevronRight, GripVertical, Play, Square, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
  closestCenter,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { Task } from "@/types/task";
import { TaskStatus } from "@/types/task";
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
  onStop: (id: number) => void;
  onToggleEnabled: (id: number, enabled: boolean) => void;
  onReorder: (fromId: number, toId: number) => void;
  isRunning?: Record<number, boolean>;
  runnablePrograms?: Record<number, boolean>;
  taskStatuses?: Record<number, TaskStatus>;
}

// ---------- 单个可排序任务行 ----------

interface SortableTaskItemProps {
  task: Task;
  running: boolean;
  programRunnable: boolean;
  isSuspended: boolean;
  onEdit: (task: Task) => void;
  onDelete: (id: number | undefined) => void;
  onRun: (id: number | undefined) => void;
  onStop: (id: number) => void;
  onToggleEnabled: (id: number, enabled: boolean) => void;
}

function SortableTaskItem({
  task,
  running,
  programRunnable,
  isSuspended,
  onEdit,
  onDelete,
  onRun,
  onStop,
  onToggleEnabled,
}: SortableTaskItemProps) {
  const { t } = useTranslation();

  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: task.id! });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const getTriggerLabel = (): string => {
    if (typeof task.trigger === "object" && task.trigger !== null && "tag" in task.trigger) {
      switch (task.trigger.tag) {
        case "Routine":
          const ms = (task.trigger as any).content;
          return t("task.every", { time: (ms / 1000).toFixed(1) });
        case "Instant":
          return t("task.onceAt", {
            time: new Date((task.trigger as any).content).toLocaleString(),
          });
        case "Startup": return t("trigger.startup");
        case "KeepAlive": return t("trigger.keepAlive");
        case "UntilSucceed": return t("trigger.untilSucceed");
        case "Manual": return t("trigger.manual");
        default: return "Unknown";
      }
    }
    return "Unknown";
  };

  const formatLastRun = (timestamp?: string): string => {
    if (!timestamp) return "-";
    try {
      return format(new Date(timestamp), "MMM dd, HH:mm:ss");
    } catch {
      return "-";
    }
  };

  const tooltip = isSuspended
    ? t("task.suspended")
    : programRunnable
      ? undefined
      : t("task.programNotRunnable");

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={cn(
        "flex items-center gap-3 p-4 bg-white border rounded-lg transition-all hover:shadow-sm",
        isDragging && "opacity-50 shadow-lg z-50",
        isSuspended
          ? "border-red-400 hover:border-red-500"
          : programRunnable
            ? "border-slate-200 hover:border-slate-300"
            : "border-yellow-400 hover:border-yellow-500"
      )}
      title={tooltip}
    >
      {/* Main task info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <h3 className="font-semibold text-slate-900 truncate">
            {task.name || t("task.defaultNameDisplay")}
          </h3>
          {running && (
            <span className="px-2 py-1 text-xs bg-green-100 text-green-800 rounded-full">
              {t("task.running")}
            </span>
          )}
        </div>
        <p className="text-xs text-slate-600 truncate mb-1">{task.program}</p>
        <div className="flex flex-wrap gap-2 text-xs text-slate-500">
          <span>{getTriggerLabel()}</span>
          {(task.last_exit_code != null) && (
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
          )}
          {task.last_run_at && (
            <span className="text-slate-400">
              {t("task.last", { time: formatLastRun(task.last_run_at) })}
            </span>
          )}
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center gap-2">
        {/* Drag handle — only this element activates dnd-kit */}
        <Button
          ref={setActivatorNodeRef}
          size="sm"
          variant="ghost"
          className="cursor-grab active:cursor-grabbing text-slate-400 hover:text-slate-600"
          title={t("button.reorder")}
          {...attributes}
          {...listeners}
        >
          <GripVertical className="h-4 w-4" />
        </Button>

        {/* Toggle switch */}
        <Switch
          checked={task.enabled}
          onCheckedChange={(checked) => task.id && onToggleEnabled(task.id, checked)}
          className="data-[state=unchecked]:bg-slate-200"
        />

        {/* Manual run/stop button */}
        {running ? (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => task.id && onStop(task.id)}
            className="bg-red-500 hover:bg-red-600 text-white hover:text-white"
            title={t("button.stop")}
          >
            <Square className="h-4 w-4" />
          </Button>
        ) : (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onRun(task.id)}
            disabled={!task.enabled}
            title={t("button.run")}
          >
            <Play className="h-4 w-4" />
          </Button>
        )}

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
          onClick={() => onDelete(task.id)}
          className="text-red-600 hover:text-red-700 hover:bg-red-50"
          title={t("button.delete")}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

// ---------- 列表主组件 ----------

export function TaskList({
  tasks,
  onEdit,
  onDelete,
  onRun,
  onStop,
  onToggleEnabled,
  onReorder,
  isRunning = {},
  runnablePrograms = {},
  taskStatuses = {},
}: TaskListProps) {
  const { t } = useTranslation();
  const [deleteTargetId, setDeleteTargetId] = useState<number | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 5 },
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    onReorder(Number(active.id), Number(over.id));
  };

  const handleDeleteRequest = (id: number | undefined) => {
    if (id) setDeleteTargetId(id);
  };

  const handleConfirmDelete = () => {
    if (deleteTargetId !== null) {
      onDelete(deleteTargetId);
      setDeleteTargetId(null);
    }
  };

  const handleRun = (id: number | undefined) => {
    if (id) onRun(id);
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
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext
          items={tasks.map((t) => t.id!)}
          strategy={verticalListSortingStrategy}
        >
          <div className="space-y-2">
            {tasks.map((task) => (
              <SortableTaskItem
                key={task.id}
                task={task}
                running={!!isRunning[task.id!]}
                programRunnable={runnablePrograms[task.id!] !== false}
                isSuspended={taskStatuses[task.id!] === TaskStatus.Suspended}
                onEdit={onEdit}
                onDelete={handleDeleteRequest}
                onRun={handleRun}
                onStop={onStop}
                onToggleEnabled={onToggleEnabled}
              />
            ))}
          </div>
        </SortableContext>
      </DndContext>

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
            <Button variant="destructive" onClick={handleConfirmDelete}>
              {t("button.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
