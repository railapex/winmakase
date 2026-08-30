//! Windows virtual-memory commit pressure.
//!
//! GlazeWM 3.10.1 can abort with `__fastfail(7)` when startup or wake display
//! work allocates during a transient commit spike. The supervisor uses the
//! same `GlobalMemoryStatusEx` signal as the upstream fix to avoid relaunching
//! the tiler into the failing window.

use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

/// Whether less than half of the system commit limit remains available.
///
/// An unreadable status is treated as no pressure: a failed diagnostic must
/// not permanently prevent the desktop from starting.
pub fn is_under_pressure() -> bool {
    // SAFETY: `MEMORYSTATUSEX` contains numeric fields and accepts zero
    // initialization. Windows requires `dwLength` before the call.
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return false;
    }
    under_pressure(status.ullTotalPageFile, status.ullAvailPageFile)
}

fn under_pressure(total_commit: u64, available_commit: u64) -> bool {
    total_commit > 0 && available_commit < total_commit / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn less_than_half_available_is_pressure() {
        assert!(under_pressure(100, 49));
        assert!(!under_pressure(100, 50));
        assert!(!under_pressure(100, 80));
        assert!(!under_pressure(0, 0));
    }
}
