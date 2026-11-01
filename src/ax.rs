//! macOS Accessibility (AX) access to the selection in the frontmost app.
//!
//! Reading `AXSelectedText` from the focused element gives the selected text
//! without touching the clipboard, and tells "nothing selected" apart from
//! "this app does not expose its text" (Chromium/Electron without the
//! `AXManualAccessibility` hint, most terminals, some Java/PDF apps). Callers
//! fall back to simulating ⌘C only when this returns `None`.
//!
//! Every call is blocking IPC to another process; run it off the main thread.
#![allow(unsafe_code)]

use std::ptr;

use accessibility_sys::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCreateApplication, AXUIElementCreateSystemWide, AXUIElementRef,
    AXUIElementSetAttributeValue, AXUIElementSetMessagingTimeout, kAXErrorSuccess,
    kAXFocusedUIElementAttribute, kAXSelectedTextAttribute, kAXTrustedCheckOptionPrompt,
};
use core_foundation::base::{CFRelease, CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

/// Seconds an unresponsive app may stall a single AX request.
const MESSAGING_TIMEOUT: f32 = 0.25;

/// Owns a +1 CoreFoundation reference and releases it on drop.
struct Owned(CFTypeRef);

impl Owned {
    fn element(&self) -> AXUIElementRef {
        self.0 as AXUIElementRef
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: `self.0` came from a Create/Copy function (+1 retain
            // count) and is released exactly once, here.
            unsafe { CFRelease(self.0) };
        }
    }
}

/// Whether Refiner may use the Accessibility API. With `prompt`, macOS shows
/// its "grant access" dialog when it may not.
pub fn is_trusted(prompt: bool) -> bool {
    if !prompt {
        // SAFETY: no preconditions.
        return unsafe { AXIsProcessTrusted() };
    }
    // SAFETY: `kAXTrustedCheckOptionPrompt` is a valid static CFString
    // provided by ApplicationServices; it is not released by us.
    let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let options =
        CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
    // SAFETY: `options` is a valid CFDictionary for the duration of the call.
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

/// Copies attribute `name` of `element`, or `None` on any AX error.
fn copy_attribute(element: AXUIElementRef, name: &'static str) -> Option<Owned> {
    let cf_name = CFString::from_static_string(name);
    let mut value: CFTypeRef = ptr::null();
    // SAFETY: `element` is a live AXUIElement, `name` a valid CFString, and
    // `value` a valid out-pointer. On success we own the returned +1 value.
    let err = unsafe {
        AXUIElementCopyAttributeValue(element, cf_name.as_concrete_TypeRef(), &mut value)
    };
    if err != kAXErrorSuccess || value.is_null() {
        debug_log!("ax: {name} failed (AXError {err})");
        return None;
    }
    Some(Owned(value))
}

fn system_wide() -> Owned {
    // SAFETY: returns a new +1 AXUIElement; `Owned` releases it.
    let system = Owned(unsafe { AXUIElementCreateSystemWide() } as CFTypeRef);
    // SAFETY: `system` is a live AXUIElement.
    unsafe { AXUIElementSetMessagingTimeout(system.element(), MESSAGING_TIMEOUT) };
    system
}

/// Asks Chromium/Electron apps to build their accessibility tree, which they
/// otherwise only do for screen readers. Harmless for other apps.
fn enable_manual_accessibility(pid: i32) {
    // SAFETY: returns a new +1 AXUIElement; `Owned` releases it.
    let app = Owned(unsafe { AXUIElementCreateApplication(pid) } as CFTypeRef);
    let name = CFString::from_static_string("AXManualAccessibility");
    let yes = CFBoolean::true_value();
    // SAFETY: all arguments are live CF objects for the duration of the call.
    unsafe {
        AXUIElementSetMessagingTimeout(app.element(), MESSAGING_TIMEOUT);
        AXUIElementSetAttributeValue(
            app.element(),
            name.as_concrete_TypeRef(),
            yes.as_concrete_TypeRef() as CFTypeRef,
        );
    }
}

fn read_selected_text() -> Option<String> {
    let system = system_wide();
    let focused = copy_attribute(system.element(), kAXFocusedUIElementAttribute)?;
    let value = copy_attribute(focused.element(), kAXSelectedTextAttribute)?;
    // SAFETY: `value.0` is a live CF object we own; `wrap_under_get_rule`
    // retains it again, so dropping both is balanced.
    let value = unsafe { CFType::wrap_under_get_rule(value.0) };
    let Some(text) = value.downcast::<CFString>() else {
        debug_log!("ax: AXSelectedText is not a string");
        return None;
    };
    let text = text.to_string();
    let text = text.trim();
    debug_log!("ax: selected text has {} chars", text.chars().count());
    (!text.is_empty()).then(|| text.to_string())
}

/// The text selected in the frontmost app, read through Accessibility.
/// `None` when nothing is selected, the app doesn't expose it, or Refiner
/// isn't trusted.
pub fn selected_text() -> Option<String> {
    if !is_trusted(false) {
        debug_log!("ax: not trusted — grant Accessibility to the app that launched Refiner");
        return None;
    }
    if let Some(text) = read_selected_text() {
        return Some(text);
    }
    // Chromium/Electron expose text only after being asked; retry once.
    let pid = frontmost_pid()?;
    if pid == std::process::id() as i32 {
        return None;
    }
    debug_log!("ax: retrying with AXManualAccessibility on pid {pid}");
    enable_manual_accessibility(pid);
    read_selected_text()
}

/// PID of the frontmost application.
pub fn frontmost_pid() -> Option<i32> {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    workspace
        .frontmostApplication()
        .map(|app| app.processIdentifier())
}

/// Global top-left screen points, as returned by AXBoundsForRange.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl SelectionRect {
    fn valid(self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|v| v.is_finite())
            && self.width > 0.
            && self.height > 0.
    }

    fn intersection(self, other: Self) -> Option<Self> {
        if !self.valid() || !other.valid() {
            return None;
        }
        let (x, y) = (self.x.max(other.x), self.y.max(other.y));
        let result = Self {
            x,
            y,
            width: (self.x + self.width).min(other.x + other.width) - x,
            height: (self.y + self.height).min(other.y + other.height) - y,
        };
        result.valid().then_some(result)
    }
}

