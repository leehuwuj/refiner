//! Clipboard-free selection capture through Windows UI Automation.
//!
//! COM objects stay on a dedicated MTA thread. A bounded queue and response
//! timeout keep a slow or broken accessibility provider off GPUI's executor.

#![allow(unsafe_code)]

use std::sync::{OnceLock, mpsc};
use std::time::Duration;

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation2, IUIAutomationTextPattern, UIA_TextPatternId,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

const TIMEOUT: Duration = Duration::from_secs(2);
const MAX_TEXT: usize = 1_000_000;

// UIA's legacy control proxies share desktop state across fixture threads.
#[cfg(test)]
pub(super) static NATIVE_UIA_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Request {
    foreground: usize,
    reply: mpsc::SyncSender<Option<String>>,
}

pub(super) struct ComApartment;

impl Drop for ComApartment {
    fn drop(&mut self) {
        // SAFETY: created only after successful CoInitializeEx on this thread;
        // UIA interfaces are dropped before the apartment guard.
        unsafe { CoUninitialize() };
    }
}

pub fn selected_text() -> Option<String> {
    // SAFETY: valid out-pointer; no ownership of the returned HWND is taken.
    let foreground = unsafe { GetForegroundWindow() };
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(foreground, &mut process_id) };
    if foreground.is_null() || process_id == 0 || process_id == std::process::id() {
        return None;
    }

    static WORKER: OnceLock<mpsc::SyncSender<Request>> = OnceLock::new();
    let worker = WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<Request>(1);
        let _ = std::thread::Builder::new()
            .name("refiner-uia".into())
            .spawn(move || run(rx));
        tx
    });
    let (reply, result) = mpsc::sync_channel(1);
    worker
        .try_send(Request {
            foreground: foreground as usize,
            reply,
        })
        .ok()?;
    result.recv_timeout(TIMEOUT).ok().flatten()
}

