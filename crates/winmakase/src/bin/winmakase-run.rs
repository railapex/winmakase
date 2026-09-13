//! Windowless PowerToys Run entrypoint for GlazeWM and Zebar.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

fn main() -> ExitCode {
    match winmakase::launcher::open_run(winmakase::launcher::OpenOptions::default()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
