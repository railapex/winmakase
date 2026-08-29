//! Supervisor integration tests.
//!
//! These drive the real supervisor against real child processes — the `_stub`
//! stand-in, never kanata or GlazeWM. Killing the actual components to watch
//! the restart logic is the cascade the supervisor exists to prevent, and on
//! this machine it would take somebody's desktop down with it.

use std::fs;
use std::io;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use winsome::component::Component;
use winsome::config::{ComponentConfig, Config, SupervisorConfig};
use winsome::control;
use winsome::health::{Health, Status};
use winsome::paths::Paths;
use winsome::supervisor::{StopReason, Supervisor};
use winsome::testutil::TempDir;

const PATIENCE: Duration = Duration::from_secs(15);

// -- harness ---------------------------------------------------------------

fn stub(args: &[&str]) -> ComponentConfig {
    let mut argv = vec!["_stub"];
    argv.extend_from_slice(args);
    ComponentConfig::new(env!("CARGO_BIN_EXE_winsome"), &argv)
}

fn fast_supervisor() -> SupervisorConfig {
    SupervisorConfig {
        poll_ms: 25,
        backoff_initial_ms: 120,
        backoff_max_ms: 480,
        healthy_reset_secs: 60,
        stop_timeout_ms: 300,
        log_max_bytes: 1 << 20,
        log_keep_files: 3,
    }
}

fn pair_config() -> Config {
    Config {
        supervisor: fast_supervisor(),
        kanata: stub(&["--tick-ms", "150"]),
        glazewm: stub(&["--tick-ms", "150"]),
    }
}

struct Harness {
    paths: Paths,
    handle: Option<JoinHandle<io::Result<StopReason>>>,
    _dir: TempDir,
}

impl Harness {
    fn start(label: &str, cfg: Config) -> Self {
        let dir = TempDir::new(label);
        let paths = Paths::at(dir.path());
        let thread_paths = paths.clone();
        let handle = thread::spawn(move || Supervisor::new(cfg, thread_paths)?.run());
        Self {
            paths,
            handle: Some(handle),
            _dir: dir,
        }
    }

    fn health(&self) -> Option<Health> {
        Health::read(&self.paths.health()).ok()
    }

    fn component(&self, c: Component) -> Option<winsome::health::ComponentHealth> {
        self.health()?.components.get(c.as_str()).cloned()
    }

    /// The pid of a component, but only while it is actually running.
    fn running_pid(&self, c: Component) -> Option<u32> {
        let h = self.component(c)?;
        (h.status == Status::Running).then_some(h.pid).flatten()
    }

    fn both_running(&self) -> Option<(u32, u32)> {
        Some((
            self.running_pid(Component::Kanata)?,
            self.running_pid(Component::Glazewm)?,
        ))
    }

    fn supervisor_log(&self) -> String {
        fs::read_to_string(self.paths.log_for("supervisor")).unwrap_or_default()
    }

    fn component_log(&self, c: Component) -> String {
        fs::read_to_string(self.paths.log_for(c.as_str())).unwrap_or_default()
    }

    fn stop(&mut self) -> StopReason {
        let Some(handle) = self.handle.take() else {
            panic!("already stopped");
        };
        control::request_stop(&self.paths).expect("write control file");
        handle
            .join()
            .expect("supervisor thread panicked")
            .expect("supervisor returned an error")
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if self.handle.is_some() {
            self.stop();
        }
    }
}

fn wait_for<T>(what: &str, mut f: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + PATIENCE;
    loop {
        if let Some(v) = f() {
            return v;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        thread::sleep(Duration::from_millis(20));
    }
}

/// Kill a child from outside, the way a crash would.
fn kill(pid: u32) {
    let status = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("run taskkill");
    assert!(status.success(), "taskkill failed for pid {pid}");
}

/// Index of the first log line containing `needle`.
fn line_of(log: &str, needle: &str) -> usize {
    log.lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("supervisor.log has no line containing {needle:?}:\n{log}"))
}

// -- the linked-pair rule ---------------------------------------------------

