//! Opening, focusing and positioning Refiner's three windows.
//!
//! The main window and the quick popup behave like menu-bar panels: they float
//! above other apps, join every Space, and close as soon as they lose focus.
//! Their state lives in models (see `state.rs`), so reopening is seamless.

use gpui_kit::*;

use crate::platform;
use crate::ui::apply_appearance;
use crate::ui::main_view::MainView;
use crate::ui::popup_view::PopupView;
#[cfg(not(target_os = "windows"))]
use crate::ui::selection_icon::SelectionIcon;
use crate::ui::settings_view::{SettingsTab, SettingsView};

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
    #[cfg(not(target_os = "windows"))]
    selection_icon: Option<(AnyWindowHandle, WeakEntity<SelectionIcon>)>,
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
    handle.is_some_and(|h| {
        h.update(cx, |_, window, _| window.activate_window())
            .is_ok()
    })
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
        Ok((handle, _)) => {
            open_windows(cx).main = Some(handle);
            #[cfg(target_os = "windows")]
            let _ = handle.update(cx, |_, window, cx| {
                window.defer(cx, |window, _| native_panel::activate(window))
            });
        }
        Err(err) => eprintln!("refiner: failed to open main window: {err:#}"),
    }
}

/// Opens the quick popup next to the mouse cursor, kept fully on screen.
pub fn show_popup(cx: &mut App) {
    if let Some(old) = open_windows(cx).popup.take() {
        let _ = old.update(cx, |_, window, _| close_panel(window));
    }
    let (w, h) = (px(POPUP_SIZE.0), px(POPUP_SIZE.1));
    let primary = cx.primary_display();
    let primary_height = primary
        .as_ref()
        .map(|d| f32::from(d.bounds().size.height))
        .unwrap_or(0.);
    let location = platform::cursor_position(primary_height);
    let cursor = location
        .map(|(x, y, _)| point(px(x), px(y)))
        .unwrap_or_else(|| {
            primary
                .as_ref()
                .map(|d| d.bounds().center())
                .unwrap_or_default()
        });

    let displays = cx.displays();
    let target_display = location
        .and_then(|(_, _, native)| native)
        .and_then(|native| {
            displays
                .iter()
                .find(|display| u64::from(display.id()) == native)
                .cloned()
        })
        .or_else(|| {
            displays
                .iter()
                .find(|display| display.bounds().contains(&cursor))
                .cloned()
        })
        .or_else(|| primary.clone());
    let screen = target_display
        .as_ref()
        .map(|display| display.visible_bounds());
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
        origin.x = origin
            .x
            .max(s.origin.x)
            .min((s.right() - w).max(s.origin.x));
        origin.y = origin
            .y
            .max(s.origin.y)
            .min((s.bottom() - h).max(s.origin.y));
    }
    let bounds = Bounds::new(origin, size(w, h));
    let mut options = panel_options(bounds);
    options.display_id = target_display.as_ref().map(|display| display.id());

    match gpui_kit::open_window(options, cx, |window, cx| {
        apply_appearance(window, cx);
        cx.new(|cx| PopupView::new(window, cx))
    }) {
        Ok((handle, _)) => {
            open_windows(cx).popup = Some(handle);
            #[cfg(target_os = "windows")]
            let _ = handle.update(cx, |_, window, cx| {
                window.defer(cx, |window, _| native_panel::activate(window))
            });
        }
        Err(err) => eprintln!("refiner: failed to open popup: {err:#}"),
    }
}

