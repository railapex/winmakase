//! The supervised set.
//!
//! kanata and GlazeWM are named types rather than map keys because the
//! linked-pair rule is a statement about *these two specifically* (spike
//! cascade incident). Making the pair a compile-time fact keeps a future third
//! component from silently inheriting rules that do not apply to it.

use std::fmt;

use crate::config::{ComponentConfig, Config};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Component {
    Kanata,
    Glazewm,
    /// The bar. Outside the linked pair: it dies and restarts alone, and its
    /// `[zebar]` config section is optional — a stack without a bar is
    /// degraded, not broken.
    Zebar,
}

impl Component {
    /// Start order: kanata first, so the mod key is live before the tiler is;
    /// the bar last, it decorates what the others provide. Shutdown walks this
    /// in reverse.
    pub const START_ORDER: [Component; 3] =
        [Component::Kanata, Component::Glazewm, Component::Zebar];

    pub fn as_str(self) -> &'static str {
        match self {
            Component::Kanata => "kanata",
            Component::Glazewm => "glazewm",
            Component::Zebar => "zebar",
        }
    }

    /// `None` means the component is not part of this supervisor's set: the
    /// bar is optional, and kanata belongs only to `input_mode = "kanata"`.
    /// The input mode, never a live kanata process, decides.
    pub fn config(self, cfg: &Config) -> Option<&ComponentConfig> {
        match self {
            Component::Kanata => cfg
                .keyboard
                .input_mode
                .runs_kanata()
                .then_some(cfg.kanata.as_ref())
                .flatten(),
            Component::Glazewm => Some(&cfg.glazewm),
            Component::Zebar => cfg.zebar.as_ref(),
        }
    }

    /// The failure group of the input/tiler: kanata plus GlazeWM when the
    /// mode runs kanata, GlazeWM alone otherwise. In start order.
    pub fn linked_group(cfg: &Config) -> Vec<Component> {
        [Component::Kanata, Component::Glazewm]
            .into_iter()
            .filter(|c| c.config(cfg).is_some())
            .collect()
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_is_the_reverse_of_start() {
        let down: Vec<_> = Component::START_ORDER.iter().rev().copied().collect();
        assert_eq!(
            down,
            vec![Component::Zebar, Component::Glazewm, Component::Kanata]
        );
    }

    #[test]
    fn the_input_mode_decides_whether_kanata_is_in_the_set() {
        use crate::config::InputMode;

        let mut cfg = Config::default();
        assert!(Component::Kanata.config(&cfg).is_some());
        assert_eq!(
            Component::linked_group(&cfg),
            vec![Component::Kanata, Component::Glazewm]
        );

        for mode in [InputMode::F13, InputMode::DirectCaps] {
            cfg.keyboard.input_mode = mode;
            assert!(
                Component::Kanata.config(&cfg).is_none(),
                "{mode:?} must not run kanata even with a [kanata] section"
            );
            assert_eq!(Component::linked_group(&cfg), vec![Component::Glazewm]);
        }
    }

    #[test]
    fn names_are_the_log_file_stems() {
        let names: Vec<&str> = Component::START_ORDER.iter().map(|c| c.as_str()).collect();
        assert_eq!(names, vec!["kanata", "glazewm", "zebar"]);
    }
}
