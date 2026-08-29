use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use clap::builder::PossibleValue;
use clap::{Parser, Subcommand, ValueEnum};

use winsome::commands;
use winsome::component::Component;
use winsome::config::Config;
use winsome::control;
use winsome::health::Health;
use winsome::paths::Paths;
use winsome::proc_alive;
use winsome::signal;
use winsome::stub::{self, StubArgs};
use winsome::supervisor::Supervisor;

#[derive(Parser)]
#[command(
    name = "winsome",
    version,
    about = "Winsome — omakase omarchy-style desktop for Windows 11",
    long_about = None
)]
struct Cli {
    /// Winsome home directory. Defaults to $WINSOME_HOME, else ~/.winsome.
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

    /// Validate the keymap file or render its GlazeWM keybindings YAML.
    Keymap {
        #[command(subcommand)]
        action: KeymapAction,
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

#[derive(Subcommand)]
enum KeymapAction {
    /// Check structural rules, chord conflicts, and print coverage.
    Check {
        /// Keymap file. Defaults to <home>/keymap/omarchy.toml.
        #[arg(long, value_name = "FILE")]
        keymap: Option<PathBuf>,
    },
    /// Render the GlazeWM `keybindings:` section from the mapped entries.
    Render {
        /// Keymap file. Defaults to <home>/keymap/omarchy.toml.
        #[arg(long, value_name = "FILE")]
        keymap: Option<PathBuf>,
        /// Write to a file instead of stdout.
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
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

    // The stub is a child process, not a Winsome command; it needs no home.
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
        Cmd::Keymap { action } => keymap_cmd(&paths, action),
        Cmd::Stub { .. } => unreachable!("handled above"),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(e),
    }
}

fn fail(e: io::Error) -> ExitCode {
    eprintln!("winsome: {e}");
    ExitCode::FAILURE
}

fn supervise(paths: &Paths, config: Option<PathBuf>) -> io::Result<()> {
    let config_path = config.unwrap_or_else(|| paths.config());
    let cfg = Config::load_or_create(&config_path)?;

    if !signal::install() {
        // Not fatal — a supervisor that cannot catch Ctrl+C is still worth
        // having — but say so, because the clean-shutdown path is gone.
        eprintln!(
            "winsome: could not install the console control handler; Ctrl+C will not shut down cleanly"
        );
    }

    println!(
        "winsome supervise — config {}, logs {}",
        config_path.display(),
        paths.logs_dir().display()
    );
    let reason = Supervisor::new(cfg, paths.clone())?.run()?;
    println!("winsome: stopped ({reason:?})");
    Ok(())
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

    if json {
        println!("{}", serde_json::to_string_pretty(&health)?);
        return Ok(());
    }

    let alive = proc_alive::is_alive(health.supervisor.pid);
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
                "winsome: no log yet for {} ({})",
                target.as_str(),
                paths.log_for(target.as_str()).display()
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn keymap_cmd(paths: &Paths, action: KeymapAction) -> io::Result<()> {
    let load = |explicit: Option<PathBuf>| -> io::Result<(PathBuf, winsome_keymap::KeymapFile)> {
        let path = explicit.unwrap_or_else(|| paths.home().join("keymap").join("omarchy.toml"));
        let text = std::fs::read_to_string(&path).map_err(|e| {
            io::Error::new(e.kind(), format!("{}: {e}", path.display()))
        })?;
        let file = winsome_keymap::parse(&text)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok((path, file))
    };
    let checked = |file: &winsome_keymap::KeymapFile| {
        winsome_keymap::check(file).map_err(|errors| {
            io::Error::new(io::ErrorKind::InvalidData, errors.join("\n"))
        })
    };

    match action {
        KeymapAction::Check { keymap } => {
            let (path, file) = load(keymap)?;
            let (_, coverage) = checked(&file)?;
            println!("{}", path.display());
            println!("{}", winsome_keymap::render_coverage(&file.meta, &coverage));
            Ok(())
        }
        KeymapAction::Render { keymap, out } => {
            let (_, file) = load(keymap)?;
            let (expanded, _) = checked(&file)?;
            let yaml = winsome_keymap::render_glazewm(&expanded);
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

fn down(paths: &Paths, timeout_secs: u64) -> ExitCode {
    // Report what we know before asking, so a stale health file cannot be
    // mistaken for a supervisor that answered.
    match Health::read(&paths.health()) {
        Ok(h) if h.supervisor.running => {
            let pid = h.supervisor.pid;
            match proc_alive::is_alive(pid) {
                Some(false) => println!(
                    "winsome: health state names supervisor pid {pid}, but that process is gone"
                ),
                _ => println!("winsome: asking supervisor pid {pid} to stop"),
            }
        }
        Ok(_) => println!("winsome: health state says no supervisor is running; asking anyway"),
        Err(_) => println!("winsome: no health state found; asking anyway"),
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
            println!("winsome: supervisor acknowledged — waiting for the stack to stop");
            loop {
                match Health::read(&paths.health()) {
                    Ok(h) if !h.supervisor.running => {
                        println!("winsome: stack stopped");
                        return ExitCode::SUCCESS;
                    }
                    Ok(h) if proc_alive::is_alive(h.supervisor.pid) == Some(false) => {
                        eprintln!(
                            "winsome: supervisor (pid {}) died mid-shutdown — components may be orphaned",
                            h.supervisor.pid
                        );
                        return ExitCode::FAILURE;
                    }
                    _ => {}
                }
                if Instant::now() >= deadline {
                    eprintln!(
                        "winsome: still shutting down after {timeout_secs}s — check `winsome status`"
                    );
                    return ExitCode::FAILURE;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
        Ok(false) => {
            eprintln!(
                "winsome: nothing picked up the request within {timeout_secs}s — request withdrawn.\n\
                 Is a supervisor running against {}?",
                paths.home().display()
            );
            ExitCode::FAILURE
        }
        Err(e) => fail(e),
    }
}
