//! The real keymap/omarchy.toml must parse, validate, and render exactly the
//! golden YAML. Regenerate the golden after a deliberate keymap change with
//! UPDATE_GOLDEN=1 (cargo test -p winmakase-keymap), and review the diff — the
//! golden IS the generated GlazeWM grammar.

use std::fs;
use std::path::PathBuf;

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

/// The golden as LF text. The renderer emits LF, and a Windows checkout with
/// `core.autocrlf=true` writes the committed LF file back as CRLF, so the
/// comparison must not depend on how the file was checked out.
fn read_golden(path: &std::path::Path) -> String {
    fs::read_to_string(path)
        .expect("golden file exists — run with UPDATE_GOLDEN=1 to create it")
        .replace("\r\n", "\n")
}

fn real_file() -> winmakase_keymap::KeymapFile {
    let text = fs::read_to_string(repo_path("keymap/omarchy.toml")).expect("keymap file readable");
    winmakase_keymap::parse(&text).expect("keymap parses")
}

#[test]
fn real_keymap_is_valid() {
    let file = real_file();
    let (_, cov) = winmakase_keymap::check(&file).expect("keymap validates");
    // Coverage acts as a drift alarm: a keymap edit that changes these counts
    // is fine, but must be a conscious edit of this assertion in the same
    // commit — never a surprise.
    // 2026-09-13: launcher uses the shared explicit-open helper (+1 mapped,
    // -1 native).
    // 2026-09-27: lock and capture bound to their Windows actions for the F13
    // super key (+2 mapped, -2 native); wm-pause added (+1 mapped, +1 chord).
    assert_eq!(
        (cov.mapped, cov.gap, cov.native, cov.app, cov.omitted),
        (80, 17, 15, 2, 30),
        "entry counts changed: {cov:?}"
    );
    assert_eq!(
        (cov.chords_mapped, cov.chords_total),
        (80, 227),
        "chord counts changed: {cov:?}"
    );
}

#[test]
fn f13_render_differs_from_golden_only_in_the_super_key() {
    let file = real_file();
    let (expanded, _) = winmakase_keymap::check(&file).expect("keymap validates");
    let rendered =
        winmakase_keymap::render_glazewm_with_super(&expanded, winmakase_keymap::SuperKey::F13);
    let golden_path = repo_path("crates/winmakase-keymap/tests/golden/glazewm-keybindings.yaml");
    let golden = read_golden(&golden_path);
    assert!(
        !golden.contains("f13"),
        "the golden must not already bind f13"
    );
    assert_eq!(rendered, golden.replace("['rwin+", "['f13+"));
}

#[test]
fn real_keymap_matches_golden() {
    let file = real_file();
    let (expanded, _) = winmakase_keymap::check(&file).expect("keymap validates");
    let rendered = winmakase_keymap::render_glazewm(&expanded);

    let golden_path = repo_path("crates/winmakase-keymap/tests/golden/glazewm-keybindings.yaml");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::create_dir_all(golden_path.parent().expect("has parent")).expect("mkdir golden");
        fs::write(&golden_path, &rendered).expect("write golden");
    }
    let golden = read_golden(&golden_path);
    assert_eq!(rendered, golden, "rendered YAML diverges from golden");
}