#[derive(Debug)]
pub struct Selection {
    pub text: String,
    pub rect: SelectionRect,
    pub pid: i32,
}

fn copy_parameter(
    element: AXUIElementRef,
    name: &'static str,
    parameter: CFTypeRef,
) -> Option<Owned> {
    let name = CFString::from_static_string(name);
    let mut value = ptr::null();
    let error = unsafe {
        accessibility_sys::AXUIElementCopyParameterizedAttributeValue(
            element,
            name.as_concrete_TypeRef(),
            parameter,
            &mut value,
        )
    };
    (error == kAXErrorSuccess && !value.is_null()).then_some(Owned(value))
}

fn range_value(value: &Owned) -> Option<core_foundation::base::CFRange> {
    use accessibility_sys::*;
    if unsafe { core_foundation::base::CFGetTypeID(value.0) } != unsafe { AXValueGetTypeID() } {
        return None;
    }
    let mut range = core_foundation::base::CFRange::init(0, 0);
    let ok = unsafe {
        AXValueGetValue(
            value.0 as AXValueRef,
            kAXValueTypeCFRange,
            (&mut range as *mut core_foundation::base::CFRange).cast(),
        )
    };
    (ok && range.location >= 0
        && range.length >= 0
        && range.location.checked_add(range.length).is_some())
    .then_some(range)
}

fn range_object(range: core_foundation::base::CFRange) -> Option<Owned> {
    let value = unsafe {
        accessibility_sys::AXValueCreate(
            accessibility_sys::kAXValueTypeCFRange,
            (&range as *const core_foundation::base::CFRange).cast(),
        )
    };
    (!value.is_null()).then_some(Owned(value as CFTypeRef))
}

