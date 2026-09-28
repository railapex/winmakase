//! Caps Lock remaps that live outside Winmakase.
//!
//! Two reasons to look. kanata on top of another Caps remap is the stuck-key
//! landmine (spike, 2026-08-28: PowerToys Keyboard Manager and kanata both on
//! Caps latched CapsLock with no escape), so the supervisor never starts
//! kanata while one exists. And `input_mode = "f13"` depends on a remap that
//! sends F13, so the supervisor reports whether it can see one.
//!
//! Two remaps are readable: the registry `Scancode Map`, which Windows applies
//! at boot below every hook, and the active PowerToys Keyboard Manager
//! profile. Both are read as configured, not as currently in effect: a
//! Scancode Map written since the last boot is not active yet, and Keyboard
//! Manager acts only while PowerToys runs. Treating configured as present is
//! the safe direction for the kanata guard, because PowerToys often starts
//! after the supervisor at logon. A firmware remap is invisible here.

use std::fmt;
use std::path::Path;

/// Scancodes and virtual-key codes for the two keys involved.
pub const SC_CAPS: u16 = 0x003A;
pub const SC_F13: u16 = 0x0064;
pub const VK_CAPITAL: &str = "20";
pub const VK_F13: &str = "124";
/// Keyboard Manager's target for a Caps key that types text.
pub const KBM_TEXT: &str = "text";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemapSource {
    ScancodeMap,
    KeyboardManager,
}

/// One configured remap of the Caps key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsRemap {
    pub source: RemapSource,
    /// True when Caps is sent as F13, the key `input_mode = "f13"` binds.
    pub to_f13: bool,
    /// What Caps becomes, in the source's own notation.
    pub target: String,
}

impl fmt::Display for CapsRemap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.source {
            RemapSource::ScancodeMap => write!(
                f,
                "registry Scancode Map sends Caps as scancode {} (active from the boot after it was written)",
                self.target
            ),
            RemapSource::KeyboardManager => write!(
                f,
                "PowerToys Keyboard Manager sends Caps as {} (while PowerToys runs)",
                self.target
            ),
        }
    }
}

/// Every Caps remap configured on this machine. Unreadable sources count as
/// absent; the caller reports what it found, not why a source was silent.
pub fn detect() -> Vec<CapsRemap> {
    let mut found = Vec::new();
    if let Some(bytes) = read_scancode_map()
        && let Ok(entries) = parse_scancode_map(&bytes)
        && let Some(to) = caps_target(&entries)
    {
        found.push(CapsRemap {
            source: RemapSource::ScancodeMap,
            to_f13: to == SC_F13,
            target: format!("0x{to:04X}"),
        });
    }
    if let Some(dir) = std::env::var_os("LOCALAPPDATA")
        && let Some(target) =
            keyboard_manager_caps_target(&Path::new(&dir).join("Microsoft/PowerToys"))
    {
        found.push(CapsRemap {
            source: RemapSource::KeyboardManager,
            to_f13: target == VK_F13,
            target: keyboard_manager_label(&target),
        });
    }
    found
}

/// `(to, from)` scancode pairs from a `Scancode Map` value: an 8-byte zero
/// header, a u32 count that includes the terminating null entry, then that
/// many 4-byte little-endian entries of target scancode then source scancode.
pub fn parse_scancode_map(bytes: &[u8]) -> Result<Vec<(u16, u16)>, String> {
    if bytes.len() < 12 {
        return Err(format!("{} bytes is shorter than the header", bytes.len()));
    }
    let count = u32::from_le_bytes(bytes[8..12].try_into().expect("four bytes")) as usize;
    let body = &bytes[12..];
    if count == 0 || body.len() < count * 4 {
        return Err(format!(
            "count {count} does not fit {} entry bytes",
            body.len()
        ));
    }
    Ok(body
        .as_chunks::<4>()
        .0
        .iter()
        .take(count)
        .map(|entry| {
            (
                u16::from_le_bytes([entry[0], entry[1]]),
                u16::from_le_bytes([entry[2], entry[3]]),
            )
        })
        .take_while(|&(to, from)| (to, from) != (0, 0))
        .collect())
}

/// What Caps is mapped to, if the map touches it. A target of 0 disables
/// the key, which is still a remap.
pub fn caps_target(entries: &[(u16, u16)]) -> Option<u16> {
    entries
        .iter()
        .find(|&&(_, from)| from == SC_CAPS)
        .map(|&(to, _)| to)
}

