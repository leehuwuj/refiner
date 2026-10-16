//! Opening, focusing and positioning Refiner's three windows.
//!
//! The main window and the quick popup behave like menu-bar panels: they float
//! above other apps, join every Space, and close as soon as they lose focus.
//! Their state lives in models (see `state.rs`), so reopening is seamless.

use gpui_kit::*;

use crate::platform;
use crate::ui::main_view::MainView;
use crate::ui::popup_view::PopupView;
use crate::ui::selection_icon::SelectionIcon;
use crate::ui::settings_view::{SettingsTab, SettingsView};
use crate::ui::apply_appearance;

pub const MAIN_SIZE: (f32, f32) = (660., 340.);
pub const POPUP_SIZE: (f32, f32) = (320., 200.);
pub const SELECTION_ICON_SIZE: f32 = 22.;
pub const SETTINGS_SIZE: (f32, f32) = (560., 560.);
const CURSOR_OFFSET: f32 = 6.;
const EDGE_MARGIN: f32 = 8.;

#[derive(Default)]
struct OpenWindows {
    main: Option<AnyWindowHandle>,
    popup: Option<AnyWindowHandle>,
    selection_icon: Option<AnyWindowHandle>,
    settings: Option<(AnyWindowHandle, WeakEntity<SettingsView>)>,
}

impl Global for OpenWindows {}

fn open_windows(cx: &mut App) -> &mut OpenWindows {
    if !cx.has_global::<OpenWindows>() {
        cx.set_global(OpenWindows::default());
    }
    cx.global_mut::<OpenWindows>()
}

/// Activates `handle` if the window is still open.
fn focus_existing(handle: Option<AnyWindowHandle>, cx: &mut App) -> bool {
    handle.is_some_and(|h| h.update(cx, |_, window, _| window.activate_window()).is_ok())
}

fn panel_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        focus: true,
        show: true,
        kind: WindowKind::PopUp,
        is_movable: true,
        app_owns_titlebar_drag: true,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Blurred,
        app_id: Some(crate::settings::APP_ID.into()),
        ..Default::default()
    }
}

/// Shows the main window: near the menu bar (macOS, top-right) or the top
/// of the screen (elsewhere).
pub fn show_main(cx: &mut App) {
    let existing = open_windows(cx).main;
    if focus_existing(existing, cx) {
        return;
    }
    let (w, h) = MAIN_SIZE;
    let Some(display) = cx.primary_display() else {
        return;
    };
    let screen = display.bounds();
    let x = if cfg!(target_os = "macos") {
        screen.right() - px(w + EDGE_MARGIN * 2.)
    } else {
        screen.origin.x + (screen.size.width - px(w)) / 2.
    };
    let y = screen.origin.y + px(if cfg!(target_os = "macos") { 34. } else { 12. });
    let bounds = Bounds::new(point(x, y), size(px(w), px(h)));

    match gpui_kit::open_window(panel_options(bounds), cx, |window, cx| {
        apply_appearance(window, cx);
        cx.new(|cx| MainView::new(window, cx))
    }) {
        Ok((handle, _)) => open_windows(cx).main = Some(handle),
        Err(err) => eprintln!("refiner: failed to open main window: {err:#}"),
    }
}

/// Opens the quick popup next to the mouse cursor, kept fully on screen.
pub fn show_popup(cx: &mut App) {
    if let Some(old) = open_windows(cx).popup.take() {
        let _ = old.update(cx, |_, window, _| window.remove_window());
    }
    let (w, h) = (px(POPUP_SIZE.0), px(POPUP_SIZE.1));
    let primary = cx.primary_display();
    let primary_height = primary
        .as_ref()
        .map(|d| f32::from(d.bounds().size.height))
        .unwrap_or(0.);
    let cursor = platform::cursor_position(primary_height)
        .map(|(x, y)| point(px(x), px(y)))
        .unwrap_or_else(|| {
            primary
                .as_ref()
                .map(|d| d.bounds().center())
                .unwrap_or_default()
        });

    let displays = cx.displays();
    let screen = displays
        .iter()
        .map(|d| d.bounds())
        .find(|b| b.contains(&cursor))
        .or_else(|| primary.as_ref().map(|d| d.bounds()));
    let offset = px(CURSOR_OFFSET);
    let mut origin = point(cursor.x + offset, cursor.y + offset);
    if let Some(s) = screen {
        // Prefer below-right of the cursor; flip when that would overflow.
        if origin.x + w > s.right() {
            origin.x = (cursor.x - w - offset).max(s.origin.x);
        }
        if origin.y + h > s.bottom() {
            origin.y = (cursor.y - h - offset).max(s.origin.y);
        }
    }
    let bounds = Bounds::new(origin, size(w, h));
    let mut options = panel_options(bounds);
    options.display_id = displays
        .iter()
        .find(|d| d.bounds().contains(&cursor))
        .map(|d| d.id());

    match gpui_kit::open_window(options, cx, |window, cx| {
        apply_appearance(window, cx);
        cx.new(|cx| PopupView::new(window, cx))
    }) {
        Ok((handle, _)) => open_windows(cx).popup = Some(handle),
        Err(err) => eprintln!("refiner: failed to open popup: {err:#}"),
    }
}

