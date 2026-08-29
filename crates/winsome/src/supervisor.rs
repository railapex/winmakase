//! The supervisor.
//!
//! DESIGN.md: "A dead tiler on Windows leaves a normal-looking desktop with
//! hotkeys silently gone — worse than a crash."
//!
//! ## The linked-pair rule
//!
//! kanata and GlazeWM are both healthy or both down. This is not symmetry for
//! its own sake; it comes out of the M0 spike's cascade incident. kanata holds
//! Caps as a real Win key and GlazeWM binds the `lwin+X` chords. Kill GlazeWM
//! and kanata keeps synthesising Win presses that nothing consumes any more, so
//! raw `Win+letter` starts firing OS shortcuts — Feedback Hub opening while the
//! desktop is still tiled and uncontrollable. So:
//!
//! - **GlazeWM exits** → stop kanata *immediately*, before any backoff, then
//!   restart both, kanata first.
//! - **kanata exits** → restart kanata alone; GlazeWM is untouched.
//!
//! Every transition is logged and lands in `health.json` before the next tick.

use std::collections::BTreeMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::backoff::Backoff;
use crate::component::Component;
use crate::config::{ComponentConfig, Config};
use crate::control;
use crate::display_watch::DisplayWatch;
use crate::health::{ComponentHealth, Health, Status, SupervisorHealth};
use crate::paths::Paths;
use crate::proc_alive;
use crate::process::{self, ExternalProcess, Poll, ProcessHandle, RunningChild};
use crate::procs;
use crate::rolling_log::RollingLog;
use crate::signal;
use crate::taskbar;
use crate::timefmt;

/// Wall-clock a console-triggered shutdown may take, start to finish.
///
/// A close, logoff, or machine shutdown gives us [`signal::GRACE_MS`] and then
/// kills the process regardless. Overrunning it is not a slow shutdown, it is a
/// half-done one: GlazeWM killed without its watcher restoring windows, or
/// kanata left holding the keyboard. So a console shutdown ignores the
/// configured stop timeouts and works to this instead.
const CONSOLE_SHUTDOWN_BUDGET_MS: u64 = 3_000;

/// Room left for the final health write and the log flush after the last child
/// is down.
const CONSOLE_SHUTDOWN_HEADROOM_MS: u64 = 500;

// Stated once, checked by the compiler: nobody can shrink the grace period or
// stretch the budget into a shutdown that gets killed half-done.
const _: () = assert!(
    CONSOLE_SHUTDOWN_BUDGET_MS + CONSOLE_SHUTDOWN_HEADROOM_MS <= signal::GRACE_MS,
    "console shutdown budget must fit inside the OS grace period"
);

/// How many consecutive unreadable polls before a child is written off.
///
/// A failing status poll means we have lost track of the process; retrying is
/// right for a blip, but repeating forever would freeze that component's
/// tracking and log four lines a second while the desktop sits broken.
const WAIT_ERROR_LIMIT: u32 = 12;

/// How long a kill gets to confirm the process is actually gone. Covers the
/// `schtasks /end` round trip for a task-hosted component; TerminateProcess
/// confirms in milliseconds.
const KILL_PATIENCE_MS: u64 = 1_500;

/// After `schtasks /run`, how long the task's process gets to appear before
/// the start counts as failed.
const TASK_START_DISCOVER_MS: u64 = 5_000;

/// A display change is acted on only after the events go quiet for this long —
/// a resolution change and a replug arrive as bursts, and bouncing the pair
/// once per event would multiply the disruption the bounce exists to fix.
const DISPLAY_DEBOUNCE_MS: u64 = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// `winsome down`.
    ControlFile,
    /// Ctrl+C, console close, logoff, or shutdown.
    ConsoleSignal,
}

impl StopReason {
    fn as_str(self) -> &'static str {
        match self {
            StopReason::ControlFile => "winsome down",
            StopReason::ConsoleSignal => "console signal",
        }
    }

    /// Total time the shutdown may take, or `None` for no limit.
    ///
    /// `winsome down` is a person waiting at a prompt — it can afford the full
    /// configured timeouts. A console signal is the OS counting down.
    fn shutdown_budget(self) -> Option<Duration> {
        match self {
            StopReason::ControlFile => None,
            StopReason::ConsoleSignal => Some(Duration::from_millis(CONSOLE_SHUTDOWN_BUDGET_MS)),
        }
    }
}

