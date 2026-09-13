//! Observed open-or-focus adapter for PowerToys Run.
//!
//! PowerToys exposes a toggle event, not an explicit show command. This module
//! serializes Winmakase callers, proves the launcher's current state, and
//! signals at most once. An uncertain signal is recorded until its process
//! generation changes or the launcher reaches a confirmed state.

use std::fmt;
use std::io;
use std::mem::{size_of, zeroed};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_FILE_NOT_FOUND, FILETIME, GetLastError, HANDLE, HWND, LPARAM,
    WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows_sys::Win32::Security::{
    GetTokenInformation, TOKEN_QUERY, TOKEN_STATISTICS, TokenStatistics,
};
use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows_sys::Win32::System::Threading::{
    CreateMutexW, EVENT_MODIFY_STATE, GetCurrentProcess, GetCurrentProcessId, GetProcessTimes,
    OpenEventW, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW, ReleaseMutex, SetEvent, WaitForSingleObject,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, EnumWindows, GetForegroundWindow, GetWindowTextW,
    GetWindowThreadProcessId, IsWindow, IsWindowVisible, SetForegroundWindow,
};

use crate::procs;

const INVOKE_EVENT: &str = r"Local\PowerToysRunInvokeEvent-30f26ad7-d36d-4c0e-ab02-68bb5ff3c4ab";
const LAUNCHER_TITLE: &str = "PowerToys.PowerLauncher";
const LAUNCHER_EXE: &str = "PowerToys.PowerLauncher.exe";
const SYNCHRONIZE: u32 = 0x0010_0000;

#[derive(Debug, Clone, Copy)]
pub struct OpenOptions {
    pub mutex_timeout: Duration,
    pub activation_timeout: Duration,
    pub poll_interval: Duration,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            mutex_timeout: Duration::from_millis(500),
            activation_timeout: Duration::from_millis(1_500),
            poll_interval: Duration::from_millis(10),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenOutcome {
    AlreadyFocused,
    FocusedExisting,
    Opened,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenReport {
    pub outcome: OpenOutcome,
    pub pid: u32,
    pub hwnd: usize,
    pub signalled: bool,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenErrorKind {
    Busy,
    NotRunning,
    NotReady,
    AmbiguousProcess,
    AmbiguousWindow,
    ProcessChanged,
    FocusDenied,
    FocusInterrupted,
    ActivationUncertain,
    ActivationPending,
    Native,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenError {
    pub kind: OpenErrorKind,
    pub message: String,
    pub signalled: bool,
}

impl OpenError {
    fn new(kind: OpenErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            signalled: false,
        }
    }

    fn after_signal(kind: OpenErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            signalled: true,
        }
    }

    fn native(context: &str) -> Self {
        Self::new(
            OpenErrorKind::Native,
            format!("{context}: {}", io::Error::last_os_error()),
        )
    }
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OpenError {}

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

struct OpenGuard(Handle);

impl Drop for OpenGuard {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.0.0);
        }
    }
}

