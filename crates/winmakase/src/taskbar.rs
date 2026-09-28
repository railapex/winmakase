//! Taskbar state, owned by the supervisor for the life of the stack.
//!
//! Winmakase's end state hides the taskbar (Zebar replaces it). Until this
//! lived here, hiding was a side effect of the bring-up script — which meant
//! a reboot brought the taskbar back even though the logon task brought the
//! stack up. The supervisor hides on start and puts back whatever state it
//! found on shutdown; `WinmakasePanic` restores independently as the belt to
//! this suspender.
//!
//! `ABM_SETSTATE(ABS_AUTOHIDE)` is required so the work area includes the
//! strip and GlazeWM tiles to the edge. On Windows 11 that flag does not
//! actually collapse the XAML bar after a hover: Start toggle hides it,
//! mouse-to-edge shows it, mouse-leave leaves it stuck painted over the tiles.
//! The peek controller owns show/hide of `Shell_TrayWnd` /
//! `Shell_SecondaryTrayWnd` so leave collapses and the edge still peeks.
//!
//! Live finding (2026-09-04): while Start is open, `EnumWindows` in this
//! process hands over the secondaries and Zebar's fake `Shell_TrayWnd` but
//! never the real primary, so the primary was never shown for Start.
//! `FindWindowExW` by class finds it every time — that is how trays are
//! enumerated now. A hold-open is also re-asserted every tick on whichever
//! trays still lag, so one missed show is a 250ms delay, not a stuck bar.
//! Explorer parks a collapsed bar just past the monitor edge and slides it
//! back on its own ~75ms after a show; moving it ourselves is ignored.

use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{GetLastError, HWND, POINT, RECT};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows_sys::Win32::UI::Shell::{
    ABM_GETSTATE, ABM_SETSTATE, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowRect,
    GetWindowThreadProcessId, HWND_TOPMOST, IsWindowVisible, SWP_HIDEWINDOW, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SetWindowPos,
};

use crate::monitors;

/// How close to a monitor's bottom edge counts as "peek" while the bar is gone.
const PEEK_PX: i32 = 8;
/// A fully-shown taskbar is ~48px; Explorer's collapsed autohide line is ~2px.
/// Only the stuck-shown state is ours to collapse.
const SHOWN_MIN_PX: i32 = 24;
/// Pause after the cursor leaves before collapsing, so a click can land.
const HIDE_AFTER: Duration = Duration::from_millis(400);