#[test]
fn glazewm_dying_takes_kanata_down_then_restarts_both_in_order() {
    let mut h = Harness::start("linked-pair", pair_config());
    let (kanata1, glazewm1) = wait_for("both components to start", || h.both_running());

    kill(glazewm1);

    let (kanata2, glazewm2) = wait_for("the pair to come back", || {
        let (k, g) = h.both_running()?;
        (k != kanata1 && g != glazewm1).then_some((k, g))
    });
    assert_ne!(kanata2, kanata1, "kanata must be a new process");
    assert_ne!(glazewm2, glazewm1, "glazewm must be a new process");

    let log = h.supervisor_log();
    let exit = line_of(&log, "glazewm exited");
    let linked = line_of(&log, "linked-pair: glazewm is down");
    let kanata_stopped = line_of(&log, &format!("kanata stopped (pid {kanata1})"));
    let kanata_started = line_of(&log, &format!("kanata started (pid {kanata2}"));
    let glazewm_started = line_of(&log, &format!("glazewm started (pid {glazewm2}"));

    // The order is the whole rule: kanata is down before anything is restarted,
    // and kanata is up before GlazeWM is.
    assert!(
        exit < linked,
        "the death must be logged before the response"
    );
    assert!(
        linked < kanata_stopped,
        "kanata must be stopped as the response to glazewm dying"
    );
    assert!(
        kanata_stopped < kanata_started,
        "kanata must go down before it comes back"
    );
    assert!(
        kanata_started < glazewm_started,
        "kanata must be up before glazewm is — a tiler with no mod key is the bad state"
    );

    // Both were restarted once, and the crash is on the record.
    assert_eq!(h.component(Component::Kanata).unwrap().restarts, 1);
    assert_eq!(h.component(Component::Glazewm).unwrap().restarts, 1);

    assert_eq!(h.stop(), StopReason::ControlFile);
}

#[test]
fn kanata_dying_restarts_kanata_alone_and_leaves_glazewm_be() {
    let mut h = Harness::start("kanata-alone", pair_config());
    let (kanata1, glazewm1) = wait_for("both components to start", || h.both_running());

    kill(kanata1);

    let kanata2 = wait_for("kanata to come back", || {
        h.running_pid(Component::Kanata).filter(|p| *p != kanata1)
    });
    assert_ne!(kanata2, kanata1);

    let glazewm = h.component(Component::Glazewm).unwrap();
    assert_eq!(glazewm.pid, Some(glazewm1), "glazewm must not be touched");
    assert_eq!(glazewm.status, Status::Running);
    assert_eq!(glazewm.restarts, 0);
    assert_eq!(h.component(Component::Kanata).unwrap().restarts, 1);

    let log = h.supervisor_log();
    assert!(log.contains("restarting kanata in 120ms"));
    assert!(
        !log.contains("linked-pair"),
        "kanata dying is the direction that does NOT take glazewm with it"
    );

    h.stop();
}

// -- backoff ----------------------------------------------------------------

#[test]
fn repeated_crashes_back_off_and_then_cap() {
    let mut cfg = pair_config();
    // kanata that dies the moment it starts, so restarts pile up on their own.
    cfg.kanata = stub(&["--exit-after-ms", "0", "--code", "3"]);
    let h = Harness::start("backoff", cfg);

    let log = wait_for("four restart attempts", || {
        let log = h.supervisor_log();
        (log.matches("restarting kanata in").count() >= 4).then_some(log)
    });

    let delays: Vec<&str> = log
        .lines()
        .filter_map(|l| l.split("restarting kanata in ").nth(1))
        .take(4)
        .collect();
    assert_eq!(
        delays,
        vec!["120ms", "240ms", "480ms", "480ms"],
        "backoff must double and then hold at the cap"
    );

    let kanata = h.component(Component::Kanata).unwrap();
    assert_eq!(kanata.last_exit_code, Some(3), "the exit code is recorded");
    assert!(kanata.restarts >= 3);
    // glazewm is untouched by kanata's crash loop.
    assert_eq!(
        h.component(Component::Glazewm).unwrap().status,
        Status::Running
    );
}

#[test]
fn a_component_that_will_not_start_never_brings_up_its_partner() {
    let mut cfg = pair_config();
    cfg.kanata = ComponentConfig::new("winsome-no-such-binary-9f3c.exe", &[]);
    let h = Harness::start("start-failure", cfg);

    let log = wait_for("the failed start to be reported", || {
        let log = h.supervisor_log();
        log.contains("not starting glazewm").then_some(log)
    });
    assert!(log.contains("kanata FAILED to start"));
    assert!(
        !log.contains("glazewm started"),
        "glazewm must never come up without kanata"
    );
    assert!(log.contains("retrying the whole set in 120ms"));
    assert_eq!(h.component(Component::Glazewm).unwrap().pid, None);
}

// -- logs and health --------------------------------------------------------

