//! Taskbar state, owned by the supervisor for the life of the stack.
//!
//! Winmakase's end state hides the taskbar (Zebar replaces it). Until this
//! lived here, hiding was a side effect of the bring-up script — which meant
//! a reboot brought the taskbar back even though the logon task brought the
//! stack up. The supervisor hides on start and puts back whatever state it
//! found on shutdown; `WinmakasePanic` restores independently as the belt to
//! this suspender.

use windows_sys::Win32::UI::Shell::{
    ABM_GETSTATE, ABM_SETSTATE, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage,
};

/// The taskbar's current appbar state bits (`ABS_AUTOHIDE` is the one used).
pub fn get_state() -> u32 {
    let mut data = zeroed();
    (unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) }) as u32
}

/// Set the appbar state bits wholesale (a value from `get_state`, or
/// [`ABS_AUTOHIDE`]).
pub fn set_state(state: u32) {
    let mut data = zeroed();
    data.lParam = state as isize;
    unsafe {
        SHAppBarMessage(ABM_SETSTATE, &mut data);
    }
}

pub const AUTOHIDE: u32 = ABS_AUTOHIDE;

fn zeroed() -> APPBARDATA {
    let mut data: APPBARDATA = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Read-only: state bits are a small set; anything else means the call
    /// itself is broken.
    #[test]
    fn reading_the_state_returns_state_bits() {
        assert!(get_state() <= 0b11);
    }

    /// Round-trips the REAL taskbar of whoever runs the test — rude on a dev
    /// desktop, so opt-in.
    #[test]
    #[ignore = "toggles the real taskbar; run explicitly"]
    fn set_and_restore_round_trip() {
        let prior = get_state();
        set_state(AUTOHIDE);
        assert_eq!(get_state(), AUTOHIDE);
        set_state(prior);
        assert_eq!(get_state(), prior);
    }
}
