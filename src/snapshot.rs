//! Dev-only window snapshots (`--features snapshot`, `REFINER_SNAPSHOT=<dir>`).
//! Renders every open window to `<dir>/<step>-<n>.png` at each step, then quits.

use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::*;

use crate::state::AppState;
use crate::ui::settings_view::SettingsTab;
use crate::windows;

fn capture(dir: &Path, step: &str, cx: &mut App) {
    eprintln!("snapshot: {step}: {} window(s)", cx.windows().len());
    for (n, handle) in cx.windows().into_iter().enumerate() {
        let result = handle.update(cx, |_, window, _| window.render_to_image());
        match result {
            Ok(Ok(image)) => {
                let path = dir.join(format!("{step}-{n}.png"));
                if let Err(err) = image.save(&path) {
                    eprintln!("snapshot: {}: {err}", path.display());
                } else {
                    eprintln!("snapshot: wrote {}", path.display());
                }
            }
            Ok(Err(err)) => eprintln!("snapshot: render failed: {err:#}"),
            Err(_) => {}
        }
    }
}

pub fn run(cx: &mut App) {
    let Some(dir) = std::env::var_os("REFINER_SNAPSHOT").map(PathBuf::from) else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    let state = AppState::get(cx).clone();
    state.main.update(cx, |m, cx| {
        m.demo(
            "Hello! Could you send me the report by tomorrow morning?",
            "Xin chào! Bạn có thể gửi cho tôi bản báo cáo trước sáng mai được không?",
            cx,
        )
    });
    cx.spawn(async move |cx| {
        let wait = |ms| cx.background_executor().timer(Duration::from_millis(ms));
        wait(1500).await;
        cx.update(|cx| capture(&dir, "1-main", cx));
        cx.update(|cx| {
            state.popup.update(cx, |p, cx| {
                p.demo(
                    "Tôi đã gửi email cho anh ấy hôm qua.",
                    "I sent him an email yesterday.",
                    cx,
                )
            });
            windows::show_popup(cx);
        });
        wait(1200).await;
        cx.update(|cx| capture(&dir, "2-popup", cx));
        cx.update(|cx| windows::show_settings(SettingsTab::General, cx));
        wait(1200).await;
        cx.update(|cx| capture(&dir, "3-settings", cx));
        cx.update(|cx| windows::show_settings(SettingsTab::Analysis, cx));
        wait(1000).await;
        cx.update(|cx| capture(&dir, "4-analysis", cx));
        cx.update(|cx| {
            windows::show_selection_icon(point(px(400.), px(300.)), "hello".into(), cx)
        });
        wait(800).await;
        cx.update(|cx| capture(&dir, "5-icon", cx));
        cx.update(|cx| cx.quit());
    })
    .detach();
}
