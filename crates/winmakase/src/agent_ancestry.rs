//! Bounded prototype for conservative agent-process ancestry classification.
//!
//! This module is intentionally not wired into window policy. It proves the
//! identity, retention and cache rules needed before that integration exists.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::procs::{self, ProcessIdentity, ProcessInfo, ProcessParentEntry};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ownership {
    Agent,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Classification {
    pub ownership: Ownership,
    pub cache_hit: bool,
}

#[derive(Clone, Debug)]
struct Node {
    parent: Option<ProcessIdentity>,
    is_agent_root: bool,
}

#[derive(Clone, Debug)]
pub struct ObservedProcess {
    pub pid: u32,
    pub creation_time: u64,
    pub parent: Option<(u32, u64)>,
    pub image_path: PathBuf,
}

impl ObservedProcess {
    fn identity(&self) -> ProcessIdentity {
        ProcessIdentity {
            pid: self.pid,
            creation_time: self.creation_time,
        }
    }

    fn parent_identity(&self) -> Option<ProcessIdentity> {
        self.parent
            .map(|(pid, creation_time)| ProcessIdentity { pid, creation_time })
    }
}

#[derive(Clone, Debug)]
pub struct AgentRootPaths {
    normalized: HashSet<String>,
}

impl AgentRootPaths {
    /// Accept only existing absolute `.exe` files. Root recognition later uses
    /// an exact, case-insensitive normalized path match. Known generic shells,
    /// runtimes and terminal hosts are rejected even when their path exists;
    /// caller-side provider validation must reject any additional broker.
    pub fn from_existing_executables(paths: impl IntoIterator<Item = PathBuf>) -> io::Result<Self> {
        let mut normalized = HashSet::new();
        for path in paths {
            if !path.is_absolute()
                || !path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "agent root must be an absolute .exe path: {}",
                        path.display()
                    ),
                ));
            }
            let canonical = std::fs::canonicalize(&path)?;
            if !canonical.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("agent root is not a file: {}", path.display()),
                ));
            }
            let basename = canonical
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if is_generic_root(basename) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("generic runtime cannot be an agent root: {basename}"),
                ));
            }
            normalized.insert(normalize_path(&canonical));
        }
        if normalized.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "at least one validated agent executable is required",
            ));
        }
        Ok(Self { normalized })
    }

    fn recognizes(&self, path: &Path) -> bool {
        self.normalized.contains(&normalize_path(path))
    }
}

fn is_generic_root(basename: &str) -> bool {
    [
        "alacritty.exe",
        "bash.exe",
        "cmd.exe",
        "conhost.exe",
        "muxel.exe",
        "node.exe",
        "nu.exe",
        "powershell.exe",
        "pwsh.exe",
        "sh.exe",
        "wezterm-gui.exe",
        "wezterm.exe",
        "windowsterminal.exe",
        "wsl.exe",
        "wt.exe",
    ]
    .iter()
    .any(|generic| basename.eq_ignore_ascii_case(generic))
}

fn normalize_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('/', "\\").to_lowercase();
    normalized
        .strip_prefix(r"\\?\")
        .unwrap_or(&normalized)
        .to_owned()
}

#[derive(Debug)]
pub struct AncestryTracker {
    roots: AgentRootPaths,
    capacity: usize,
    nodes: HashMap<ProcessIdentity, Node>,
    insertion_order: VecDeque<ProcessIdentity>,
    cache: HashMap<ProcessIdentity, Ownership>,
}