struct RunProcess {
    pid: u32,
    created: u64,
    handle: Handle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct UncertainMarker {
    pid: u32,
    created: u64,
    recorded_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObservedAction {
    ReturnFocused,
    Focus,
    Signal,
}

fn decide(visible: bool, foreground: bool) -> ObservedAction {
    if visible && foreground {
        ObservedAction::ReturnFocused
    } else if visible {
        ObservedAction::Focus
    } else {
        ObservedAction::Signal
    }
}

/// Opens PowerToys Run or focuses the already-visible launcher.
pub fn open_run(options: OpenOptions) -> Result<OpenReport, OpenError> {
    let started = Instant::now();
    let original_foreground = unsafe { GetForegroundWindow() };
    let mutex_deadline = started + options.mutex_timeout;
    let _guard = acquire(mutex_deadline)?;
    let process = discover_process()?;
    let deadline = Instant::now() + options.activation_timeout;

    let hwnd = wait_for_window(&process, deadline, options.poll_interval)?;
    let visible = window_visible(hwnd, process.pid)?;
    let foreground = unsafe { GetForegroundWindow() } == hwnd;

    match decide(visible, foreground) {
        ObservedAction::ReturnFocused => {
            clear_uncertain(&process);
            Ok(report(
                OpenOutcome::AlreadyFocused,
                &process,
                hwnd,
                false,
                started,
            ))
        }
        ObservedAction::Focus => focus_existing(
            &process,
            hwnd,
            original_foreground,
            deadline,
            options,
            started,
        ),
        ObservedAction::Signal => signal_hidden(
            &process,
            hwnd,
            original_foreground,
            deadline,
            options,
            started,
        ),
    }
}

fn focus_existing(
    process: &RunProcess,
    hwnd: HWND,
    original_foreground: HWND,
    deadline: Instant,
    options: OpenOptions,
    started: Instant,
) -> Result<OpenReport, OpenError> {
    revalidate(process, hwnd)?;
    let foreground = unsafe { GetForegroundWindow() };
    if foreground_changed(original_foreground, foreground, hwnd) {
        return Err(OpenError::new(
            OpenErrorKind::FocusInterrupted,
            "foreground changed before PowerToys Run could be focused",
        ));
    }
    unsafe {
        SetForegroundWindow(hwnd);
    }
    loop {
        ensure_alive(process)?;
        let foreground = unsafe { GetForegroundWindow() };
        if foreground == hwnd && window_visible(hwnd, process.pid)? {
            clear_uncertain(process);
            return Ok(report(
                OpenOutcome::FocusedExisting,
                process,
                hwnd,
                false,
                started,
            ));
        }
        if foreground != original_foreground && foreground != hwnd {
            return Err(OpenError::new(
                OpenErrorKind::FocusInterrupted,
                "foreground changed while focusing PowerToys Run",
            ));
        }
        if Instant::now() >= deadline {
            return Err(OpenError::new(
                OpenErrorKind::FocusDenied,
                "PowerToys Run stayed visible but Windows did not grant foreground focus",
            ));
        }
        sleep_until(deadline, options.poll_interval);
    }
}

fn signal_hidden(
    process: &RunProcess,
    hwnd: HWND,
    original_foreground: HWND,
    deadline: Instant,
    options: OpenOptions,
    started: Instant,
) -> Result<OpenReport, OpenError> {
    if uncertain_is_live(process) {
        return Err(OpenError::new(
            OpenErrorKind::ActivationPending,
            "a previous PowerToys Run activation is still uncertain; refusing a second toggle",
        ));
    }
    let event = wait_for_event(process, deadline, options.poll_interval)?;
    revalidate(process, hwnd)?;
    if window_visible(hwnd, process.pid)? {
        return focus_existing(
            process,
            hwnd,
            original_foreground,
            deadline,
            options,
            started,
        );
    }

    let foreground = unsafe { GetForegroundWindow() };
    if foreground_changed(original_foreground, foreground, hwnd) {
        return Err(OpenError::new(
            OpenErrorKind::FocusInterrupted,
            "foreground changed before PowerToys Run could be signalled",
        ));
    }
    unsafe {
        let _ = AllowSetForegroundWindow(process.pid);
    }
    // Journal before the irreversible edge. Every post-signal early return
    // must leave the next caller unable to queue an opposite toggle while
    // PowerToys may still be handling this one.
    write_uncertain(process)?;
    if unsafe { SetEvent(event.0) } == 0 {
        clear_uncertain(process);
        return Err(OpenError::native(
            "could not signal the PowerToys Run invoke event",
        ));
    }

    let mut focus_attempted = false;
    loop {
        ensure_alive_after_signal(process)?;
        let visible = window_visible(hwnd, process.pid)
            .map_err(|error| mark_signalled(error, OpenErrorKind::ProcessChanged))?;
        let foreground = unsafe { GetForegroundWindow() };
        if visible && foreground == hwnd {
            clear_uncertain(process);
            return Ok(report(OpenOutcome::Opened, process, hwnd, true, started));
        }
        if visible && !focus_attempted && foreground == original_foreground {
            unsafe {
                SetForegroundWindow(hwnd);
            }
            focus_attempted = true;
        } else if foreground != original_foreground && foreground != hwnd {
            return Err(OpenError::after_signal(
                OpenErrorKind::FocusInterrupted,
                "foreground changed while PowerToys Run was opening",
            ));
        }
        if Instant::now() >= deadline {
            return Err(OpenError::after_signal(
                OpenErrorKind::ActivationUncertain,
                "PowerToys Run did not become visible and focused before the activation deadline",
            ));
        }
        sleep_until(deadline, options.poll_interval);
    }
}

fn report(
    outcome: OpenOutcome,
    process: &RunProcess,
    hwnd: HWND,
    signalled: bool,
    started: Instant,
) -> OpenReport {
    OpenReport {
        outcome,
        pid: process.pid,
        hwnd: hwnd as usize,
        signalled,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

fn expected_image_paths() -> Result<Vec<String>, OpenError> {
    let mut roots = Vec::new();
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        roots.push(PathBuf::from(program_files));
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        roots.push(PathBuf::from(local_app_data));
    }
    if roots.is_empty() {
        return Err(OpenError::new(
            OpenErrorKind::NotRunning,
            "ProgramFiles and LOCALAPPDATA are unavailable; cannot locate a supported PowerToys installation",
        ));
    }
    let mut paths = roots
        .into_iter()
        .flat_map(|root| {
            let install = root.join("PowerToys");
            [
                install.join(LAUNCHER_EXE),
                install.join("WinUI3Apps").join(LAUNCHER_EXE),
            ]
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| normalize_path(path));
    paths.dedup_by(|left, right| normalize_path(left) == normalize_path(right));
    Ok(paths)
}

fn discover_process() -> Result<RunProcess, OpenError> {
    let expected = expected_image_paths()?;
    let current_pid = unsafe { GetCurrentProcessId() };
    let current_session = session_id(current_pid)?;
    let current_auth = authentication_id(unsafe { GetCurrentProcess() })?;
    let mut matches = Vec::new();
    let mut pids = Vec::new();
    for path in &expected {
        pids.extend(procs::pids_for_image_path(path).map_err(|error| {
            OpenError::new(
                OpenErrorKind::Native,
                format!("could not inspect PowerToys Run processes: {error}"),
            )
        })?);
    }
    pids.sort_unstable();
    pids.dedup();

    for pid in pids {
        let raw = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid) };
        if raw.is_null() {
            continue;
        }
        let handle = Handle(raw);
        let actual = normalize_path(&image_path(&handle)?);
        if !process_alive(&handle)?
            || session_id(pid)? != current_session
            || authentication_id(handle.0)? != current_auth
            || !expected.iter().any(|path| normalize_path(path) == actual)
        {
            continue;
        }
        let created = creation_time(&handle)?;
        if process_alive(&handle)? {
            matches.push(RunProcess {
                pid,
                created,
                handle,
            });
        }
    }

    match matches.len() {
        0 => Err(OpenError::new(
            OpenErrorKind::NotRunning,
            format!(
                "PowerToys Run is not running from a supported path ({})",
                expected.join(" or ")
            ),
        )),
        1 => Ok(matches.remove(0)),
        count => Err(OpenError::new(
            OpenErrorKind::AmbiguousProcess,
            format!("found {count} PowerToys Run processes in this logon session"),
        )),
    }
}

fn wait_for_window(
    process: &RunProcess,
    deadline: Instant,
    poll_interval: Duration,
) -> Result<HWND, OpenError> {
    loop {
        ensure_alive(process)?;
        if let Some(hwnd) = launcher_window(process.pid)? {
            return Ok(hwnd);
        }
        if Instant::now() >= deadline {
            return Err(OpenError::new(
                OpenErrorKind::NotReady,
                "PowerToys Run is running but its launcher window is not ready",
            ));
        }
        sleep_until(deadline, poll_interval);
    }
}

fn wait_for_event(
    process: &RunProcess,
    deadline: Instant,
    poll_interval: Duration,
) -> Result<Handle, OpenError> {
    let name = wide(INVOKE_EVENT);
    loop {
        ensure_alive(process)?;
        let raw = unsafe { OpenEventW(EVENT_MODIFY_STATE, 0, name.as_ptr()) };
        if !raw.is_null() {
            return Ok(Handle(raw));
        }
        let code = unsafe { GetLastError() };
        if code != ERROR_FILE_NOT_FOUND {
            return Err(OpenError::new(
                OpenErrorKind::Native,
                format!(
                    "could not open the PowerToys Run invoke event: {}",
                    io::Error::from_raw_os_error(code as i32)
                ),
            ));
        }
        if Instant::now() >= deadline {
            return Err(OpenError::new(
                OpenErrorKind::NotReady,
                "PowerToys Run is running but its invoke event is not ready",
            ));
        }
        sleep_until(deadline, poll_interval);
    }
}

struct WindowSearch {
    pid: u32,
    matches: Vec<HWND>,
}

unsafe extern "system" fn visit_window(hwnd: HWND, context: LPARAM) -> i32 {
    let search = unsafe { &mut *(context as *mut WindowSearch) };
    let mut pid = 0;
    if unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } == 0 || pid != search.pid {
        return 1;
    }
    let mut title = [0_u16; 256];
    let length = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32) };
    if length > 0 && String::from_utf16_lossy(&title[..length as usize]) == LAUNCHER_TITLE {
        search.matches.push(hwnd);
    }
    1
}

