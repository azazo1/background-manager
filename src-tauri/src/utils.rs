use std::path::Path;

use tauri::{AppHandle, Manager};

pub(crate) trait EnsureDirExists: Sized {
    fn ensure_dir_exists(self) -> crate::Result<Self>;
}

impl<T> EnsureDirExists for T
where
    T: AsRef<Path>,
{
    fn ensure_dir_exists(self) -> crate::Result<Self> {
        std::fs::create_dir_all(self.as_ref()).map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Io,
                format!("failed to create directory: {}", self.as_ref().display()),
                Box::new(e),
            )
        })?;
        Ok(self)
    }
}

/// 在 dock 栏中显示/隐藏图标 (仅 macos)
pub(crate) fn toggle_dock_icon(show: bool) {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSApplication;
        use objc2_app_kit::NSApplicationActivationPolicy;

        // 获取当前应用实例
        let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
        // 设置为 Accessory 模式（即从 Dock 移除，但在托盘可见）
        app.setActivationPolicy(if show {
            NSApplicationActivationPolicy::Regular
        } else {
            NSApplicationActivationPolicy::Accessory
        });
    }
}

pub(crate) fn toggle_window(app: &AppHandle, show: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if show {
            window.unminimize().ok();
            window.show().ok();
            window.set_focus().ok();
        } else {
            window.minimize().ok();
        }
    }
    toggle_dock_icon(show);
}
