//! Supervisor integration tests.
//!
//! These drive the real supervisor against real child processes — the `_stub`
//! stand-in, never kanata or GlazeWM. Killing the actual components to watch
//! the restart logic is the cascade the supervisor exists to prevent, and on
//! this machine it would take somebody's desktop down with it.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use winsome::component::Component;
use winsome::config::{ComponentConfig, Config, SupervisorConfig};
use winsome::control;
use winsome::health::{Health, Status, SupervisorHealth};
use winsome::paths::Paths;
use winsome::supervisor::{StopReason, Supervisor};
use winsome::testutil::{FakeHandle, FakePoll, TempDir};

// Generous on purpose: these tests run alongside a full workspace build on
// loaded machines (CI runners, parallel test threads); a starved harness
// thread must not read as a supervisor bug.
const PATIENCE: Duration = Duration::from_secs(45);

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
        bounce_on_display_change: false,
        // Never true in tests: it would toggle the real taskbar of whoever
        // runs `cargo test`.
        hide_taskbar: false,
    }
}

fn pair_config() -> Config {
    Config {
        supervisor: fast_supervisor(),
        kanata: stub(&["--tick-ms", "150"]),
        glazewm: stub(&["--tick-ms", "150"]),
        zebar: None,
        scratchpad: Default::default(),
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

/// Every kanata start/stop in the log, in order, as (event, pid).
fn kanata_events(log: &str) -> Vec<(&'static str, u32)> {
    let mut events = Vec::new();
    for line in log.lines() {
        for (marker, event) in [
            ("kanata started (pid ", "started"),
            ("kanata stopped (pid ", "stopped"),
        ] {
            if let Some((_, rest)) = line.split_once(marker) {
                let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
                if let Ok(pid) = digits.parse::<u32>() {
                    events.push((event, pid));
                }
            }
        }
    }
    events
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

#[test]
fn a_half_started_pair_is_rolled_back_instead_of_leaving_an_orphan() {
    // kanata starts fine, glazewm cannot start at all. The retry must not
    // respawn kanata on top of the one already running: `children.insert`
    // would drop the old handle, and dropping a `Child` does not kill it, so
    // the desktop would accumulate a keyboard remapper per retry.
    let mut cfg = pair_config();
    cfg.glazewm = ComponentConfig::new("winsome-no-such-binary-7a21.exe", &[]);
    let mut h = Harness::start("partial-start", cfg);

    wait_for("three start attempts", || {
        (h.supervisor_log().matches("kanata started (pid ").count() >= 3).then_some(())
    });
    h.stop();

    let log = h.supervisor_log();

    // Nothing survived the run. This is the assertion that matters: a leaked
    // kanata is a second keyboard remapper on a live desktop.
    let events = kanata_events(&log);
    for (event, pid) in &events {
        if *event == "started" {
            assert_eq!(
                winsome::proc_alive::is_alive(*pid),
                Some(false),
                "kanata pid {pid} was orphaned; events: {events:?}"
            );
        }
    }

    // Start and stop strictly alternate, on matching pids: at no point were two
    // kanata processes live.
    assert!(events.len() >= 6, "{events:?}");
    for pair in events.chunks(2) {
        assert_eq!(pair[0].0, "started", "{events:?}");
        let Some(stop) = pair.get(1) else { continue };
        assert_eq!(stop.0, "stopped", "{events:?}");
        assert_eq!(
            stop.1, pair[0].1,
            "the kanata taken down must be the one that came up: {events:?}"
        );
    }

    assert!(log.contains("linked-pair: rolling kanata back"));
    assert!(
        !log.contains("glazewm started"),
        "glazewm never came up, so nothing should claim it did"
    );
    assert!(
        !log.contains("already running — not starting another"),
        "rollback should have left nothing for the guard to catch"
    );
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

// -- the optional third component --------------------------------------------

#[test]
fn zebar_joins_the_set_and_restarts_alone() {
    let mut cfg = pair_config();
    cfg.zebar = Some(stub(&["--tick-ms", "150"]));
    let mut h = Harness::start("zebar-set", cfg);

    let (kanata, glazewm) = wait_for("the pair to start", || h.both_running());
    let zebar1 = wait_for("zebar to start", || h.running_pid(Component::Zebar));

    kill(zebar1);
    let zebar2 = wait_for("zebar to come back", || {
        h.running_pid(Component::Zebar).filter(|p| *p != zebar1)
    });
    assert_ne!(zebar2, zebar1);

    // The pair is untouched: a dead bar is cosmetic, not a keyboard hazard.
    assert_eq!(h.running_pid(Component::Kanata), Some(kanata));
    assert_eq!(h.running_pid(Component::Glazewm), Some(glazewm));
    assert_eq!(h.component(Component::Zebar).unwrap().restarts, 1);
    let log = h.supervisor_log();
    assert!(log.contains("restarting zebar in"), "{log}");
    assert!(
        !log.contains("linked-pair"),
        "zebar must not trigger pair rules:\n{log}"
    );

    // Shutdown stops the bar first (reverse start order).
    h.stop();
    let log = h.supervisor_log();
    let zebar_stopped = line_of(&log, &format!("zebar stopped (pid {zebar2})"));
    let glazewm_stopped = line_of(&log, &format!("glazewm stopped (pid {glazewm})"));
    assert!(zebar_stopped < glazewm_stopped);
}

#[test]
fn a_config_without_zebar_runs_a_two_component_set() {
    let mut h = Harness::start("without-bar", pair_config());
    wait_for("the pair to start", || h.both_running());

    let state = h.health().unwrap();
    assert_eq!(state.components.len(), 2);
    assert!(!state.components.contains_key("zebar"));

    h.stop();
    let log = h.supervisor_log();
    assert!(
        !log.contains("zebar"),
        "an unconfigured component should never be mentioned:\n{log}"
    );
}

// -- adoption and external processes ----------------------------------------

/// A copy of our binary under a unique name and directory, so full-image-path
/// matching can never collide with the other tests' stub children.
fn copy_of_our_binary(dir: &Path, name: &str) -> PathBuf {
    let dest = dir.join(name);
    fs::copy(env!("CARGO_BIN_EXE_winsome"), &dest).expect("copy test binary");
    dest
}

#[test]
fn a_running_component_is_adopted_not_started_again() {
    let bin_dir = TempDir::new("adopt-bin");
    let adoptee = copy_of_our_binary(bin_dir.path(), "winsome-adoptee.exe");
    let adoptee_str = adoptee.to_str().unwrap().to_string();

    // Running before the supervisor exists — the live-desktop takeover case.
    let mut external = std::process::Command::new(&adoptee)
        .args(["_stub", "--tick-ms", "100"])
        .spawn()
        .expect("start the adoptee");
    let external_pid = external.id();

    let mut cfg = pair_config();
    cfg.kanata = ComponentConfig::new(adoptee_str, &["_stub", "--tick-ms", "100"]).adopting();
    let mut h = Harness::start("adopt", cfg);

    let (kanata, _) = wait_for("both components to start", || h.both_running());
    assert_eq!(
        kanata, external_pid,
        "the running process must be adopted, not replaced"
    );
    let log = h.supervisor_log();
    assert!(
        log.contains(&format!(
            "kanata started (pid {external_pid}, restarts 0): adopted"
        )),
        "the log must say it adopted:\n{log}"
    );

    // Shutdown owns the adopted process like any other: it dies with the set.
    h.stop();
    assert_eq!(winsome::proc_alive::is_alive(external_pid), Some(false));
    let _ = external.wait();
}

#[test]
fn a_second_supervisor_against_the_same_home_is_refused() {
    let dir = TempDir::new("singleton");
    let paths = Paths::at(dir.path());

    // A stand-in for the first supervisor: any process that stays alive.
    let mut other = std::process::Command::new("cmd")
        .args(["/c", "ping -n 30 127.0.0.1 > nul"])
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();

    let mut sup = Supervisor::new(pair_config(), paths.clone()).unwrap();
    Health::new(
        SupervisorHealth {
            running: true,
            pid: other.id(),
            since: winsome::timefmt::now_iso8601(),
        },
        std::collections::BTreeMap::new(),
    )
    .write(&paths.health())
    .unwrap();

    let err = sup.run().expect_err("a live supervisor must be refused");
    assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    assert!(err.to_string().contains(&other.id().to_string()));

    kill(other.id());
    let _ = other.wait();
}

// -- scripted-handle escalation (unreachable with real children) -------------

#[test]
fn an_unreadable_component_is_written_off_after_the_retry_budget() {
    let dir = TempDir::new("unreadable");
    let mut cfg = pair_config();
    // Backoff so long the scheduled recovery can never fire mid-test.
    cfg.supervisor.backoff_initial_ms = 600_000;
    cfg.supervisor.backoff_max_ms = 600_000;
    let mut sup = Supervisor::new(cfg, Paths::at(dir.path())).unwrap();

    sup.inject_running(
        Component::Kanata,
        Box::new(FakeHandle::new(
            4242,
            vec![FakePoll::Error("scripted blip")],
        )),
    );
    for _ in 0..12 {
        sup.tick();
    }

    let log = fs::read_to_string(Paths::at(dir.path()).log_for("supervisor")).unwrap();
    assert!(
        log.contains("kanata: cannot read child status: scripted blip — retrying"),
        "{log}"
    );
    assert_eq!(
        log.matches("cannot read child status").count(),
        1,
        "the retry must not spam the log:\n{log}"
    );
    assert!(
        log.contains("status unreadable for 12 polls"),
        "after the budget it gives up:\n{log}"
    );
    assert!(log.contains("kanata stopped (pid 4242)"));
    assert_eq!(
        sup.snapshot().components["kanata"].status,
        Status::Restarting,
        "a written-off component goes through normal recovery"
    );
}

#[test]
fn glazewm_becoming_unreadable_still_triggers_the_linked_pair_rule() {
    let dir = TempDir::new("unreadable-pair");
    let mut cfg = pair_config();
    cfg.supervisor.backoff_initial_ms = 600_000;
    cfg.supervisor.backoff_max_ms = 600_000;
    let mut sup = Supervisor::new(cfg, Paths::at(dir.path())).unwrap();

    sup.inject_running(
        Component::Kanata,
        Box::new(FakeHandle::new(111, vec![FakePoll::Running])),
    );
    sup.inject_running(
        Component::Glazewm,
        Box::new(FakeHandle::new(
            222,
            vec![FakePoll::Error("scripted fault")],
        )),
    );
    for _ in 0..12 {
        sup.tick();
    }

    let log = fs::read_to_string(Paths::at(dir.path()).log_for("supervisor")).unwrap();
    assert!(
        log.contains("linked-pair: glazewm is down"),
        "an unwatchable tiler is a down tiler:\n{log}"
    );
    assert!(log.contains("kanata stopped (pid 111)"), "{log}");
    let snap = sup.snapshot();
    assert_eq!(snap.components["kanata"].status, Status::Restarting);
    assert_eq!(snap.components["glazewm"].status, Status::Restarting);
}

// -- display changes ---------------------------------------------------------

#[test]
fn a_display_change_bounces_the_pair_only_when_configured() {
    let dir = TempDir::new("display-bounce");
    let mut cfg = pair_config();
    cfg.supervisor.bounce_on_display_change = true;
    let mut sup = Supervisor::new(cfg, Paths::at(dir.path())).unwrap();
    sup.start_now();

    let before = sup.snapshot();
    let old_kanata = before.components["kanata"].pid.unwrap();
    sup.simulate_display_change();
    sup.tick(); // bounce: both down, restart scheduled at zero delay
    sup.tick(); // the scheduled restart fires

    let log = fs::read_to_string(Paths::at(dir.path()).log_for("supervisor")).unwrap();
    assert!(log.contains("display change: bouncing the pair"), "{log}");
    let after = sup.snapshot();
    let new_kanata = after.components["kanata"].pid.unwrap();
    assert_ne!(new_kanata, old_kanata, "the pair must be new processes");
    assert_eq!(after.components["kanata"].restarts, 1);
    assert_eq!(after.components["glazewm"].restarts, 1);

    sup.shutdown_now();
}

#[test]
fn a_display_change_without_the_flag_changes_nothing() {
    let dir = TempDir::new("display-quiet");
    let mut sup = Supervisor::new(pair_config(), Paths::at(dir.path())).unwrap();
    sup.start_now();

    let before = sup.snapshot().components["kanata"].pid;
    sup.simulate_display_change();
    sup.tick();
    sup.tick();

    let log = fs::read_to_string(Paths::at(dir.path()).log_for("supervisor")).unwrap();
    assert!(!log.contains("bouncing the pair"), "{log}");
    assert_eq!(sup.snapshot().components["kanata"].pid, before);

    sup.shutdown_now();
}

// -- task-hosted components ---------------------------------------------------

/// The real `schtasks` round trip: /run starts it, discovery finds it, a crash
/// restarts it through the task, /end stops it.
///
/// Ignored by default: a scheduled task launching a console binary flashes a
/// console window on an interactive desktop. Run explicitly
/// (`cargo test -- --ignored`) or in CI, where no one is watching the desktop.
#[test]
#[ignore = "registers and runs a real scheduled task; flashes a console window"]
fn a_task_hosted_component_is_started_watched_and_stopped_through_its_task() {
    let bin_dir = TempDir::new("task-bin");
    let hosted = copy_of_our_binary(bin_dir.path(), "winsome-task-stub.exe");
    let task = format!("WinsomeTest-{}", std::process::id());

    let create = std::process::Command::new("schtasks")
        .args([
            "/create",
            "/tn",
            &task,
            "/tr",
            &format!("\"{}\" _stub --tick-ms 100", hosted.display()),
            "/sc",
            "once",
            "/st",
            "00:00",
            "/f",
        ])
        .status()
        .expect("run schtasks /create");
    assert!(create.success(), "could not register the test task");

    let result = std::panic::catch_unwind(|| {
        let mut cfg = pair_config();
        cfg.kanata = ComponentConfig::new(hosted.to_str().unwrap(), &[]).hosted_by_task(&task);
        let mut h = Harness::start("task-hosted", cfg);

        let (kanata1, _) = wait_for("both components to start", || h.both_running());
        assert!(
            h.supervisor_log().contains(&format!("via task {task}")),
            "kanata must have come up through the task"
        );

        // A crash must be recovered through the task as well.
        kill(kanata1);
        let kanata2 = wait_for("kanata to come back", || {
            h.running_pid(Component::Kanata).filter(|p| *p != kanata1)
        });

        h.stop();
        assert_eq!(winsome::proc_alive::is_alive(kanata2), Some(false));
    });

    let _ = std::process::Command::new("schtasks")
        .args(["/delete", "/tn", &task, "/f"])
        .status();
    if let Err(p) = result {
        std::panic::resume_unwind(p);
    }
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
