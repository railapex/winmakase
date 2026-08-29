//! Finding a component's process when we did not spawn it.
//!
//! Two callers: adopt-first (a component already running when the supervisor
//! arrives — taking over a live desktop must not bounce it), and task-hosted
//! start (`schtasks /run` hands back no pid, so the process has to be found).
//!
//! Matching is on the **full image path**, not the basename. The basename is
//! how the snapshot filters cheaply, but the path is what makes the answer
//! safe: `winsome.exe` test stubs, or some unrelated kanata, must never be
//! adopted because they share a filename.

use std::io;
use std::path::Path;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};

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
    for (pid, base) in snapshot()? {
        if base != wanted_base {
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

/// Every process as (pid, lowercased exe basename).
fn snapshot() -> io::Result<Vec<(u32, String)>> {
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

        let mut out = Vec::new();
        let mut ok = Process32FirstW(snap, &mut entry);
        while ok != 0 {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let base = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
            out.push((entry.th32ProcessID, base));
            ok = Process32NextW(snap, &mut entry);
        }
        CloseHandle(snap);
        Ok(out)
    }
}

/// The full Win32 path of a pid's executable, when the process lets us ask.
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
        let elsewhere = format!("C:/winsome-no-such-dir-1c9/{name}");
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