/// Shows the selection button above the text's top-left on Windows, or beside
/// the selection-ending cursor on macOS, without taking focus from its app.
#[cfg(any(not(target_os = "windows"), feature = "snapshot", test))]
pub fn show_selection_icon(at: Point<Pixels>, text: String, cx: &mut App) {
    let side = px(SELECTION_ICON_SIZE);
    let mut origin = if cfg!(target_os = "windows") {
        point(at.x, at.y - side - px(6.))
    } else {
        point(at.x + px(8.), at.y + px(10.))
    };
    if let Some(s) = cx
        .displays()
        .iter()
        .map(|d| d.bounds())
        .find(|b| b.contains(&at))
    {
        #[cfg(target_os = "windows")]
        {
            origin.x = origin
                .x
                .max(s.origin.x + px(4.))
                .min(s.right() - side - px(4.));
            origin.y = origin
                .y
                .max(s.origin.y + px(4.))
                .min(s.bottom() - side - px(4.));
        }
        if origin.x + side > s.right() {
            origin.x = at.x - side - px(10.);
        }
        if origin.y + side > s.bottom() {
            origin.y = at.y - side - px(12.);
        }
    }
    let display_id = cx
        .displays()
        .iter()
        .find(|display| display.bounds().contains(&at))
        .map(|display| display.id());
    #[cfg(target_os = "windows")]
    {
        crate::windows_selection_button::show(origin, display_id, text, cx);
    }
    #[cfg(not(target_os = "windows"))]
    {
        hide_selection_icon(cx);
        let mut options = panel_options(Bounds::new(origin, size(side, side)));
        options.display_id = display_id;
        options.focus = false;
        options.is_movable = false;
        options.window_background = WindowBackgroundAppearance::Transparent;
        match gpui_kit::open_window(options, cx, |window, cx| {
            apply_appearance(window, cx);
            cx.new(|cx| SelectionIcon::new(text, window, cx))
        }) {
            Ok((handle, view)) => {
                debug_log!("selection: icon shown at {origin:?}");
                open_windows(cx).selection_icon = Some((handle, view.downgrade()))
            }
            Err(err) => eprintln!("refiner: failed to open selection icon: {err:#}"),
        }
    }
}