fn rect_value(value: &Owned) -> Option<SelectionRect> {
    use accessibility_sys::*;
    if unsafe { core_foundation::base::CFGetTypeID(value.0) } != unsafe { AXValueGetTypeID() } {
        return None;
    }
    let mut rect = objc2_foundation::NSRect::default();
    let ok = unsafe {
        AXValueGetValue(
            value.0 as AXValueRef,
            kAXValueTypeCGRect,
            (&mut rect as *mut objc2_foundation::NSRect).cast(),
        )
    };
    let rect = SelectionRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    };
    (ok && rect.valid()).then_some(rect)
}

fn intersect_ranges(
    a: core_foundation::base::CFRange,
    b: core_foundation::base::CFRange,
) -> Option<core_foundation::base::CFRange> {
    let start = a.location.max(b.location);
    let end = a
        .location
        .checked_add(a.length)?
        .min(b.location.checked_add(b.length)?);
    (start >= 0 && end > start).then(|| core_foundation::base::CFRange::init(start, end - start))
}

fn text_bounds(element: AXUIElementRef, selected: &Owned) -> Option<SelectionRect> {
    use accessibility_sys::*;
    use core_foundation::number::CFNumber;
    let selected = range_value(selected)?;
    if selected.length == 0 {
        return None;
    }
    // Clip the character range before asking for a line: the selection may
    // begin above the viewport. Never scroll the source app to find it.
    let visible =
        copy_attribute(element, kAXVisibleCharacterRangeAttribute).and_then(|v| range_value(&v));
    let range = match visible {
        Some(visible) => intersect_ranges(selected, visible)?,
        None => selected,
    };
    let index = CFNumber::from(range.location as i64);
    let line = copy_parameter(
        element,
        kAXLineForIndexParameterizedAttribute,
        index.as_CFTypeRef(),
    );
    let first_line = line
        .and_then(|line| copy_parameter(element, kAXRangeForLineParameterizedAttribute, line.0))
        .and_then(|line| range_value(&line))
        .and_then(|line| intersect_ranges(range, line));
    if let Some(line) = first_line {
        let line = range_object(line)?;
        let bounds = copy_parameter(element, kAXBoundsForRangeParameterizedAttribute, line.0)?;
        return rect_value(&bounds);
    }
    // Some single-line controls expose bounds but not line queries. For a
    // multiline provider without line queries, use the first character's
    // bounds rather than the union of unrelated lines.
    let first = copy_parameter(
        element,
        kAXRangeForIndexParameterizedAttribute,
        index.as_CFTypeRef(),
    )
    .and_then(|value| range_value(&value))
    .and_then(|first| intersect_ranges(range, first))
    .unwrap_or(core_foundation::base::CFRange::init(range.location, 1));
    let first = range_object(first)?;
    let first = rect_value(&copy_parameter(
        element,
        kAXBoundsForRangeParameterizedAttribute,
        first.0,
    )?)?;
    let full = range_object(range)?;
    let full = copy_parameter(element, kAXBoundsForRangeParameterizedAttribute, full.0)
        .and_then(|value| rect_value(&value));
    Some(
        full.filter(|full| {
            (full.y - first.y).abs() < 1. && (full.height - first.height).abs() < 1.
        })
        .unwrap_or(first),
    )
}

// WebKit/Chromium expose document selections through text markers rather
// than the numeric ranges used by native text controls. These CF functions
// are declared by AXUIElement.h but missing from accessibility-sys 0.2.
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXTextMarkerRangeGetTypeID() -> core_foundation::base::CFTypeID;
    fn AXTextMarkerRangeCopyStartMarker(range: CFTypeRef) -> CFTypeRef;
    fn AXTextMarkerRangeCopyEndMarker(range: CFTypeRef) -> CFTypeRef;
}

fn string_value(value: &Owned) -> Option<String> {
    unsafe { CFType::wrap_under_get_rule(value.0) }
        .downcast::<CFString>()
        .map(|text| text.to_string())
}

fn marker_pair(range: &Owned) -> Option<(Owned, Owned)> {
    if unsafe { core_foundation::base::CFGetTypeID(range.0) != AXTextMarkerRangeGetTypeID() } {
        return None;
    }
    let start = Owned(unsafe { AXTextMarkerRangeCopyStartMarker(range.0) });
    let end = Owned(unsafe { AXTextMarkerRangeCopyEndMarker(range.0) });
    (!start.0.is_null() && !end.0.is_null()).then_some((start, end))
}