fn run(requests: mpsc::Receiver<Request>) {
    // SAFETY: this fresh thread owns its MTA and all UIA interfaces. Nothing
    // except owned Strings and numeric window identities crosses threads.
    unsafe {
        if CoInitializeEx(None, COINIT_MULTITHREADED).is_err() {
            return;
        }
        let _apartment = ComApartment;
        let automation: IUIAutomation2 =
            match CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) {
                Ok(automation) => automation,
                Err(err) => {
                    eprintln!("refiner: UI Automation unavailable: {err}");
                    return;
                }
            };
        let _ = automation.SetConnectionTimeout(500);
        let _ = automation.SetTransactionTimeout(500);
        while let Ok(request) = requests.recv() {
            let text = read_selection(&automation, request.foreground, false)
                .map(|selection| selection.text);
            let _ = request.reply.try_send(text);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Selection {
    pub text: String,
    /// Physical screen pixels; never divide these by a different monitor's DPI.
    pub at: Option<(i32, i32)>,
    pub foreground: usize,
}

pub(super) fn current_selection(automation: &IUIAutomation2) -> Option<Selection> {
    let foreground = unsafe { GetForegroundWindow() };
    let mut process_id = 0;
    unsafe {
        GetWindowThreadProcessId(foreground, &mut process_id);
    }
    if foreground.is_null() || process_id == 0 || process_id == std::process::id() {
        return None;
    }
    read_selection(automation, foreground as usize, true)
}

fn read_selection(
    automation: &IUIAutomation2,
    foreground: usize,
    with_bounds: bool,
) -> Option<Selection> {
    // SAFETY: all interfaces belong to this initialized COM thread. Windows
    // owns HWNDs; they are only compared, never dereferenced.
    unsafe {
        if GetForegroundWindow() as usize != foreground {
            return None;
        }
        let focused = automation.GetFocusedElement().ok()?;
        let mut element = focused.clone();
        let walker = automation.ControlViewWalker().ok()?;
        let mut pattern = None;
        let mut belongs_to_foreground = false;
        for _ in 0..32 {
            if element.CurrentIsPassword().ok()?.as_bool() {
                return None;
            }
            if pattern.is_none() {
                pattern = element
                    .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
                    .ok();
            }
            if element
                .CurrentNativeWindowHandle()
                .ok()
                .is_some_and(|hwnd| hwnd.0 as usize == foreground)
            {
                belongs_to_foreground = true;
                break;
            }
            // A focused link can expose text on its containing document.
            // Validate ancestry rather than PID: browsers have multiple processes.
            element = walker.GetParentElement(&element).ok()?;
        }
        if !belongs_to_foreground {
            return None;
        }
        let pattern = pattern?;
        let text = selection_from_pattern(&pattern).ok().flatten()?;
        let at = if with_bounds {
            selection_anchor(&pattern)
        } else {
            None
        };
        let still_focused = automation
            .CompareElements(&focused, &automation.GetFocusedElement().ok()?)
            .ok()?
            .as_bool();
        (still_focused && GetForegroundWindow() as usize == foreground).then_some(Selection {
            text,
            at,
            foreground,
        })
    }
}

fn selection_anchor(pattern: &IUIAutomationTextPattern) -> Option<(i32, i32)> {
    unsafe {
        let ranges = pattern.GetSelection().ok()?;
        let count = ranges.Length().ok()?;
        if !(1..=32).contains(&count) {
            return None;
        }
        (0..count)
            .filter_map(|index| rectangle_anchor(&ranges.GetElement(index).ok()?))
            .min_by_key(|&(x, y)| (y, x))
    }
}

fn rectangle_anchor(
    range: &windows::Win32::UI::Accessibility::IUIAutomationTextRange,
) -> Option<(i32, i32)> {
    use windows::Win32::System::Com::SAFEARRAY;
    use windows::Win32::System::Ole::{
        SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetDim, SafeArrayGetElemsize,
        SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
    };
    struct Rectangles(*mut SAFEARRAY);
    impl Drop for Rectangles {
        fn drop(&mut self) {
            unsafe {
                let _ = SafeArrayDestroy(self.0);
            }
        }
    }
    unsafe {
        let array = Rectangles(range.GetBoundingRectangles().ok()?);
        if array.0.is_null() || SafeArrayGetDim(array.0) != 1 || SafeArrayGetElemsize(array.0) != 8
        {
            return None;
        }
        let low = SafeArrayGetLBound(array.0, 1).ok()?;
        let high = SafeArrayGetUBound(array.0, 1).ok()?;
        let count = high.checked_sub(low)?.checked_add(1)? as usize;
        if count == 0 || count > 4096 || !count.is_multiple_of(4) {
            return None;
        }
        let mut data = std::ptr::null_mut();
        SafeArrayAccessData(array.0, &mut data).ok()?;
        if data.is_null() {
            let _ = SafeArrayUnaccessData(array.0);
            return None;
        }
        let at = top_left_rectangle(std::slice::from_raw_parts(data as *const f64, count));
        let _ = SafeArrayUnaccessData(array.0);
        at
    }
}

fn selection_from_pattern(
    pattern: &IUIAutomationTextPattern,
) -> windows::core::Result<Option<String>> {
    // SAFETY: UIA returns owned COM range interfaces and BSTRs, released by
    // windows-rs. Empty/caret ranges deliberately produce no selection.
    unsafe {
        let ranges = pattern.GetSelection()?;
        let count = ranges.Length()?;
        if !(1..=32).contains(&count) {
            return Ok(None);
        }
        let mut text = String::new();
        let mut remaining = MAX_TEXT;
        for index in 0..count {
            let range = ranges.GetElement(index)?;
            let part = range.GetText((remaining + 1) as i32)?;
            if part.len() > remaining {
                return Ok(None); // Never submit a silently truncated selection.
            }
            remaining -= part.len();
            text.push_str(&part.to_string());
        }
        let text = text.trim().to_owned();
        Ok((!text.is_empty()).then_some(text))
    }
}

fn top_left_rectangle(rectangles: &[f64]) -> Option<(i32, i32)> {
    rectangles
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|r| r.iter().all(|n| n.is_finite()) && r[2] > 0. && r[3] > 0.)
        .min_by(|a, b| a[1].total_cmp(&b[1]).then_with(|| a[0].total_cmp(&b[0])))
        .map(|r| (r[0].round() as i32, r[1].round() as i32))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_visual_top_left_without_accepting_empty_or_invalid_rectangles() {
        assert_eq!(
            top_left_rectangle(&[
                10.,
                200.,
                50.,
                20.,
                300.,
                100.,
                50.,
                20.,
                0.,
                0.,
                0.,
                20.,
                100.,
                100.,
                50.,
                20.,
                f64::NAN,
                0.,
                10.,
                20.,
            ]),
            Some((100, 100))
        );
        assert_eq!(
            top_left_rectangle(&[-2000., -300., 40., 20.]),
            Some((-2000, -300))
        );
        assert_eq!(top_left_rectangle(&[0., 0., 0., 0.]), None);
    }
    use windows::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, MSG, PM_REMOVE, PeekMessageW,
        SendMessageW, WS_POPUP,
    };

    #[test]
    fn native_edit_selection_and_empty_caret() {
        let _serial = NATIVE_UIA_TEST.lock().unwrap();
        // A real Win32 EDIT provider, with its window thread pumping messages
        // while UIA reads from another MTA thread. No clipboard or key input.
        unsafe {
            let class: Vec<u16> = "EDIT\0".encode_utf16().collect();
            let content: Vec<u16> = "Hello 世界\0".encode_utf16().collect();
            let hwnd = CreateWindowExW(
                0,
                class.as_ptr(),
                content.as_ptr(),
                WS_POPUP,
                0,
                0,
                200,
                50,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            assert!(!hwnd.is_null());
            struct Fixture(windows_sys::Win32::Foundation::HWND);
            impl Drop for Fixture {
                fn drop(&mut self) {
                    unsafe {
                        DestroyWindow(self.0);
                    }
                }
            }
            let _fixture = Fixture(hwnd);
            // EM_SETSEL selects UTF-16 offsets 6..8, including both CJK chars.
            const EM_SETSEL: u32 = 0x00B1;
            SendMessageW(hwnd, EM_SETSEL, 6, 8);
            let (tx, rx) = mpsc::sync_channel(1);
            let handle = hwnd as usize;
            std::thread::spawn(move || {
                let result = (|| -> windows::core::Result<_> {
                    CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
                    let _apartment = ComApartment;
                    let automation: IUIAutomation2 =
                        CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)?;
                    let element = automation.ElementFromHandle(HWND(handle as _))?;
                    let pattern = element
                        .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)?;
                    let selected = selection_from_pattern(&pattern)?;
                    SendMessageW(handle as _, EM_SETSEL, 3, 3);
                    let caret = selection_from_pattern(&pattern)?;
                    Ok((selected, caret))
                })();
                let _ = tx.send(result);
            });
            let started = std::time::Instant::now();
            let result = loop {
                if let Ok(result) = rx.try_recv() {
                    break result;
                }
                assert!(
                    started.elapsed() < Duration::from_secs(10),
                    "UIA fixture timed out"
                );
                let mut message: MSG = std::mem::zeroed();
                while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    DispatchMessageW(&message);
                }
                std::thread::sleep(Duration::from_millis(5));
            };
            let (selected, caret) = result.expect("Win32 EDIT must expose TextPattern");
            assert_eq!(selected.as_deref(), Some("世界"));
            assert_eq!(caret, None);
        }
    }
}