impl AncestryTracker {
    pub fn new(roots: AgentRootPaths, capacity: usize) -> io::Result<Self> {
        if capacity == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ancestry capacity must be nonzero",
            ));
        }
        Ok(Self {
            roots,
            capacity,
            nodes: HashMap::with_capacity(capacity),
            insertion_order: VecDeque::with_capacity(capacity),
            cache: HashMap::with_capacity(capacity),
        })
    }

    /// Retain edges only after the caller validated both creation identities.
    /// A missing parent must be represented by omitting that parent node, which
    /// makes classification stop at unknown rather than reconstructing history.
    pub fn observe(&mut self, observations: impl IntoIterator<Item = ObservedProcess>) {
        self.cache.clear();
        for observation in observations {
            let identity = observation.identity();
            let parent = observation
                .parent_identity()
                .filter(|parent| parent.creation_time <= identity.creation_time);
            let node = Node {
                parent,
                is_agent_root: self.roots.recognizes(&observation.image_path),
            };
            if let Some(existing) = self.nodes.get_mut(&identity) {
                *existing = node;
                continue;
            }
            self.nodes.insert(identity, node);
            self.insertion_order.push_back(identity);
            while self.nodes.len() > self.capacity {
                if let Some(expired) = self.insertion_order.pop_front() {
                    self.nodes.remove(&expired);
                    self.cache.remove(&expired);
                }
            }
        }
    }

    /// Revalidate the queried PID's creation identity before every answer,
    /// including cache hits. Ancestors may have exited after their edges were
    /// observed; their retained identities remain valid history for a live
    /// descendant.
    pub fn classify_pid(
        &mut self,
        pid: u32,
        mut current_identity: impl FnMut(u32) -> Option<(u32, u64)>,
    ) -> Classification {
        let Some((live_pid, creation_time)) = current_identity(pid) else {
            return Classification {
                ownership: Ownership::Unknown,
                cache_hit: false,
            };
        };
        if live_pid != pid {
            return Classification {
                ownership: Ownership::Unknown,
                cache_hit: false,
            };
        }
        let identity = ProcessIdentity { pid, creation_time };
        if let Some(ownership) = self.cache.get(&identity).copied() {
            return Classification {
                ownership,
                cache_hit: true,
            };
        }

        let ownership = self.classify_identity(identity);
        if self.nodes.contains_key(&identity) {
            self.cache.insert(identity, ownership);
        }
        Classification {
            ownership,
            cache_hit: false,
        }
    }

    pub fn retained_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn cached_answers(&self) -> usize {
        self.cache.len()
    }

    fn classify_identity(&self, mut identity: ProcessIdentity) -> Ownership {
        let mut visited = HashSet::new();
        while visited.insert(identity) {
            let Some(node) = self.nodes.get(&identity) else {
                return Ownership::Unknown;
            };
            if node.is_agent_root {
                return Ownership::Agent;
            }
            let Some(parent) = node.parent else {
                return Ownership::Unknown;
            };
            identity = parent;
        }
        Ownership::Unknown
    }

    pub fn observe_native_chain(&mut self, owner_pid: u32) -> io::Result<NativeCapture> {
        let total_started = Instant::now();

        let started = Instant::now();
        let first_snapshot = procs::process_parent_snapshot()?;
        let first_snapshot_time = started.elapsed();
        let first_by_pid = snapshot_by_pid(&first_snapshot);

        let started = Instant::now();
        let mut queried = Vec::new();
        let mut cursor = owner_pid;
        let mut seen = HashSet::new();
        let mut failure = None;
        let mut chain_ended = false;
        while seen.insert(cursor) && queried.len() < self.capacity {
            let Some(entry) = first_by_pid.get(&cursor) else {
                failure = Some(NativeGap::MissingSnapshotEntry);
                break;
            };
            let Some(info) = procs::process_info(cursor) else {
                failure = Some(NativeGap::InaccessibleProcess);
                break;
            };
            let is_root = self.roots.recognizes(Path::new(&info.image_path));
            queried.push(QueriedProcess {
                info,
                parent_pid: (!is_root && entry.parent_pid != 0).then_some(entry.parent_pid),
            });
            if is_root || entry.parent_pid == 0 {
                chain_ended = true;
                break;
            }
            cursor = entry.parent_pid;
        }
        if failure.is_none() && !chain_ended {
            failure = Some(NativeGap::CycleOrDepthLimit);
        }
        let chain_query_time = started.elapsed();

        let started = Instant::now();
        let verification_snapshot = procs::process_parent_snapshot()?;
        let verification_snapshot_time = started.elapsed();
        let verification_by_pid = snapshot_by_pid(&verification_snapshot);

        let started = Instant::now();
        if failure.is_none() {
            failure = validate_queried_chain(&queried, &verification_by_pid);
        }
        let chain_revalidation_time = started.elapsed();

        if failure.is_none() {
            let observations = queried
                .iter()
                .map(|process| ObservedProcess {
                    pid: process.info.identity.pid,
                    creation_time: process.info.identity.creation_time,
                    parent: process.parent_pid.and_then(|parent_pid| {
                        queried
                            .iter()
                            .find(|candidate| candidate.info.identity.pid == parent_pid)
                            .map(|parent| {
                                (parent.info.identity.pid, parent.info.identity.creation_time)
                            })
                    }),
                    image_path: PathBuf::from(&process.info.image_path),
                })
                .collect::<Vec<_>>();
            self.observe(observations);
        }

        let started = Instant::now();
        let classification = if failure.is_none() {
            self.classify_pid(owner_pid, |pid| {
                procs::process_identity(pid).map(|identity| (identity.pid, identity.creation_time))
            })
        } else {
            Classification {
                ownership: Ownership::Unknown,
                cache_hit: false,
            }
        };
        let classification_time = started.elapsed();

        Ok(NativeCapture {
            classification,
            observed_nodes: queried.len(),
            failure,
            timings: NativeTimings {
                first_snapshot: first_snapshot_time,
                chain_queries: chain_query_time,
                verification_snapshot: verification_snapshot_time,
                chain_revalidation: chain_revalidation_time,
                classification: classification_time,
                total: total_started.elapsed(),
            },
        })
    }

    pub fn classify_native_pid(&mut self, pid: u32) -> Classification {
        self.classify_pid(pid, |queried_pid| {
            procs::process_identity(queried_pid)
                .map(|identity| (identity.pid, identity.creation_time))
        })
    }
}

