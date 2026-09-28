use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use clap::builder::PossibleValue;
use clap::{Parser, Subcommand, ValueEnum};

use winmakase::commands;
use winmakase::component::Component;
use winmakase::config::{Config, InputMode};
use winmakase::control;
use winmakase::health::Health;
use winmakase::paths::Paths;
use winmakase::proc_alive;
use winmakase::signal;
use winmakase::stub::{self, StubArgs};
use winmakase::supervisor::Supervisor;

#[derive(Parser)]
#[command(
    name = "winmakase",
    version,
    about = "Winmakase — omakase omarchy-style desktop for Windows 11",
    long_about = None
)]
struct Cli {
    /// Winmakase home directory. Defaults to $WINMAKASE_HOME, else ~/.winmakase.
    #[arg(long, global = true, value_name = "DIR")]
    home: Option<PathBuf>,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the supervisor in the foreground: start the components and keep them
    /// healthy. Ctrl+C shuts the stack down cleanly.
    Supervise {
        /// Config file. Defaults to <home>/config.toml, created if absent.
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },

    /// Show what the supervisor believes about each component.
    Status {
        /// Print the raw health file instead of the table.
        #[arg(long)]
        json: bool,
    },

    /// Tail a component's log.
    Logs {
        component: LogTarget,
        /// Number of lines to show.
        #[arg(short = 'n', long = "lines", default_value_t = 50, value_name = "N")]
        lines: usize,
    },

    /// Ask a running supervisor to stop.
    Down {
        /// How long to wait for the supervisor to acknowledge.
        #[arg(long, default_value_t = 15, value_name = "SECONDS")]
        timeout: u64,
    },

    /// Ask a running supervisor to re-read config.toml, applying what it can
    /// live and bouncing only the components whose configuration changed.
    Reload {
        /// How long to wait for the supervisor to act.
        #[arg(long, default_value_t = 30, value_name = "SECONDS")]
        timeout: u64,
    },

    /// Validate the keymap file or render its GlazeWM keybindings YAML.
    Keymap {
        #[command(subcommand)]
        action: KeymapAction,
    },

    /// Render the kanata .kbd config for the `[keyboard]` mode in config.toml.
    Kanata {
        #[command(subcommand)]
        action: KanataAction,
    },

    /// Open or focus PowerToys Run without toggling an open launcher closed.
    Launcher {
        #[command(subcommand)]
        action: LauncherAction,
    },

    /// Explain a documented gap — bound to chords whose omarchy feature the
    /// stack cannot provide, so the key explains itself instead of dying
    /// silently.
    Gap {
        #[arg(value_enum)]
        name: GapName,
    },

    /// Re-orient the focused row of windows in place (omarchy Super+J).
    Reflow {
        /// Print the move plan without executing it.
        #[arg(long)]
        dry_run: bool,
    },

    /// Float a tiled window; return any non-tiled window directly to tiling.
    ToggleFloating,

    /// Toggle a named scratchpad window (see [scratchpad.*] in config.toml).
    Scratchpad {
        #[command(subcommand)]
        action: ScratchpadAction,
    },