fn marker_range(element: AXUIElementRef, start: &Owned, end: &Owned) -> Option<Owned> {
    let markers = core_foundation::array::CFArray::from_CFTypes(&[
        unsafe { CFType::wrap_under_get_rule(start.0) },
        unsafe { CFType::wrap_under_get_rule(end.0) },
    ]);
    copy_parameter(
        element,
        "AXTextMarkerRangeForUnorderedTextMarkers",
        markers.as_CFTypeRef(),
    )
}

fn marker_length(element: AXUIElementRef, range: &Owned) -> Option<i64> {
    let value = copy_parameter(element, "AXLengthForTextMarkerRange", range.0)?;
    unsafe { CFType::wrap_under_get_rule(value.0) }
        .downcast::<core_foundation::number::CFNumber>()?
        .to_i64()
}

fn marker_bounds(element: AXUIElementRef, selected: &Owned) -> Option<SelectionRect> {
    let (a, b) = marker_pair(selected)?;
    // The selection's anchor/focus can be reversed after a backwards drag.
    let normalized = marker_range(element, &a, &b)?;
    let (start, _) = marker_pair(&normalized)?;
    let length = marker_length(element, &normalized)?;
    if length <= 0 {
        return None;
    }
    let end = copy_parameter(element, "AXNextLineEndTextMarkerForTextMarker", start.0)?;
    let line = marker_range(element, &start, &end)?;
    let line_length = marker_length(element, &line)?;
    if line_length <= 0 {
        return None;
    }
    let first_line = if length <= line_length {
        &normalized
    } else {
        &line
    };
    rect_value(&copy_parameter(
        element,
        "AXBoundsForTextMarkerRange",
        first_line.0,
    )?)
}

/// Returns the exact selection identity to revalidate after the geometry IPC.
fn selection_contents(
    element: AXUIElementRef,
) -> Option<(String, SelectionRect, &'static str, Owned)> {
    use accessibility_sys::*;
    if let Some(range) = copy_attribute(element, kAXSelectedTextRangeAttribute)
        && let Some(rect) = text_bounds(element, &range)
        && let Some(text) =
            copy_attribute(element, kAXSelectedTextAttribute).and_then(|v| string_value(&v))
        && !text.trim().is_empty()
    {
        return Some((text, rect, kAXSelectedTextRangeAttribute, range));
    }
    let range = copy_attribute(element, "AXSelectedTextMarkerRange")?;
    let rect = marker_bounds(element, &range)?;
    let text = copy_parameter(element, "AXStringForTextMarkerRange", range.0)
        .and_then(|v| string_value(&v))?;
    (!text.trim().is_empty()).then_some((text, rect, "AXSelectedTextMarkerRange", range))
}

fn element_rect(element: AXUIElementRef) -> Option<SelectionRect> {
    use accessibility_sys::*;
    let position = copy_attribute(element, kAXPositionAttribute)?;
    let size = copy_attribute(element, kAXSizeAttribute)?;
    // Check CF types before treating provider data as AXValue objects.
    if unsafe {
        core_foundation::base::CFGetTypeID(position.0) != AXValueGetTypeID()
            || core_foundation::base::CFGetTypeID(size.0) != AXValueGetTypeID()
    } {
        return None;
    }
    let mut p = objc2_foundation::NSPoint::default();
    let mut s = objc2_foundation::NSSize::default();
    let point_ok = unsafe {
        AXValueGetValue(
            position.0 as AXValueRef,
            kAXValueTypeCGPoint,
            (&mut p as *mut objc2_foundation::NSPoint).cast(),
        )
    };
    let size_ok = unsafe {
        AXValueGetValue(
            size.0 as AXValueRef,
            kAXValueTypeCGSize,
            (&mut s as *mut objc2_foundation::NSSize).cast(),
        )
    };
    let rect = SelectionRect {
        x: p.x,
        y: p.y,
        width: s.width,
        height: s.height,
    };
    (point_ok && size_ok && rect.valid()).then_some(rect)
}

