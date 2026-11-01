//! Exact-size, non-activating native Windows selection button.
#![allow(unsafe_code)]

use gpui_kit::{App, Global};
#[cfg(any(feature = "snapshot", test))]
use gpui_kit::{DisplayId, Pixels, Point};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
        WindowsAndMessaging::*,
    },
};

const CLASS: windows::core::PCWSTR = windows::core::w!("RefinerSelectionButton");

struct State {
    text: String,
    clicks: smol::channel::Sender<String>,
}
struct Button {
    hwnd: HWND,
    _state: Box<State>,
}
impl Drop for Button {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
        }
    }
}
struct Buttons {
    button: Option<Button>,
    clicks: smol::channel::Sender<String>,
}
impl Global for Buttons {}

#[cfg(any(feature = "snapshot", test))]
pub fn show(origin: Point<Pixels>, display: Option<DisplayId>, text: String, cx: &mut App) {
    let mut dpi = 96;
    if let Some(display) = display {
        let mut y = 96;
        unsafe {
            GetDpiForMonitor(
                u64::from(display) as HMONITOR,
                MDT_EFFECTIVE_DPI,
                &mut dpi,
                &mut y,
            );
        }
    }
    let scale = dpi as f32 / 96.;
    show_pixels(
        (f32::from(origin.x) * scale).round() as i32,
        (f32::from(origin.y) * scale).round() as i32,
        (crate::windows::SELECTION_ICON_SIZE * scale).round() as i32,
        text,
        cx,
    );
}

/// Keep UIA's physical coordinates intact: mixed-DPI logical display bounds
/// can overlap and cannot reliably identify which native monitor owns text.
pub fn show_at_text(anchor: (i32, i32), text: String, cx: &mut App) {
    unsafe {
        let monitor = MonitorFromPoint(
            POINT {
                x: anchor.0,
                y: anchor.1,
            },
            MONITOR_DEFAULTTONEAREST,
        );
        let (mut dpi, mut y_dpi) = (96, 96);
        if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi, &mut y_dpi) < 0 {
            hide(cx);
            return;
        }
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            hide(cx);
            return;
        }
        let (x, y, side) = placement(
            anchor,
            (
                info.rcWork.left,
                info.rcWork.top,
                info.rcWork.right,
                info.rcWork.bottom,
            ),
            dpi,
        );
        show_pixels(x, y, side, text, cx);
    }
}

fn placement(anchor: (i32, i32), work: (i32, i32, i32, i32), dpi: u32) -> (i32, i32, i32) {
    let scale = dpi as f32 / 96.;
    let side = (crate::windows::SELECTION_ICON_SIZE * scale).round() as i32;
    let gap = (6. * scale).round() as i32;
    let margin = (4. * scale).round() as i32;
    let x = anchor
        .0
        .max(work.0 + margin)
        .min((work.2 - side - margin).max(work.0 + margin));
    let y = (anchor.1 - side - gap)
        .max(work.1 + margin)
        .min((work.3 - side - margin).max(work.1 + margin));
    (x, y, side)
}

fn show_pixels(x: i32, y: i32, side: i32, text: String, cx: &mut App) {
    if !cx.has_global::<Buttons>() {
        let (clicks, receiver) = smol::channel::unbounded::<String>();
        cx.set_global(Buttons {
            button: None,
            clicks,
        });
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
    let buttons = cx.global_mut::<Buttons>();
    unsafe {
        if buttons
            .button
            .as_ref()
            .is_some_and(|button| IsWindow(button.hwnd) == 0)
        {
            buttons.button.take();
        }
        if buttons.button.is_none() {
            let module = GetModuleHandleW(std::ptr::null());
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: module,
                hCursor: LoadCursorW(std::ptr::null_mut(), IDC_HAND),
                lpszClassName: CLASS.as_ptr(),
                ..std::mem::zeroed()
            };
            RegisterClassW(&class);
            let mut state = Box::new(State {
                text: text.clone(),
                clicks: buttons.clicks.clone(),
            });
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED,
                CLASS.as_ptr(),
                windows::core::w!("Refiner selection").as_ptr(),
                WS_POPUP,
                x,
                y,
                side,
                side,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                module,
                &mut *state as *mut State as _,
            );
            if hwnd.is_null() {
                eprintln!(
                    "refiner: cannot create selection button: {}",
                    std::io::Error::last_os_error()
                );
                return;
            }
            SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
            buttons.button = Some(Button {
                hwnd,
                _state: state,
            });
        }
        let button = buttons.button.as_mut().unwrap();
        // Use the same stable pointer as WndProc; do not create a second
        // mutable reference through the owning Box while native messages run.
        let state = GetWindowLongPtrW(button.hwnd, GWLP_USERDATA) as *mut State;
        (*state).text = text;
        SetWindowPos(
            button.hwnd,
            HWND_TOPMOST,
            x,
            y,
            side,
            side,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        let region = CreateEllipticRgn(0, 0, side, side);
        if !region.is_null() && SetWindowRgn(button.hwnd, region, 1) == 0 {
            DeleteObject(region);
        }
        InvalidateRect(button.hwnd, std::ptr::null(), 0);
        UpdateWindow(button.hwnd);
    }
}

pub(super) fn is_button(hwnd: HWND) -> bool {
    let mut class = [0u16; 64];
    let length = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
    length > 0 && class[..length as usize] == unsafe { CLASS.as_wide() }[..]
}

pub fn hide(cx: &mut App) {
    if cx.has_global::<Buttons>()
        && let Some(button) = &cx.global::<Buttons>().button
    {
        unsafe {
            ShowWindow(button.hwnd, SW_HIDE);
        }
    }
}

