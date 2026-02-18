import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import type { AppConfig } from "@/types/config";
import { configApi } from "@/lib/api";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

interface AppConfigDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  isLoading?: boolean;
}

export function AppConfigDialog({
  open,
  onOpenChange,
  isLoading = false,
}: AppConfigDialogProps) {
  const { t } = useTranslation();
  const [config, setConfig] = useState<AppConfig>({
    quiet_launch: false,
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (open) {
      loadConfig();
    }
  }, [open]);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const loadedConfig = await configApi.getConfig();
      setConfig(loadedConfig);
    } catch (err) {
      console.error("Failed to load config:", err);
      toast.error(t("toast.loadConfigFailed"));
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      await configApi.saveConfig(config);
      onOpenChange(false);
      toast.success(t("toast.saveSuccess"));
    } catch (err) {
      console.error("Failed to save config:", err);
      toast.error(t("toast.saveFailed"), {
        description: err instanceof Error ? err.message : t("toast.unknownError"),
      });
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>{t("dialog.settingsTitle")}</DialogTitle>
          <DialogDescription>
            {t("dialog.settingsDesc")}
          </DialogDescription>
        </DialogHeader>

        {loading ? (
          <div className="flex items-center justify-center py-8">
            <div className="space-y-2 text-center">
              <div className="h-6 w-6 border-3 border-slate-200 border-t-slate-900 rounded-full animate-spin mx-auto" />
              <p className="text-xs text-slate-600">{t("status.loading")}</p>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            {/* Quiet Launch */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="quiet-launch">{t("form.quietLaunch")}</Label>
                <Switch
                  id="quiet-launch"
                  checked={config.quiet_launch}
                  onCheckedChange={(checked) =>
                    setConfig((prev) => ({
                      ...prev,
                      quiet_launch: checked,
                    }))
                  }
                />
              </div>
              <p className="text-xs text-slate-500">{t("form.quietLaunchDesc")}</p>
            </div>
          </div>
        )}

        <DialogFooter className="gap-2">
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isLoading || loading}
          >
            {t("button.cancel")}
          </Button>
          <Button onClick={handleSave} disabled={isLoading || loading}>
            {isLoading ? t("button.saving") : t("button.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
