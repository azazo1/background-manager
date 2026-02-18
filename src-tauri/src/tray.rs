use tauri::{
    App, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tracing::debug;

use crate::{config::PKG_NAME, utils::toggle_window};

pub(crate) fn init_tray(app: &mut App) -> crate::Result<()> {
    let failed_to_create_menu = |e| {
        crate::Error::with_source(
            crate::ErrorKind::Tray,
            "failed to create tray menu",
            Box::new(e),
        )
    };
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(failed_to_create_menu)?;
    let show_i = MenuItem::with_id(app, "show", "显示", true, None::<&str>)
        .map_err(failed_to_create_menu)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i]).map_err(failed_to_create_menu)?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip(PKG_NAME)
        .show_menu_on_left_click(false)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                let handle = app.clone();
                tokio::spawn(async move {
                    let state = handle.state();
                    crate::commands::exit(handle.clone(), state).await.ok();
                });
            }
            "show" => {
                toggle_window(app, true);
            }
            _ => {
                debug!("menu item {:?} not handled", event.id);
            }
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                // 点击托盘图标展示并聚焦于主窗口
                toggle_window(tray.app_handle(), true);
            }
            _ => {
                debug!("unhandled tray event: {event:?}");
            }
        })
        .build(app)
        .map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Tray,
                "failed to build tray icon",
                Box::new(e),
            )
        })?;
    Ok(())
}
