//! Reusable, nonactivating 22pt selection button in native screen coordinates.
#![allow(unsafe_code)]

use std::{cell::RefCell, io::Cursor};

use gpui_kit::{App, DisplayId, Global, Pixels, Point};
use objc2::rc::Retained;
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::*;
use objc2_foundation::{NSData, NSNumber, NSPoint, NSRect, NSSize, NSString};

use crate::ax::SelectionRect;

const SIDE: f64 = crate::windows::SELECTION_ICON_SIZE as f64;

struct ButtonState {
    text: RefCell<String>,
    clicks: smol::channel::Sender<String>,
}

define_class!(
    #[unsafe(super(NSButton))]
    #[thread_kind = MainThreadOnly]
    #[name = "RefinerMacSelectionButton"]
    #[ivars = ButtonState]
    struct SelectionButton;

    impl SelectionButton {
        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _: Option<&NSEvent>) -> bool { true }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, _: &NSEvent) {
            self.submit();
        }

        #[unsafe(method(accessibilityPerformPress))]
        fn accessibility_press(&self) -> bool {
            self.submit();
            true
        }

        #[unsafe(method(resetCursorRects))]
        fn reset_cursor_rects(&self) {
            self.addCursorRect_cursor(self.bounds(), &NSCursor::pointingHandCursor());
        }
    }
);

impl SelectionButton {
    fn submit(&self) {
        if let Some(window) = self.window() {
            window.orderOut(None);
        }
        let _ = self
            .ivars()
            .clicks
            .try_send(self.ivars().text.borrow().clone());
    }
}

define_class!(
    #[unsafe(super(NSPanel))]
    #[thread_kind = MainThreadOnly]
    #[name = "RefinerMacSelectionPanel"]
    struct SelectionPanel;

    impl SelectionPanel {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> bool { false }
        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main(&self) -> bool { false }
    }
);

struct Button {
    panel: Retained<SelectionPanel>,
    view: Retained<SelectionButton>,
}
impl Global for Button {}

impl Drop for Button {
    fn drop(&mut self) {
        self.panel.close();
    }
}

/// Both inputs are AppKit screen points (bottom-left origin), including
/// negative origins on displays to the left or below the primary display.
fn placement(anchor: (f64, f64), work: NSRect) -> NSPoint {
    NSPoint::new(
        anchor
            .0
            .max(work.origin.x + 4.)
            .min((work.origin.x + work.size.width - SIDE - 4.).max(work.origin.x + 4.)),
        (anchor.1 + SIDE + 6.)
            .max(work.origin.y + SIDE + 4.)
            .min((work.origin.y + work.size.height - 4.).max(work.origin.y + SIDE + 4.)),
    )
}

/// Preserve the source artwork's colors, with antialiased transparent corners.
fn artwork() -> image::RgbaImage {
    let side = 66u32;
    let mut canvas = image::RgbaImage::new(side, side);
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        let distance = ((x as f64 + 0.5 - 33.).powi(2) + (y as f64 + 0.5 - 33.).powi(2)).sqrt();
        let alpha = ((33. - distance).clamp(0., 1.) * 255.).round() as u8;
        let color = if distance > 30. {
            [203, 213, 225]
        } else {
            [255, 255, 255]
        };
        *pixel = image::Rgba([color[0], color[1], color[2], alpha]);
    }
    let source = image::load_from_memory(include_bytes!("../assets/icons/128x128@2x.png"))
        .expect("valid embedded icon");
    let art = image::imageops::resize(
        &source.to_rgba8(),
        52,
        52,
        image::imageops::FilterType::Lanczos3,
    );
    image::imageops::overlay(&mut canvas, &art, 7, 7);
    canvas
}

