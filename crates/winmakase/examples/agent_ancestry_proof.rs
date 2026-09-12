//! A1 native ancestry benchmark using only hidden copies of this executable.

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
    let bridge_stop = fixture_dir.join("bridge.stop");
    let leaf_stop = fixture_dir.join("leaf.stop");
    let mut root = hidden(&provider_exe)
        .args([
            "fixture-root",
            worker_exe.to_str().unwrap(),
            root_stop.to_str().unwrap(),
            bridge_stop.to_str().unwrap(),
            leaf_stop.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let line = read_one_line(root.stdout.take().unwrap())?;
    let fixture_pids = line
        .split_whitespace()
        .map(|value| value.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;
    if fixture_pids.len() != 2 {
        return Err(io::Error::other("fixture returned an invalid PID record"));
    }
    let bridge_pid = fixture_pids[0];
    let leaf_pid = fixture_pids[1];
    let bridge_handle = open_wait_handle(bridge_pid)?;
    let leaf_handle = open_wait_handle(leaf_pid)?;

    let roots = AgentRootPaths::from_existing_executables([provider_exe.clone()])?;
    let mut tracker = AncestryTracker::new(roots, 128)?;
    let mut first_snapshot = Vec::new();
    let mut chain_queries = Vec::new();
    let mut verification_snapshot = Vec::new();
    let mut chain_revalidation = Vec::new();
    let mut miss_classification = Vec::new();
    let mut total = Vec::new();
    let mut misses = 0usize;

    for _ in 0..25 {
        let capture = tracker.observe_native_chain(leaf_pid)?;
        if capture.failure.is_some() || capture.classification.ownership != Ownership::Agent {
            return Err(io::Error::other(format!(
                "fixture ancestry capture failed: {:?}",
                capture.failure
            )));
        }
        misses += usize::from(!capture.classification.cache_hit);
        first_snapshot.push(capture.timings.first_snapshot);
        chain_queries.push(capture.timings.chain_queries);
        verification_snapshot.push(capture.timings.verification_snapshot);
        chain_revalidation.push(capture.timings.chain_revalidation);
        miss_classification.push(capture.timings.classification);
        total.push(capture.timings.total);
    }

    // The retained edge, rather than a later PID walk, must carry ancestry once
    // the already-observed root and bridge are gone.
    fs::write(&root_stop, b"stop")?;
    fs::write(&bridge_stop, b"stop")?;
    root.wait()?;
    wait_exact_handle(bridge_handle)?;

    let retained = tracker.classify_native_pid(leaf_pid);
    if retained.ownership != Ownership::Agent || !retained.cache_hit {
        return Err(io::Error::other(
            "retained ancestry failed after root and bridge exit",
        ));
    }

    let mut cache_hit_revalidation = Vec::new();
    let mut hits = 0usize;
    for _ in 0..200 {
        let started = Instant::now();
        let classification = tracker.classify_native_pid(leaf_pid);
        cache_hit_revalidation.push(started.elapsed());
        if classification.ownership != Ownership::Agent || !classification.cache_hit {
            return Err(io::Error::other("cache-hit revalidation failed"));
        }
        hits += 1;
    }

    fs::write(&leaf_stop, b"stop")?;
    wait_exact_handle(leaf_handle)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "scope": "hidden fixture-owned process chain only",
            "requested_model": "gpt-5.6-sol/high builder",
            "observed_model": "not exposed to the worker runtime",
            "captures": 25,
            "observed_chain_nodes": 3,
            "classification_misses": misses,
            "cache_hits_after_ancestor_exit": hits,
            "retained_after_root_and_bridge_exit": true,
            "timings_ns": {
                "first_full_snapshot": distribution(&first_snapshot),
                "exact_chain_identity_and_image_queries": distribution(&chain_queries),
                "verification_full_snapshot": distribution(&verification_snapshot),
                "exact_chain_identity_revalidation": distribution(&chain_revalidation),
                "miss_classification": distribution(&miss_classification),
                "full_capture": distribution(&total),
                "cache_hit_owner_identity_revalidation": distribution(&cache_hit_revalidation),
            }
        }))?
    );

    drop(root);
    let _ = fs::remove_dir_all(&fixture_dir);
    Ok(())
}

fn fixture_root(args: &[std::ffi::OsString]) -> io::Result<()> {
    if args.len() != 4 {
        return Err(io::Error::other("invalid fixture-root arguments"));
    }
    let worker_exe = PathBuf::from(&args[0]);
    let root_stop = PathBuf::from(&args[1]);
    let bridge_stop = PathBuf::from(&args[2]);
    let leaf_stop = PathBuf::from(&args[3]);
    let mut bridge = hidden(&worker_exe)
        .args([
            "fixture-bridge",
            bridge_stop.to_str().unwrap(),
            leaf_stop.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .spawn()?;
    let leaf_pid = read_one_line(bridge.stdout.take().unwrap())?
        .trim()
        .parse::<u32>()
        .map_err(io::Error::other)?;
    println!("{} {leaf_pid}", bridge.id());
    io::stdout().flush()?;
    wait_for_marker(&root_stop);
    drop(bridge);
    Ok(())
}

fn fixture_bridge(args: &[std::ffi::OsString]) -> io::Result<()> {
    if args.len() != 2 {
        return Err(io::Error::other("invalid fixture-bridge arguments"));
    }
    let bridge_stop = PathBuf::from(&args[0]);
    let leaf_stop = PathBuf::from(&args[1]);
    let leaf = hidden(&std::env::current_exe()?)
        .args(["fixture-leaf", leaf_stop.to_str().unwrap()])
        .spawn()?;
    println!("{}", leaf.id());
    io::stdout().flush()?;
    wait_for_marker(&bridge_stop);
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

fn read_one_line(stdout: impl io::Read) -> io::Result<String> {
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line)?;
    if line.is_empty() {
        return Err(io::Error::other("fixture exited before reporting its PID"));
    }
    Ok(line)
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
            "fixture process wait failed: {result:#x}"
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
    let median = ns[ns.len() / 2];
    let p95_index = (ns.len() * 95).div_ceil(100).saturating_sub(1);
    json!({
        "median": median,
        "p95": ns[p95_index],
        "max": ns[ns.len() - 1]
    })
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
