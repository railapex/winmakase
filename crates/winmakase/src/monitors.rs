//! Per-monitor working-area reserves — the evidence for the bar-dock check.
//!
//! Zebar's `dockToEdge` registers an appbar per bar window, and the
//! registrations race (live findings, 2026-08-28: with the systray provider
//! active, one or two of the three bars would reliably lose their dock — bars
//! render, but windows tile underneath them). A lost dock shows up as a
//! monitor whose working area starts at its bounds' top while its siblings
//! are reserved. The supervisor reads this and bounces the bar; a restart
//! demonstrably re-wins the race.

use windows_sys::Win32::Foundation::{LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};

/// Each monitor's reserved space at the top edge, in pixels
/// (`work.top - bounds.top`).
pub fn top_reserves() -> Vec<i32> {
    let mut out: Vec<i32> = Vec::new();
    unsafe extern "system" fn callback(
        monitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> i32 {
        unsafe {
            let out = &mut *(lparam as *mut Vec<i32>);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut info) != 0 {
                out.push(info.rcWork.top - info.rcMonitor.top);
            }
            1
        }
    }
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(callback),
            &mut out as *mut Vec<i32> as LPARAM,
        );
    }
    out
}

/// What the reserves say about the bar dock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockVerdict {
    /// Every monitor reserved, or none — either the dock fully took, or the
    /// bar does not dock at all (also a consistent choice). Nothing to do.
    Consistent,
    /// Some monitors reserved, some not: the dock race was lost somewhere.
    Partial,
}

pub fn dock_verdict(reserves: &[i32]) -> DockVerdict {
    let reserved = reserves.iter().filter(|r| **r > 0).count();
    if reserved == 0 || reserved == reserves.len() {
        DockVerdict::Consistent
    } else {
        DockVerdict::Partial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_or_nothing_is_consistent() {
        assert_eq!(dock_verdict(&[40, 40, 40]), DockVerdict::Consistent);
        assert_eq!(dock_verdict(&[0, 0, 0]), DockVerdict::Consistent);
        assert_eq!(dock_verdict(&[]), DockVerdict::Consistent);
        assert_eq!(dock_verdict(&[48]), DockVerdict::Consistent);
    }

    #[test]
    fn a_mixed_reserve_is_a_lost_race() {
        assert_eq!(dock_verdict(&[40, 0, 40]), DockVerdict::Partial);
        assert_eq!(dock_verdict(&[0, 40, 0]), DockVerdict::Partial);
    }

    #[test]
    fn the_real_machine_reports_at_least_one_monitor() {
        assert!(!top_reserves().is_empty());
    }
}