#[cfg(test)]
pub(super) fn handle(cx: &App) -> usize {
    cx.global::<Buttons>().button.as_ref().unwrap().hwnd as usize
}

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_NCCREATE => {
                let creation = &*(lparam as *const CREATESTRUCTW);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, creation.lpCreateParams as isize);
                return 1;
            }
            WM_MOUSEACTIVATE => return MA_NOACTIVATE as isize,
            WM_ERASEBKGND => return 1,
            WM_LBUTTONDOWN => {
                let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
                if let Some(state) = state.as_ref() {
                    ShowWindow(hwnd, SW_HIDE);
                    let _ = state.clicks.try_send(state.text.clone());
                }
                return 0;
            }
            WM_PAINT => {
                let mut paint: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut paint);
                let mut rect: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                draw(hdc, rect.right - rect.left);
                EndPaint(hwnd, &paint);
                return 0;
            }
            WM_NCDESTROY => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            _ => {}
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}

unsafe fn draw(hdc: HDC, side: i32) {
    unsafe {
        let key = CreateSolidBrush(0x00ffffff);
        let rect = RECT {
            left: 0,
            top: 0,
            right: side,
            bottom: side,
        };
        FillRect(hdc, &rect, key);
        DeleteObject(key);
        let scale = side as f32 / crate::windows::SELECTION_ICON_SIZE;
        let white = CreateSolidBrush(0x00ffffff);
        let pen = CreatePen(PS_SOLID, (scale.round() as i32).max(1), 0x00e1d5cb);
        let old_brush = SelectObject(hdc, white);
        let old_pen = SelectObject(hdc, pen);
        Ellipse(hdc, 1, 1, side - 1, side - 1);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        DeleteObject(white);
        DeleteObject(pen);
        // Use the same embedded app artwork as the tray. Composite its alpha
        // onto white before handing the bitmap to GDI.
        static APP_ICON: std::sync::OnceLock<image::RgbaImage> = std::sync::OnceLock::new();
        let source = APP_ICON.get_or_init(|| {
            image::load_from_memory(include_bytes!("../assets/icons/128x128@2x.png"))
                .expect("embedded app icon is valid PNG")
                .into_rgba8()
        });
        let art_side = ((side as f32 * 0.8).round() as u32).max(1);
        let art = image::imageops::resize(
            source,
            art_side,
            art_side,
            image::imageops::FilterType::Lanczos3,
        );
        let mut bgra = Vec::with_capacity((art_side * art_side * 4) as usize);
        for pixel in art.pixels() {
            let alpha = u32::from(pixel[3]);
            let blend = |channel: u8| {
                ((u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8
            };
            bgra.extend_from_slice(&[blend(pixel[2]), blend(pixel[1]), blend(pixel[0]), 0]);
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: art_side as i32,
                biHeight: -(art_side as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..std::mem::zeroed()
            },
            ..std::mem::zeroed()
        };
        let inset = (side - art_side as i32) / 2;
        SetDIBitsToDevice(
            hdc,
            inset,
            inset,
            art_side,
            art_side,
            0,
            0,
            0,
            art_side,
            bgra.as_ptr() as _,
            &info,
            DIB_RGB_COLORS,
        );
    }
}

#[cfg(feature = "snapshot")]
pub(super) fn snapshot(cx: &App, path: &std::path::Path) {
    if cx.has_global::<Buttons>()
        && let Some(button) = &cx.global::<Buttons>().button
    {
        let mut rect: RECT = unsafe { std::mem::zeroed() };
        if unsafe { GetClientRect(button.hwnd, &mut rect) } != 0 && rect.right > 0 {
            save_preview(rect.right as u32, path);
        }
    }
}

#[cfg(any(test, feature = "snapshot"))]
pub(super) fn save_preview(side: u32, path: impl AsRef<std::path::Path>) {
    unsafe {
        let dc = CreateCompatibleDC(std::ptr::null_mut());
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: side as i32,
                biHeight: -(side as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..std::mem::zeroed()
            },
            ..std::mem::zeroed()
        };
        let mut pixels = std::ptr::null_mut();
        let bitmap = CreateDIBSection(
            dc,
            &info,
            DIB_RGB_COLORS,
            &mut pixels,
            std::ptr::null_mut(),
            0,
        );
        assert!(!bitmap.is_null());
        let old = SelectObject(dc, bitmap);
        draw(dc, side as i32);
        GdiFlush();
        let bytes = std::slice::from_raw_parts(pixels as *const u8, (side * side * 4) as usize);
        let mut rgba = Vec::with_capacity(bytes.len());
        let region = CreateEllipticRgn(0, 0, side as i32, side as i32);
        for (index, p) in bytes.as_chunks::<4>().0.iter().enumerate() {
            let alpha = if PtInRegion(
                region,
                index as i32 % side as i32,
                index as i32 / side as i32,
            ) != 0
            {
                255
            } else {
                0
            };
            rgba.extend_from_slice(&[p[2], p[1], p[0], alpha]);
        }
        DeleteObject(region);
        SelectObject(dc, old);
        DeleteObject(bitmap);
        DeleteDC(dc);
        image::RgbaImage::from_raw(side, side, rgba)
            .unwrap()
            .save(path)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::placement;
    #[test]
    fn positions_in_physical_pixels_on_mixed_dpi_and_negative_monitors() {
        assert_eq!(
            placement((2200, 300), (1920, 0, 4480, 1400), 144),
            (2200, 258, 33)
        );
        assert_eq!(
            placement((-2000, -200), (-2560, -300, 0, 1100), 192),
            (-2000, -256, 44)
        );
        assert_eq!(
            placement((-2500, 10), (-2560, 0, 0, 1400), 192),
            (-2500, 8, 44)
        );
        assert_eq!(
            placement((4478, 1395), (1920, 0, 4480, 1400), 144),
            (4441, 1353, 33)
        );
    }
}