pub fn hide_selection_icon(cx: &mut App) {
    #[cfg(target_os = "windows")]
    crate::windows_selection_button::hide(cx);
    #[cfg(not(target_os = "windows"))]
    if let Some((handle, _)) = open_windows(cx).selection_icon.take() {
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
    #[cfg(target_os = "windows")]
    {
        native_panel::watch(window, cx)
    }
    #[cfg(not(target_os = "windows"))]
    {
        cx.observe_window_activation(window, |_, window, _| {
            let snapshot =
                cfg!(feature = "snapshot") && std::env::var_os("REFINER_SNAPSHOT").is_some();
            if !window.is_window_active() && !snapshot {
                window.remove_window();
            }
        })
    }
}
/// Close a floating panel, including its native Windows surface.
pub fn close_panel(window: &mut Window) {
    #[cfg(target_os = "windows")]
    native_panel::hide(window);
    window.remove_window();
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod native_panel {
    use super::*;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    #[cfg(test)]
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SW_HIDE, SetForegroundWindow, ShowWindowAsync, WA_INACTIVE, WM_ACTIVATE, WM_NCDESTROY,
    };

    fn hwnd(window: &Window) -> Option<HWND> {
        match HasWindowHandle::window_handle(window).ok()?.as_raw() {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as HWND),
            _ => None,
        }
    }

    pub fn hide(window: &Window) {
        if let Some(hwnd) = hwnd(window) {
            // Queue native hiding rather than re-entering GPUI's activation
            // callback through synchronous ShowWindow.
            unsafe {
                ShowWindowAsync(hwnd, SW_HIDE);
            }
        }
    }

    pub fn activate(window: &Window) {
        if let Some(hwnd) = hwnd(window) {
            // Showing a tool window does not guarantee foreground activation.
            // A shortcut request must activate it so clicking elsewhere sends
            // a real WM_ACTIVATE/WA_INACTIVE to this HWND.
            let activated = unsafe { SetForegroundWindow(hwnd) };
            debug_log!("panel: requested native activation, result={activated}");
        }
    }

    const SUBCLASS_ID: usize = 0x5246504e;

    unsafe extern "system" fn panel_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        id: usize,
        data: usize,
    ) -> LRESULT {
        // The subclass owns this sender until WM_NCDESTROY. Windows invokes
        // it on the window's UI thread; never borrow GPUI from a WndProc.
        unsafe {
            if message == WM_ACTIVATE && (wparam & 0xffff) == WA_INACTIVE as usize {
                ShowWindowAsync(hwnd, SW_HIDE);
                let sender = &*(data as *const smol::channel::Sender<()>);
                let _ = sender.try_send(());
            }
            if message == WM_NCDESTROY {
                RemoveWindowSubclass(hwnd, Some(panel_proc), id);
                let result = DefSubclassProc(hwnd, message, wparam, lparam);
                drop(Box::from_raw(data as *mut smol::channel::Sender<()>));
                return result;
            }
            DefSubclassProc(hwnd, message, wparam, lparam)
        }
    }

    pub fn watch<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Subscription {
        let snapshot = cfg!(feature = "snapshot") && std::env::var_os("REFINER_SNAPSHOT").is_some();
        if !snapshot && let Some(hwnd) = hwnd(window) {
            let (tx, rx) = smol::channel::bounded::<()>(1);
            let data = Box::into_raw(Box::new(tx));
            let installed =
                unsafe { SetWindowSubclass(hwnd, Some(panel_proc), SUBCLASS_ID, data as usize) };
            if installed != 0 {
                let handle = Window::window_handle(window);
                cx.spawn(async move |_, cx| {
                    if rx.recv().await.is_ok() {
                        debug_log!("panel: native WA_INACTIVE; removing panel");
                        let _ = handle.update(cx, |_, window, _| close_panel(window));
                    }
                })
                .detach();
            } else {
                unsafe {
                    drop(Box::from_raw(data));
                }
                eprintln!("refiner: failed to install native panel focus handler");
            }
        }
        cx.observe_window_activation(window, move |_, window, _| {
            debug_log!(
                "panel: activation event active={}",
                window.is_window_active()
            );
            if !snapshot && !window.is_window_active() {
                close_panel(window);
            }
        })
    }
    #[cfg(test)]
    #[::core::prelude::v1::test]
    #[ignore = "requires an interactive Windows desktop and GPU"]
    fn selection_updates_leave_exactly_one_native_surface() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
        };
        unsafe extern "system" fn count(hwnd: HWND, data: LPARAM) -> i32 {
            unsafe {
                let mut pid = 0;
                GetWindowThreadProcessId(hwnd, &mut pid);
                if pid == std::process::id() && IsWindowVisible(hwnd) != 0 {
                    *(data as *mut usize) += 1;
                }
            }
            1
        }
        fn visible_windows() -> usize {
            let mut count_windows = 0usize;
            unsafe {
                EnumWindows(Some(count), &mut count_windows as *mut usize as LPARAM);
            }
            count_windows
        }
        let passed = Arc::new(AtomicBool::new(false));
        let result = passed.clone();
        gpui_kit::application()
            .with_assets(crate::assets::AppAssets)
            .run(move |cx| {
                cx.set_quit_mode(QuitMode::Explicit);
                gpui_kit::init(cx);
                crate::state::AppState::init(cx);
                crate::windows_selection_button::show_at_text((300, 200), "First".into(), cx);
                for i in 0..40 {
                    crate::windows_selection_button::show_at_text((300 + i, 200), format!("Selection {i}"), cx);
                }
                cx.spawn(async move |cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(400))
                        .await;
                    let during = visible_windows();
                    let circular_and_shadowless = cx.update(|cx| unsafe {
                        use windows_sys::Win32::{Graphics::Gdi::*, UI::WindowsAndMessaging::*};
                        let native = crate::windows_selection_button::handle(cx) as HWND;
                        let mut rect: windows_sys::Win32::Foundation::RECT = std::mem::zeroed();
                        GetClientRect(native, &mut rect);
                        let square = rect.right == rect.bottom && rect.right > 0;
                        let region = CreateRectRgn(0, 0, 0, 0);
                        GetWindowRgn(native, region);
                        let round = PtInRegion(region, 0, 0) == 0
                            && PtInRegion(region, rect.right / 2, rect.bottom / 2) != 0;
                        DeleteObject(region);
                        crate::windows_selection_button::save_preview(rect.right as u32, "target/selection-icon.png");
                        square && round
                            && GetWindowLongPtrW(native, GWL_EXSTYLE) as u32 & WS_EX_LAYERED != 0
                            && GetClassLongPtrW(native, GCL_STYLE) as u32 & CS_DROPSHADOW == 0
                    });
                    cx.update(hide_selection_icon);
                    let after = visible_windows();
                    result.store(during == 1 && after == 0 && circular_and_shadowless, Ordering::SeqCst);
                    eprintln!("selection icon: circular region and disabled native shadow={circular_and_shadowless}");
                    eprintln!(
                        "selection burst native windows: visible={during}, after_dismiss={after}"
                    );
                    cx.update(|cx| cx.quit());
                })
                .detach();
            });
        assert!(
            passed.load(Ordering::SeqCst),
            "selection updates must leave one surface and dismissal must leave none"
        );
    }

    #[cfg(test)]
    #[::core::prelude::v1::test]
    #[ignore = "requires an interactive Windows desktop and GPU"]
    fn compact_popup_hides_after_focus_moves_to_another_window_thread() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DispatchMessageW, GetForegroundWindow, GetMessageW, IsWindow,
            IsWindowVisible, MSG, PostMessageW, WM_CLOSE, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        };

        // A separate Win32 input queue exercises asynchronous activation, as
        // happens when clicking another application rather than GPUI Settings.
        let (ready, receiver) = std::sync::mpsc::sync_channel(1);
        let fixture = std::thread::spawn(move || unsafe {
            let native = CreateWindowExW(
                0,
                windows::core::w!("STATIC").as_ptr(),
                windows::core::w!("Refiner focus test").as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                10,
                10,
                260,
                120,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            assert!(!native.is_null());
            ready.send(native as usize).unwrap();
            let mut message: MSG = std::mem::zeroed();
            while IsWindow(native) != 0 && GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0
            {
                DispatchMessageW(&message);
            }
        });
        let other = receiver.recv_timeout(Duration::from_secs(5)).unwrap();

        let passed = Arc::new(AtomicBool::new(false));
        let result = passed.clone();
        gpui_kit::application().with_assets(crate::assets::AppAssets).run(move |cx| {
            cx.set_quit_mode(QuitMode::Explicit);
            gpui_kit::init(cx);
            crate::state::AppState::init(cx);
            crate::ui::popup_view::bind_keys(cx);
            unsafe { SetForegroundWindow(other as HWND); }
            show_selection_icon(point(px(300.), px(200.)), "Selected text".into(), cx);
            let icon_native = crate::windows_selection_button::handle(cx);
            for i in 0..40 {
                show_selection_icon(point(px(300. + i as f32), px(200.)), format!("Selection {i}"), cx);
                assert_eq!(crate::windows_selection_button::handle(cx), icon_native);
            }            cx.spawn(async move |cx| {
                cx.background_executor().timer(Duration::from_millis(200)).await;
                let icon_visible = unsafe { IsWindowVisible(icon_native as HWND) } != 0;
                let icon_preserves_focus = unsafe { GetForegroundWindow() } == other as HWND;
                let (popup, native) = cx.update(|cx| {
                    hide_selection_icon(cx);
                    show_popup(cx);
                    let popup = open_windows(cx).popup.expect("popup opened");
                    let native = popup.update(cx, |_, window, _| hwnd(window).unwrap() as usize).unwrap();
                    (popup, native)
                });
                cx.background_executor().timer(Duration::from_millis(700)).await;
                let was_visible = unsafe { IsWindowVisible(native as HWND) } != 0;
                let was_focused = unsafe { GetForegroundWindow() } == native as HWND;
                unsafe { SetForegroundWindow(other as HWND); }
                cx.background_executor().timer(Duration::from_millis(600)).await;
                let removed = popup.update(cx, |_, _, _| ()).is_err();
                let hidden = unsafe { IsWindowVisible(native as HWND) } == 0;
                let focus_moved = unsafe { GetForegroundWindow() } == other as HWND;
                let icon_hidden = unsafe { IsWindowVisible(icon_native as HWND) } == 0;
                result.store(icon_visible && icon_preserves_focus && icon_hidden && was_visible && was_focused && focus_moved && removed && hidden, Ordering::SeqCst);
                eprintln!("selection icon: visible={icon_visible}, preserves_source_focus={icon_preserves_focus}, hidden_on_dismiss={icon_hidden}");
                eprintln!("popup native lifecycle: initially_visible={was_visible}, initially_focused={was_focused}, focus_moved={focus_moved}, removed={removed}, hidden={hidden}");
                cx.update(|cx| cx.quit());
            }).detach();
        });
        unsafe {
            PostMessageW(other as HWND, WM_CLOSE, 0, 0);
        }
        fixture.join().unwrap();
        assert!(
            passed.load(Ordering::SeqCst),
            "compact popup must disappear while the application stays alive"
        );
    }
}