fn launcher_window(pid: u32) -> Result<Option<HWND>, OpenError> {
    let mut search = WindowSearch {
        pid,
        matches: Vec::new(),
    };
    if unsafe {
        EnumWindows(
            Some(visit_window),
            (&mut search as *mut WindowSearch) as LPARAM,
        )
    } == 0
    {
        return Err(OpenError::native(
            "could not enumerate PowerToys Run windows",
        ));
    }
    match search.matches.as_slice() {
        [] => Ok(None),
        [hwnd] => Ok(Some(*hwnd)),
        many => Err(OpenError::new(
            OpenErrorKind::AmbiguousWindow,
            format!("found {} PowerToys Run launcher windows", many.len()),
        )),
    }
}

fn window_visible(hwnd: HWND, expected_pid: u32) -> Result<bool, OpenError> {
    let mut pid = 0;
    if unsafe { IsWindow(hwnd) } == 0
        || unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } == 0
        || pid != expected_pid
    {
        return Err(OpenError::new(
            OpenErrorKind::ProcessChanged,
            "PowerToys Run window identity changed",
        ));
    }
    let mut cloaked = 0_u32;
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED as u32,
            (&mut cloaked as *mut u32).cast(),
            size_of::<u32>() as u32,
        )
    };
    if result < 0 {
        return Err(OpenError::new(
            OpenErrorKind::Native,
            format!("PowerToys Run cloak query failed: {result:#x}"),
        ));
    }
    Ok(unsafe { IsWindowVisible(hwnd) } != 0 && cloaked == 0)
}

