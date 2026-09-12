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

use windows_sys::Wdk::System::SystemInformation::NtQuerySystemInformation;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, INVALID_HANDLE_VALUE, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW, WaitForSingleObject,
};

const SYNCHRONIZE: u32 = 0x0010_0000;
const SYSTEM_BASIC_PROCESS_INFORMATION: i32 = 252;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004u32 as i32;
const STATUS_INVALID_INFO_CLASS: i32 = 0xC000_0003u32 as i32;
const MAX_BASIC_PROCESS_BUFFER: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SequenceProcess {
    pub pid: u32,
    pub parent_pid: u32,
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProcessInfo {
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

/// Every process as PID and lowercased executable basename for the existing
/// supervisor adoption lookup.
fn image_snapshot() -> io::Result<Vec<(u32, String)>> {
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

/// Full image path from a process proven live across the query.
pub(crate) fn process_info(pid: u32) -> Option<ProcessInfo> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid);
        if handle.is_null() {
            return None;
        }
        if WaitForSingleObject(handle, 0) != WAIT_TIMEOUT {
            CloseHandle(handle);
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let got_path = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        let still_live = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;
        CloseHandle(handle);
        (got_path != 0 && still_live).then(|| ProcessInfo {
            image_path: String::from_utf16_lossy(&buf[..len as usize]),
        })
    }
}

/// Windows 11 26100.4770+ process identities. This reads only PID, PPID and the
/// unique sequence number; image-name pointers in the returned buffer are not
/// dereferenced. There is no creation-time or Toolhelp ancestry fallback.
pub(crate) fn sequence_process_snapshot(max_entries: usize) -> io::Result<Vec<SequenceProcess>> {
    let mut bytes = 64 * 1024usize;
    loop {
        if bytes > MAX_BASIC_PROCESS_BUFFER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot exceeded bounded buffer",
            ));
        }
        let words = bytes.div_ceil(std::mem::size_of::<u64>());
        let mut buffer = vec![0u64; words];
        let buffer_bytes = buffer.len() * std::mem::size_of::<u64>();
        let mut returned = 0u32;
        let status = unsafe {
            NtQuerySystemInformation(
                SYSTEM_BASIC_PROCESS_INFORMATION,
                buffer.as_mut_ptr().cast(),
                u32::try_from(buffer_bytes).unwrap_or(u32::MAX),
                &mut returned,
            )
        };
        if status == STATUS_INVALID_INFO_CLASS {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "SystemBasicProcessInformation is unavailable",
            ));
        }
        if status == STATUS_INFO_LENGTH_MISMATCH {
            bytes = usize::try_from(returned)
                .unwrap_or(MAX_BASIC_PROCESS_BUFFER + 1)
                .max(bytes.saturating_mul(2));
            continue;
        }
        if status < 0 {
            return Err(io::Error::other(format!(
                "NtQuerySystemInformation failed with NTSTATUS {status:#x}"
            )));
        }
        let used = usize::try_from(returned).unwrap_or(buffer_bytes);
        if used > buffer_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot reported a truncated buffer",
            ));
        }
        return parse_sequence_processes(buffer.as_ptr().cast(), used, max_entries);
    }
}

