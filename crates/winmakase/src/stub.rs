//! A stand-in child process, for exercising the supervisor.
//!
//! The supervisor cannot be tested against the real components: kanata and
//! GlazeWM are the live desktop, and killing them to watch the restart logic is
//! exactly the cascade the supervisor exists to prevent. So the tests point the
//! config at this instead — a process that ticks on stdout, can be told to exit
//! with a chosen code, and dies when killed.
//!
//! Hidden from `--help`; it is plumbing, not a verb.

use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct StubArgs {
    pub tick_ms: u64,
    pub exit_after_ms: Option<u64>,
    pub code: i32,
    pub stderr: bool,
}

pub fn run(args: StubArgs) -> i32 {
    let start = Instant::now();
    let deadline = args.exit_after_ms.map(Duration::from_millis);
    let pid = std::process::id();
    let mut tick: u64 = 0;
    loop {
        if let Some(d) = deadline
            && start.elapsed() >= d
        {
            println!("stub pid {pid} exiting with {}", args.code);
            let _ = io::stdout().flush();
            return args.code;
        }
        tick += 1;
        println!("stub pid {pid} tick {tick}");
        let _ = io::stdout().flush();
        if args.stderr {
            eprintln!("stub pid {pid} stderr {tick}");
            let _ = io::stderr().flush();
        }
        thread::sleep(Duration::from_millis(args.tick_ms.max(1)));
    }
}
