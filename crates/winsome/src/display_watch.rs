//! Display-change events for the supervisor.
//!
//! glzr-io/glazewm#1233 reports GlazeWM needing a restart after a monitor
//! reconnect. The M0 spike could NOT reproduce it on 3.10.1 — a full
//! teleprompter add/remove/re-add cycle ran clean — so the automatic bounce is
//! insurance, off by default (`bounce_on_display_change`), and would be the
//! wrong default anyway: the occasional-display toggle is a routine workflow
//! here, and bouncing the tiler on every toggle would manufacture the very
//! disruption #1233 describes. The events themselves are always logged, so the
//! day the bug does show up, the log already ties it to the replug.
//!
//! Mechanism: `WM_DISPLAYCHANGE` is a broadcast to top-level windows —
//! message-only windows do not receive broadcasts — so a hidden top-level
//! window on its own message-loop thread counts the events.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetMessageW,
    MSG, PostMessageW, RegisterClassW, SetWindowLongPtrW, TranslateMessage, WM_CLOSE,
    WM_DESTROY, WM_DISPLAYCHANGE, WM_NCCREATE, WNDCLASSW,
};

/// Counts `WM_DISPLAYCHANGE` broadcasts on a hidden window's thread; the
/// supervisor drains the count from its poll loop.
pub struct DisplayWatch {
    changes: Arc<AtomicU64>,
    hwnd: isize,
    thread: Option<JoinHandle<()>>,
}

// The window handle is only ever used with PostMessageW, which is documented
// cross-thread safe.
unsafe impl Send for DisplayWatch {}

impl DisplayWatch {
    /// Create the hidden window and start its message loop. Errors instead of
    /// pretending: a supervisor that silently is not watching would be worse
    /// than one that says it could not start the watch.
    pub fn start() -> io::Result<Self> {
        let changes = Arc::new(AtomicU64::new(0));
        let for_thread = Arc::clone(&changes);
        let (tx, rx) = mpsc::channel::<Result<isize, String>>();

        let thread = thread::Builder::new()
            .name("display-watch".to_string())
            .spawn(move || message_loop(for_thread, &tx))?;

        match rx.recv() {
            Ok(Ok(hwnd)) => Ok(Self {
                changes,
                hwnd,
                thread: Some(thread),
            }),
            Ok(Err(msg)) => {
                let _ = thread.join();
                Err(io::Error::other(msg))
            }
            Err(_) => {
                let _ = thread.join();
                Err(io::Error::other("display-watch thread died before reporting"))
            }
        }
    }

    /// Display changes since the last call.
    pub fn take_changes(&self) -> u64 {
        self.changes.swap(0, Ordering::SeqCst)
    }

    /// Deliver a synthetic `WM_DISPLAYCHANGE`, for tests — firing a real one
    /// means actually changing somebody's display mode.
    #[doc(hidden)]
    pub fn post_display_change(&self) {
        unsafe {
            PostMessageW(self.hwnd as HWND, WM_DISPLAYCHANGE, 0, 0);
        }
    }
}

impl Drop for DisplayWatch {
    fn drop(&mut self) {
        unsafe {
            PostMessageW(self.hwnd as HWND, WM_CLOSE, 0, 0);
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn message_loop(changes: Arc<AtomicU64>, ready: &mpsc::Sender<Result<isize, String>>) {
    // The counter travels to the wndproc through the create params and lands in
    // GWLP_USERDATA; a leaked Arc reference keeps it alive as long as the
    // window (the window dies with this thread, which `DisplayWatch` joins).
    let counter_ptr = Arc::into_raw(changes);

    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        // Unique per process: RegisterClassW fails on a duplicate name, and two
        // watches in one process (tests) must not fight over a class.
        let class_name = wide(&format!(
            "WinsomeDisplayWatch-{}-{:p}",
            std::process::id(),
            counter_ptr
        ));

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: module,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        if RegisterClassW(&wc) == 0 {
            let _ = ready.send(Err(format!(
                "RegisterClassW failed (Win32 error {})",
                GetLastError()
            )));
            drop(Arc::from_raw(counter_ptr));
            return;
        }

        // Top-level (no parent), never shown. WS_OVERLAPPED with no ShowWindow
        // stays invisible but still receives broadcasts.
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            module,
            counter_ptr as *mut _,
        );
        if hwnd.is_null() {
            let _ = ready.send(Err(format!(
                "CreateWindowExW failed (Win32 error {})",
                GetLastError()
            )));
            drop(Arc::from_raw(counter_ptr));
            return;
        }
        let _ = ready.send(Ok(hwnd as isize));

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        drop(Arc::from_raw(counter_ptr));
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCCREATE => {
                let create = lparam as *const CREATESTRUCTW;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*create).lpCreateParams as isize);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DISPLAYCHANGE => {
                let ptr = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    hwnd,
                    GWLP_USERDATA,
                ) as *const AtomicU64;
                if !ptr.is_null() {
                    (*ptr).fetch_add(1, Ordering::SeqCst);
                }
                0
            }
            WM_DESTROY => {
                windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn counts_posted_display_changes_and_drains_on_take() {
        let watch = DisplayWatch::start().unwrap();
        assert_eq!(watch.take_changes(), 0);

        watch.post_display_change();
        watch.post_display_change();

        // The message crosses a thread; poll briefly rather than sleep blind.
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut seen = 0;
        while seen < 2 && Instant::now() < deadline {
            seen += watch.take_changes();
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(seen, 2, "both posted events must be counted");
        assert_eq!(watch.take_changes(), 0, "take drains the count");
    }

    #[test]
    fn two_watches_in_one_process_do_not_collide() {
        let a = DisplayWatch::start().unwrap();
        let b = DisplayWatch::start().unwrap();
        a.post_display_change();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut seen = 0;
        while seen < 1 && Instant::now() < deadline {
            seen += a.take_changes();
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(seen, 1);
        // b's window also received the (posted-to-a) message? No: PostMessageW
        // targets one window, so b saw nothing. A real WM_DISPLAYCHANGE is a
        // broadcast and both would count it — same counter semantics either way.
        assert_eq!(b.take_changes(), 0);
    }
}