    /// Stand-in child process for supervisor tests. Not a user-facing verb.
    #[command(name = "_stub", hide = true)]
    Stub {
        #[arg(long, default_value_t = 200)]
        tick_ms: u64,
        #[arg(long)]
        exit_after_ms: Option<u64>,
        #[arg(long, default_value_t = 0)]
        code: i32,
        #[arg(long)]
        stderr: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum GapName {
    /// Window grouping / tabs (omarchy Super+G family).
    Grouping,
}

#[derive(Subcommand)]
enum ScratchpadAction {
    /// Summon the named window from the scratch workspace, banish it there,
    /// or launch it — whichever applies.
    Toggle { name: String },
    /// The anonymous rwin+s key: rescue an unpresentable focused window, else
    /// pull the newest window out of scratch, else quietly do nothing.
    Summon,
}

#[derive(Subcommand)]
enum KanataAction {
    /// Print (or write) the .kbd for the configured keyboard mode.
    Render {
        /// Config file. Defaults to <home>/config.toml, created if absent.
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
        /// Write to a file instead of stdout.
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum LauncherAction {
    /// Observe PowerToys Run, then focus it or signal its invoke event once.
    Open {
        /// Print a machine-readable result (including structured failures).
        #[arg(long)]
        json: bool,
        /// Total readiness/activation deadline after acquiring the caller lock.
        #[arg(long, default_value_t = 1500, value_name = "MILLISECONDS")]
        timeout_ms: u64,
    },
}

#[derive(Subcommand)]
enum KeymapAction {
    /// Check structural rules, chord conflicts, and print coverage.
    Check {
        /// Keymap file. Defaults to <home>/keymap/omarchy.toml.
        #[arg(long, value_name = "FILE")]
        keymap: Option<PathBuf>,
        /// Local overrides file. Defaults to <home>/keymap/local.toml,
        /// merged only when it exists.
        #[arg(long, value_name = "FILE")]
        local: Option<PathBuf>,
    },
    /// Render GlazeWM bindings, optionally composed with a binding-free base.
    Render {
        /// Keymap file. Defaults to <home>/keymap/omarchy.toml.
        #[arg(long, value_name = "FILE")]
        keymap: Option<PathBuf>,
        /// Local overrides file. Defaults to <home>/keymap/local.toml,
        /// merged only when it exists.
        #[arg(long, value_name = "FILE")]
        local: Option<PathBuf>,
        /// Product config containing `[apps]`. Defaults to
        /// <home>/config.toml, created if absent. An explicit path must exist.
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
        /// Binding-free GlazeWM base. When set, render a complete config;
        /// without it, render only the `keybindings:` section.
        #[arg(long, value_name = "FILE")]
        base: Option<PathBuf>,
        /// Write to a file instead of stdout.
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
        /// Preview override for the physical SUPER owner. Without this flag,
        /// `[keyboard].input_mode` is authoritative.
        #[arg(long, value_enum)]
        super_key: Option<SuperKeyArg>,
    },
}

#[derive(Copy, Clone, Default, ValueEnum)]
enum SuperKeyArg {
    /// Caps is mapped to right-Win by kanata; physical left Win stays native.
    #[default]
    RightWin,
    /// Patched GlazeWM owns Caps directly as its leader.
    CapsLock,
    /// A registry, PowerToys or firmware remap sends F13 for Caps.
    F13,
}

impl From<SuperKeyArg> for InputMode {
    fn from(value: SuperKeyArg) -> Self {
        match value {
            SuperKeyArg::RightWin => Self::Kanata,
            SuperKeyArg::CapsLock => Self::DirectCaps,
            SuperKeyArg::F13 => Self::F13,
        }
    }
}

/// A `logs` target: any supervised component, plus the supervisor's own log.
///
/// The component names come from [`Component`] rather than being spelled again
/// here, so renaming one cannot leave the CLI pointing at a log file that no
/// longer exists.
#[derive(Copy, Clone, PartialEq, Eq)]
enum LogTarget {
    Component(Component),
    Supervisor,
}

impl LogTarget {
    const VARIANTS: [LogTarget; 4] = [
        LogTarget::Component(Component::Kanata),
        LogTarget::Component(Component::Glazewm),
        LogTarget::Component(Component::Zebar),
        LogTarget::Supervisor,
    ];

    fn as_str(self) -> &'static str {
        match self {
            LogTarget::Component(c) => c.as_str(),
            LogTarget::Supervisor => "supervisor",
        }
    }
}

// Add a component and this stops compiling until `logs` can reach its log.
const _: () = assert!(
    LogTarget::VARIANTS.len() == Component::START_ORDER.len() + 1,
    "every component needs a logs target, plus one for the supervisor"
);

impl ValueEnum for LogTarget {
    fn value_variants<'a>() -> &'a [Self] {
        &Self::VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.as_str()))
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // The stub is a child process, not a Winmakase command; it needs no home.
    if let Cmd::Stub {
        tick_ms,
        exit_after_ms,
        code,
        stderr,
    } = cli.command
    {
        let rc = stub::run(StubArgs {
            tick_ms,
            exit_after_ms,
            code,
            stderr,
        });
        return ExitCode::from(rc.clamp(0, 255) as u8);
    }

    let paths = match cli.home {
        Some(dir) => Paths::at(dir),
        None => match Paths::resolve() {
            Ok(p) => p,
            Err(e) => return fail(e),
        },
    };

    let result = match cli.command {
        Cmd::Supervise { config } => supervise(&paths, config),
        Cmd::Status { json } => status(&paths, json),
        Cmd::Logs { component, lines } => logs(&paths, component, lines),
        Cmd::Down { timeout } => return down(&paths, timeout),
        Cmd::Reload { timeout } => return reload(&paths, timeout),
        Cmd::Keymap { action } => keymap_cmd(&paths, action),
        Cmd::Kanata { action } => kanata_cmd(&paths, action),
        Cmd::Launcher { action } => return launcher_cmd(action),
        Cmd::Gap { name } => {
            gap_cmd(name);
            Ok(())
        }
        Cmd::Reflow { dry_run } => reflow_cmd(&paths, dry_run),
        Cmd::ToggleFloating => toggle_floating_cmd(&paths),
        Cmd::Scratchpad { action } => scratchpad_cmd(&paths, action),
        Cmd::Stub { .. } => unreachable!("handled above"),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(e),
    }
}

fn launcher_cmd(action: LauncherAction) -> ExitCode {
    match action {
        LauncherAction::Open { json, timeout_ms } => {
            let options = winmakase::launcher::OpenOptions {
                activation_timeout: Duration::from_millis(timeout_ms),
                ..Default::default()
            };
            match winmakase::launcher::open_run(options) {
                Ok(report) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string(&report).expect("launcher report serializes")
                        );
                    } else {
                        println!(
                            "PowerToys Run: {:?} ({} ms)",
                            report.outcome, report.elapsed_ms
                        );
                    }
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string(&error).expect("launcher error serializes")
                        );
                    } else {
                        eprintln!("winmakase: {error}");
                    }
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn fail(e: io::Error) -> ExitCode {
    eprintln!("winmakase: {e}");
    ExitCode::FAILURE
}

fn supervise(paths: &Paths, config: Option<PathBuf>) -> io::Result<()> {
    let config_path = config.unwrap_or_else(|| paths.config());
    let cfg = Config::load_or_create(&config_path)?;

    if !signal::install() {
        // Not fatal — a supervisor that cannot catch Ctrl+C is still worth
        // having — but say so, because the clean-shutdown path is gone.
        eprintln!(
            "winmakase: could not install the console control handler; Ctrl+C will not shut down cleanly"
        );
    }

    println!(
        "winmakase supervise — config {}, logs {}",
        config_path.display(),
        paths.logs_dir().display()
    );
    let mut sup = Supervisor::new(cfg, paths.clone())?;
    sup.set_config_path(config_path);
    let reason = sup.run()?;
    println!("winmakase: stopped ({reason:?})");
    Ok(())
}

fn reload(paths: &Paths, timeout_secs: u64) -> ExitCode {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let token = match control::request_reload(paths) {
        Ok(t) => t,
        Err(e) => return fail(e),
    };
    match control::wait_for_ack(paths, Duration::from_secs(timeout_secs)) {
        Ok(true) => loop {
            // The ack means the request was heard; the result file (under our
            // token) is the reload actually being done.
            if let Ok(text) = std::fs::read_to_string(paths.reload_result())
                && text.lines().next() == Some(token.as_str())
            {
                let report: Vec<&str> = text.lines().skip(1).collect();
                for line in &report {
                    println!("winmakase: {line}");
                }
                return if report.iter().any(|l| l.starts_with("REJECTED")) {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                };
            }
            if Instant::now() >= deadline {
                eprintln!("winmakase: reload was heard but no result arrived in time");
                return ExitCode::FAILURE;
            }
            thread::sleep(Duration::from_millis(50));
        },
        Ok(false) => {
            eprintln!(
                "winmakase: no supervisor picked the reload up within {timeout_secs}s — is the stack running?"
            );
            ExitCode::FAILURE
        }
        Err(e) => fail(e),
    }
}

fn status(paths: &Paths, json: bool) -> io::Result<()> {
    let path = paths.health();
    let health = match Health::read(&path) {
        Ok(h) => h,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            println!("{}", commands::no_health_state(&path));
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let alive = proc_alive::is_alive(health.supervisor.pid);

    if json {
        // The health file records transitions, not liveness — a supervisor
        // killed outright leaves a file that reads "running" forever. The
        // table view already checks the pid; machine consumers (the Zebar
        // health dot) need the same truth, added as a sibling field so the
        // schema-1 shape is untouched. `null` = could not tell.
        let mut value = serde_json::to_value(&health)?;
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "supervisorProcessAlive".to_string(),
                serde_json::to_value(alive)?,
            );
        }
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    println!("{}", commands::render_status(&health, &path, alive));
    Ok(())
}

fn logs(paths: &Paths, target: LogTarget, lines: usize) -> io::Result<()> {
    match commands::tail_log(paths, target.as_str(), lines) {
        Ok((text, _)) => {
            println!("{text}");
            Ok(())
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!(
                "winmakase: no log yet for {} ({})",
                target.as_str(),
                paths.log_for(target.as_str()).display()
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn keymap_cmd(paths: &Paths, action: KeymapAction) -> io::Result<()> {
    let invalid = |msg: String| io::Error::new(io::ErrorKind::InvalidData, msg);
    // Loads the stock keymap, then folds in local overrides: an explicit
    // --local path must exist; the default <home>/keymap/local.toml is
    // merged only when present.
    let load = |keymap: Option<PathBuf>,
                local: Option<PathBuf>|
     -> io::Result<(PathBuf, winmakase_keymap::KeymapFile, Option<String>)> {
        let path = keymap.unwrap_or_else(|| paths.home().join("keymap").join("omarchy.toml"));
        let text = std::fs::read_to_string(&path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
        let mut file = winmakase_keymap::parse(&text).map_err(invalid)?;

        let (local_path, required) = match local {
            Some(p) => (p, true),
            None => (paths.home().join("keymap").join("local.toml"), false),
        };
        let mut merged = None;
        if required || local_path.exists() {
            let text = std::fs::read_to_string(&local_path)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", local_path.display())))?;
            let overrides = winmakase_keymap::parse_overrides(&text).map_err(invalid)?;
            let report = winmakase_keymap::merge(&mut file, overrides)
                .map_err(|errors| invalid(errors.join("\n")))?;
            merged = Some(format!("local {} — {report}", local_path.display()));
        }
        Ok((path, file, merged))
    };
    let checked = |file: &winmakase_keymap::KeymapFile| {
        winmakase_keymap::check(file).map_err(|errors| invalid(errors.join("\n")))
    };

    match action {
        KeymapAction::Check { keymap, local } => {
            let (path, file, merged) = load(keymap, local)?;
            let (_, coverage) = checked(&file)?;
            println!("{}", path.display());
            if let Some(note) = merged {
                println!("{note}");
            }
            println!(
                "{}",
                winmakase_keymap::render_coverage(&file.meta, &coverage)
            );
            Ok(())
        }
        KeymapAction::Render {
            keymap,
            local,
            config,
            base,
            out,
            super_key,
        } => {
            let (_, file, _) = load(keymap, local)?;
            let (expanded, _) = checked(&file)?;
            let (cfg, config_source) = match config {
                Some(path) => {
                    let source = path.display().to_string();
                    (Config::load(&path)?, source)
                }
                None => {
                    let path = paths.config();
                    let source = path.display().to_string();
                    (Config::load_or_create(&path)?, source)
                }
            };
            let roles =
                winmakase::app_roles::from_config(&config_source, &cfg.apps).map_err(invalid)?;
            let input_mode = super_key.map_or(cfg.keyboard.input_mode, Into::into);
            let yaml = match base {
                Some(path) => {
                    let text = std::fs::read_to_string(&path).map_err(|e| {
                        io::Error::new(e.kind(), format!("{}: {e}", path.display()))
                    })?;
                    winmakase::app_roles::compose_glazewm(winmakase::app_roles::CompositionInput {
                        base_yaml: &text,
                        theme: None,
                        keymap: &expanded,
                        roles: &roles,
                        input_mode,
                    })
                    .map_err(invalid)?
                }
                None if roles.is_empty() && input_mode != InputMode::DirectCaps => {
                    winmakase_keymap::render_glazewm_with_super(&expanded, input_mode.super_key())
                }
                None if input_mode == InputMode::DirectCaps => {
                    return Err(invalid(
                        "direct-Caps rendering requires --base so the required general.keybinding_leader can be generated"
                            .into(),
                    ));
                }
                None => {
                    return Err(invalid(
                        "configured app window rules require --base so workspaces and existing host rules can be validated"
                            .into(),
                    ));
                }
            };
            match out {
                Some(dest) => std::fs::write(dest, yaml),
                None => {
                    print!("{yaml}");
                    Ok(())
                }
            }
        }
    }
}

fn gap_cmd(name: GapName) {
    match name {
        GapName::Grouping => winmakase::notify::info(
            "Winmakase — window grouping",
            "Window grouping (omarchy's Super+G tabs) is a documented gap: \
             GlazeWM has no grouping/container primitive, and Winmakase cannot \
             fake one from outside the window manager.\n\n\
             The genuine product route is the parked Glazemakaze core track: \
             add the primitive inside Glaze, then expose it here.\n\n\
             Details: docs/plans/glazemakaze-core.md and DESIGN.md § Documented gaps.",
        ),
    }
}

fn kanata_cmd(paths: &Paths, action: KanataAction) -> io::Result<()> {
    match action {
        KanataAction::Render { config, out } => {
            let cfg = Config::load_or_create(&config.unwrap_or_else(|| paths.config()))?;
            let kbd = winmakase::kanata_kbd::render(&cfg.keyboard);
            match out {
                Some(dest) => std::fs::write(dest, kbd),
                None => {
                    print!("{kbd}");
                    Ok(())
                }
            }
        }
    }
}

fn toggle_floating_cmd(paths: &Paths) -> io::Result<()> {
    let cfg = Config::load_or_create(&paths.config())?;
    let client = winmakase::glazewm::Client::from_config(&cfg)?;
    let Some(window) = client.query_focused()?.filter(|node| node.is_window()) else {
        return Ok(());
    };

    // Glaze's native toggle restores prevState, which can cycle forever
    // between floating and fullscreen. This key always toggles against tiling.
    let args: &[&str] = match window.state_type() {
        Some("tiling") => &["set-floating", "--centered"],
        Some("floating" | "fullscreen" | "minimized") => &["set-tiling"],
        state => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("cannot toggle window with unknown state: {state:?}"),
            ));
        }
    };
    // Target the queried window even if focus changes during the round trip.
    client.command_for(&window.id, args)
}

fn reflow_cmd(paths: &Paths, dry_run: bool) -> io::Result<()> {
    let cfg = Config::load_or_create(&paths.config())?;
    let client = winmakase::glazewm::Client::from_config(&cfg)?;
    let workspaces = client.query_workspaces()?;
    let plan = winmakase::reflow::plan(&workspaces)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    println!(
        "reflow: {} -> {} ({} move{})",
        plan.from,
        plan.to,
        plan.moves.len(),
        if plan.moves.len() == 1 { "" } else { "s" }
    );
    for m in &plan.moves {
        println!("  move {} {:?}", m.direction, m.title);
        if !dry_run {
            client.command_for(&m.window_id, &["move", "--direction", m.direction])?;
        }
    }
    if dry_run {
        println!("(dry run — nothing moved)");
    }
    Ok(())
}

fn scratchpad_cmd(paths: &Paths, action: ScratchpadAction) -> io::Result<()> {
    let cfg = Config::load_or_create(&paths.config())?;
    let name = match action {
        ScratchpadAction::Summon => {
            let client = winmakase::glazewm::Client::from_config(&cfg)?;
            let outcome = winmakase::scratchpad::summon_key(&client)?;
            println!("{outcome}");
            return Ok(());
        }
        ScratchpadAction::Toggle { name } => name,
    };
    let Some(pad) = cfg.scratchpad.get(&name) else {
        let known: Vec<&str> = cfg.scratchpad.keys().map(String::as_str).collect();
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "no [scratchpad.{name}] in {} (configured: {})",
                paths.config().display(),
                if known.is_empty() {
                    "none".to_string()
                } else {
                    known.join(", ")
                }
            ),
        ));
    };
    let client = winmakase::glazewm::Client::from_config(&cfg)?;
    let outcome = winmakase::scratchpad::toggle(&client, pad)?;
    println!("{outcome}");
    Ok(())
}

fn down(paths: &Paths, timeout_secs: u64) -> ExitCode {
    // Report what we know before asking, so a stale health file cannot be
    // mistaken for a supervisor that answered.
    match Health::read(&paths.health()) {
        Ok(h) if h.supervisor.running => {
            let pid = h.supervisor.pid;
            match proc_alive::is_alive(pid) {
                Some(false) => println!(
                    "winmakase: health state names supervisor pid {pid}, but that process is gone"
                ),
                _ => println!("winmakase: asking supervisor pid {pid} to stop"),
            }
        }
        Ok(_) => println!("winmakase: health state says no supervisor is running; asking anyway"),
        Err(_) => println!("winmakase: no health state found; asking anyway"),
    }

    // One deadline for the whole operation, hearing AND finishing.
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    if let Err(e) = control::request_stop(paths) {
        return fail(e);
    }
    match control::wait_for_ack(paths, Duration::from_secs(timeout_secs)) {
        Ok(true) => {
            // The ack means the request was heard, not done: the supervisor is
            // still stopping components. Callers act on "down came back", so
            // success has to mean finished (the panic path learned this live —
            // it killed the supervisor mid-shutdown and orphaned the pair).
            println!("winmakase: supervisor acknowledged — waiting for the stack to stop");
            loop {
                match Health::read(&paths.health()) {
                    Ok(h) if !h.supervisor.running => {
                        println!("winmakase: stack stopped");
                        return ExitCode::SUCCESS;
                    }
                    Ok(h) if proc_alive::is_alive(h.supervisor.pid) == Some(false) => {
                        eprintln!(
                            "winmakase: supervisor (pid {}) died mid-shutdown — components may be orphaned",
                            h.supervisor.pid
                        );
                        return ExitCode::FAILURE;
                    }
                    _ => {}
                }
                if Instant::now() >= deadline {
                    eprintln!(
                        "winmakase: still shutting down after {timeout_secs}s — check `winmakase status`"
                    );
                    return ExitCode::FAILURE;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
        Ok(false) => {
            eprintln!(
                "winmakase: nothing picked up the request within {timeout_secs}s — request withdrawn.\n\
                 Is a supervisor running against {}?",
                paths.home().display()
            );
            ExitCode::FAILURE
        }
        Err(e) => fail(e),
    }
}
