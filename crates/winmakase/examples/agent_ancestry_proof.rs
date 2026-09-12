//! A1 feed benchmark using only hidden, explicitly owned fixture processes.

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{ChildStdout, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::json;
use windows_sys::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject};
use winmakase::agent_ancestry::{AgentRootPaths, AncestryTracker, Ownership};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const SYNCHRONIZE: u32 = 0x0010_0000;
const FIXTURE_TIMEOUT: Duration = Duration::from_secs(30);

fn main() -> io::Result<()> {
    let args = std::env::args_os().collect::<Vec<_>>();
    match args.get(1).and_then(|value| value.to_str()) {
        Some("fixture-root") => fixture_root(&args[2..]),
        Some("fixture-bridge") => fixture_bridge(&args[2..]),
        Some("fixture-leaf") => fixture_leaf(&args[2..]),
        _ => benchmark(),
    }
}

fn benchmark() -> io::Result<()> {
    let original_exe = std::env::current_exe()?;
    let fixture_dir = fixture_dir();
    fs::create_dir(&fixture_dir)?;
    let provider_exe = fixture_dir.join("validated-provider-root.exe");
    let worker_exe = fixture_dir.join("fixture-worker.exe");
    fs::copy(&original_exe, &provider_exe)?;
    fs::copy(&original_exe, &worker_exe)?;

    let root_stop = fixture_dir.join("root.stop");
    let spawn_bridge = fixture_dir.join("spawn-bridge");
    let bridge_stop = fixture_dir.join("bridge.stop");
    let spawn_leaf = fixture_dir.join("spawn-leaf");
    let leaf_stop = fixture_dir.join("leaf.stop");
    let mut root = hidden(&provider_exe)
        .arg("fixture-root")
        .arg(&root_stop)
        .arg(&spawn_bridge)
        .arg(&worker_exe)
        .arg(&bridge_stop)
        .arg(&spawn_leaf)
        .arg(&leaf_stop)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut root_output = BufReader::new(root.stdout.take().unwrap());
    expect_line(&mut root_output, "ready")?;

    let roots = AgentRootPaths::from_existing_executables([provider_exe])?;
    let mut tracker = AncestryTracker::new(roots, 128, 16_384)?;
    tracker.enroll_root_pid(root.id())?;

    // A root is not trusted until the same sequence and exact live image path
    // are present in two consecutive complete observations.
    tracker.refresh_native()?;
    let root_promotion = tracker.refresh_native()?;
    if root_promotion.observation.promoted_roots != 1 {
        return Err(io::Error::other(
            "root did not pass two-observation binding",
        ));
    }

    fs::write(&spawn_bridge, b"spawn")?;
    let bridge_pid = read_pid(&mut root_output)?;
    let bridge_handle = open_wait_handle(bridge_pid)?;
    tracker.refresh_native()?;

    fs::write(&spawn_leaf, b"spawn")?;
    let leaf_pid = read_pid(&mut root_output)?;
    let leaf_handle = open_wait_handle(leaf_pid)?;
    tracker.refresh_native()?;

    let first = tracker.classify_current(leaf_pid);
    if first.ownership != Ownership::Agent || first.cache_hit {
        return Err(io::Error::other("first supplied-snapshot lookup failed"));
    }

    let mut supplied_snapshot_lookup = Vec::new();
    for _ in 0..200 {
        let started = Instant::now();
        let classification = tracker.classify_current(leaf_pid);
        supplied_snapshot_lookup.push(started.elapsed());
        if classification.ownership != Ownership::Agent || !classification.cache_hit {
            return Err(io::Error::other("supplied-snapshot cache lookup failed"));
        }
    }

    let mut fresh_snapshot = Vec::new();
    let mut root_validation = Vec::new();
    let mut feed_update = Vec::new();
    let mut fresh_lookup = Vec::new();
    let mut end_to_end = Vec::new();
    let mut fresh_misses = 0usize;
    for _ in 0..100 {
        let result = tracker.classify_native_pid(leaf_pid)?;
        if result.classification.ownership != Ownership::Agent {
            return Err(io::Error::other("fresh classification lost ownership"));
        }
        if !result.classification.cache_hit {
            fresh_misses += 1;
            continue;
        }
        fresh_snapshot.push(result.refresh.snapshot);
        root_validation.push(result.refresh.root_validation);
        feed_update.push(result.refresh.feed_update);
        fresh_lookup.push(result.supplied_snapshot_lookup);
        end_to_end.push(result.total);
        if end_to_end.len() == 50 {
            break;
        }
    }
    if end_to_end.len() != 50 {
        return Err(io::Error::other("could not collect 50 fresh cache hits"));
    }

    fs::write(&root_stop, b"stop")?;
    fs::write(&bridge_stop, b"stop")?;
    root.wait()?;
    wait_exact_handle(bridge_handle)?;
    let retained = tracker.classify_native_pid(leaf_pid)?;
    if retained.classification.ownership != Ownership::Agent {
        return Err(io::Error::other("retained edge failed after ancestor exit"));
    }

    fs::write(&leaf_stop, b"stop")?;
    wait_exact_handle(leaf_handle)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "scope": "hidden explicitly enrolled fixture-owned chain only",
            "identity": "PID plus SystemBasicProcessInformation SequenceNumber",
            "requested_model": "gpt-5.6-sol/high builder",
            "observed_model": "not exposed to the worker runtime",
            "safe_edges_observed": 2,
            "supplied_snapshot_cache_hits": 200,
            "fresh_snapshot_cache_hits": 50,
            "fresh_snapshot_cache_misses_from_new_observed_descendants": fresh_misses,
            "retained_after_root_and_bridge_exit": true,
            "timings_ns": {
                "lookup_with_supplied_current_snapshot": distribution(&supplied_snapshot_lookup),
                "fresh_sequence_snapshot": distribution(&fresh_snapshot),
                "exact_enrolled_root_live_image_validation": distribution(&root_validation),
                "feed_update": distribution(&feed_update),
                "lookup_after_fresh_snapshot": distribution(&fresh_lookup),
                "fresh_snapshot_to_answer": distribution(&end_to_end),
            }
        }))?
    );

    let _ = fs::remove_dir_all(&fixture_dir);
    Ok(())
}