#[test]
fn child_output_lands_in_the_components_own_log() {
    let mut cfg = pair_config();
    cfg.kanata = stub(&["--tick-ms", "30", "--stderr"]);
    let mut h = Harness::start("child-logs", cfg);
    wait_for("both components to start", || h.both_running());

    let log = wait_for("kanata stdout and stderr", || {
        let log = h.component_log(Component::Kanata);
        (log.contains("[out] stub pid") && log.contains("[err] stub pid")).then_some(log)
    });
    // Every captured line is timestamped and tagged by stream.
    for line in log.lines().take(5) {
        let (stamp, rest) = line.split_once(' ').unwrap();
        assert!(winsome::timefmt::parse_iso8601(stamp).is_some(), "{line}");
        assert!(
            rest.starts_with("[out] ") || rest.starts_with("[err] "),
            "{line}"
        );
    }
    wait_for("glazewm's output in its own file", || {
        h.component_log(Component::Glazewm)
            .contains("[out] stub pid")
            .then_some(())
    });
    assert!(
        !h.component_log(Component::Glazewm).contains("[err]"),
        "a component's log holds only its own output"
    );
    h.stop();
}

#[test]
fn health_state_describes_the_live_stack() {
    let mut h = Harness::start("health-live", pair_config());
    let (kanata, glazewm) = wait_for("both components to start", || h.both_running());

    let state = h.health().unwrap();
    assert_eq!(state.schema, 1);
    assert!(state.supervisor.running);
    assert_eq!(state.supervisor.pid, std::process::id());
    assert_eq!(state.components.len(), 2);
    assert_eq!(state.components["kanata"].pid, Some(kanata));
    assert_eq!(state.components["glazewm"].pid, Some(glazewm));
    for c in state.components.values() {
        assert_eq!(c.status, Status::Running);
        assert_eq!(c.restarts, 0);
        assert!(winsome::timefmt::parse_iso8601(&c.since).is_some());
    }

    h.stop();
    let after = h.health().unwrap();
    assert!(!after.supervisor.running);
    for c in after.components.values() {
        assert_eq!(c.status, Status::Stopped);
        assert_eq!(c.pid, None);
    }
}

// -- shutdown ---------------------------------------------------------------

#[test]
fn shutdown_stops_components_in_reverse_order_via_their_stop_command() {
    let mut cfg = pair_config();
    // A stop command that returns immediately without stopping anything: the
    // supervisor must fall through to killing the child rather than hang.
    cfg.glazewm = stub(&["--tick-ms", "150"]).with_stop(
        env!("CARGO_BIN_EXE_winsome"),
        &["_stub", "--exit-after-ms", "0"],
    );
    let mut h = Harness::start("shutdown", cfg);
    let (kanata, glazewm) = wait_for("both components to start", || h.both_running());

    assert_eq!(h.stop(), StopReason::ControlFile);

    let log = h.supervisor_log();
    let requested = line_of(&log, "shutdown requested (winsome down)");
    let asked = line_of(&log, "asking glazewm to exit");
    let forced = line_of(&log, "glazewm still up after");
    let glazewm_stopped = line_of(&log, &format!("glazewm stopped (pid {glazewm})"));
    let kanata_stopped = line_of(&log, &format!("kanata stopped (pid {kanata})"));
    let complete = line_of(&log, "shutdown complete");

    assert!(requested < asked);
    assert!(
        asked < forced,
        "the CLI is asked before the process is killed"
    );
    assert!(forced < glazewm_stopped);
    assert!(
        glazewm_stopped < kanata_stopped,
        "reverse of start order: glazewm first, kanata last"
    );
    assert!(kanata_stopped < complete);

    // Nothing left running.
    assert_eq!(winsome::proc_alive::is_alive(kanata), Some(false));
    assert_eq!(winsome::proc_alive::is_alive(glazewm), Some(false));
}

#[test]
fn a_stale_stop_request_does_not_kill_the_next_supervisor() {
    let dir = TempDir::new("stale-stop");
    let paths = Paths::at(dir.path());
    control::request_stop(&paths).unwrap();

    let cfg = pair_config();
    let thread_paths = paths.clone();
    let handle = thread::spawn(move || Supervisor::new(cfg, thread_paths)?.run());

    // It must come up and stay up despite the request that predates it.
    let started = wait_for("the supervisor to come up", || {
        let h = Health::read(&paths.health()).ok()?;
        h.components
            .values()
            .all(|c| c.status == Status::Running)
            .then_some(true)
    });
    assert!(started);
    thread::sleep(Duration::from_millis(200));
    assert!(
        Health::read(&paths.health()).unwrap().supervisor.running,
        "a stale request must be cleared at startup, not obeyed"
    );

    control::request_stop(&paths).unwrap();
    assert_eq!(
        handle.join().unwrap().unwrap(),
        StopReason::ControlFile,
        "a fresh request still stops it"
    );
}
