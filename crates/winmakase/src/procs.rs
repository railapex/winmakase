//! Finding a component's process when we did not spawn it.
//!
//! Two callers: adopt-first (a component already running when the supervisor
//! arrives — taking over a live desktop must not bounce it), and task-hosted
//! start (`schtasks /run` hands back no pid, so the process has to be found).
//!
//! Matching is on the **full image path**, not the basename. The basename is
//! how the snapshot filters cheaply, but the path is what makes the answer
//! safe: `winmakase.exe` test stubs, or some unrelated kanata, must never be
//! adopted because they share a filename.

use std::io;
use std::path::Path;

use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ProcessIdentity {
    pub pid: u32,
    pub creation_time: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProcessParentEntry {
    pub pid: u32,
    pub parent_pid: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProcessInfo {
    pub identity: ProcessIdentity,
    pub image_path: String,
}

/// Pids whose executable is exactly `image_path` (case-insensitive, slashes
/// normalized). Pids we cannot open or query are skipped, not guessed at.
pub fn pids_for_image_path(image_path: &str) -> io::Result<Vec<u32>> {
    let wanted_base = match Path::new(image_path).file_name() {
        Some(n) => n.to_string_lossy().to_lowercase(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{image_path}: no file name to match on"),
            ));
        }
    };
    let wanted_path = normalize(image_path);

    let mut out = Vec::new();
    for (pid, image_base) in image_snapshot()? {
        if image_base != wanted_base {
            continue;
        }
        if let Some(full) = full_image_path(pid)
            && normalize(&full) == wanted_path
        {
            out.push(pid);
        }
    }
    Ok(out)
}

fn normalize(p: &str) -> String {
    p.replace('/', "\\").to_lowercase()
}

/// The full Win32 path of a PID's executable, when the process lets us ask.
fn full_image_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let got = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);
        (got != 0).then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// Every process as PID and parent PID only.
///
/// Callers must treat PID/PPID as a point-in-time hint. The ancestry prototype
/// pairs it with separately queried creation identities before retaining an
/// edge. This function intentionally does not query or expose global paths.
pub(crate) fn process_parent_snapshot() -> io::Result<Vec<ProcessParentEntry>> {
    let mut out = Vec::new();
    for_each_process(|entry| {
        out.push(ProcessParentEntry {
            pid: entry.th32ProcessID,
            parent_pid: entry.th32ParentProcessID,
        });
    })?;
    Ok(out)
}

/// Every process as PID and lowercased executable basename for the existing
/// supervisor adoption lookup.
fn image_snapshot() -> io::Result<Vec<(u32, String)>> {
    let mut out = Vec::new();
    for_each_process(|entry| {
        let len = entry
            .szExeFile
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.szExeFile.len());
        let base = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
        out.push((entry.th32ProcessID, base));
    })?;
    Ok(out)
}

fn for_each_process(mut visit: impl FnMut(&PROCESSENTRY32W)) -> io::Result<()> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return Err(io::Error::other(format!(
                "CreateToolhelp32Snapshot failed (Win32 error {})",
                GetLastError()
            )));
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut ok = Process32FirstW(snap, &mut entry);
        while ok != 0 {
            visit(&entry);
            ok = Process32NextW(snap, &mut entry);
        }
        CloseHandle(snap);
        Ok(())
    }
}

/// Creation identity and full image path from one process handle.
pub(crate) fn process_info(pid: u32) -> Option<ProcessInfo> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }

        let mut creation: FILETIME = std::mem::zeroed();
        let mut exit: FILETIME = std::mem::zeroed();
        let mut kernel: FILETIME = std::mem::zeroed();
        let mut user: FILETIME = std::mem::zeroed();
        let got_times = GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user);
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let got_path = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);
        (got_times != 0 && got_path != 0).then(|| ProcessInfo {
            identity: ProcessIdentity {
                pid,
                creation_time: (u64::from(creation.dwHighDateTime) << 32)
                    | u64::from(creation.dwLowDateTime),
            },
            image_path: String::from_utf16_lossy(&buf[..len as usize]),
        })
    }
}

/// Current creation identity, queried from a newly opened process handle.
pub(crate) fn process_identity(pid: u32) -> Option<ProcessIdentity> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut creation: FILETIME = std::mem::zeroed();
        let mut exit: FILETIME = std::mem::zeroed();
        let mut kernel: FILETIME = std::mem::zeroed();
        let mut user: FILETIME = std::mem::zeroed();
        let got = GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user);
        CloseHandle(handle);
        (got != 0).then(|| ProcessIdentity {
            pid,
            creation_time: (u64::from(creation.dwHighDateTime) << 32)
                | u64::from(creation.dwLowDateTime),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_our_own_process_by_its_full_path() {
        let me = std::env::current_exe().unwrap();
        let pids = pids_for_image_path(me.to_str().unwrap()).unwrap();
        assert!(
            pids.contains(&std::process::id()),
            "our own pid must be found by our own image path; got {pids:?}"
        );
    }

    #[test]
    fn forward_slashes_match_the_same_image() {
        let me = std::env::current_exe().unwrap();
        let forward = me.to_str().unwrap().replace('\\', "/");
        let pids = pids_for_image_path(&forward).unwrap();
        assert!(pids.contains(&std::process::id()));
    }

    #[test]
    fn a_basename_collision_with_a_different_path_does_not_match() {
        let me = std::env::current_exe().unwrap();
        let name = me.file_name().unwrap().to_string_lossy();
        let elsewhere = format!("C:/winmakase-no-such-dir-1c9/{name}");
        let pids = pids_for_image_path(&elsewhere).unwrap();
        assert!(
            pids.is_empty(),
            "same basename under a different directory must not match: {pids:?}"
        );
    }

    #[test]
    fn a_path_without_a_file_name_is_rejected() {
        assert!(pids_for_image_path("C:/").is_err());
    }
}
