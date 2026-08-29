//! The logon-task host: `winsome supervise` with no console.
//!
//! A scheduled task running a console binary in the interactive session puts a
//! console window on the desktop; a GUI-subsystem binary never has one. The
//! cost is that console control events do not exist here — session end arrives
//! as `WM_ENDSESSION` on the display watch's window instead (see
//! `display_watch`), funneling into the same shutdown flag.
//!
//! No arguments, no output: configuration is `~/.winsome/config.toml`
//! ($WINSOME_HOME respected), and anything worth saying goes to the supervisor
//! log — a windowless process printing to a stdout nobody holds would be
//! logging to nowhere.
#![windows_subsystem = "windows"]

use std::fs;
use std::io::Write;
use std::process::ExitCode;

use winsome::config::Config;
use winsome::paths::Paths;
use winsome::signal;
use winsome::supervisor::Supervisor;
use winsome::timefmt;

fn main() -> ExitCode {
    let paths = match Paths::resolve() {
        Ok(p) => p,
        // No home directory to resolve means nowhere to log either.
        Err(_) => return ExitCode::FAILURE,
    };

    let run = || -> std::io::Result<()> {
        let cfg = Config::load_or_create(&paths.config())?;
        // Best effort: a GUI process has no console, but a handler that is
        // never called costs nothing and covers a debugger-attached console.
        let _ = signal::install();
        Supervisor::new(cfg, paths.clone())?.run()?;
        Ok(())
    };

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            note_failure(&paths, &e.to_string());
            ExitCode::FAILURE
        }
    }
}

/// A start-up failure must leave a line somewhere findable. The rolling log
/// machinery may be exactly what failed, so this appends plainly.
fn note_failure(paths: &Paths, msg: &str) {
    let path = paths.log_for("supervisor");
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{} winsomed could not run: {msg}", timefmt::now_iso8601());
    }
}