/// Fixed `SYSTEM_BASICPROCESS_INFORMATION` record. `image_name.buffer` is
/// validated as an opaque pointer and is never dereferenced.
#[repr(C)]
#[derive(Clone, Copy)]
struct BasicProcessInformation {
    next_entry_offset: u32,
    unique_process_id: *mut std::ffi::c_void,
    inherited_from_unique_process_id: *mut std::ffi::c_void,
    sequence_number: u64,
    image_name: UnicodeStringFields,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct UnicodeStringFields {
    length: u16,
    maximum_length: u16,
    buffer: *const u16,
}

fn parse_sequence_processes(
    buffer: *const u8,
    used: usize,
    max_entries: usize,
) -> io::Result<Vec<SequenceProcess>> {
    let header_size = std::mem::size_of::<BasicProcessInformation>();
    let header_alignment = std::mem::align_of::<BasicProcessInformation>();
    if !(buffer as usize).is_multiple_of(header_alignment) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "basic process snapshot buffer was misaligned",
        ));
    }
    let mut offset = 0usize;
    let mut out = Vec::new();
    loop {
        if !offset.is_multiple_of(header_alignment) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot entry was misaligned",
            ));
        }
        if offset.checked_add(header_size).is_none_or(|end| end > used) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot contained a truncated entry",
            ));
        }
        if out.len() == max_entries {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot exceeded the entry cap",
            ));
        }
        let fields = unsafe {
            std::ptr::read_unaligned(buffer.add(offset).cast::<BasicProcessInformation>())
        };
        if fields.image_name.length > fields.image_name.maximum_length
            || fields.image_name.length % 2 != 0
            || fields.image_name.maximum_length % 2 != 0
            || (fields.image_name.maximum_length != 0 && fields.image_name.buffer.is_null())
            || (!fields.image_name.buffer.is_null()
                && !(fields.image_name.buffer as usize).is_multiple_of(std::mem::align_of::<u16>()))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot contained an invalid image-name descriptor",
            ));
        }
        let pid = u32::try_from(fields.unique_process_id as usize)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "PID overflow"))?;
        let parent_pid = u32::try_from(fields.inherited_from_unique_process_id as usize)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "PPID overflow"))?;
        out.push(SequenceProcess {
            pid,
            parent_pid,
            sequence: fields.sequence_number,
        });
        if fields.next_entry_offset == 0 {
            return Ok(out);
        }
        let next = usize::try_from(fields.next_entry_offset).unwrap_or(usize::MAX);
        if next < header_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot contained an invalid next offset",
            ));
        }
        if !next.is_multiple_of(header_alignment) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "basic process snapshot contained a misaligned next offset",
            ));
        }
        offset = offset.checked_add(next).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "snapshot offset overflow")
        })?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    fn identity_fields(next_entry_offset: u32) -> BasicProcessInformation {
        BasicProcessInformation {
            next_entry_offset,
            unique_process_id: 10usize as _,
            inherited_from_unique_process_id: 1usize as _,
            sequence_number: 100,
            image_name: UnicodeStringFields {
                length: 0,
                maximum_length: 0,
                buffer: std::ptr::null(),
            },
        }
    }

    fn encoded_identities(fields: &[BasicProcessInformation]) -> Vec<u64> {
        let mut words =
            vec![0u64; std::mem::size_of_val(fields).div_ceil(std::mem::size_of::<u64>())];
        for (index, fields) in fields.iter().copied().enumerate() {
            unsafe {
                std::ptr::write_unaligned(
                    words
                        .as_mut_ptr()
                        .cast::<u8>()
                        .add(index * std::mem::size_of::<BasicProcessInformation>())
                        .cast::<BasicProcessInformation>(),
                    fields,
                );
            }
        }
        words
    }

    #[test]
    fn finds_our_own_process_by_its_full_path() {
        let me = std::env::current_exe().unwrap();
        let pids = pids_for_image_path(me.to_str().unwrap()).unwrap();
        assert!(pids.contains(&std::process::id()));
    }

    #[test]
    fn forward_slashes_match_the_same_image() {
        let me = std::env::current_exe().unwrap();
        let forward = me.to_str().unwrap().replace('\\', "/");
        assert!(
            pids_for_image_path(&forward)
                .unwrap()
                .contains(&std::process::id())
        );
    }

    #[test]
    fn a_basename_collision_with_a_different_path_does_not_match() {
        let me = std::env::current_exe().unwrap();
        let name = me.file_name().unwrap().to_string_lossy();
        let elsewhere = format!("C:/winmakase-no-such-dir-1c9/{name}");
        assert!(pids_for_image_path(&elsewhere).unwrap().is_empty());
    }

    #[test]
    fn a_path_without_a_file_name_is_rejected() {
        assert!(pids_for_image_path("C:/").is_err());
    }

    #[test]
    fn sequence_snapshot_contains_our_live_unique_identity() {
        let snapshot = match sequence_process_snapshot(16_384) {
            Ok(snapshot) => snapshot,
            Err(error) if error.kind() == io::ErrorKind::Unsupported => return,
            Err(error) => panic!("native sequence snapshot failed: {error}"),
        };
        let me = snapshot
            .iter()
            .find(|process| process.pid == std::process::id())
            .unwrap();
        assert_ne!(me.sequence, 0);
    }

    #[test]
    fn sequence_parser_rejects_an_incomplete_terminal_entry() {
        let header_size = std::mem::size_of::<BasicProcessInformation>();
        let words = encoded_identities(&[
            identity_fields(u32::try_from(header_size).unwrap()),
            identity_fields(0),
        ]);
        let used = header_size * 2 - 1;
        assert!(parse_sequence_processes(words.as_ptr().cast(), used, 1).is_err());
    }

    #[test]
    fn sequence_parser_rejects_an_offset_inside_the_fixed_record() {
        let header_size = std::mem::size_of::<BasicProcessInformation>();
        let words = encoded_identities(&[identity_fields(
            u32::try_from(header_size - std::mem::align_of::<BasicProcessInformation>()).unwrap(),
        )]);
        assert!(parse_sequence_processes(words.as_ptr().cast(), header_size, 1).is_err());
    }

    #[test]
    fn sequence_parser_rejects_a_misaligned_next_offset() {
        let header_size = std::mem::size_of::<BasicProcessInformation>();
        let words = encoded_identities(&[identity_fields(u32::try_from(header_size + 1).unwrap())]);
        assert!(parse_sequence_processes(words.as_ptr().cast(), header_size, 1).is_err());
    }

    #[test]
    fn sequence_parser_validates_the_opaque_image_name_descriptor() {
        let mut fields = identity_fields(0);
        fields.image_name.length = 2;
        let words = encoded_identities(&[fields]);
        let used = words.len() * std::mem::size_of::<u64>();
        assert!(parse_sequence_processes(words.as_ptr().cast(), used, 1).is_err());
    }

    #[test]
    fn sequence_parser_rejects_an_entry_over_the_cap() {
        let words = encoded_identities(&[identity_fields(0)]);
        let used = words.len() * std::mem::size_of::<u64>();
        assert!(parse_sequence_processes(words.as_ptr().cast(), used, 0).is_err());
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn sequence_parser_rejects_a_pid_that_does_not_fit_win32() {
        let mut fields = identity_fields(0);
        fields.unique_process_id = ((u32::MAX as usize) + 1) as _;
        let words = encoded_identities(&[fields]);
        let used = words.len() * std::mem::size_of::<u64>();
        assert!(parse_sequence_processes(words.as_ptr().cast(), used, 1).is_err());
    }

    #[test]
    fn process_info_rejects_an_exited_but_held_process() {
        let mut child = Command::new("cmd")
            .args(["/d", "/c", "exit", "0"])
            .creation_flags(0x0800_0000)
            .spawn()
            .unwrap();
        let pid = child.id();
        child.wait().unwrap();
        assert!(process_info(pid).is_none());
        drop(child);
    }
}