const TRAY_CLASSES: &[&str] = &["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

/// Flyouts / Start / overflow that should keep the bar up even if the cursor
/// has left the tray rect.
const HOLD_CLASSES: &[&str] = &[
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "NotifyIconOverflowWindow",
    "TopLevelWindowForOverflowXamlIsland",
    "XamlExplorerHostIslandWindow",
    "TaskListOverlayWnd",
    "TaskListThumbnailWnd",
    "#32768",
];

const HOLD_PROCESSES: &[&str] = &[
    "startmenuexperiencehost.exe",
    "searchhost.exe",
    "shellexperiencehost.exe",
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pt {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    fn from_win(r: RECT) -> Self {
        Self {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }

    fn width(self) -> i32 {
        self.right - self.left
    }

    fn height(self) -> i32 {
        self.bottom - self.top
    }

    fn contains(self, p: Pt) -> bool {
        p.x >= self.left && p.x < self.right && p.y >= self.top && p.y < self.bottom
    }

    fn is_bar_shape(self) -> bool {
        let w = self.width();
        let h = self.height();
        w > 0 && h > 0 && ((h <= 96 && w >= 200) || (w <= 96 && h >= 200))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tray {
    hwnd: isize,
    rect: Rect,
    visible: bool,
}

impl Tray {
    fn fully_shown(self) -> bool {
        self.visible && self.rect.height() >= SHOWN_MIN_PX
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Show,
    Hide,
    None,
}

struct View {
    cursor: Pt,
    monitors: Vec<Rect>,
    trays: Vec<Tray>,
    hold_open: bool,
}

impl View {
    fn want_open(&self) -> bool {
        self.hold_open || self.cursor_in_peek() || self.cursor_over_shown_tray()
    }

    fn cursor_in_peek(&self) -> bool {
        self.monitors.iter().any(|m| {
            let strip = Rect {
                left: m.left,
                top: m.bottom - PEEK_PX,
                right: m.right,
                bottom: m.bottom,
            };
            strip.contains(self.cursor)
        })
    }

    fn cursor_over_shown_tray(&self) -> bool {
        self.trays
            .iter()
            .any(|t| t.fully_shown() && t.rect.contains(self.cursor))
    }

    fn any_fully_shown(&self) -> bool {
        self.trays.iter().any(|t| t.fully_shown())
    }

    fn all_fully_shown(&self) -> bool {
        !self.trays.is_empty() && self.trays.iter().all(|t| t.fully_shown())
    }

    fn lagging(&self) -> Vec<Tray> {
        self.trays
            .iter()
            .copied()
            .filter(|t| !t.fully_shown())
            .collect()
    }
}

fn decide(view: &View, outside_since: Option<Instant>, now: Instant) -> (Action, Option<Instant>) {
    if view.want_open() {
        // Every tray, not any: Explorer can swallow the primary's show while
        // Start opens, and the secondaries alone would otherwise satisfy us.
        let action = if view.all_fully_shown() {
            Action::None
        } else {
            Action::Show
        };
        return (action, None);
    }
    if view.any_fully_shown() {
        let since = outside_since.unwrap_or(now);
        if now.saturating_duration_since(since) >= HIDE_AFTER {
            (Action::Hide, Some(since))
        } else {
            (Action::None, Some(since))
        }
    } else {
        (Action::None, None)
    }
}

/// Poll-loop peek controller. Collapse on leave; show on the bottom edge.
#[derive(Default)]
pub struct Peek {
    outside_since: Option<Instant>,
    /// One "re-shown" line per hold-open episode, not one per tick.
    lag_logged: bool,
}

impl Peek {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one decision. Returns a log line only when visibility changes.
    pub fn tick(&mut self) -> Option<String> {
        let view = snapshot();
        let (action, since) = decide(&view, self.outside_since, Instant::now());
        self.outside_since = since;
        match action {
            Action::Show => {
                let first = !view.any_fully_shown();
                let lagging = view.lagging();
                if lagging.is_empty() {
                    return None;
                }
                let failed = show_trays(&lagging);
                let mut msg = if first {
                    self.lag_logged = false;
                    Some("taskbar peeked (cursor on edge or flyout)".to_string())
                } else if !self.lag_logged {
                    self.lag_logged = true;
                    Some(format!(
                        "taskbar re-shown ({} tray(s) lagged the first show)",
                        lagging.len()
                    ))
                } else {
                    None
                };
                if !failed.is_empty() && msg.is_some() {
                    msg = msg.map(|m| format!("{m}; SetWindowPos failed: {failed:?}"));
                }
                msg
            }
            Action::Hide => {
                self.lag_logged = false;
                hide_trays(&view.trays);
                Some("taskbar collapsed (cursor left)".into())
            }
            Action::None => None,
        }
    }

    /// Hide every real tray window now (start / hide_taskbar on). No delay.
    pub fn collapse_now(&mut self) -> Option<String> {
        let view = snapshot();
        self.outside_since = None;
        if !view.any_fully_shown() {
            return None;
        }
        hide_trays(&view.trays);
        Some("taskbar collapsed".into())
    }

    /// Put the windows back (shutdown / hide_taskbar off). Does not touch the
    /// appbar flag — the caller restores that.
    pub fn release(&mut self) {
        let view = snapshot();
        self.outside_since = None;
        show_trays(&view.trays);
    }
}

fn snapshot() -> View {
    View {
        cursor: cursor_pos(),
        monitors: monitors::bounds().into_iter().map(Rect::from_win).collect(),
        trays: tray_windows(),
        hold_open: foreground_holds_open(),
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn cursor_pos() -> Pt {
    let mut pt = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut pt) } == 0 {
        return Pt {
            x: i32::MIN,
            y: i32::MIN,
        };
    }
    Pt { x: pt.x, y: pt.y }
}

/// The tray windows, found by class with `FindWindowExW` — not
/// `EnumWindows`. Live 2026-09-04: while Start is opening, EnumWindows in
/// this process hands over the secondaries and Zebar's fake `Shell_TrayWnd`
/// but never the real primary; FindWindowEx by class finds it every time.
fn tray_windows() -> Vec<Tray> {
    let mut out: Vec<Tray> = Vec::new();
    for class in TRAY_CLASSES {
        let wclass = wide(class);
        let mut hwnd: HWND = std::ptr::null_mut();
        loop {
            hwnd = unsafe {
                FindWindowExW(
                    std::ptr::null_mut(),
                    hwnd,
                    wclass.as_ptr(),
                    std::ptr::null(),
                )
            };
            if hwnd.is_null() {
                break;
            }
            let mut rc = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if unsafe { GetWindowRect(hwnd, &mut rc) } == 0 {
                continue;
            }
            let rect = Rect::from_win(rc);
            // Zebar hosts the systray in its own `Shell_TrayWnd`; shape
            // tells the real bars from that and any other impostor.
            if !rect.is_bar_shape() {
                continue;
            }
            out.push(Tray {
                hwnd: hwnd as isize,
                rect,
                visible: unsafe { IsWindowVisible(hwnd) } != 0,
            });
        }
    }
    out
}

/// Show each tray in place. Returns (hwnd, Win32 error) for every call that
/// failed, so a refusal shows up in the log instead of as a missing bar.
fn show_trays(trays: &[Tray]) -> Vec<(isize, u32)> {
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW;
    let mut failed = Vec::new();
    for t in trays {
        let ok = unsafe { SetWindowPos(t.hwnd as HWND, HWND_TOPMOST, 0, 0, 0, 0, flags) };
        if ok == 0 {
            failed.push((t.hwnd, unsafe { GetLastError() }));
        }
    }
    failed
}

fn hide_trays(trays: &[Tray]) {
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_HIDEWINDOW;
    for t in trays {
        unsafe {
            SetWindowPos(t.hwnd as HWND, HWND_TOPMOST, 0, 0, 0, 0, flags);
        }
    }
}

fn foreground_holds_open() -> bool {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return false;
    }
    let class = class_name(hwnd);
    if HOLD_CLASSES.iter().any(|c| class.eq_ignore_ascii_case(c)) {
        return true;
    }
    if class.eq_ignore_ascii_case("Windows.UI.Core.CoreWindow")
        && let Some(name) = process_file_name(hwnd)
    {
        return HOLD_PROCESSES.iter().any(|p| name.eq_ignore_ascii_case(p))
            || name.eq_ignore_ascii_case("explorer.exe");
    }
    if let Some(name) = process_file_name(hwnd) {
        return HOLD_PROCESSES.iter().any(|p| name.eq_ignore_ascii_case(p));
    }
    false
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if n <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

fn process_file_name(hwnd: HWND) -> Option<String> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == 0 {
        return None;
    }
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }
    let mut buf = [0u16; 512];
    let mut size = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size) };
    unsafe {
        windows_sys::Win32::Foundation::CloseHandle(handle);
    }
    if ok == 0 || size == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(&buf[..size as usize]);
    path.rsplit('\\').next().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor() -> Rect {
        Rect {
            left: 0,
            top: 0,
            right: 3840,
            bottom: 1600,
        }
    }

    fn shown_tray() -> Tray {
        Tray {
            hwnd: 1,
            rect: Rect {
                left: 0,
                top: 1552,
                right: 3840,
                bottom: 1600,
            },
            visible: true,
        }
    }

    fn hidden_tray() -> Tray {
        let mut t = shown_tray();
        t.visible = false;
        t
    }

    fn collapsed_tray() -> Tray {
        Tray {
            hwnd: 1,
            rect: Rect {
                left: 0,
                top: 1598,
                right: 3840,
                bottom: 1600,
            },
            visible: true,
        }
    }

    fn view(cursor: Pt, trays: Vec<Tray>, hold: bool) -> View {
        View {
            cursor,
            monitors: vec![monitor()],
            trays,
            hold_open: hold,
        }
    }

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

    #[test]
    fn a_ghost_shell_tray_is_not_a_bar_shape() {
        let ghost = Rect {
            left: 261,
            top: 301,
            right: 3141,
            bottom: 1450,
        };
        assert!(!ghost.is_bar_shape());
        assert!(shown_tray().rect.is_bar_shape());
    }

    #[test]
    fn hover_over_shown_bar_holds_it() {
        let now = Instant::now();
        let (action, since) = decide(
            &view(Pt { x: 100, y: 1570 }, vec![shown_tray()], false),
            None,
            now,
        );
        assert_eq!(action, Action::None);
        assert_eq!(since, None);
    }

    #[test]
    fn cursor_on_bottom_edge_shows_a_hidden_bar() {
        let now = Instant::now();
        let (action, since) = decide(
            &view(Pt { x: 100, y: 1596 }, vec![hidden_tray()], false),
            None,
            now,
        );
        assert_eq!(action, Action::Show);
        assert_eq!(since, None);
    }

    #[test]
    fn cursor_in_the_tile_strip_does_not_count_as_peek_while_hidden() {
        // y=1570 is in the 48px the bar occupies when shown, but 30px above
        // the edge — that's a tiled window, not the peek zone.
        let now = Instant::now();
        let (action, since) = decide(
            &view(Pt { x: 100, y: 1570 }, vec![hidden_tray()], false),
            None,
            now,
        );
        assert_eq!(action, Action::None);
        assert_eq!(since, None);
    }

    #[test]
    fn leave_hides_only_after_the_delay() {
        let t0 = Instant::now();
        let v = view(Pt { x: 100, y: 800 }, vec![shown_tray()], false);
        let (action, since) = decide(&v, None, t0);
        assert_eq!(action, Action::None);
        assert_eq!(since, Some(t0));

        let (action, _) = decide(&v, Some(t0), t0 + Duration::from_millis(399));
        assert_eq!(action, Action::None);

        let (action, _) = decide(&v, Some(t0), t0 + Duration::from_millis(400));
        assert_eq!(action, Action::Hide);
    }

    #[test]
    fn start_or_flyout_holds_a_shown_bar() {
        let now = Instant::now();
        let (action, since) = decide(
            &view(Pt { x: 100, y: 800 }, vec![shown_tray()], true),
            Some(now),
            now + Duration::from_secs(5),
        );
        assert_eq!(action, Action::None);
        assert_eq!(since, None);
    }

    #[test]
    fn start_re_shows_a_tray_that_lagged_the_first_show() {
        // Live 2026-09-04: the primary was missing from one enumeration while
        // Start opened; the two secondaries were up. `any` would have stopped
        // here and left the primary hidden for the whole Start session.
        let now = Instant::now();
        let (action, since) = decide(
            &view(
                Pt { x: 100, y: 800 },
                vec![shown_tray(), hidden_tray()],
                true,
            ),
            None,
            now,
        );
        assert_eq!(action, Action::Show);
        assert_eq!(since, None);
    }

    #[test]
    fn explorer_collapsed_line_is_left_alone() {
        let now = Instant::now();
        let (action, since) = decide(
            &view(Pt { x: 100, y: 800 }, vec![collapsed_tray()], false),
            None,
            now,
        );
        assert_eq!(action, Action::None);
        assert_eq!(since, None);
    }
}
