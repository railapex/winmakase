//! Console control events.
//!
//! Ctrl+C must not kill the supervisor outright. GlazeWM has to be asked to
//! exit through its own CLI so `glazewm-watcher` puts every window back where
//! it was; kanata has to die too, or the desktop is left with a mod key and
//! nothing listening for it. So the handler only raises a flag and the main
//! loop does the work.
//!
//! Close/logoff/shutdown events are different: Windows terminates the process a
//! few seconds after the handler returns. For those we block inside the handler
//! until the main loop reports it has finished, which is the only way the
//! desktop gets restored on a logoff.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use windows_sys::Win32::System::Console::{
    CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
    SetConsoleCtrlHandler,
};
use windows_sys::core::BOOL;

static REQUESTED: AtomicBool = AtomicBool::new(false);
static FINISHED: AtomicBool = AtomicBool::new(false);

/// Windows gives a close/logoff handler roughly five seconds before it
/// terminates the process; stay inside it with a second to spare.
///
/// Public because the supervisor sizes its console-shutdown budget against this
/// and asserts the relationship at compile time — a shutdown that outruns the
/// grace period is killed half-done, with GlazeWM's windows unrestored and
/// kanata still holding the keyboard.
pub const GRACE_MS: u64 = 4_000;
const GRACE: Duration = Duration::from_millis(GRACE_MS);

pub fn install() -> bool {
    unsafe { SetConsoleCtrlHandler(Some(handler), 1) != 0 }
}

pub fn shutdown_requested() -> bool {
    REQUESTED.load(Ordering::SeqCst)
}

/// Ask the main loop to shut down, from any thread. The windowless supervisor
/// (`winmakased`, a GUI-subsystem process with no console) gets its session-end
/// notice as `WM_ENDSESSION` on the display watch's window rather than as a
/// console event; both funnel here.
pub fn request_shutdown() {
    REQUESTED.store(true, Ordering::SeqCst);
}

/// Block until the main loop reports the shutdown finished, up to the OS grace
/// period. What the close/logoff paths do to keep Windows from terminating the
/// process before GlazeWM's windows are restored.
pub fn block_until_finished_within_grace() {
    let deadline = Instant::now() + GRACE;
    while !FINISHED.load(Ordering::SeqCst) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }
}

/// Called once the supervisor has stopped its children and written final state.
pub fn mark_finished() {
    FINISHED.store(true, Ordering::SeqCst);
}

unsafe extern "system" fn handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT => {
            request_shutdown();
            1
        }
        CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => {
            request_shutdown();
            block_until_finished_within_grace();
            1
        }
        _ => 0,
    }
}
