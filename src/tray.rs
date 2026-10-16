//! Menu-bar / system-tray icon. Left click opens the main window; the menu
//! offers Settings, Analysis and Quit.

use gpui_kit::*;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::ui::settings_view::SettingsTab;
use crate::windows;

const ICON_PNG: &[u8] = include_bytes!("../assets/icons/128x128@2x.png");

/// The menu bar draws status items at most 22pt tall; Apple suggests ~16pt of
/// artwork. Pixels per point for the rendered bitmap (sharp on Retina).
const BAR_HEIGHT_PT: u32 = 22;
const ART_HEIGHT_PT: u32 = 16;
const SCALE: u32 = 3;

/// Keeps the tray icon alive for the lifetime of the app.
struct Tray {
    _icon: TrayIcon,
}

impl Global for Tray {}

enum TrayAction {
    ShowMain,
    Settings,
    Analysis,
    Quit,
}

/// The app icon at 16pt, centered on a transparent 22pt-tall canvas.
fn load_icon() -> anyhow::Result<Icon> {
    use image::{RgbaImage, imageops};

    let source = image::load_from_memory(ICON_PNG)?.into_rgba8();
    let art = ART_HEIGHT_PT * SCALE;
    let art = imageops::resize(&source, art, art, imageops::FilterType::Lanczos3);
    let side = BAR_HEIGHT_PT * SCALE;
    let mut canvas = RgbaImage::new(side, side);
    let offset = i64::from((side - art.height()) / 2);
    imageops::overlay(&mut canvas, &art, offset, offset);
    Ok(Icon::from_rgba(canvas.into_raw(), side, side)?)
}

/// Creates the tray icon. Must run on the main thread after launch.
pub fn init(cx: &mut App) {
    let menu = Menu::new();
    let items = [
        MenuItem::with_id("settings", "Settings", true, None),
        MenuItem::with_id("analysis", "Analysis", true, None),
    ];
    let quit = MenuItem::with_id("quit", "Quit Refiner", true, None);
    let built = menu
        .append_items(&[&items[0], &items[1], &PredefinedMenuItem::separator(), &quit])
        .map_err(anyhow::Error::from)
        .and_then(|_| load_icon())
        .and_then(|icon| {
            TrayIconBuilder::new()
                .with_id("refiner")
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .with_icon_templated(icon)
                .with_tooltip("Refiner")
                .build()
                .map_err(anyhow::Error::from)
        });
    let icon = match built {
        Ok(icon) => icon,
        Err(err) => {
            eprintln!("refiner: failed to create tray icon: {err:#}");
            return;
        }
    };
    cx.set_global(Tray { _icon: icon });

    let (tx, rx) = smol::channel::unbounded::<TrayAction>();
    let click_tx = tx.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let _ = click_tx.try_send(TrayAction::ShowMain);
        }
    }));
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let action = match event.id.as_ref() {
            "settings" => TrayAction::Settings,
            "analysis" => TrayAction::Analysis,
            "quit" => TrayAction::Quit,
            _ => return,
        };
        let _ = tx.try_send(action);
    }));

    cx.spawn(async move |cx| {
        while let Ok(action) = rx.recv().await {
            cx.update(|cx| match action {
                TrayAction::ShowMain => windows::show_main(cx),
                TrayAction::Settings => windows::show_settings(SettingsTab::General, cx),
                TrayAction::Analysis => windows::show_settings(SettingsTab::Analysis, cx),
                TrayAction::Quit => cx.quit(),
            });
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::{ART_HEIGHT_PT, ICON_PNG, SCALE, load_icon};

    #[test]
    fn icon_is_padded_to_menu_bar_height() {
        let source = image::load_from_memory(ICON_PNG).unwrap().into_rgba8();
        assert!(source.width() >= ART_HEIGHT_PT * SCALE);
        assert!(load_icon().is_ok());
    }
}
