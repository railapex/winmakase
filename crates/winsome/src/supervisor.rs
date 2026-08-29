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
use crate::config::Config;
use crate::control;
use crate::health::{ComponentHealth, Health, Status, SupervisorHealth};
use crate::paths::Paths;
use crate::process::{self, RunningChild};
use crate::rolling_log::RollingLog;
use crate::signal;
use crate::timefmt;

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
    log: Arc<Mutex<RollingLog>>,
    child_logs: BTreeMap<Component, Arc<Mutex<RollingLog>>>,
    children: BTreeMap<Component, RunningChild>,
    backoff: BTreeMap<Component, Backoff>,
    health: BTreeMap<Component, ComponentHealth>,
    pending: Option<PendingRestart>,
    started_at: SystemTime,
    shutting_down: bool,
}

impl Supervisor {
    pub fn new(cfg: Config, paths: Paths) -> io::Result<Self> {
        paths.ensure_dirs()?;
        let (max_bytes, keep) = (cfg.supervisor.log_max_bytes, cfg.supervisor.log_keep_files);
        let log = Arc::new(Mutex::new(RollingLog::open(
            paths.log_for("supervisor"),
            max_bytes,
            keep,
        )?));

        let mut child_logs = BTreeMap::new();
        let mut backoff = BTreeMap::new();
        let mut health = BTreeMap::new();
        for c in Component::START_ORDER {
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

        Ok(Self {
            cfg,
            paths,
            log,
            child_logs,
            children: BTreeMap::new(),
            backoff,
            health,
            pending: None,
            started_at: SystemTime::now(),
            shutting_down: false,
        })
    }

    /// Start the set and watch it until told to stop. Returns why it stopped.
    pub fn run(&mut self) -> io::Result<StopReason> {
        self.log(format!(
            "supervisor starting (pid {}) home={}",
            std::process::id(),
            self.paths.home().display()
        ));
        // Anything left in the control file predates us and is not our order.
        control::clear(&self.paths)?;

        self.start_set(Component::START_ORDER.to_vec(), false);

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

            self.reap_exits();
            self.fire_due_restart();
            self.reset_recovered_backoffs();
            thread::sleep(poll);
        };

        self.shutdown(reason);
        Ok(reason)
    }

    // -- child lifecycle ---------------------------------------------------

    /// Returns false if the component could not be spawned.
    fn start_component(&mut self, c: Component, is_restart: bool) -> bool {
        let ccfg = c.config(&self.cfg).clone();
        let log = Arc::clone(&self.child_logs[&c]);
        match process::spawn(&ccfg, log) {
            Ok(running) => {
                let pid = running.pid;
                self.children.insert(c, running);
                let h = self.health_mut(c);
                h.set(Status::Running, Some(pid));
                if is_restart {
                    h.restarts = h.restarts.saturating_add(1);
                }
                let n = self.health[&c].restarts;
                self.log(format!(
                    "{c} started (pid {pid}, restarts {n}): {}",
                    process::render(&ccfg)
                ));
                true
            }
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

    fn reap_exits(&mut self) {
        let mut exited: Vec<(Component, Option<i32>, Duration)> = Vec::new();
        for (c, running) in self.children.iter_mut() {
            match running.child.try_wait() {
                Ok(Some(status)) => exited.push((*c, status.code(), running.started.elapsed())),
                Ok(None) => {}
                Err(e) => {
                    // Losing track of a child is itself a reportable failure.
                    let msg = format!("{c}: cannot read child status: {e}");
                    let mut g = self.log.lock().unwrap_or_else(|p| p.into_inner());
                    let _ = g.write_line(&msg);
                }
            }
        }
        for (c, code, uptime) in exited {
            self.children.remove(&c);
            self.on_exit(c, code, uptime);
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
            Component::Kanata => {
                if self.pending.is_some() {
                    self.log("kanata restart already scheduled — leaving it be");
                } else {
                    let delay = self.backoff_mut(Component::Kanata).next_delay();
                    self.schedule(vec![Component::Kanata], delay, true);
                    self.log(format!("restarting kanata in {delay:?}"));
                }
            }
        }
        self.write_health();
    }

    fn schedule(&mut self, order: Vec<Component>, delay: Duration, is_restart: bool) {
        for c in &order {
            self.health_mut(*c).set(Status::Restarting, None);
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
    /// the linked-pair rule exists to prevent, so a failure part-way through
    /// abandons the rest and reschedules the whole set.
    fn start_set(&mut self, order: Vec<Component>, is_restart: bool) {
        for (i, c) in order.iter().enumerate() {
            if self.start_component(*c, is_restart) {
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
            let delay = self.backoff_mut(*c).next_delay();
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

    /// Kill now, no grace period. The crash-recovery path: the desktop is
    /// already in a bad state and speed is the whole point.
    fn force_stop(&mut self, c: Component) {
        let Some(mut running) = self.children.remove(&c) else {
            self.health_mut(c).set(Status::Stopped, None);
            return;
        };
        let pid = running.pid;
        match running.child.kill() {
            Ok(()) => {
                let _ = running.child.wait();
                self.log(format!("{c} stopped (pid {pid})"));
            }
            Err(e) => self.log(format!("{c} (pid {pid}) could not be killed: {e}")),
        }
        self.health_mut(c).set(Status::Stopped, None);
    }

    /// Ask nicely, then insist. GlazeWM's `wm-exit` lets its watcher restore
    /// every window; killing it skips that and leaves the desktop tiled.
    fn stop_gracefully(&mut self, c: Component) {
        let Some(mut running) = self.children.remove(&c) else {
            self.health_mut(c).set(Status::Stopped, None);
            return;
        };
        let pid = running.pid;
        let ccfg = c.config(&self.cfg).clone();
        let timeout = Duration::from_millis(self.cfg.supervisor.stop_timeout_ms);

        if let Some(stop_cmd) = ccfg.stop_command.as_deref() {
            self.log(format!("asking {c} to exit: {stop_cmd}"));
            match process::run_to_completion(stop_cmd, &ccfg.stop_args, timeout) {
                Ok(Some(s)) if !s.success() => {
                    self.log(format!("{c} stop command exited with {s}"))
                }
                Ok(None) => self.log(format!("{c} stop command did not finish in time")),
                Ok(Some(_)) => {}
                Err(e) => self.log(format!("{c} stop command failed to run: {e}")),
            }
            if process::wait_for_exit(&mut running.child, timeout).is_some() {
                self.log(format!("{c} exited cleanly (pid {pid})"));
                self.health_mut(c).set(Status::Stopped, None);
                return;
            }
            self.log(format!(
                "{c} still up after {} — killing it",
                timefmt::human_duration(timeout)
            ));
        }

        match running.child.kill() {
            Ok(()) => {
                let _ = running.child.wait();
                self.log(format!("{c} stopped (pid {pid})"));
            }
            Err(e) => self.log(format!("{c} (pid {pid}) could not be killed: {e}")),
        }
        self.health_mut(c).set(Status::Stopped, None);
    }

    fn shutdown(&mut self, reason: StopReason) {
        self.shutting_down = true;
        self.pending = None;
        self.log(format!(
            "shutdown requested ({}) — stopping components in reverse order",
            reason.as_str()
        ));
        for c in Component::START_ORDER.iter().rev().copied() {
            self.stop_gracefully(c);
        }
        self.write_health();
        self.log("shutdown complete");
        signal::mark_finished();
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
