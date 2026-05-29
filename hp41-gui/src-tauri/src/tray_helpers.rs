//! Pure helper logic for the macOS menu-bar popover (Task 3 of the menu-bar plan).
//!
//! These functions take plain numbers (physical pixels / durations) instead of
//! Tauri types so they can be unit-tested without a running application. The
//! tray event handlers in `tray.rs` call them.

use std::time::{Duration, Instant};

/// Horizontal/vertical position (physical pixels) for the popover window so it
/// sits centered under the tray icon, just below the menu bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopoverPos {
    pub x: i32,
    pub y: i32,
}

/// Center the window horizontally under the icon and place its top at the icon's
/// bottom edge. `icon_x` is the icon rect's left; `icon_w` its width; `icon_bottom`
/// the icon rect's bottom edge; `window_w` the window width. All physical pixels.
pub fn compute_popover_position(
    icon_x: i32,
    icon_w: i32,
    icon_bottom: i32,
    window_w: i32,
) -> PopoverPos {
    let icon_center_x = icon_x + icon_w / 2;
    PopoverPos {
        x: icon_center_x - window_w / 2,
        y: icon_bottom,
    }
}

/// Decide whether a tray left-click on a currently-hidden window should SHOW it.
///
/// When the popover is open and the user clicks the tray icon, macOS fires a
/// window blur (which hides the popover and records `last_hidden`) *before* the
/// tray click event. Without a guard, the click would immediately re-show the
/// just-hidden window ("flicker"). If the window was hidden within `debounce` of
/// now, we treat this click as the closing click and do NOT re-show.
pub fn should_show_after_hide(
    last_hidden: Option<Instant>,
    now: Instant,
    debounce: Duration,
) -> bool {
    match last_hidden {
        Some(t) => now.duration_since(t) >= debounce,
        None => true,
    }
}

/// Clamp the popover's inner height to a fraction of the screen height so it never
/// runs off the bottom of the display. Returns the height to apply (logical px).
pub fn fit_inner_height(design_h: f64, screen_h: f64, max_fraction: f64) -> f64 {
    let cap = screen_h * max_fraction;
    design_h.min(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_window_under_icon() {
        // icon at x=100, width=24 -> center 112; window 440 wide -> x = 112-220 = -108
        let pos = compute_popover_position(100, 24, 30, 440);
        assert_eq!(pos, PopoverPos { x: -108, y: 30 });
    }

    #[test]
    fn places_top_at_icon_bottom() {
        let pos = compute_popover_position(500, 24, 38, 440);
        assert_eq!(pos.y, 38);
    }

    #[test]
    fn show_allowed_when_never_hidden() {
        assert!(should_show_after_hide(None, Instant::now(), Duration::from_millis(250)));
    }

    #[test]
    fn show_suppressed_within_debounce() {
        let now = Instant::now();
        assert!(!should_show_after_hide(Some(now), now, Duration::from_millis(250)));
    }

    #[test]
    fn show_allowed_after_debounce_elapsed() {
        let now = Instant::now();
        let long_ago = now - Duration::from_millis(500);
        assert!(should_show_after_hide(Some(long_ago), now, Duration::from_millis(250)));
    }

    #[test]
    fn height_uncapped_on_tall_screen() {
        // 1020 design fits within 92% of a 1440 screen (1324) -> stays 1020
        assert_eq!(fit_inner_height(1020.0, 1440.0, 0.92), 1020.0);
    }

    #[test]
    fn height_capped_on_short_screen() {
        // 92% of 800 = 736 -> capped
        assert_eq!(fit_inner_height(1020.0, 800.0, 0.92), 736.0);
    }
}
