//! The real keymap/omarchy.toml must parse, validate, and render exactly the
//! golden YAML. Regenerate the golden after a deliberate keymap change with
//! UPDATE_GOLDEN=1 (cargo test -p winsome-keymap), and review the diff — the
//! golden IS the generated GlazeWM grammar.

use std::fs;
use std::path::PathBuf;

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

fn real_file() -> winsome_keymap::KeymapFile {
    let text = fs::read_to_string(repo_path("keymap/omarchy.toml")).expect("keymap file readable");
    winsome_keymap::parse(&text).expect("keymap parses")
}

#[test]
fn real_keymap_is_valid() {
    let file = real_file();
    let (_, cov) = winsome_keymap::check(&file).expect("keymap validates");
    // Coverage acts as a drift alarm: a keymap edit that changes these counts
    // is fine, but must be a conscious edit of this assertion in the same
    // commit — never a surprise.
    assert_eq!(
        (cov.mapped, cov.gap, cov.native, cov.app, cov.omitted),
        (73, 19, 18, 2, 30),
        "entry counts changed: {cov:?}"
    );
    assert_eq!((cov.chords_mapped, cov.chords_total), (73, 226), "chord counts changed: {cov:?}");
}

#[test]
fn real_keymap_matches_golden() {
    let file = real_file();
    let (expanded, _) = winsome_keymap::check(&file).expect("keymap validates");
    let rendered = winsome_keymap::render_glazewm(&expanded);

    let golden_path = repo_path("crates/winsome-keymap/tests/golden/glazewm-keybindings.yaml");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::create_dir_all(golden_path.parent().expect("has parent")).expect("mkdir golden");
        fs::write(&golden_path, &rendered).expect("write golden");
    }
    let golden = fs::read_to_string(&golden_path)
        .expect("golden file exists — run with UPDATE_GOLDEN=1 to create it");
    assert_eq!(rendered, golden, "rendered YAML diverges from golden");
}