fn fixture_root(args: &[std::ffi::OsString]) -> io::Result<()> {
    if args.len() != 6 {
        return Err(io::Error::other("invalid fixture-root arguments"));
    }
    println!("ready");
    io::stdout().flush()?;
    wait_for_marker(Path::new(&args[1]));
    let mut bridge = hidden(Path::new(&args[2]))
        .arg("fixture-bridge")
        .arg(&args[3])
        .arg(&args[4])
        .arg(&args[5])
        .stdout(Stdio::piped())
        .spawn()?;
    println!("{}", bridge.id());
    io::stdout().flush()?;
    let mut bridge_output = BufReader::new(bridge.stdout.take().unwrap());
    let leaf = read_pid(&mut bridge_output)?;
    println!("{leaf}");
    io::stdout().flush()?;
    wait_for_marker(Path::new(&args[0]));
    drop(bridge);
    Ok(())
}

fn fixture_bridge(args: &[std::ffi::OsString]) -> io::Result<()> {
    if args.len() != 3 {
        return Err(io::Error::other("invalid fixture-bridge arguments"));
    }
    wait_for_marker(Path::new(&args[1]));
    let leaf = hidden(&std::env::current_exe()?)
        .arg("fixture-leaf")
        .arg(&args[2])
        .spawn()?;
    println!("{}", leaf.id());
    io::stdout().flush()?;
    wait_for_marker(Path::new(&args[0]));
    drop(leaf);
    Ok(())
}

fn fixture_leaf(args: &[std::ffi::OsString]) -> io::Result<()> {
    if args.len() != 1 {
        return Err(io::Error::other("invalid fixture-leaf arguments"));
    }
    wait_for_marker(Path::new(&args[0]));
    Ok(())
}

fn hidden(exe: &Path) -> Command {
    let mut command = Command::new(exe);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

fn expect_line(output: &mut BufReader<ChildStdout>, expected: &str) -> io::Result<()> {
    let mut line = String::new();
    output.read_line(&mut line)?;
    if line.trim() != expected {
        return Err(io::Error::other("fixture readiness record was invalid"));
    }
    Ok(())
}

fn read_pid(output: &mut BufReader<ChildStdout>) -> io::Result<u32> {
    let mut line = String::new();
    output.read_line(&mut line)?;
    line.trim().parse::<u32>().map_err(io::Error::other)
}

fn wait_for_marker(marker: &Path) {
    let started = Instant::now();
    while !marker.exists() && started.elapsed() < FIXTURE_TIMEOUT {
        thread::sleep(Duration::from_millis(10));
    }
}

fn open_wait_handle(pid: u32) -> io::Result<*mut std::ffi::c_void> {
    let handle = unsafe { OpenProcess(SYNCHRONIZE, 0, pid) };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    Ok(handle)
}

fn wait_exact_handle(handle: *mut std::ffi::c_void) -> io::Result<()> {
    let result = unsafe { WaitForSingleObject(handle, 5_000) };
    unsafe { CloseHandle(handle) };
    if result != WAIT_OBJECT_0 {
        return Err(io::Error::other(format!(
            "fixture wait failed: {result:#x}"
        )));
    }
    Ok(())
}

fn distribution(samples: &[Duration]) -> serde_json::Value {
    let mut ns = samples
        .iter()
        .map(|sample| u64::try_from(sample.as_nanos()).unwrap_or(u64::MAX))
        .collect::<Vec<_>>();
    ns.sort_unstable();
    let p95 = (ns.len() * 95).div_ceil(100).saturating_sub(1);
    json!({"median": ns[ns.len() / 2], "p95": ns[p95], "max": ns[ns.len() - 1]})
}

fn fixture_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "winmakase-agent-ancestry-{}-{nonce}",
        std::process::id()
    ))
}
