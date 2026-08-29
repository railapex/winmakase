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
}

impl Component {
    /// Start order: kanata first, so the mod key is live before the tiler is.
    /// Shutdown walks this in reverse.
    pub const START_ORDER: [Component; 2] = [Component::Kanata, Component::Glazewm];

    pub fn as_str(self) -> &'static str {
        match self {
            Component::Kanata => "kanata",
            Component::Glazewm => "glazewm",
        }
    }

    pub fn config(self, cfg: &Config) -> &ComponentConfig {
        match self {
            Component::Kanata => &cfg.kanata,
            Component::Glazewm => &cfg.glazewm,
        }
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
        assert_eq!(down, vec![Component::Glazewm, Component::Kanata]);
    }

    #[test]
    fn names_are_the_log_file_stems() {
        let names: Vec<&str> = Component::START_ORDER.iter().map(|c| c.as_str()).collect();
        assert_eq!(names, vec!["kanata", "glazewm"]);
    }
}