fn snapshot_by_pid(entries: &[ProcessParentEntry]) -> HashMap<u32, &ProcessParentEntry> {
    entries.iter().map(|entry| (entry.pid, entry)).collect()
}

#[derive(Clone, Debug)]
struct QueriedProcess {
    info: ProcessInfo,
    parent_pid: Option<u32>,
}

fn validate_queried_chain(
    chain: &[QueriedProcess],
    verification: &HashMap<u32, &ProcessParentEntry>,
) -> Option<NativeGap> {
    for process in chain {
        let Some(current_entry) = verification.get(&process.info.identity.pid) else {
            return Some(NativeGap::MissingSnapshotEntry);
        };
        if process
            .parent_pid
            .is_some_and(|parent_pid| current_entry.parent_pid != parent_pid)
        {
            return Some(NativeGap::ChangedDuringCapture);
        }
        let Some(current_identity) = procs::process_identity(process.info.identity.pid) else {
            return Some(NativeGap::InaccessibleProcess);
        };
        if current_identity != process.info.identity {
            return Some(NativeGap::ChangedDuringCapture);
        }
    }
    for pair in chain.windows(2) {
        let child = &pair[0];
        let parent = &pair[1];
        if child.parent_pid != Some(parent.info.identity.pid) {
            return Some(NativeGap::MissingValidatedParent);
        }
        if parent.info.identity.creation_time > child.info.identity.creation_time {
            return Some(NativeGap::ReusedParentPid);
        }
    }
    None
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeGap {
    MissingSnapshotEntry,
    InaccessibleProcess,
    ChangedDuringCapture,
    MissingValidatedParent,
    ReusedParentPid,
    CycleOrDepthLimit,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeTimings {
    pub first_snapshot: Duration,
    pub chain_queries: Duration,
    pub verification_snapshot: Duration,
    pub chain_revalidation: Duration,
    pub classification: Duration,
    pub total: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeCapture {
    pub classification: Classification,
    pub observed_nodes: usize,
    pub failure: Option<NativeGap>,
    pub timings: NativeTimings,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots() -> AgentRootPaths {
        AgentRootPaths::from_existing_executables([std::env::current_exe().unwrap()]).unwrap()
    }

    fn observed(
        pid: u32,
        created: u64,
        parent: Option<(u32, u64)>,
        path: PathBuf,
    ) -> ObservedProcess {
        ObservedProcess {
            pid,
            creation_time: created,
            parent,
            image_path: path,
        }
    }

    #[test]
    fn complete_observed_chain_reaches_exact_configured_root() {
        let root_path = std::env::current_exe().unwrap();
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([
            observed(10, 100, None, root_path),
            observed(
                20,
                200,
                Some((10, 100)),
                PathBuf::from("C:/Windows/cmd.exe"),
            ),
            observed(30, 300, Some((20, 200)), PathBuf::from("C:/fixture.exe")),
        ]);

        let result = tracker.classify_pid(30, |_| Some((30, 300)));
        assert_eq!(result.ownership, Ownership::Agent);
        assert!(!result.cache_hit);
    }

    #[test]
    fn generic_shell_and_same_basename_are_not_roots() {
        let configured = std::env::current_exe().unwrap();
        let same_name = PathBuf::from("C:/elsewhere").join(configured.file_name().unwrap());
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([
            observed(10, 100, None, same_name),
            observed(
                20,
                200,
                Some((10, 100)),
                PathBuf::from("C:/Windows/cmd.exe"),
            ),
        ]);

        assert_eq!(
            tracker.classify_pid(20, |_| Some((20, 200))).ownership,
            Ownership::Unknown
        );
    }

    #[test]
    fn configured_generic_shell_path_is_rejected() {
        let windows = std::env::var_os("WINDIR").unwrap();
        let cmd = PathBuf::from(windows).join("System32/cmd.exe");
        assert!(AgentRootPaths::from_existing_executables([cmd]).is_err());
    }

    #[test]
    fn missing_parent_history_stays_unknown() {
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([observed(
            30,
            300,
            Some((20, 200)),
            PathBuf::from("C:/fixture.exe"),
        )]);

        assert_eq!(
            tracker.classify_pid(30, |_| Some((30, 300))).ownership,
            Ownership::Unknown
        );
    }

    #[test]
    fn reused_parent_pid_does_not_bridge_to_a_new_identity() {
        let root_path = std::env::current_exe().unwrap();
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([
            observed(20, 250, None, root_path),
            observed(30, 300, Some((20, 200)), PathBuf::from("C:/fixture.exe")),
        ]);

        assert_eq!(
            tracker.classify_pid(30, |_| Some((30, 300))).ownership,
            Ownership::Unknown
        );
    }

    #[test]
    fn newer_reused_parent_identity_is_rejected_at_observation() {
        let root_path = std::env::current_exe().unwrap();
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([
            observed(20, 400, None, root_path),
            observed(30, 300, Some((20, 400)), PathBuf::from("C:/fixture.exe")),
        ]);

        assert_eq!(
            tracker.classify_pid(30, |_| Some((30, 300))).ownership,
            Ownership::Unknown
        );
    }

    #[test]
    fn inaccessible_parent_cannot_be_inferred_from_its_pid() {
        let root_path = std::env::current_exe().unwrap();
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([
            observed(10, 100, None, root_path),
            // The raw PPID may have been 10, but without access to its creation
            // identity the child has no validated parent edge to retain.
            observed(20, 200, None, PathBuf::from("C:/fixture.exe")),
        ]);

        assert_eq!(
            tracker.classify_pid(20, |_| Some((20, 200))).ownership,
            Ownership::Unknown
        );
    }

    #[test]
    fn observed_lineage_survives_parent_and_root_exit() {
        let root_path = std::env::current_exe().unwrap();
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([
            observed(10, 100, None, root_path),
            observed(20, 200, Some((10, 100)), PathBuf::from("C:/bridge.exe")),
            observed(30, 300, Some((20, 200)), PathBuf::from("C:/leaf.exe")),
        ]);

        // Only the queried descendant is revalidated. The retained edge is the
        // evidence for ancestors that have since exited.
        assert_eq!(
            tracker
                .classify_pid(30, |pid| (pid == 30).then_some((30, 300)))
                .ownership,
            Ownership::Agent
        );
    }

    #[test]
    fn cache_hits_still_revalidate_and_reject_pid_reuse_or_inaccessibility() {
        let root_path = std::env::current_exe().unwrap();
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        tracker.observe([
            observed(10, 100, None, root_path),
            observed(20, 200, Some((10, 100)), PathBuf::from("C:/leaf.exe")),
        ]);
        let mut calls = 0;
        let first = tracker.classify_pid(20, |_| {
            calls += 1;
            Some((20, 200))
        });
        let second = tracker.classify_pid(20, |_| {
            calls += 1;
            Some((20, 200))
        });
        let reused = tracker.classify_pid(20, |_| {
            calls += 1;
            Some((20, 999))
        });
        let inaccessible = tracker.classify_pid(20, |_| {
            calls += 1;
            None
        });

        assert!(!first.cache_hit);
        assert!(second.cache_hit);
        assert_eq!(reused.ownership, Ownership::Unknown);
        assert!(!reused.cache_hit);
        assert_eq!(inaccessible.ownership, Ownership::Unknown);
        assert!(!inaccessible.cache_hit);
        assert_eq!(calls, 4);
    }

    #[test]
    fn bounded_retention_evicts_old_history_and_answers() {
        let root_path = std::env::current_exe().unwrap();
        let mut tracker = AncestryTracker::new(roots(), 2).unwrap();
        tracker.observe([
            observed(10, 100, None, root_path),
            observed(20, 200, Some((10, 100)), PathBuf::from("C:/leaf.exe")),
        ]);
        assert_eq!(
            tracker.classify_pid(20, |_| Some((20, 200))).ownership,
            Ownership::Agent
        );
        tracker.observe([observed(30, 300, None, PathBuf::from("C:/human.exe"))]);

        assert_eq!(tracker.retained_nodes(), 2);
        assert!(tracker.cached_answers() <= 2);
        assert_eq!(
            tracker.classify_pid(20, |_| Some((20, 200))).ownership,
            Ownership::Unknown
        );

        for pid in 100..1_000 {
            let _ = tracker.classify_pid(pid, |_| Some((pid, u64::from(pid))));
        }
        assert!(tracker.cached_answers() <= tracker.retained_nodes());
    }

    #[test]
    fn native_self_capture_uses_validated_path_and_revalidates_cache_hit() {
        let mut tracker = AncestryTracker::new(roots(), 8).unwrap();
        let first = tracker.observe_native_chain(std::process::id()).unwrap();
        assert_eq!(first.failure, None);
        assert_eq!(first.classification.ownership, Ownership::Agent);
        assert!(!first.classification.cache_hit);

        let cached = tracker.classify_native_pid(std::process::id());
        assert_eq!(cached.ownership, Ownership::Agent);
        assert!(cached.cache_hit);
    }
}