/// Keyboard Manager's target for Caps (`"124"`, `"164;32"` for a shortcut,
/// or [`KBM_TEXT`]), when the module is enabled and its active profile
/// remaps Caps. A missing file or field falls back to PowerToys' own default
/// (module on, profile `default`), because a missed remap is the unsafe
/// direction for the kanata guard.
pub fn keyboard_manager_caps_target(powertoys: &Path) -> Option<String> {
    let general = read_json(&powertoys.join("settings.json"))?;
    // PowerToys enables Keyboard Manager by default; only an explicit false
    // turns it off.
    if general["enabled"]["Keyboard Manager"] == serde_json::Value::Bool(false) {
        return None;
    }
    let kbm = powertoys.join("Keyboard Manager");
    let profile = read_json(&kbm.join("settings.json"))
        .and_then(|s| {
            s["properties"]["activeConfiguration"]["value"]
                .as_str()
                .map(str::to_string)
        })
        .unwrap_or_else(|| "default".into());
    // The name becomes a file name; anything beyond these could leave the
    // directory (`..`, a separator, or a drive-relative `C:`).
    if profile.is_empty()
        || !profile
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-'))
    {
        return None;
    }
    let remaps = read_json(&kbm.join(format!("{profile}.json")))?;
    let caps_entry = |section: &str| {
        remaps[section]["inProcess"]
            .as_array()?
            .iter()
            .find(|entry| entry["originalKeys"].as_str() == Some(VK_CAPITAL))
            .cloned()
    };
    if let Some(entry) = caps_entry("remapKeys") {
        return entry["newRemapKeys"].as_str().map(str::to_string);
    }
    caps_entry("remapKeysToText").map(|_| KBM_TEXT.to_string())
}

