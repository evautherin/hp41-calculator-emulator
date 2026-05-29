//! macOS menu-bar (status-item) mode for the HP-41 GUI.
//!
//! Everything here is macOS-only. On other platforms the app keeps its normal
//! decorated window (see `apply_menu_bar_mode` callers in lib.rs). The frontend
//! and IPC contract are unchanged; this module only manages the window's
//! presentation (Accessory policy, decorations off, always-on-top, hidden start)
//! and a tray icon whose left-click toggles a borderless popover and whose
//! right-click opens a small menu.

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;

use crate::tray_helpers::{compute_popover_position, fit_inner_height, should_show_after_hide};

/// Shared state for the popover toggle. Managed via `app.manage`.
/// - `last_hidden`: when the window was last hidden (flicker-guard, see tray_helpers).
/// - `suppress_hide`: true while a native file dialog is open, so the blur it
///   causes does not hide the popover (set/cleared in commands.rs).
#[derive(Default)]
pub struct PopoverState {
    pub last_hidden: Mutex<Option<Instant>>,
    // Read/written by commands.rs (file-dialog suppression) in a later task of
    // the menu-bar plan; declared here so the managed state already carries the
    // field. Scoped allow (not blanket) until that wiring lands.
    #[allow(dead_code)]
    pub suppress_hide: AtomicBool,
}

const DEBOUNCE: Duration = Duration::from_millis(250);
const DESIGN_HEIGHT: f64 = 1020.0;
const DESIGN_WIDTH: f64 = 440.0;
const MAX_SCREEN_FRACTION: f64 = 0.92;

/// Resize the popover to fit the current monitor height, position it centered
/// under the tray icon, then show + focus it.
fn show_popover(window: &WebviewWindow, icon_x: i32, icon_w: i32, icon_bottom: i32) {
    // Clamp height to the monitor so a 1020px layout fits short laptop screens.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = monitor.scale_factor();
        let screen_h_logical = monitor.size().height as f64 / scale;
        let h = fit_inner_height(DESIGN_HEIGHT, screen_h_logical, MAX_SCREEN_FRACTION);
        let _ = window.set_size(LogicalSize::new(DESIGN_WIDTH, h));
    }

    // Position using physical pixels (icon rect is physical).
    let win_w_physical = window.outer_size().map(|s| s.width as i32).unwrap_or(440);
    let pos = compute_popover_position(icon_x, icon_w, icon_bottom, win_w_physical);
    let _ = window.set_position(PhysicalPosition::new(pos.x, pos.y));

    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_popover(app: &AppHandle, window: &WebviewWindow) {
    let _ = window.hide();
    if let Some(state) = app.try_state::<PopoverState>() {
        if let Ok(mut g) = state.last_hidden.lock() {
            *g = Some(Instant::now());
        }
    }
}

/// Build the tray icon, its menu, and event handlers. macOS only.
pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let handle = app.handle();

    // ---- right-click menu ----
    let about_i = MenuItemBuilder::with_id("about", "About HP-41 Calculator").build(app)?;
    let start_login_i = CheckMenuItemBuilder::with_id("start_login", "Start at Login")
        .checked(handle.autolaunch().is_enabled().unwrap_or(false))
        .build(app)?;
    let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&about_i)
        .item(&start_login_i)
        .separator()
        .item(&quit_i)
        .build()?;

    let icon_path = app
        .path()
        .resolve("icons/tray-template.png", tauri::path::BaseDirectory::Resource)
        .unwrap_or_else(|_| "icons/tray-template.png".into());

    TrayIconBuilder::with_id("hp41-tray")
        .icon(tauri::image::Image::from_path(icon_path)?)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false) // left click is the popover toggle, not the menu
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "about" => {
                let version = env!("CARGO_PKG_VERSION");
                app.dialog()
                    .message(format!("HP-41 Calculator\nVersion {version}"))
                    .title("About")
                    .blocking_show();
            }
            "start_login" => {
                let mgr = app.autolaunch();
                let now_enabled = mgr.is_enabled().unwrap_or(false);
                let _ = if now_enabled { mgr.disable() } else { mgr.enable() };
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                let Some(window) = app.get_webview_window("main") else {
                    return;
                };
                let visible = window.is_visible().unwrap_or(false);
                if visible {
                    hide_popover(app, &window);
                    return;
                }
                // Hidden: apply flicker-guard before re-showing.
                let last_hidden = app
                    .try_state::<PopoverState>()
                    .and_then(|s| s.last_hidden.lock().ok().map(|g| *g))
                    .flatten();
                if !should_show_after_hide(last_hidden, Instant::now(), DEBOUNCE) {
                    return; // this click is the one that just closed the popover
                }
                // `rect.position` / `rect.size` are `dpi::Position` / `dpi::Size`
                // enums in 2.11. Convert to physical pixels using the window's
                // scale factor, then feed plain integers to the pure helpers.
                let scale = window.scale_factor().unwrap_or(1.0);
                let pos_phys = rect.position.to_physical::<i32>(scale);
                let size_phys = rect.size.to_physical::<i32>(scale);
                let icon_x = pos_phys.x;
                let icon_w = size_phys.width;
                let icon_bottom = pos_phys.y + size_phys.height;
                show_popover(&window, icon_x, icon_w, icon_bottom);
            }
        })
        .build(app)?;

    Ok(())
}

/// Apply pure menu-bar presentation to the main window: no Dock icon, no
/// decorations, always on top, hidden until the tray toggles it.
pub fn apply_menu_bar_mode(app: &mut App) -> tauri::Result<()> {
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_decorations(false);
        let _ = win.set_always_on_top(true);
        let _ = win.hide();
    }
    setup_tray(app)
}
