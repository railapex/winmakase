//! Is that pid still there?
//!
//! `health.json` records transitions, not heartbeats, so its age says nothing
//! about whether the supervisor is alive. A supervisor killed outright — task
//! ended, machine wedged — leaves a file that reads "running" forever. Checking
//! the pid is what turns `winmakase status` from a file dump into an answer.

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, GetLastError, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, WaitForSingleObject,
};

/// Standard access right; `WaitForSingleObject` fails without it, and
/// windows-sys only exposes the constant under its file-system feature.
const SYNCHRONIZE: u32 = 0x0010_0000;

/// `Some(true)`/`Some(false)` when we can tell, `None` when we cannot.
///
/// Access-denied is treated as alive on purpose: you cannot be denied access to
/// a process that does not exist, so a denial is evidence of existence. Saying
/// "gone" there would be an inference dressed up as an observation.
///
/// Liveness comes from waiting on the process handle with a zero timeout rather
/// than from `GetExitCodeProcess`, which cannot tell a running process from one
/// that exited with code 259.
pub fn is_alive(pid: u32) -> Option<bool> {
    if pid == 0 {
        return Some(false);
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid);
        if handle.is_null() {
            return match GetLastError() {
                ERROR_ACCESS_DENIED => Some(true),
                _ => Some(false),
            };
        }
        let waited = WaitForSingleObject(handle, 0);
        CloseHandle(handle);
        match waited {
            // Signalled means the process object has been set — it has exited.
            WAIT_OBJECT_0 => Some(false),
            WAIT_TIMEOUT => Some(true),
            // WAIT_FAILED or anything else: say so rather than guess.
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn our_own_process_is_alive() {
        assert_eq!(is_alive(std::process::id()), Some(true));
    }

    #[test]
    fn pid_zero_is_not_a_process() {
        assert_eq!(is_alive(0), Some(false));
    }

    #[test]
    fn a_finished_process_reads_as_gone() {
        let mut child = std::process::Command::new("cmd")
            .args(["/c", "exit", "0"])
            .spawn()
            .expect("cmd.exe is always present on Windows");
        let pid = child.id();
        child.wait().unwrap();
        // The pid could be recycled in principle; not within a few
        // milliseconds on the same machine.
        assert_eq!(is_alive(pid), Some(false));
    }
}
