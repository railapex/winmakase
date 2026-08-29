//! The `winmakase down` channel.
//!
//! **Mechanism: a polled control file** at `~/.winmakase/state/control`. `down`
//! writes `stop <timestamp>`; the supervisor notices within one poll tick,
//! deletes the file (which is the acknowledgement `down` waits on), and shuts
//! down gracefully.
//!
//! A named pipe would be lower latency and no more correct. The file wins on
//! the properties that matter here: it survives the supervisor not being up
//! yet, it needs no security descriptor to cross the elevated/non-elevated
//! boundary this stack straddles, it is inspectable with `type`, and it is
//! drivable from a test or a PowerShell one-liner without an IPC client. A
//! quarter-second to shut a desktop down is not a cost anyone can feel.
//!
//! Stale requests are handled at both ends: the supervisor clears the file at
//! startup, and `down` removes its own request if nobody picks it up.

use std::fs;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

use crate::paths::Paths;
use crate::timefmt;

const STOP: &str = "stop";

/// Ask a running supervisor to stop.
pub fn request_stop(paths: &Paths) -> io::Result<()> {
    fs::create_dir_all(paths.state_dir())?;
    fs::write(
        paths.control(),
        format!("{STOP} {}\n", timefmt::now_iso8601()),
    )
}

/// Drop any request left over from a previous run. Called at supervisor
/// startup: a `stop` written while nothing was listening must not shut down the
/// next supervisor the moment it comes up.
pub fn clear(paths: &Paths) -> io::Result<()> {
    match fs::remove_file(paths.control()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// True if a stop is pending. Consumes the request — removing the file is how
/// `down` learns the supervisor heard it.
pub fn take_stop_request(paths: &Paths) -> io::Result<bool> {
    let path = paths.control();
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let verb = text.split_whitespace().next().unwrap_or_default();
    if verb == STOP {
        clear(paths)?;
        return Ok(true);
    }
    // Unknown verb: drop it rather than re-reading it every tick, but say so.
    clear(paths)?;
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("unknown control request {verb:?} (expected {STOP:?})"),
    ))
}

pub fn is_pending(paths: &Paths) -> bool {
    paths.control().exists()
}

/// Block until the supervisor consumes the request. `Ok(true)` means it was
/// acknowledged; `Ok(false)` means the timeout ran out and the request was
/// withdrawn so it cannot ambush the next startup.
pub fn wait_for_ack(paths: &Paths, timeout: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !is_pending(paths) {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(50));
    }
    if is_pending(paths) {
        clear(paths)?;
        return Ok(false);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn a_request_is_written_seen_once_and_consumed() {
        let dir = TempDir::new("control");
        let paths = Paths::at(dir.path());
        assert!(!take_stop_request(&paths).unwrap());
        request_stop(&paths).unwrap();
        assert!(is_pending(&paths));
        assert!(take_stop_request(&paths).unwrap());
        assert!(!is_pending(&paths));
        assert!(!take_stop_request(&paths).unwrap());
    }

    #[test]
    fn clearing_at_startup_defuses_a_stale_request() {
        let dir = TempDir::new("control-stale");
        let paths = Paths::at(dir.path());
        request_stop(&paths).unwrap();
        clear(&paths).unwrap();
        assert!(!take_stop_request(&paths).unwrap());
    }

    #[test]
    fn a_junk_request_errors_and_is_removed() {
        let dir = TempDir::new("control-junk");
        let paths = Paths::at(dir.path());
        fs::create_dir_all(paths.state_dir()).unwrap();
        fs::write(paths.control(), "explode\n").unwrap();
        assert!(take_stop_request(&paths).is_err());
        assert!(!is_pending(&paths));
    }

    #[test]
    fn an_unheard_request_withdraws_itself() {
        let dir = TempDir::new("control-timeout");
        let paths = Paths::at(dir.path());
        request_stop(&paths).unwrap();
        assert!(!wait_for_ack(&paths, Duration::from_millis(150)).unwrap());
        assert!(!is_pending(&paths));
    }
}
