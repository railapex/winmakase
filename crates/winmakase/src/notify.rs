//! Native user-facing notices.
//!
//! A MessageBox, not a toast: zero dependencies, no AppID/COM registration,
//! works from any context including a chord's `shell-exec`. The callers are
//! rare, deliberate explainers (documented-gap chords), where modal is
//! acceptable — never use this on a hot path.

use windows_sys::Win32::UI::WindowsAndMessaging::{
    MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_TOPMOST, MessageBoxW,
};

pub fn info(title: &str, body: &str) {
    let t: Vec<u16> = title.encode_utf16().chain([0]).collect();
    let b: Vec<u16> = body.encode_utf16().chain([0]).collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            b.as_ptr(),
            t.as_ptr(),
            MB_OK | MB_ICONINFORMATION | MB_SETFOREGROUND | MB_TOPMOST,
        );
    }
}