fn keyboard_manager_label(target: &str) -> String {
    match target {
        VK_F13 => "F13 (VK 124)".into(),
        KBM_TEXT => "typed text".into(),
        other => format!("VK {other}"),
    }
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(path).ok()?;
    // PowerToys writes UTF-8, sometimes with a BOM.
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

fn read_scancode_map() -> Option<Vec<u8>> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY_LOCAL_MACHINE, RRF_RT_REG_BINARY, RegGetValueW,
    };

    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let subkey = wide(r"SYSTEM\CurrentControlSet\Control\Keyboard Layout");
    let value = wide("Scancode Map");
    let mut len: u32 = 0;
    // SAFETY: both strings are NUL-terminated and outlive the calls; the
    // first call only reports the size, the second writes at most `len`
    // bytes into a buffer of exactly that size.
    unsafe {
        let status = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_BINARY,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut len,
        );
        if status != ERROR_SUCCESS || len == 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        let status = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_BINARY,
            std::ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut len,
        );
        if status != ERROR_SUCCESS {
            return None;
        }
        buf.truncate(len as usize);
        Some(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    /// Caps→F13 and ScrLk→Caps, as the desk's planned map writes them.
    const DESK_MAP: [u8; 24] = [
        0, 0, 0, 0, 0, 0, 0, 0, // version, flags
        3, 0, 0, 0, // two entries plus the null terminator
        0x64, 0x00, 0x3A, 0x00, // F13 <- Caps
        0x3A, 0x00, 0x46, 0x00, // Caps <- ScrLk
        0, 0, 0, 0,
    ];

    #[test]
    fn a_scancode_map_yields_its_caps_target() {
        let entries = parse_scancode_map(&DESK_MAP).unwrap();
        assert_eq!(entries, vec![(SC_F13, SC_CAPS), (SC_CAPS, 0x0046)]);
        assert_eq!(caps_target(&entries), Some(SC_F13));
    }

    #[test]
    fn a_map_that_only_targets_caps_does_not_remap_it() {
        // ScrLk→Caps alone makes another key a Caps; Caps itself is untouched.
        let map = [
            0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0x3A, 0, 0x46, 0, 0, 0, 0, 0,
        ];
        assert_eq!(caps_target(&parse_scancode_map(&map).unwrap()), None);
    }

    #[test]
    fn malformed_scancode_maps_are_rejected() {
        assert!(parse_scancode_map(&[0; 8]).is_err());
        // Claims three entries, carries one.
        let short = [0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0x64, 0, 0x3A, 0];
        assert!(parse_scancode_map(&short).is_err());
    }

    fn powertoys(dir: &Path, enabled: Option<bool>, remap_keys: &str) {
        let kbm = dir.join("Keyboard Manager");
        std::fs::create_dir_all(&kbm).unwrap();
        let enabled = match enabled {
            Some(on) => format!(r#"{{"enabled":{{"Keyboard Manager":{on}}}}}"#),
            None => "{}".into(),
        };
        std::fs::write(dir.join("settings.json"), enabled).unwrap();
        std::fs::write(
            kbm.join("settings.json"),
            r#"{"properties":{"activeConfiguration":{"value":"default"}},"name":"Keyboard Manager"}"#,
        )
        .unwrap();
        std::fs::write(
            kbm.join("default.json"),
            format!("\u{feff}{{\"remapKeys\":{{\"inProcess\":[{remap_keys}]}}}}"),
        )
        .unwrap();
    }

    #[test]
    fn keyboard_manager_caps_to_f13_is_found() {
        let dir = TempDir::new("kbm-f13");
        powertoys(
            dir.path(),
            Some(true),
            r#"{"originalKeys":"145","newRemapKeys":"20"},{"originalKeys":"20","newRemapKeys":"124"}"#,
        );
        assert_eq!(
            keyboard_manager_caps_target(dir.path()).as_deref(),
            Some(VK_F13)
        );
    }

    #[test]
    fn keyboard_manager_counts_only_when_enabled_and_remapping_caps() {
        let dir = TempDir::new("kbm-off");
        powertoys(
            dir.path(),
            Some(false),
            r#"{"originalKeys":"20","newRemapKeys":"164;32"}"#,
        );
        assert_eq!(keyboard_manager_caps_target(dir.path()), None);

        let dir = TempDir::new("kbm-default-on");
        powertoys(
            dir.path(),
            None,
            r#"{"originalKeys":"20","newRemapKeys":"164;32"}"#,
        );
        assert_eq!(
            keyboard_manager_caps_target(dir.path()).as_deref(),
            Some("164;32"),
            "an absent enabled flag is PowerToys' default: on"
        );

        let dir = TempDir::new("kbm-scrlk-only");
        powertoys(
            dir.path(),
            Some(true),
            r#"{"originalKeys":"145","newRemapKeys":"20"}"#,
        );
        assert_eq!(keyboard_manager_caps_target(dir.path()), None);
    }

    #[test]
    fn no_powertoys_means_no_keyboard_manager_remap() {
        let dir = TempDir::new("kbm-absent");
        assert_eq!(keyboard_manager_caps_target(dir.path()), None);
    }

    #[test]
    fn a_missing_module_settings_file_means_the_default_profile() {
        let dir = TempDir::new("kbm-no-settings");
        powertoys(
            dir.path(),
            Some(true),
            r#"{"originalKeys":"20","newRemapKeys":"124"}"#,
        );
        std::fs::remove_file(dir.path().join("Keyboard Manager/settings.json")).unwrap();
        assert_eq!(
            keyboard_manager_caps_target(dir.path()).as_deref(),
            Some(VK_F13)
        );
    }

    #[test]
    fn a_profile_name_that_could_leave_the_directory_is_refused() {
        for name in ["C:evil", "../up", "a.b", r"sub\x", ""] {
            let dir = TempDir::new("kbm-profile-name");
            powertoys(
                dir.path(),
                Some(true),
                r#"{"originalKeys":"20","newRemapKeys":"124"}"#,
            );
            std::fs::write(
                dir.path().join("Keyboard Manager/settings.json"),
                serde_json::json!({"properties":{"activeConfiguration":{"value":name}}})
                    .to_string(),
            )
            .unwrap();
            assert_eq!(keyboard_manager_caps_target(dir.path()), None, "{name:?}");
        }
    }

    #[test]
    fn caps_typing_text_is_a_remap() {
        let dir = TempDir::new("kbm-text");
        powertoys(dir.path(), Some(true), "");
        std::fs::write(
            dir.path().join("Keyboard Manager/default.json"),
            r#"{"remapKeys":{"inProcess":[]},"remapKeysToText":{"inProcess":[{"originalKeys":"20","unicodeText":"hi"}]}}"#,
        )
        .unwrap();
        assert_eq!(
            keyboard_manager_caps_target(dir.path()).as_deref(),
            Some(KBM_TEXT)
        );
    }
}