fn read_selection(pid: i32) -> Option<Selection> {
    use accessibility_sys::*;
    let app = Owned(unsafe { AXUIElementCreateApplication(pid) } as CFTypeRef);
    unsafe {
        AXUIElementSetMessagingTimeout(app.element(), MESSAGING_TIMEOUT);
    }
    let focused = copy_attribute(app.element(), kAXFocusedUIElementAttribute)?;
    let mut source = Owned(unsafe { core_foundation::base::CFRetain(focused.0) });
    // A selected link can leave keyboard focus on a child of its document.
    // Walk only a bounded ancestry; never scan unrelated application text.
    for _ in 0..12 {
        let role = copy_attribute(source.element(), kAXRoleAttribute)
            .and_then(|value| string_value(&value));
        let secure = copy_attribute(source.element(), kAXSubroleAttribute)
            .and_then(|value| string_value(&value));
        if secure.as_deref() == Some(kAXSecureTextFieldSubrole) {
            return None;
        }
        if let Some((text, mut rect, name, selected)) = selection_contents(source.element()) {
            if let Some(viewport) = element_rect(source.element()) {
                rect = rect.intersection(viewport)?;
            }
            if let Some(window) = copy_attribute(source.element(), kAXWindowAttribute)
                && let Some(viewport) = element_rect(window.element())
            {
                rect = rect.intersection(viewport)?;
            }
            let still_focused = copy_attribute(app.element(), kAXFocusedUIElementAttribute)?;
            let still_selected = copy_attribute(source.element(), name)?;
            if !unsafe {
                core_foundation::base::CFEqual(focused.0, still_focused.0) != 0
                    && core_foundation::base::CFEqual(selected.0, still_selected.0) != 0
            } {
                return None;
            }
            return Some(Selection { text, rect, pid });
        }
        if role.as_deref() == Some(kAXWindowRole) {
            break;
        }
        source = copy_attribute(source.element(), kAXParentAttribute)?;
    }
    None
}

/// One coherent text/bounds snapshot from the expected application. Does not
/// touch the clipboard or require UI work on the calling thread.
pub fn selection(pid: i32) -> Option<Selection> {
    if !is_trusted(false) || pid == std::process::id() as i32 {
        return None;
    }
    read_selection(pid).or_else(|| {
        enable_manual_accessibility(pid);
        read_selection(pid)
    })
}

/// AX objects and their CFRunLoop source stay on the selection worker.
/// Callbacks only enqueue a notification; they never perform provider reads.
pub struct SelectionObserver {
    observer: Owned,
    app: Owned,
    focused: Option<Owned>,
    context: Box<ObserverContext>,
}
struct ObserverContext {
    notify: Box<dyn Fn(bool)>,
    focus_changed: std::cell::Cell<bool>,
}

unsafe extern "C" fn selection_changed(
    _: accessibility_sys::AXObserverRef,
    _: AXUIElementRef,
    name: core_foundation::string::CFStringRef,
    context: *mut std::ffi::c_void,
) {
    let context = unsafe { &*(context as *const ObserverContext) };
    let name = unsafe { CFString::wrap_under_get_rule(name) };
    let focus = name == accessibility_sys::kAXFocusedUIElementChangedNotification;
    if focus {
        context.focus_changed.set(true);
    }
    (context.notify)(focus);
}

