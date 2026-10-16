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
    AXUIElementCopyAttributeValue, AXUIElementCreateApplication, AXUIElementCreateSystemWide,
    AXUIElementRef, AXUIElementSetAttributeValue,
    AXUIElementSetMessagingTimeout, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
    kAXErrorSuccess, kAXFocusedUIElementAttribute, kAXSelectedTextAttribute,
    kAXTrustedCheckOptionPrompt,
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
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
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
