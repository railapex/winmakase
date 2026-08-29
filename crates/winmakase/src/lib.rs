//! Winmakase — the CLI and supervisor for an omakase omarchy-style Windows 11
//! desktop.
//!
//! The library half exists so the supervisor's behaviour can be driven from
//! tests. See [`supervisor`] for the linked-pair rule that shapes most of it.

pub mod backoff;
pub mod commands;
pub mod component;
pub mod config;
pub mod control;
pub mod display_watch;
pub mod glazewm;
pub mod health;
pub mod kanata_kbd;
pub mod monitors;
pub mod notify;
pub mod paths;
pub mod proc_alive;
pub mod process;
pub mod procs;
pub mod reflow;
pub mod rolling_log;
pub mod scratchpad;
pub mod signal;
pub mod stub;
pub mod supervisor;
pub mod taskbar;
pub mod timefmt;

#[doc(hidden)]
pub mod testutil;
