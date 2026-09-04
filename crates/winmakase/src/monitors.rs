//! Per-monitor working-area reserves — the evidence for the bar-dock check.
//!
//! Zebar's `dockToEdge` registers an appbar per bar window, and the
//! registrations race (live findings, 2026-08-28 and 2026-08-30: with the
//! systray provider active, one, two, or all three bars can lose their dock —
//! bars render, but windows tile underneath them). A lost dock shows up as a
//! monitor whose working area starts at its bounds' top. The supervisor reads
//! this and bounces the bar; a restart demonstrably re-wins the race.

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

/// Each monitor's full bounds (`rcMonitor`), virtual-screen coordinates.
pub fn bounds() -> Vec<RECT> {
    let mut out: Vec<RECT> = Vec::new();
    unsafe extern "system" fn callback(
        monitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> i32 {
        unsafe {
            let out = &mut *(lparam as *mut Vec<RECT>);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut info) != 0 {
                out.push(info.rcMonitor);
            }
            1
        }
    }
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(callback),
            &mut out as *mut Vec<RECT> as LPARAM,
        );
    }
    out
}

/// What the reserves say about the bar dock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockVerdict {
    /// Every enumerated monitor reserved space. Nothing to do.
    Consistent,
    /// No enumerated monitor reserved space. A running configured bar should
    /// dock, so all-zero is a total dock failure rather than consistency.
    Missing,
    /// Some monitors reserved, some not: the dock race was lost somewhere.
    Partial,
    /// Monitor enumeration returned no evidence. Do not churn the bar on an
    /// unreadable state.
    Unknown,
}

pub fn dock_verdict(reserves: &[i32]) -> DockVerdict {
    if reserves.is_empty() {
        return DockVerdict::Unknown;
    }
    let reserved = reserves.iter().filter(|r| **r > 0).count();
    if reserved == reserves.len() {
        DockVerdict::Consistent
    } else if reserved == 0 {
        DockVerdict::Missing
    } else {
        DockVerdict::Partial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_monitor_reserved_is_consistent() {
        assert_eq!(dock_verdict(&[40, 40, 40]), DockVerdict::Consistent);
        assert_eq!(dock_verdict(&[48]), DockVerdict::Consistent);
    }

    #[test]
    fn all_zero_is_a_total_dock_failure() {
        assert_eq!(dock_verdict(&[0, 0, 0]), DockVerdict::Missing);
        assert_eq!(dock_verdict(&[0]), DockVerdict::Missing);
    }

    #[test]
    fn no_monitor_evidence_is_unknown() {
        assert_eq!(dock_verdict(&[]), DockVerdict::Unknown);
    }

    #[test]
    fn a_mixed_reserve_is_a_lost_race() {
        assert_eq!(dock_verdict(&[40, 0, 40]), DockVerdict::Partial);
        assert_eq!(dock_verdict(&[0, 40, 0]), DockVerdict::Partial);
    }

    #[test]
    fn the_real_machine_reports_at_least_one_monitor() {
        assert!(!top_reserves().is_empty());
        assert_eq!(bounds().len(), top_reserves().len());
    }
}