/// How long a single wait inside a shutdown may take: never more than the
/// configured timeout, and never more than half of what is left in the budget.
///
/// Halving is what makes the total provably bounded — each wait can consume at
/// most half the remainder, so any number of them stays inside the budget.
fn bounded(configured: Duration, remaining: Option<Duration>) -> Duration {
    match remaining {
        None => configured,
        Some(left) => configured.min(left / 2),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaitErrorAction {
    /// Say it once, so a transient fault still leaves a line.
    Report,
    /// Keep retrying without repeating ourselves.
    Quiet,
    /// Stop believing the child is tracked.
    GiveUp,
}

fn wait_error_action(consecutive: u32) -> WaitErrorAction {
    match consecutive {
        0 | 1 => WaitErrorAction::Report,
        n if n >= WAIT_ERROR_LIMIT => WaitErrorAction::GiveUp,
        _ => WaitErrorAction::Quiet,
    }
}

struct PendingRestart {
    at: Instant,
    /// Started in this order; a failure part-way through reschedules the whole
    /// set rather than leaving half a pair up.
    order: Vec<Component>,
    /// False for a first start that failed and is being retried — it never ran,
    /// so counting it as a restart would overstate the crash history.
    is_restart: bool,
}

pub struct Supervisor {
    cfg: Config,
    paths: Paths,
    /// The components this supervisor actually runs: `START_ORDER` filtered to
    /// what the config declares (the bar is optional).
    active: Vec<Component>,
    log: Arc<Mutex<RollingLog>>,
    child_logs: BTreeMap<Component, Arc<Mutex<RollingLog>>>,
    children: BTreeMap<Component, RunningChild>,
    backoff: BTreeMap<Component, Backoff>,
    health: BTreeMap<Component, ComponentHealth>,
    /// Consecutive polls where a child's status could not be read.
    wait_errors: BTreeMap<Component, u32>,
    pending: Option<PendingRestart>,
    display_watch: Option<DisplayWatch>,
    /// Set when a display change arrives; acted on once the burst goes quiet.
    display_event_at: Option<Instant>,
    /// Taskbar state found at startup, put back on shutdown.
    prior_taskbar: Option<u32>,
    started_at: SystemTime,
    shutting_down: bool,
}

impl Supervisor {
    pub fn new(mut cfg: Config, paths: Paths) -> io::Result<Self> {
        paths.ensure_dirs()?;
        // Clamp before anything reads these. A hand-edited zero would otherwise
        // reach the backoff as a zero-delay restart loop.
        let notes = cfg.supervisor.sanitize();
        let (max_bytes, keep) = (cfg.supervisor.log_max_bytes, cfg.supervisor.log_keep_files);
        let log = Arc::new(Mutex::new(RollingLog::open(
            paths.log_for("supervisor"),
            max_bytes,
            keep,
        )?));

        let active: Vec<Component> = Component::START_ORDER
            .into_iter()
            .filter(|c| c.config(&cfg).is_some())
            .collect();

        let mut child_logs = BTreeMap::new();
        let mut backoff = BTreeMap::new();
        let mut health = BTreeMap::new();
        for c in active.iter().copied() {
            child_logs.insert(
                c,
                Arc::new(Mutex::new(RollingLog::open(
                    paths.log_for(c.as_str()),
                    max_bytes,
                    keep,
                )?)),
            );
            backoff.insert(
                c,
                Backoff::new(
                    Duration::from_millis(cfg.supervisor.backoff_initial_ms),
                    Duration::from_millis(cfg.supervisor.backoff_max_ms),
                ),
            );
            health.insert(c, ComponentHealth::stopped_now());
        }

        let supervisor = Self {
            cfg,
            paths,
            active,
            log,
            child_logs,
            children: BTreeMap::new(),
            backoff,
            health,
            wait_errors: BTreeMap::new(),
            pending: None,
            display_watch: None,
            display_event_at: None,
            prior_taskbar: None,
            started_at: SystemTime::now(),
            shutting_down: false,
        };
        for note in notes {
            supervisor.log(format!("config: {note}"));
        }
        Ok(supervisor)
    }

    /// Start the set and watch it until told to stop. Returns why it stopped.
    pub fn run(&mut self) -> io::Result<StopReason> {
        // Two supervisors adopting the same pair would both claim it and race
        // every restart. The logon task plus a curious `winsome supervise` in
        // a console is exactly how that happens.
        if let Ok(h) = Health::read(&self.paths.health()) {
            let pid = h.supervisor.pid;
            if h.supervisor.running
                && pid != std::process::id()
                && proc_alive::is_alive(pid) == Some(true)
            {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "a supervisor (pid {pid}) is already running against {}",
                        self.paths.home().display()
                    ),
                ));
            }
        }

        self.log(format!(
            "supervisor starting (pid {}) home={}",
            std::process::id(),
            self.paths.home().display()
        ));
        // Anything left in the control file predates us and is not our order.
        control::clear(&self.paths)?;

        match DisplayWatch::start() {
            Ok(w) => self.display_watch = Some(w),
            // Not fatal: the watch is #1233 insurance, the stack is the job.
            Err(e) => self.log(format!("display watch could not start: {e}")),
        }

        if self.cfg.supervisor.hide_taskbar {
            let prior = taskbar::get_state();
            taskbar::set_state(taskbar::AUTOHIDE);
            self.prior_taskbar = Some(prior);
            self.log(format!(
                "taskbar auto-hidden (prior state {prior}, restored on shutdown)"
            ));
        }

        self.start_set(self.active.clone(), false);

        let poll = Duration::from_millis(self.cfg.supervisor.poll_ms.max(1));
        let reason = loop {
            if signal::shutdown_requested() {
                break StopReason::ConsoleSignal;
            }
            match control::take_stop_request(&self.paths) {
                Ok(true) => break StopReason::ControlFile,
                Ok(false) => {}
                Err(e) => self.log(format!("control file ignored: {e}")),
            }

            self.tick();
            thread::sleep(poll);
        };

        self.shutdown(reason);
        Ok(reason)
    }

    /// One pass of the poll loop. Public (hidden) so tests can step the
    /// supervisor deterministically instead of racing a thread.
    #[doc(hidden)]
    pub fn tick(&mut self) {
        self.reap_exits();
        self.check_display_changes();
        self.fire_due_restart();
        self.reset_recovered_backoffs();
    }

    /// Start the component set without entering the poll loop, for stepping
    /// tests that drive `tick()` by hand.
    #[doc(hidden)]
    pub fn start_now(&mut self) {
        self.start_set(self.active.clone(), false);
    }

    /// Pretend a display change happened and its debounce window has passed.
    /// The window plumbing has its own tests; this exercises the response.
    #[doc(hidden)]
    pub fn simulate_display_change(&mut self) {
        self.display_event_at =
            Some(Instant::now() - Duration::from_millis(DISPLAY_DEBOUNCE_MS + 1));
    }

    /// Stop everything the way `run()` would on a control-file request.
    #[doc(hidden)]
    pub fn shutdown_now(&mut self) {
        self.shutdown(StopReason::ControlFile);
    }

    /// Hand the supervisor an already-running component, for tests that need a
    /// handle whose failures can be scripted.
    #[doc(hidden)]
    pub fn inject_running(&mut self, c: Component, handle: Box<dyn ProcessHandle>) {
        let pid = handle.pid();
        self.children.insert(
            c,
            RunningChild {
                handle,
                started: Instant::now(),
            },
        );
        self.health_mut(c).set(Status::Running, Some(pid));
        self.write_health();
    }

    // -- child lifecycle ---------------------------------------------------

    /// Returns false if the component could not be brought up.
    fn start_component(&mut self, c: Component, is_restart: bool) -> bool {
        let Some(ccfg) = c.config(&self.cfg).cloned() else {
            // Unreachable via the active set; a bug elsewhere must not panic
            // the process holding the desktop together.
            self.log(format!("{c} has no configuration — not starting it"));
            return false;
        };

        // Adopt-first: the exact executable already running means the desktop
        // is already using it — starting a second instance or bouncing the
        // first is precisely what taking over a live machine must not do.
        if ccfg.adopt {
            match procs::pids_for_image_path(&ccfg.command) {
                Ok(pids) if !pids.is_empty() => {
                    if pids.len() > 1 {
                        self.log(format!(
                            "{c}: {} instances of {} are running ({pids:?}) — adopting the first",
                            pids.len(),
                            ccfg.command
                        ));
                    }
                    match ExternalProcess::open(pids[0], ccfg.task.clone()) {
                        Ok(ext) => {
                            return self.record_started(
                                c,
                                RunningChild {
                                    handle: Box::new(ext),
                                    started: Instant::now(),
                                },
                                is_restart,
                                "adopted — already running; output not captured until its next restart",
                            );
                        }
                        Err(e) => {
                            // Running but unwatchable: starting another anyway
                            // would put two instances on the desktop. Fail the
                            // start and let the backoff retry.
                            self.health_mut(c).set(Status::Stopped, None);
                            self.log(format!(
                                "{c} FAILED to start: pid {} is running but cannot be watched ({e})",
                                pids[0]
                            ));
                            return false;
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => self.log(format!(
                    "{c}: could not scan for a running instance ({e}) — starting fresh"
                )),
            }
        }

        if let Some(task) = ccfg.task.clone() {
            return self.start_via_task(c, &ccfg, &task, is_restart);
        }

        let log = Arc::clone(&self.child_logs[&c]);
        match process::spawn(&ccfg, log) {
            Ok(running) => self.record_started(c, running, is_restart, &process::render(&ccfg)),
            // 740 = ERROR_ELEVATION_REQUIRED: a UIAccess manifest (GlazeWM
            // ships asInvoker + uiAccess=true) refuses plain CreateProcess.
            // The shell path launches it un-elevated; output is not captured.
            Err(e) if e.raw_os_error() == Some(740) => match process::shell_spawn(&ccfg) {
                Ok(running) => self.record_started(
                    c,
                    running,
                    is_restart,
                    &format!(
                        "{} — via shell (UIAccess manifest); output not captured",
                        process::render(&ccfg)
                    ),
                ),
                Err(e) => {
                    self.health_mut(c).set(Status::Stopped, None);
                    self.log(format!(
                        "{c} FAILED to start via the shell fallback: {e} — command was {}",
                        process::render(&ccfg)
                    ));
                    false
                }
            },
            Err(e) => {
                self.health_mut(c).set(Status::Stopped, None);
                self.log(format!(
                    "{c} FAILED to start: {e} — command was {}",
                    process::render(&ccfg)
                ));
                false
            }
        }
    }

    /// `schtasks /run`, then find the process the task started. The scheduler
    /// hands back no pid, so the exact image path is the identity.
    fn start_via_task(
        &mut self,
        c: Component,
        ccfg: &ComponentConfig,
        task: &str,
        is_restart: bool,
    ) -> bool {
        let run = process::run_to_completion(
            "schtasks",
            &["/run".to_string(), "/tn".to_string(), task.to_string()],
            Duration::from_millis(TASK_START_DISCOVER_MS),
        );
        match run {
            Ok(Some(s)) if s.success() => {}
            Ok(Some(s)) => {
                self.health_mut(c).set(Status::Stopped, None);
                self.log(format!(
                    "{c} FAILED to start: schtasks /run /tn {task} exited with {s}"
                ));
                return false;
            }
            Ok(None) => {
                self.health_mut(c).set(Status::Stopped, None);
                self.log(format!(
                    "{c} FAILED to start: schtasks /run /tn {task} did not finish in time"
                ));
                return false;
            }
            Err(e) => {
                self.health_mut(c).set(Status::Stopped, None);
                self.log(format!(
                    "{c} FAILED to start: schtasks /run /tn {task} failed to run: {e}"
                ));
                return false;
            }
        }

        let deadline = Instant::now() + Duration::from_millis(TASK_START_DISCOVER_MS);
        loop {
            match procs::pids_for_image_path(&ccfg.command) {
                Ok(pids) if !pids.is_empty() => match ExternalProcess::open(pids[0], Some(task.to_string())) {
                    Ok(ext) => {
                        return self.record_started(
                            c,
                            RunningChild {
                                handle: Box::new(ext),
                                started: Instant::now(),
                            },
                            is_restart,
                            &format!("via task {task}"),
                        );
                    }
                    Err(e) => {
                        self.health_mut(c).set(Status::Stopped, None);
                        self.log(format!(
                            "{c} FAILED to start: task {task} produced pid {} but it cannot be watched ({e})",
                            pids[0]
                        ));
                        return false;
                    }
                },
                Ok(_) => {}
                Err(e) => self.log(format!("{c}: process scan failed ({e}) — retrying")),
            }
            if Instant::now() >= deadline {
                self.health_mut(c).set(Status::Stopped, None);
                self.log(format!(
                    "{c} FAILED to start: task {task} ran but no {} process appeared within {}ms",
                    ccfg.command, TASK_START_DISCOVER_MS
                ));
                return false;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn record_started(
        &mut self,
        c: Component,
        running: RunningChild,
        is_restart: bool,
        how: &str,
    ) -> bool {
        let pid = running.pid();
        self.children.insert(c, running);
        let h = self.health_mut(c);
        h.set(Status::Running, Some(pid));
        if is_restart {
            h.restarts = h.restarts.saturating_add(1);
        }
        let n = self.health[&c].restarts;
        self.log(format!("{c} started (pid {pid}, restarts {n}): {how}"));
        true
    }

    fn reap_exits(&mut self) {
        let mut exited: Vec<(Component, Option<i32>, Duration)> = Vec::new();
        let mut unreadable: Vec<(Component, String, Duration)> = Vec::new();
        let mut healthy: Vec<Component> = Vec::new();
        for (c, running) in self.children.iter_mut() {
            match running.handle.poll() {
                Ok(Poll::Exited(code)) => exited.push((*c, code, running.started.elapsed())),
                Ok(Poll::Running) => healthy.push(*c),
                Err(e) => unreadable.push((*c, e.to_string(), running.started.elapsed())),
            }
        }
        for c in healthy {
            // One good poll clears the run; the limit is for a persistent fault.
            self.wait_errors.remove(&c);
        }
        for (c, code, uptime) in exited {
            self.wait_errors.remove(&c);
            self.children.remove(&c);
            self.on_exit(c, code, uptime);
        }
        for (c, msg, uptime) in unreadable {
            self.on_unreadable_status(c, &msg, uptime);
        }
    }

    /// A child whose status cannot be read. Retry quietly for a while; if it
    /// never clears, stop pretending the component is tracked and put it
    /// through the normal down path so the linked-pair rule still applies.
    fn on_unreadable_status(&mut self, c: Component, msg: &str, uptime: Duration) {
        let n = {
            let slot = self.wait_errors.entry(c).or_insert(0);
            *slot += 1;
            *slot
        };
        match wait_error_action(n) {
            WaitErrorAction::Report => {
                self.log(format!("{c}: cannot read child status: {msg} — retrying"));
            }
            // Silent in between: at a 250ms poll this would be four identical
            // lines a second, which buries the log it is meant to help.
            WaitErrorAction::Quiet => {}
            WaitErrorAction::GiveUp => {
                self.log(format!(
                    "{c}: status unreadable for {n} polls ({msg}) after {} — killing it and treating it as down",
                    timefmt::human_duration(uptime)
                ));
                self.wait_errors.remove(&c);
                self.force_stop(c);
                self.health_mut(c).last_exit_code = None;
                self.plan_recovery(c);
            }
        }
    }

    fn on_exit(&mut self, c: Component, code: Option<i32>, uptime: Duration) {
        let code_str = code.map_or_else(|| "none".to_string(), |c| c.to_string());
        self.log(format!(
            "{c} exited (code {code_str}) after {}",
            timefmt::human_duration(uptime)
        ));
        {
            let h = self.health_mut(c);
            h.set(Status::Stopped, None);
            h.last_exit_code = code;
        }
        self.plan_recovery(c);
    }

    /// Decide what happens now that `c` is down. Shared by a clean exit and by
    /// a child we had to write off.
    fn plan_recovery(&mut self, c: Component) {
        if self.shutting_down {
            self.write_health();
            return;
        }

        match c {
            // Linked pair, the dangerous direction. kanata goes down first and
            // without waiting: every millisecond it outlives GlazeWM is a
            // millisecond of raw Win+letter reaching Windows.
            Component::Glazewm => {
                self.log(
                    "linked-pair: glazewm is down — stopping kanata now (raw Win-shortcut guard)",
                );
                self.force_stop(Component::Kanata);
                let delay = self.backoff_mut(Component::Glazewm).next_delay();
                self.schedule(vec![Component::Kanata, Component::Glazewm], delay, true);
                self.log(format!(
                    "restarting the pair in {delay:?} (kanata, then glazewm)"
                ));
            }
            // kanata alone: GlazeWM keeps tiling, chords come back in seconds.
            // The bar likewise restarts alone — a missing bar is cosmetic.
            Component::Kanata | Component::Zebar => {
                if self.pending.is_some() {
                    self.log(format!("{c} restart already scheduled — leaving it be"));
                } else {
                    let delay = self.backoff_mut(c).next_delay();
                    self.schedule(vec![c], delay, true);
                    self.log(format!("restarting {c} in {delay:?}"));
                }
            }
        }
        self.write_health();
    }

    fn schedule(&mut self, order: Vec<Component>, delay: Duration, is_restart: bool) {
        for c in &order {
            // A component that is still up is not restarting. Saying otherwise
            // puts a lie in health.json, which is what Zebar renders.
            if !self.children.contains_key(c) {
                self.health_mut(*c).set(Status::Restarting, None);
            }
        }
        self.pending = Some(PendingRestart {
            at: Instant::now() + delay,
            order,
            is_restart,
        });
    }

    /// Start a set in order, all or none.
    ///
    /// Starting GlazeWM after kanata failed to come up is precisely the state
    /// the linked-pair rule exists to prevent. So a failure part-way through
    /// does two things: it abandons the components after the failure, and it
    /// rolls back the ones this pass already started. Rolling back is not
    /// tidiness — without it the retry respawns on top of a live child,
    /// `children.insert` drops the old handle, and `Child`'s drop does not
    /// kill, leaving a second kanata holding the keyboard forever.
    fn start_set(&mut self, order: Vec<Component>, is_restart: bool) {
        let mut started_here: Vec<Component> = Vec::new();
        for (i, c) in order.iter().copied().enumerate() {
            if self.children.contains_key(&c) {
                // Belt and braces against a future caller: never spawn a second
                // instance of something already tracked.
                self.log(format!("{c} is already running — not starting another"));
                continue;
            }
            if self.start_component(c, is_restart) {
                started_here.push(c);
                continue;
            }

            let rest = &order[i + 1..];
            if !rest.is_empty() {
                let skipped: Vec<&str> = rest.iter().map(|c| c.as_str()).collect();
                self.log(format!(
                    "not starting {} — {c} would not come up, and the pair starts together",
                    skipped.join(", ")
                ));
                for c in rest {
                    self.health_mut(*c).set(Status::Stopped, None);
                }
            }
            for done in started_here.into_iter().rev() {
                self.log(format!(
                    "linked-pair: rolling {done} back — the set could not come up whole"
                ));
                self.force_stop(done);
            }

            let delay = self.backoff_mut(c).next_delay();
            self.log(format!("retrying the whole set in {delay:?}"));
            self.schedule(order.clone(), delay, is_restart);
            self.write_health();
            return;
        }
        self.write_health();
    }

    fn fire_due_restart(&mut self) {
        let Some(p) = self.pending.as_ref() else {
            return;
        };
        if Instant::now() < p.at {
            return;
        }
        let (order, is_restart) = (p.order.clone(), p.is_restart);
        self.pending = None;
        self.start_set(order, is_restart);
    }

    /// A component that has held together for `healthy_reset_secs` counts as
    /// recovered, so the next crash restarts in a second rather than thirty.
    fn reset_recovered_backoffs(&mut self) {
        let threshold = Duration::from_secs(self.cfg.supervisor.healthy_reset_secs);
        let recovered: Vec<Component> = self
            .children
            .iter()
            .filter(|(c, r)| r.started.elapsed() >= threshold && !self.backoff[c].is_reset())
            .map(|(c, _)| *c)
            .collect();
        for c in recovered {
            self.backoff_mut(c).reset();
            self.log(format!(
                "{c} healthy for {} — backoff reset",
                timefmt::human_duration(threshold)
            ));
        }
    }

    /// Kill, confirm, log, record. The tail of both stop paths. `patience` is
    /// how long the kill gets to confirm death — bounded by the caller when a
    /// shutdown budget is running.
    fn kill_and_record(&mut self, c: Component, mut running: RunningChild, patience: Duration) {
        let pid = running.pid();
        match running.handle.kill(patience) {
            Ok(()) => self.log(format!("{c} stopped (pid {pid})")),
            Err(e) => self.log(format!("{c} (pid {pid}) could not be killed: {e}")),
        }
        self.health_mut(c).set(Status::Stopped, None);
    }

    /// Kill now, no grace period. The crash-recovery path: the desktop is
    /// already in a bad state and speed is the whole point.
    fn force_stop(&mut self, c: Component) {
        let Some(running) = self.children.remove(&c) else {
            self.health_mut(c).set(Status::Stopped, None);
            return;
        };
        self.kill_and_record(c, running, Duration::from_millis(KILL_PATIENCE_MS));
    }

    /// Ask nicely, then insist. GlazeWM's `wm-exit` lets its watcher restore
    /// every window; killing it skips that and leaves the desktop tiled.
    ///
    /// `deadline` bounds the whole shutdown. When one is set, each wait takes
    /// at most half the time left, so asking politely can never eat the budget
    /// that the kill and the next component still need.
    fn stop_gracefully(&mut self, c: Component, deadline: Option<Instant>) {
        let Some(mut running) = self.children.remove(&c) else {
            self.health_mut(c).set(Status::Stopped, None);
            return;
        };
        let pid = running.pid();
        let ccfg = c.config(&self.cfg).cloned().unwrap_or_default();
        let configured = Duration::from_millis(self.cfg.supervisor.stop_timeout_ms);
        let left = || deadline.map(|d| d.saturating_duration_since(Instant::now()));

        if let Some(stop_cmd) = ccfg.stop_command.as_deref() {
            let ask = bounded(configured, left());
            self.log(format!("asking {c} to exit: {stop_cmd}"));
            match process::run_to_completion(stop_cmd, &ccfg.stop_args, ask) {
                Ok(Some(s)) if !s.success() => {
                    self.log(format!("{c} stop command exited with {s}"))
                }
                Ok(None) => self.log(format!("{c} stop command did not finish in time")),
                Ok(Some(_)) => {}
                Err(e) => self.log(format!("{c} stop command failed to run: {e}")),
            }
            let settle = bounded(configured, left());
            if process::wait_for_exit(running.handle.as_mut(), settle).is_some() {
                self.log(format!("{c} exited cleanly (pid {pid})"));
                self.health_mut(c).set(Status::Stopped, None);
                return;
            }
            self.log(format!(
                "{c} still up after {} — killing it",
                timefmt::human_duration(settle)
            ));
        }

        // Floor the confirmation window: a fully spent budget must still leave
        // the kill enough time to observe the death it just caused.
        let patience =
            bounded(Duration::from_millis(KILL_PATIENCE_MS), left()).max(Duration::from_millis(100));
        self.kill_and_record(c, running, patience);
    }

    fn shutdown(&mut self, reason: StopReason) {
        self.shutting_down = true;
        self.pending = None;
        let budget = reason.shutdown_budget();
        let deadline = budget.map(|b| Instant::now() + b);
        self.log(format!(
            "shutdown requested ({}) — stopping components in reverse order{}",
            reason.as_str(),
            match budget {
                // Console shutdowns run against the OS clock, not the config.
                Some(b) => format!(" within {b:?}"),
                None => String::new(),
            }
        ));
        for c in self.active.clone().into_iter().rev() {
            self.stop_gracefully(c, deadline);
        }
        if let Some(prior) = self.prior_taskbar.take() {
            taskbar::set_state(prior);
            self.log(format!("taskbar state restored ({prior})"));
        }
        self.write_health();
        self.log("shutdown complete");
        signal::mark_finished();
    }

    /// Display changes: always logged; the pair bounce only when configured
    /// (glazewm#1233 insurance) and only after the event burst goes quiet.
    fn check_display_changes(&mut self) {
        let events = match &self.display_watch {
            Some(w) => w.take_changes(),
            None => 0,
        };
        if events > 0 {
            self.log(format!(
                "display change detected ({events} event{})",
                if events == 1 { "" } else { "s" }
            ));
            self.display_event_at = Some(Instant::now());
        }

        let Some(at) = self.display_event_at else {
            return;
        };
        if at.elapsed() < Duration::from_millis(DISPLAY_DEBOUNCE_MS) {
            return;
        }
        self.display_event_at = None;

        if !self.cfg.supervisor.bounce_on_display_change {
            return;
        }
        if self.pending.is_some() {
            self.log("display change: a restart is already scheduled — leaving it be");
            return;
        }
        self.log("display change: bouncing the pair (bounce_on_display_change is on)");
        // Down in the safe order — kanata first, so it never outlives GlazeWM
        // synthesizing Win presses nothing consumes — then GlazeWM politely, so
        // its watcher restores window positions. The bar is not bounced: zebar
        // handles monitor changes itself.
        self.force_stop(Component::Kanata);
        self.stop_gracefully(Component::Glazewm, None);
        self.schedule(
            vec![Component::Kanata, Component::Glazewm],
            Duration::ZERO,
            true,
        );
        self.write_health();
    }

    // -- state -------------------------------------------------------------

    fn health_mut(&mut self, c: Component) -> &mut ComponentHealth {
        self.health
            .get_mut(&c)
            .expect("every component is seeded in new()")
    }

    fn backoff_mut(&mut self, c: Component) -> &mut Backoff {
        self.backoff
            .get_mut(&c)
            .expect("every component is seeded in new()")
    }

    /// Snapshot for `winsome status` and Zebar. Written on every transition.
    pub fn snapshot(&self) -> Health {
        let components = self
            .health
            .iter()
            .map(|(c, h)| (c.as_str().to_string(), h.clone()))
            .collect();
        Health::new(
            SupervisorHealth {
                running: !self.shutting_down,
                pid: std::process::id(),
                since: timefmt::iso8601(self.started_at),
            },
            components,
        )
    }

    fn write_health(&self) {
        if let Err(e) = self.snapshot().write(&self.paths.health()) {
            self.log(format!("could not write health state: {e}"));
        }
    }

    fn log(&self, msg: impl AsRef<str>) {
        let mut g = self.log.lock().unwrap_or_else(|p| p.into_inner());
        let _ = g.write_line(msg.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SupervisorConfig;

    #[test]
    fn a_console_shutdown_fits_inside_the_os_grace_period() {
        let budget = StopReason::ConsoleSignal
            .shutdown_budget()
            .expect("a console shutdown is budgeted");
        let configured = Duration::from_millis(SupervisorConfig::default().stop_timeout_ms);
        assert!(
            configured > budget,
            "if the configured timeout already fit, the budget would be pointless"
        );

        // Worst case: every wait times out in full. Six is more waits than the
        // shutdown ever performs (stop command + settle + kill confirmation,
        // for each component).
        let mut remaining = budget;
        let mut spent = Duration::ZERO;
        for _ in 0..6 {
            let w = bounded(configured, Some(remaining));
            spent += w;
            remaining -= w;
        }
        assert!(spent < budget, "spent {spent:?} of {budget:?}");
        assert!(
            spent + Duration::from_millis(CONSOLE_SHUTDOWN_HEADROOM_MS)
                <= Duration::from_millis(signal::GRACE_MS),
            "the worst-case shutdown must leave room for the final health write"
        );
    }

    #[test]
    fn a_control_file_shutdown_keeps_the_configured_timeouts() {
        assert_eq!(StopReason::ControlFile.shutdown_budget(), None);
        let configured = Duration::from_secs(5);
        assert_eq!(bounded(configured, None), configured);
    }

    #[test]
    fn a_bounded_wait_takes_at_most_half_of_what_is_left() {
        let five = Duration::from_secs(5);
        assert_eq!(
            bounded(five, Some(Duration::from_millis(3_000))),
            Duration::from_millis(1_500)
        );
        // The configured timeout still wins when it is the smaller of the two.
        assert_eq!(
            bounded(Duration::from_millis(100), Some(Duration::from_secs(10))),
            Duration::from_millis(100)
        );
        // Out of budget: stop waiting, do not go negative or block.
        assert_eq!(bounded(five, Some(Duration::ZERO)), Duration::ZERO);
    }

    #[test]
    fn unreadable_status_reports_once_then_gives_up() {
        assert_eq!(wait_error_action(1), WaitErrorAction::Report);
        for n in 2..WAIT_ERROR_LIMIT {
            assert_eq!(wait_error_action(n), WaitErrorAction::Quiet, "poll {n}");
        }
        assert_eq!(wait_error_action(WAIT_ERROR_LIMIT), WaitErrorAction::GiveUp);
        assert_eq!(
            wait_error_action(WAIT_ERROR_LIMIT + 50),
            WaitErrorAction::GiveUp
        );
    }
}