fn revalidate(process: &RunProcess, hwnd: HWND) -> Result<(), OpenError> {
    ensure_alive(process)?;
    let _ = window_visible(hwnd, process.pid)?;
    Ok(())
}

fn ensure_alive(process: &RunProcess) -> Result<(), OpenError> {
    if process_alive(&process.handle)? {
        Ok(())
    } else {
        Err(OpenError::new(
            OpenErrorKind::ProcessChanged,
            "PowerToys Run exited during activation",
        ))
    }
}

fn ensure_alive_after_signal(process: &RunProcess) -> Result<(), OpenError> {
    ensure_alive(process).map_err(|error| mark_signalled(error, OpenErrorKind::ProcessChanged))
}

fn mark_signalled(mut error: OpenError, kind: OpenErrorKind) -> OpenError {
    error.kind = kind;
    error.signalled = true;
    error
}

fn acquire(deadline: Instant) -> Result<OpenGuard, OpenError> {
    let (high, low) = authentication_id(unsafe { GetCurrentProcess() })?;
    let name = wide(&format!(
        r"Local\Winmakase.Run.Open.{:08x}.{:08x}",
        high as u32, low
    ));
    let raw = unsafe { CreateMutexW(null(), 0, name.as_ptr()) };
    if raw.is_null() {
        return Err(OpenError::native("could not create the launcher mutex"));
    }
    let handle = Handle(raw);
    let millis = deadline
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(u128::from(u32::MAX - 1)) as u32;
    match unsafe { WaitForSingleObject(handle.0, millis) } {
        WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(OpenGuard(handle)),
        WAIT_TIMEOUT => Err(OpenError::new(
            OpenErrorKind::Busy,
            "another Winmakase launcher request is still running",
        )),
        _ => Err(OpenError::native("could not wait for the launcher mutex")),
    }
}

fn session_id(pid: u32) -> Result<u32, OpenError> {
    let mut session = 0;
    if unsafe { ProcessIdToSessionId(pid, &mut session) } == 0 {
        Err(OpenError::native("could not read the process session"))
    } else {
        Ok(session)
    }
}

fn authentication_id(process: HANDLE) -> Result<(i32, u32), OpenError> {
    let mut token = null_mut();
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
        return Err(OpenError::native("could not open the process token"));
    }
    let token = Handle(token);
    let mut stats: TOKEN_STATISTICS = unsafe { zeroed() };
    let mut returned = 0;
    if unsafe {
        GetTokenInformation(
            token.0,
            TokenStatistics,
            (&mut stats as *mut TOKEN_STATISTICS).cast(),
            size_of::<TOKEN_STATISTICS>() as u32,
            &mut returned,
        )
    } == 0
    {
        return Err(OpenError::native(
            "could not read the process logon identity",
        ));
    }
    Ok((
        stats.AuthenticationId.HighPart,
        stats.AuthenticationId.LowPart,
    ))
}