impl SelectionObserver {
    pub fn new(pid: i32, notify: impl Fn(bool) + 'static) -> Option<Self> {
        use accessibility_sys::*;
        if !is_trusted(false) {
            return None;
        }
        enable_manual_accessibility(pid);
        let mut observer = ptr::null_mut();
        if unsafe { AXObserverCreate(pid, selection_changed, &mut observer) } != kAXErrorSuccess {
            return None;
        }
        let app = Owned(unsafe { AXUIElementCreateApplication(pid) } as CFTypeRef);
        unsafe {
            AXUIElementSetMessagingTimeout(app.element(), MESSAGING_TIMEOUT);
        }
        let mut this = Self {
            observer: Owned(observer as CFTypeRef),
            app,
            focused: None,
            context: Box::new(ObserverContext {
                notify: Box::new(notify),
                focus_changed: std::cell::Cell::new(true),
            }),
        };
        this.subscribe(this.app.element(), kAXFocusedUIElementChangedNotification);
        this.subscribe(this.app.element(), kAXSelectedTextChangedNotification);
        unsafe {
            core_foundation::runloop::CFRunLoopAddSource(
                core_foundation::runloop::CFRunLoopGetCurrent(),
                AXObserverGetRunLoopSource(observer),
                core_foundation::runloop::kCFRunLoopDefaultMode,
            );
        }
        this.refresh_focus();
        Some(this)
    }

    fn subscribe(&mut self, element: AXUIElementRef, name: &'static str) {
        let name = CFString::from_static_string(name);
        let result = unsafe {
            accessibility_sys::AXObserverAddNotification(
                self.observer.0 as _,
                element,
                name.as_concrete_TypeRef(),
                (&mut *self.context as *mut ObserverContext).cast(),
            )
        };
        debug_log!("selection: observe {name}: AXError {result}");
    }

    pub fn refresh_focus(&mut self) {
        if !self.context.focus_changed.replace(false) {
            return;
        }
        if let Some(old) = self.focused.take() {
            let name =
                CFString::from_static_string(accessibility_sys::kAXSelectedTextChangedNotification);
            unsafe {
                accessibility_sys::AXObserverRemoveNotification(
                    self.observer.0 as _,
                    old.element(),
                    name.as_concrete_TypeRef(),
                );
            }
        }
        if let Some(focused) = copy_attribute(self.app.element(), kAXFocusedUIElementAttribute) {
            self.subscribe(
                focused.element(),
                accessibility_sys::kAXSelectedTextChangedNotification,
            );
            self.focused = Some(focused);
        }
    }
}
impl Drop for SelectionObserver {
    fn drop(&mut self) {
        unsafe {
            core_foundation::runloop::CFRunLoopRemoveSource(
                core_foundation::runloop::CFRunLoopGetCurrent(),
                accessibility_sys::AXObserverGetRunLoopSource(self.observer.0 as _),
                core_foundation::runloop::kCFRunLoopDefaultMode,
            );
        }
        // Unregister before dropping the callback context or its AX elements.
        // The observer is released by Owned after Drop returns.
        for element in [Some(&self.app), self.focused.as_ref()]
            .into_iter()
            .flatten()
        {
            for name in [
                accessibility_sys::kAXSelectedTextChangedNotification,
                accessibility_sys::kAXFocusedUIElementChangedNotification,
            ] {
                let name = CFString::from_static_string(name);
                unsafe {
                    accessibility_sys::AXObserverRemoveNotification(
                        self.observer.0 as _,
                        element.element(),
                        name.as_concrete_TypeRef(),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod geometry_tests {
    use super::*;
    use core_foundation::base::CFRange;
    #[test]
    fn range_intersection_rejects_empty_and_overflow() {
        let range = intersect_ranges(CFRange::init(20, 30), CFRange::init(35, 30)).unwrap();
        assert_eq!((range.location, range.length), (35, 15));
        assert!(intersect_ranges(CFRange::init(0, 5), CFRange::init(5, 5)).is_none());
        assert!(intersect_ranges(CFRange::init(isize::MAX, 5), CFRange::init(0, 5)).is_none());
    }
    #[test]
    fn bounds_reject_nan_and_offscreen_rectangles() {
        let rect = SelectionRect {
            x: -1800.,
            y: 40.,
            width: 80.,
            height: 20.,
        };
        assert!(rect.valid());
        assert!(
            !SelectionRect {
                x: f64::NAN,
                ..rect
            }
            .valid()
        );
        assert!(
            rect.intersection(SelectionRect {
                x: 0.,
                y: 0.,
                width: 100.,
                height: 100.
            })
            .is_none()
        );
        assert_eq!(
            rect.intersection(SelectionRect { x: -1780., ..rect })
                .unwrap()
                .width,
            60.
        );
    }
}
