import { useState, useEffect } from "react";
import { Plus, Trash2, FolderOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import type { Task, Trigger } from "@/types/task";
import { taskApi } from "@/lib/api";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface TaskEditDialogProps {
  open: boolean;
  task?: Task;
  onOpenChange: (open: boolean) => void;
  onSave: (task: Task) => void;
  isLoading?: boolean;
}

type TriggerType = "Manual" | "Startup" | "KeepAlive" | "Routine" | "Instant" | "UntilSucceed";

const getProgramBaseName = (programPath: string) => {
  if (!programPath) return "";
  let normalized = programPath.replace(/\\/g, "/");
  normalized = normalized.replace(/\/*$/, '');
  return normalized.split("/").pop() || "";
};

export function TaskEditDialog({
  open,
  task,
  onOpenChange,
  onSave,
  isLoading = false,
}: TaskEditDialogProps) {
  const { t } = useTranslation();
  const [formData, setFormData] = useState<Task>({
    name: "",
    program: "",
    args: [],
    trigger: { tag: "Manual" },
    enabled: true,
  });

  const [triggerType, setTriggerType] = useState<TriggerType>("Manual");
  const [routineMs, setRoutineMs] = useState<number>(5000);
  const [instantTime, setInstantTime] = useState<string>("");
  const [browsingProgram, setBrowsingProgram] = useState(false);
  const [isNameAuto, setIsNameAuto] = useState(true);

  useEffect(() => {
    if (task) {
      setFormData(task);
      setIsNameAuto(false);
      if (typeof task.trigger === "object" && "tag" in task.trigger) {
        setTriggerType(task.trigger.tag);
        if (task.trigger.tag === "Routine") {
          setRoutineMs((task.trigger as any).content);
        } else if (task.trigger.tag === "Instant") {
          setInstantTime((task.trigger as any).content);
        }
      }
    } else {
      setFormData({
        name: "",
        program: "",
        args: [],
        trigger: { tag: "Manual" },
        enabled: true,
      });
      setTriggerType("Manual");
      setRoutineMs(5000);
      setInstantTime("");
      setIsNameAuto(true);
    }
  }, [task, open]);

  const handleProgramChange = (value: string) => {
    const shouldAutoName = !formData.name || isNameAuto;
    const derivedName = shouldAutoName ? getProgramBaseName(value) : formData.name;

    setFormData((prev) => ({
      ...prev,
      program: value,
      name: derivedName,
    }));

    if (shouldAutoName) {
      setIsNameAuto(true);
    }
  };

  const handleBrowseProgram = async () => {
    try {
      setBrowsingProgram(true);
      const filePath = await taskApi.pickFile();
      if (filePath) {
        handleProgramChange(filePath);
      }
    } catch (err) {
      console.error("Failed to pick file:", err);
    } finally {
      setBrowsingProgram(false);
    }
  };

  const handleTriggerTypeChange = (type: TriggerType) => {
    setTriggerType(type);
    let newTrigger: Trigger;

    switch (type) {
      case "Routine":
        newTrigger = { tag: "Routine", content: routineMs };
        break;
      case "Instant":
        newTrigger = { tag: "Instant", content: instantTime };
        break;
      case "Startup":
        newTrigger = { tag: "Startup" };
        break;
      case "KeepAlive":
        newTrigger = { tag: "KeepAlive" };
        break;
      case "UntilSucceed":
        newTrigger = { tag: "UntilSucceed" };
        break;
      case "Manual":
      default:
        newTrigger = { tag: "Manual" };
    }

    setFormData((prev) => ({ ...prev, trigger: newTrigger }));
  };

  const handleRoutineChange = (ms: number) => {
    setRoutineMs(ms);
    setFormData((prev) => ({
      ...prev,
      trigger: { tag: "Routine", content: ms },
    }));
  };

  const handleInstantChange = (time: string) => {
    setInstantTime(time);
    setFormData((prev) => ({
      ...prev,
      trigger: { tag: "Instant", content: time },
    }));
  };

  const handleArgChange = (index: number, value: string) => {
    const newArgs = [...formData.args];
    newArgs[index] = value;
    setFormData((prev) => ({ ...prev, args: newArgs }));
  };

  const handleAddArg = () => {
    setFormData((prev) => ({
      ...prev,
      args: [...prev.args, ""],
    }));
  };

  const handleRemoveArg = (index: number) => {
    setFormData((prev) => ({
      ...prev,
      args: prev.args.filter((_, i) => i !== index),
    }));
  };

  const handleSave = () => {
    if (!formData.program) {
      toast.error(t("validation.required"), {
        description: t("validation.fillRequired")
      });
      return;
    }
    const trimmedName = formData.name.trim();
    const derivedName = trimmedName || getProgramBaseName(formData.program);
    onSave({ ...formData, name: derivedName });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {task?.id ? t("dialog.editTitle") : t("dialog.createTitle")}
          </DialogTitle>
          <DialogDescription>
            {task?.id ? t("dialog.editDesc") : t("dialog.createDesc")}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Task Name */}
          <div className="space-y-2">
            <Label htmlFor="task-name">
              {t("form.taskName")}
            </Label>
            <Input
              id="task-name"
              placeholder="e.g., Database Backup"
              value={formData.name}
              onChange={(e) => {
                const value = e.target.value;
                setFormData((prev) => ({ ...prev, name: value }));
                setIsNameAuto(value.trim().length === 0);
              }}
            />
          </div>

          {/* Program Path */}
          <div className="space-y-2">
            <Label htmlFor="program">
              {t("form.programPath")} {t("form.required")}
            </Label>
            <div className="flex gap-2">
              <Input
                id="program"
                placeholder="/path/to/program or program.exe"
                value={formData.program}
                onChange={(e) => handleProgramChange(e.target.value)}
                className="flex-1"
              />
              <Button
                size="sm"
                variant="outline"
                onClick={handleBrowseProgram}
                disabled={browsingProgram}
                className="shrink-0"
              >
                <FolderOpen className="h-4 w-4 mr-1" />
                {t("button.browse")}
              </Button>
            </div>
          </div>

          {/* Arguments */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label>{t("form.arguments")}</Label>
              <Button
                size="sm"
                variant="outline"
                onClick={handleAddArg}
                className="text-xs"
              >
                <Plus className="h-3 w-3 mr-1" />
                {t("button.addArgument")}
              </Button>
            </div>
            <p className="text-xs text-slate-500">{t("form.argumentsNote")}</p>
            {formData.args.length > 0 ? (
              <div className="space-y-2">
                {formData.args.map((arg, index) => (
                  <div key={index} className="flex gap-2">
                    <Input
                      placeholder={t("form.argumentPlaceholder", { number: index + 1 })}
                      value={arg}
                      onChange={(e) => handleArgChange(index, e.target.value)}
                      className="flex-1"
                    />
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleRemoveArg(index)}
                      className="text-red-600 hover:text-red-700 hover:bg-red-50"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-slate-500">{t("status.noArguments")}</p>
            )}
          </div>

          {/* Trigger Type */}
          <div className="space-y-2">
            <Label htmlFor="trigger">{t("form.triggerMode")}</Label>
            <Select value={triggerType} onValueChange={handleTriggerTypeChange}>
              <SelectTrigger id="trigger">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="Manual">{t("trigger.manual")}</SelectItem>
                <SelectItem value="Startup">{t("trigger.startup")}</SelectItem>
                <SelectItem value="KeepAlive">{t("trigger.keepAlive")}</SelectItem>
                <SelectItem value="UntilSucceed">{t("trigger.untilSucceed")}</SelectItem>
                <SelectItem value="Routine">{t("trigger.routine")}</SelectItem>
                <SelectItem value="Instant">{t("trigger.instant")}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Trigger-specific options */}
          {triggerType === "Routine" && (
            <div className="space-y-2">
              <Label htmlFor="routine-interval">{t("form.intervalMs")}</Label>
              <Input
                id="routine-interval"
                type="number"
                placeholder="5000"
                value={routineMs}
                onChange={(e) => handleRoutineChange(Number(e.target.value))}
                min="100"
              />
              <p className="text-xs text-slate-500">
                {t("form.intervalNote", { seconds: (routineMs / 1000).toFixed(1) })}
              </p>
            </div>
          )}

          {triggerType === "Instant" && (
            <div className="space-y-2">
              <Label htmlFor="instant-time">{t("form.scheduledTime")}</Label>
              <Input
                id="instant-time"
                type="datetime-local"
                value={instantTime}
                onChange={(e) => handleInstantChange(e.target.value)}
              />
            </div>
          )}

          {/* File Redirections */}
          <div className="border-t pt-4 space-y-3">
            <p className="font-semibold text-sm">{t("form.fileRedirections")}</p>

            <div className="space-y-2">
              <Label htmlFor="stdin">{t("form.standardInput")}</Label>
              <Input
                id="stdin"
                placeholder="/path/to/input/file"
                value={formData.stdin || ""}
                onChange={(e) =>
                  setFormData((prev) => ({
                    ...prev,
                    stdin: e.target.value || undefined,
                  }))
                }
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="stdout">{t("form.standardOutput")}</Label>
              <Input
                id="stdout"
                placeholder="/path/to/output/file"
                value={formData.stdout || ""}
                onChange={(e) =>
                  setFormData((prev) => ({
                    ...prev,
                    stdout: e.target.value || undefined,
                  }))
                }
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="stderr">{t("form.standardError")}</Label>
              <Input
                id="stderr"
                placeholder="/path/to/error/file"
                value={formData.stderr || ""}
                onChange={(e) =>
                  setFormData((prev) => ({
                    ...prev,
                    stderr: e.target.value || undefined,
                  }))
                }
              />
            </div>
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isLoading}
          >
            {t("button.cancel")}
          </Button>
          <Button onClick={handleSave} disabled={isLoading || browsingProgram}>
            {isLoading ? t("button.saving") : task?.id ? t("button.update") : t("button.create")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
