//! Rendered kanata configs — the mode templates behind `winmakase kanata render`.
//!
//! The templates are the spike configs: `caps` is v4.1, live-proven; `apps`
//! validates with `kanata --check` but has never been live-tested, and the
//! two-hooks finding (spike/NOTES.md) predicts desync risk for its synthesized
//! multi-modifier chord — the template says so in its own header. Rendering is
//! substitution only; judgment lives in the template text.
//!
//! Game-foreground auto-suspend is deliberately NOT here: kanata has no
//! foreground-window awareness (komokana/kanawin/qanata exist precisely to
//! bolt it on externally). The winmakase version is a supervisor-side watcher
//! flipping layers over kanata's TCP server — its own TODO item.

use crate::config::{KeyboardConfig, KeyboardMode};

const CAPS_TEMPLATE: &str = include_str!("../templates/caps.kbd");
const APPS_TEMPLATE: &str = include_str!("../templates/apps.kbd");

pub fn render(kb: &KeyboardConfig) -> String {
    match kb.mode {
        KeyboardMode::Caps => CAPS_TEMPLATE.to_string(),
        KeyboardMode::Apps => APPS_TEMPLATE
            .replace("{tap_ms}", &kb.tap_ms.to_string())
            .replace("{hold_ms}", &kb.hold_ms.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kb(mode: KeyboardMode) -> KeyboardConfig {
        KeyboardConfig {
            mode,
            ..KeyboardConfig::default()
        }
    }

    #[test]
    fn caps_mode_ships_the_scrlk_hatch() {
        // The hard rule: a busted remap must always leave one raw exit.
        let out = render(&kb(KeyboardMode::Caps));
        assert!(out.contains("(defsrc caps slck)"), "{out}");
        assert!(out.contains("(deflayer base rmet caps)"), "{out}");
    }

    #[test]
    fn apps_mode_leaves_caps_native() {
        // In apps mode the raw CapsLock exit is caps itself — the template
        // must not touch it.
        let out = render(&kb(KeyboardMode::Apps));
        assert!(out.contains("(defsrc menu)"), "{out}");
    }

    #[test]
    fn apps_mode_substitutes_the_tap_hold_window() {
        let out = render(&KeyboardConfig {
            mode: KeyboardMode::Apps,
            tap_ms: 150,
            hold_ms: 250,
            ..KeyboardConfig::default()
        });
        assert!(out.contains("tap-hold-press 150 250"), "{out}");
    }

    #[test]
    fn no_placeholder_survives_rendering() {
        for mode in [KeyboardMode::Caps, KeyboardMode::Apps] {
            let out = render(&kb(mode));
            assert!(
                !out.contains('{'),
                "unresolved placeholder in {mode:?}: {out}"
            );
        }
    }

    #[test]
    fn every_template_says_it_is_generated() {
        for mode in [KeyboardMode::Caps, KeyboardMode::Apps] {
            assert!(render(&kb(mode)).contains("do not edit"));
        }
    }
}
