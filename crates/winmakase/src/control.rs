//! The `winmakase down` / `winmakase reload` channel.
//!
//! **Mechanism: a polled control file** at `~/.winmakase/state/control`. `down`
//! writes `stop <timestamp>`; `reload` writes `reload <token>`. The supervisor
//! notices within one poll tick, deletes the file (which is the
//! acknowledgement the requester waits on), and acts. A reload additionally
//! writes its outcome to `state/reload-result`, first line the token, so the
//! CLI can tell this reload's report from a stale one.
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
const RELOAD: &str = "reload";

/// What a control file asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Stop,
    Reload { token: String },
}

/// Ask a running supervisor to stop.
pub fn request_stop(paths: &Paths) -> io::Result<()> {
    fs::create_dir_all(paths.state_dir())?;
    fs::write(
        paths.control(),
        format!("{STOP} {}\n", timefmt::now_iso8601()),
    )
}

/// Ask a running supervisor to re-read its config. Returns the token the
/// supervisor will echo as the first line of the reload result.
pub fn request_reload(paths: &Paths) -> io::Result<String> {
    // The counter matters: two reloads inside one second from one process
    // (a script, a test) must not share a token, or the second read of the
    // result file matches the first reload's report.
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let token = format!("{}#{}-{n}", timefmt::now_iso8601(), std::process::id());
    fs::create_dir_all(paths.state_dir())?;
    fs::write(paths.control(), format!("{RELOAD} {token}\n"))?;
    Ok(token)
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

/// The pending request, if any. Consumes it — removing the file is how the
/// requester learns the supervisor heard it.
pub fn take_request(paths: &Paths) -> io::Result<Option<Request>> {
    let path = paths.control();
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut fields = text.split_whitespace();
    let verb = fields.next().unwrap_or_default();
    match verb {
        STOP => {
            clear(paths)?;
            Ok(Some(Request::Stop))
        }
        RELOAD => {
            let token = fields.next().unwrap_or_default().to_string();
            clear(paths)?;
            Ok(Some(Request::Reload { token }))
        }
        _ => {
            // Unknown verb: drop it rather than re-reading it every tick,
            // but say so.
            clear(paths)?;
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown control request {verb:?} (expected {STOP:?} or {RELOAD:?})"),
            ))
        }
    }
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
        assert_eq!(take_request(&paths).unwrap(), None);
        request_stop(&paths).unwrap();
        assert!(is_pending(&paths));
        assert_eq!(take_request(&paths).unwrap(), Some(Request::Stop));
        assert!(!is_pending(&paths));
        assert_eq!(take_request(&paths).unwrap(), None);
    }

    #[test]
    fn a_reload_request_carries_its_token_back() {
        let dir = TempDir::new("control-reload");
        let paths = Paths::at(dir.path());
        let token = request_reload(&paths).unwrap();
        assert!(!token.is_empty());
        assert_eq!(
            take_request(&paths).unwrap(),
            Some(Request::Reload {
                token: token.clone()
            })
        );
        assert!(!is_pending(&paths));
    }

    #[test]
    fn clearing_at_startup_defuses_a_stale_request() {
        let dir = TempDir::new("control-stale");
        let paths = Paths::at(dir.path());
        request_stop(&paths).unwrap();
        clear(&paths).unwrap();
        assert_eq!(take_request(&paths).unwrap(), None);
    }

    #[test]
    fn a_junk_request_errors_and_is_removed() {
        let dir = TempDir::new("control-junk");
        let paths = Paths::at(dir.path());
        fs::create_dir_all(paths.state_dir()).unwrap();
        fs::write(paths.control(), "explode\n").unwrap();
        assert!(take_request(&paths).is_err());
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