fn create(cx: &mut App, mtm: MainThreadMarker) {
    if cx.has_global::<Button>() {
        return;
    }
    let (clicks, receiver) = smol::channel::unbounded();
    let panel: Retained<SelectionPanel> = unsafe {
        msg_send![SelectionPanel::alloc(mtm), initWithContentRect: NSRect::new(NSPoint::new(0., 0.), NSSize::new(SIDE, SIDE)), styleMask: NSWindowStyleMask::NonactivatingPanel, backing: NSBackingStoreType::Buffered, defer: false]
    };
    unsafe {
        panel.setReleasedWhenClosed(false);
    }
    panel.setOpaque(false);
    panel.setBackgroundColor(Some(&NSColor::clearColor()));
    panel.setHasShadow(false);
    panel.setHidesOnDeactivate(false);
    panel.setMovable(false);
    panel.setLevel(NSPopUpMenuWindowLevel);
    panel.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    let allocated = SelectionButton::alloc(mtm).set_ivars(ButtonState {
        text: RefCell::new(String::new()),
        clicks,
    });
    let view: Retained<SelectionButton> = unsafe {
        msg_send![super(allocated), initWithFrame: NSRect::new(NSPoint::new(0., 0.), NSSize::new(SIDE, SIDE))]
    };
    view.setTitle(&NSString::from_str("Translate and correct selection"));
    view.setBordered(false);
    view.setImagePosition(NSCellImagePosition::ImageOnly);
    view.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
    view.setFocusRingType(NSFocusRingType::None);
    view.setToolTip(Some(&NSString::from_str("Translate and correct selection")));
    let mut bytes = Cursor::new(Vec::new());
    artwork()
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("encode icon");
    let data = NSData::with_bytes(bytes.get_ref());
    let icon = NSImage::initWithData(NSImage::alloc(), &data).expect("decode icon");
    icon.setSize(NSSize::new(SIDE, SIDE));
    icon.setTemplate(false);
    view.setImage(Some(&icon));
    panel.setContentView(Some(&view));
    cx.set_global(Button { panel, view });
    cx.spawn(async move |cx| {
        while let Ok(text) = receiver.recv().await {
            cx.update(|cx| {
                hide(cx);
                let popup = crate::state::AppState::get(cx).popup.clone();
                popup.update(cx, |popup, cx| popup.run(text, cx));
                crate::windows::show_popup(cx);
            });
        }
    })
    .detach();
}

/// AX geometry is in global top-left screen points. Only the native window
/// boundary flips Y; no backing-scale multiplication is appropriate here.
pub fn show_at_text(rect: SelectionRect, text: String, cx: &mut App) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let screens = NSScreen::screens(mtm);
    let Some(primary) = screens.firstObject() else {
        return;
    };
    let anchor = (rect.x, primary.frame().size.height - rect.y);
    let Some(screen) = screens.iter().find(|screen| {
        let f = screen.frame();
        anchor.0 >= f.origin.x
            && anchor.0 < f.origin.x + f.size.width
            && anchor.1 > f.origin.y
            && anchor.1 <= f.origin.y + f.size.height
    }) else {
        hide(cx);
        return;
    };
    create(cx, mtm);
    let button = cx.global::<Button>();
    *button.view.ivars().text.borrow_mut() = text;
    button
        .panel
        .setFrameTopLeftPoint(placement(anchor, screen.visibleFrame()));
    button.panel.orderFrontRegardless();
    debug_log!("selection: native icon at {:?}", button.panel.frame());
}

pub fn is_button(window: &NSWindow) -> bool {
    window.downcast_ref::<SelectionPanel>().is_some()
}

pub fn hide(cx: &mut App) {
    if cx.has_global::<Button>() {
        cx.global::<Button>().panel.orderOut(None);
    }
}