/// Shows the floating selection button just below-right of `at` (the mouse
/// position where the selection ended), without taking focus from the app
/// that owns the selection.
pub fn show_selection_icon(at: Point<Pixels>, text: String, cx: &mut App) {
    hide_selection_icon(cx);
    let side = px(SELECTION_ICON_SIZE);
    let mut origin = point(at.x + px(8.), at.y + px(10.));
    if let Some(s) = cx
        .displays()
        .iter()
        .map(|d| d.bounds())
        .find(|b| b.contains(&at))
    {
        if origin.x + side > s.right() {
            origin.x = at.x - side - px(10.);
        }
        if origin.y + side > s.bottom() {
            origin.y = at.y - side - px(12.);
        }
    }
    let mut options = panel_options(Bounds::new(origin, size(side, side)));
    options.focus = false;
    options.is_movable = false;
    options.window_background = WindowBackgroundAppearance::Transparent;
    match gpui_kit::open_window(options, cx, |window, cx| {
        apply_appearance(window, cx);
        cx.new(|cx| SelectionIcon::new(text, window, cx))
    }) {
        Ok((handle, _)) => {
            debug_log!("selection: icon shown at {origin:?}");
            open_windows(cx).selection_icon = Some(handle)
        }
        Err(err) => eprintln!("refiner: failed to open selection icon: {err:#}"),
    }
}

pub fn hide_selection_icon(cx: &mut App) {
    if let Some(handle) = open_windows(cx).selection_icon.take() {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }
}

/// Opens (or focuses) the settings window on `tab`.
pub fn show_settings(tab: SettingsTab, cx: &mut App) {
    cx.activate(true);
    if let Some((handle, view)) = open_windows(cx).settings.clone() {
        let alive = handle
            .update(cx, |_, window, cx| {
                window.activate_window();
                window.set_window_title(tab.label());
                let _ = view.update(cx, |v, cx| v.set_tab(tab, cx));
            })
            .is_ok();
        if alive {
            return;
        }
    }
    let (w, h) = SETTINGS_SIZE;
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(w), px(h)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Refiner Settings".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        focus: true,
        show: true,
        kind: WindowKind::Normal,
        is_resizable: true,
        window_min_size: Some(size(px(440.), px(480.))),
        app_id: Some(crate::settings::APP_ID.into()),
        ..Default::default()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        apply_appearance(window, cx);
        cx.new(|cx| SettingsView::new(tab, window, cx))
    }) {
        Ok((handle, view)) => open_windows(cx).settings = Some((handle, view.downgrade())),
        Err(err) => eprintln!("refiner: failed to open settings: {err:#}"),
    }
}

/// Closes a panel window when it loses focus. Call from the view's constructor.
pub fn close_on_blur<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Subscription {
    cx.observe_window_activation(window, |_, window, _| {
        if std::env::var_os("REFINER_DEBUG").is_some() {
            eprintln!("refiner: activation changed: active={}", window.is_window_active());
        }
        // Snapshot runs must not depend on which app is frontmost.
        let snapshot = cfg!(feature = "snapshot") && std::env::var_os("REFINER_SNAPSHOT").is_some();
        if !window.is_window_active() && !snapshot {
            window.remove_window();
        }
    })
}