fn process_alive(handle: &Handle) -> Result<bool, OpenError> {
    match unsafe { WaitForSingleObject(handle.0, 0) } {
        WAIT_TIMEOUT => Ok(true),
        WAIT_OBJECT_0 => Ok(false),
        _ => Err(OpenError::native(
            "could not inspect the PowerToys Run process",
        )),
    }
}

fn creation_time(handle: &Handle) -> Result<u64, OpenError> {
    let (mut created, mut exited, mut kernel, mut user): (FILETIME, FILETIME, FILETIME, FILETIME) =
        unsafe { zeroed() };
    if unsafe { GetProcessTimes(handle.0, &mut created, &mut exited, &mut kernel, &mut user) } == 0
    {
        return Err(OpenError::native(
            "could not read PowerToys Run process identity",
        ));
    }
    Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

fn image_path(handle: &Handle) -> Result<String, OpenError> {
    let mut buffer = vec![0_u16; 32_768];
    let mut length = buffer.len() as u32;
    if unsafe { QueryFullProcessImageNameW(handle.0, 0, buffer.as_mut_ptr(), &mut length) } == 0 {
        return Err(OpenError::native(
            "could not read the PowerToys Run image path",
        ));
    }
    Ok(String::from_utf16_lossy(&buffer[..length as usize]))
}

fn normalize_path(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

fn foreground_changed(original: HWND, current: HWND, target: HWND) -> bool {
    current != original && current != target
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn sleep_until(deadline: Instant, interval: Duration) {
    thread::sleep(interval.min(deadline.saturating_duration_since(Instant::now())));
}

fn marker_path(process: &RunProcess) -> PathBuf {
    std::env::temp_dir().join(format!("winmakase-run-uncertain-{}.json", process.pid))
}

fn now_ms() -> Option<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

fn uncertain_is_live(process: &RunProcess) -> bool {
    let path = marker_path(process);
    let marker = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<UncertainMarker>(&text).ok());
    match marker {
        Some(marker) if marker.pid == process.pid && marker.created == process.created => true,
        _ => {
            let _ = std::fs::remove_file(path);
            false
        }
    }
}

fn write_uncertain(process: &RunProcess) -> Result<(), OpenError> {
    let Some(recorded_ms) = now_ms() else {
        return Err(OpenError::new(
            OpenErrorKind::Native,
            "system clock cannot record a safe launcher activation guard",
        ));
    };
    let marker = UncertainMarker {
        pid: process.pid,
        created: process.created,
        recorded_ms,
    };
    let text = serde_json::to_string(&marker).map_err(|error| {
        OpenError::new(
            OpenErrorKind::Native,
            format!("could not serialize the launcher activation guard: {error}"),
        )
    })?;
    std::fs::write(marker_path(process), text).map_err(|error| {
        OpenError::new(
            OpenErrorKind::Native,
            format!("could not persist the launcher activation guard: {error}"),
        )
    })
}

fn clear_uncertain(process: &RunProcess) {
    let _ = std::fs::remove_file(marker_path(process));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_matrix_never_signals_a_visible_launcher() {
        assert_eq!(decide(true, true), ObservedAction::ReturnFocused);
        assert_eq!(decide(true, false), ObservedAction::Focus);
        assert_eq!(decide(false, false), ObservedAction::Signal);
    }

    #[test]
    fn paths_compare_case_and_separator_insensitively() {
        assert_eq!(
            normalize_path(r"C:\Program Files\PowerToys\PowerToys.PowerLauncher.exe"),
            normalize_path("c:/program files/powertoys/powertoys.powerlauncher.exe")
        );
    }

    #[test]
    fn default_deadlines_are_bounded() {
        let options = OpenOptions::default();
        assert!(options.mutex_timeout <= Duration::from_secs(1));
        assert!(options.activation_timeout <= Duration::from_secs(2));
        assert!(!options.poll_interval.is_zero());
    }

    #[test]
    fn a_new_foreground_destination_cancels_mutation() {
        let original = 1_usize as HWND;
        let target = 2_usize as HWND;
        assert!(!foreground_changed(original, original, target));
        assert!(!foreground_changed(original, target, target));
        assert!(foreground_changed(original, 3_usize as HWND, target));
    }
}