/// GPUI's macOS backend expects display-local coordinates at window creation.
/// Select screens using native global frames, then convert once at this edge.
pub fn popup_location() -> Option<(Point<Pixels>, gpui_kit::Bounds<Pixels>, DisplayId)> {
    let mtm = MainThreadMarker::new()?;
    let cursor = NSEvent::mouseLocation();
    let screens = NSScreen::screens(mtm);
    let screen = screens.iter().find(|screen| {
        let f = screen.frame();
        cursor.x >= f.origin.x
            && cursor.x < f.origin.x + f.size.width
            && cursor.y >= f.origin.y
            && cursor.y < f.origin.y + f.size.height
    })?;
    let frame = screen.frame();
    let visible = screen.visibleFrame();
    let number = screen
        .deviceDescription()
        .objectForKey(&NSString::from_str("NSScreenNumber"))?;
    let number = number.downcast_ref::<NSNumber>()?;
    Some((
        gpui_kit::point(
            gpui_kit::px((cursor.x - frame.origin.x) as f32),
            gpui_kit::px((frame.origin.y + frame.size.height - cursor.y) as f32),
        ),
        gpui_kit::Bounds::new(
            gpui_kit::point(
                gpui_kit::px((visible.origin.x - frame.origin.x) as f32),
                gpui_kit::px(
                    (frame.origin.y + frame.size.height - visible.origin.y - visible.size.height)
                        as f32,
                ),
            ),
            gpui_kit::size(
                gpui_kit::px(visible.size.width as f32),
                gpui_kit::px(visible.size.height as f32),
            ),
        ),
        DisplayId::from(number.unsignedLongLongValue()),
    ))
}

#[cfg(feature = "snapshot")]
pub fn snapshot(cx: &mut App, path: &std::path::Path) {
    if !cx.has_global::<Button>() {
        return;
    }
    let button = cx.global::<Button>();
    assert_eq!(button.panel.frame().size, NSSize::new(SIDE, SIDE));
    assert!(!button.panel.hasShadow());
    assert!(!button.panel.isOpaque());
    assert!(!button.panel.canBecomeKeyWindow());
    assert!(!button.panel.isKeyWindow());
    let frame = button.view.bounds();
    let bitmap = button
        .view
        .bitmapImageRepForCachingDisplayInRect(frame)
        .expect("allocate native snapshot");
    button
        .view
        .cacheDisplayInRect_toBitmapImageRep(frame, &bitmap);
    let data = unsafe {
        bitmap.representationUsingType_properties(
            NSBitmapImageFileType::PNG,
            &objc2_foundation::NSDictionary::new(),
        )
    }
    .expect("encode native snapshot");
    std::fs::write(path, data.to_vec()).expect("save native snapshot");
    let number = button.panel.windowNumber();
    let frame = button.panel.frame();
    let text = button.view.ivars().text.borrow().clone();
    let mtm = MainThreadMarker::new().unwrap();
    let primary_height = NSScreen::screens(mtm)
        .firstObject()
        .unwrap()
        .frame()
        .size
        .height;
    let rect = SelectionRect {
        x: frame.origin.x,
        y: primary_height - frame.origin.y + 6.,
        width: 40.,
        height: 16.,
    };
    let foreground = crate::ax::frontmost_pid();
    for _ in 0..40 {
        show_at_text(rect, text.clone(), cx);
    }
    assert_eq!(cx.global::<Button>().panel.windowNumber(), number);
    assert_eq!(crate::ax::frontmost_pid(), foreground);
    hide(cx);
    assert!(!cx.global::<Button>().panel.isVisible());
    eprintln!(
        "snapshot: native selection panel: 22pt, transparent, shadowless, non-key; reused across 40 updates, hidden on dismiss; wrote {}",
        path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placement_respects_negative_screens_and_menu_bar() {
        let work = NSRect::new(NSPoint::new(-1920., -200.), NSSize::new(1920., 1080.));
        assert_eq!(placement((-1800., 400.), work), NSPoint::new(-1800., 428.));
        assert_eq!(placement((-10., 879.), work), NSPoint::new(-26., 876.));
        assert_eq!(
            placement((-1919., -199.), work),
            NSPoint::new(-1916., -171.)
        );
    }
    #[test]
    fn artwork_is_circular_and_preserves_app_artwork() {
        let art = artwork();
        assert_eq!(art.dimensions(), (66, 66));
        assert_eq!(art.get_pixel(0, 0)[3], 0);
        assert!(
            art.pixels()
                .any(|p| p[3] > 200 && p[0] < 40 && p[1] < 40 && p[2] < 40)
        );
        assert_eq!(*art.get_pixel(33, 3), image::Rgba([255, 255, 255, 255]));
    }
}
