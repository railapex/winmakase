//! Unwired A1 prototype: bounded process-start observation by unique sequence.
//!
//! A child edge is retained only when the same unique parent identity existed
//! in the immediately previous and current complete observations, while the
//! child identity is new in the current one. There is no retrospective walk.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::procs;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProcessKey {
    pub pid: u32,
    pub sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotProcess {
    pub pid: u32,
    pub parent_pid: u32,
    pub sequence: u64,
}

impl SnapshotProcess {
    fn key(self) -> ProcessKey {
        ProcessKey {
            pid: self.pid,
            sequence: self.sequence,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcessSnapshot {
    by_pid: HashMap<u32, SnapshotProcess>,
}

impl ProcessSnapshot {
    pub fn new(
        max_entries: usize,
        processes: impl IntoIterator<Item = SnapshotProcess>,
    ) -> io::Result<Self> {
        let mut by_pid = HashMap::new();
        for process in processes {
            if by_pid.len() == max_entries {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process snapshot exceeded the entry cap",
                ));
            }
            if by_pid.insert(process.pid, process).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process snapshot contained a duplicate PID",
                ));
            }
        }
        Ok(Self { by_pid })
    }

    fn capture_native(max_entries: usize) -> io::Result<Self> {
        let entries = procs::sequence_process_snapshot(max_entries)?
            .into_iter()
            .map(|process| SnapshotProcess {
                pid: process.pid,
                parent_pid: process.parent_pid,
                sequence: process.sequence,
            });
        Self::new(max_entries, entries)
    }

    pub fn key_for_pid(&self, pid: u32) -> Option<ProcessKey> {
        self.by_pid.get(&pid).copied().map(SnapshotProcess::key)
    }

    fn len(&self) -> usize {
        self.by_pid.len()
    }
}

#[derive(Clone, Debug)]
pub struct AgentRootPaths {
    normalized: HashSet<String>,
}

impl AgentRootPaths {
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

    fn normalized_match(&self, path: &Path) -> Option<String> {
        let normalized = normalize_path(path);
        self.normalized.contains(&normalized).then_some(normalized)
    }
}

fn normalize_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('/', "\\").to_lowercase();
    normalized
        .strip_prefix(r"\\?\")
        .unwrap_or(&normalized)
        .to_owned()
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
    parent: Option<ProcessKey>,
    is_agent_root: bool,
}

#[derive(Clone, Debug)]
struct RootEnrollment {
    bound_identity: Option<ProcessKey>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedRoot {
    identity: ProcessKey,
    normalized_path: String,
}

#[derive(Debug)]
pub struct AncestryTracker {
    roots: AgentRootPaths,
    node_capacity: usize,
    snapshot_capacity: usize,
    nodes: HashMap<ProcessKey, Node>,
    insertion_order: VecDeque<ProcessKey>,
    cache: HashMap<ProcessKey, Ownership>,
    previous: Option<ProcessSnapshot>,
    previous_root_validations: HashMap<ProcessKey, String>,
    root_enrollments: HashMap<u32, RootEnrollment>,
}

impl AncestryTracker {
    pub fn new(
        roots: AgentRootPaths,
        node_capacity: usize,
        snapshot_capacity: usize,
    ) -> io::Result<Self> {
        if node_capacity == 0 || snapshot_capacity == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ancestry and snapshot capacities must be nonzero",
            ));
        }
        Ok(Self {
            roots,
            node_capacity,
            snapshot_capacity,
            nodes: HashMap::with_capacity(node_capacity),
            insertion_order: VecDeque::with_capacity(node_capacity),
            cache: HashMap::with_capacity(node_capacity),
            previous: None,
            previous_root_validations: HashMap::new(),
            root_enrollments: HashMap::new(),
        })
    }

    /// Explicit proof-only enrollment. Automatic provider candidate discovery
    /// is intentionally outside this prototype.
    pub fn enroll_root_pid(&mut self, pid: u32) -> io::Result<()> {
        if pid == 0
            || (!self.root_enrollments.contains_key(&pid)
                && self.root_enrollments.len() == self.node_capacity)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid or over-capacity root enrollment",
            ));
        }
        self.root_enrollments.entry(pid).or_insert(RootEnrollment {
            bound_identity: None,
        });
        Ok(())
    }

    /// A failed, truncated or unavailable observation breaks adjacency. Old
    /// validated history remains, but the next complete snapshot is baseline
    /// only and cannot create edges from the pre-gap snapshot.
    pub fn invalidate_continuity(&mut self) {
        self.previous = None;
        self.previous_root_validations.clear();
        self.cache.clear();
    }

    pub fn observe(&mut self, current: ProcessSnapshot) -> io::Result<Observation> {
        if current.len() > self.snapshot_capacity {
            self.invalidate_continuity();
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process snapshot exceeded the tracker entry cap",
            ));
        }
        Ok(self.advance(current, Vec::new()))
    }

    pub fn refresh_native(&mut self) -> io::Result<NativeRefresh> {
        let total_started = Instant::now();
        let started = Instant::now();
        let snapshot = match ProcessSnapshot::capture_native(self.snapshot_capacity) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.invalidate_continuity();
                return Err(error);
            }
        };
        let snapshot_time = started.elapsed();

        let started = Instant::now();
        let validated_roots = self.validate_enrolled_roots(&snapshot);
        let root_validation_time = started.elapsed();

        let started = Instant::now();
        let observation = self.advance(snapshot, validated_roots);
        let feed_update_time = started.elapsed();
        Ok(NativeRefresh {
            observation,
            snapshot: snapshot_time,
            root_validation: root_validation_time,
            feed_update: feed_update_time,
            total: total_started.elapsed(),
        })
    }

    pub fn classify_current(&mut self, pid: u32) -> Classification {
        let Some(identity) = self
            .previous
            .as_ref()
            .and_then(|snapshot| snapshot.key_for_pid(pid))
        else {
            return Classification {
                ownership: Ownership::Unknown,
                cache_hit: false,
            };
        };
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

    pub fn classify_native_pid(&mut self, pid: u32) -> io::Result<NativeClassification> {
        let total_started = Instant::now();
        let refresh = self.refresh_native()?;
        let started = Instant::now();
        let classification = self.classify_current(pid);
        let lookup = started.elapsed();
        Ok(NativeClassification {
            classification,
            refresh,
            supplied_snapshot_lookup: lookup,
            total: total_started.elapsed(),
        })
    }

    pub fn retained_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn cached_answers(&self) -> usize {
        self.cache.len()
    }

    fn validate_enrolled_roots(&mut self, snapshot: &ProcessSnapshot) -> Vec<ValidatedRoot> {
        let mut validated = Vec::new();
        for (&pid, enrollment) in &mut self.root_enrollments {
            let Some(identity) = snapshot.key_for_pid(pid) else {
                continue;
            };
            let bound = enrollment.bound_identity.get_or_insert(identity);
            if *bound != identity {
                continue;
            }
            let Some(info) = procs::process_info(pid) else {
                continue;
            };
            if let Some(normalized_path) = self.roots.normalized_match(Path::new(&info.image_path))
            {
                validated.push(ValidatedRoot {
                    identity,
                    normalized_path,
                });
            }
        }
        validated
    }

    fn advance(
        &mut self,
        current: ProcessSnapshot,
        validated_roots: Vec<ValidatedRoot>,
    ) -> Observation {
        let previous = self.previous.take();
        let baseline_only = previous.is_none();
        let mut added_edges = 0;
        let mut promoted_roots = 0;
        let mut graph_changed = false;
        let current_root_validations = validated_roots
            .into_iter()
            .map(|root| (root.identity, root.normalized_path))
            .collect::<HashMap<_, _>>();
        let promotable_roots = current_root_validations
            .iter()
            .filter_map(|(identity, path)| {
                (self.previous_root_validations.get(identity) == Some(path)
                    && previous
                        .as_ref()
                        .and_then(|snapshot| snapshot.key_for_pid(identity.pid))
                        == Some(*identity)
                    && current.key_for_pid(identity.pid) == Some(*identity))
                .then_some(*identity)
            })
            .collect::<HashSet<_>>();

        if let Some(previous) = &previous {
            for child in current.by_pid.values() {
                let child_key = child.key();
                if previous.key_for_pid(child.pid) == Some(child_key) {
                    continue;
                }
                let Some(parent_before) = previous.key_for_pid(child.parent_pid) else {
                    continue;
                };
                let Some(parent_now) = current.key_for_pid(child.parent_pid) else {
                    continue;
                };
                if parent_before != parent_now
                    || (!self.nodes.contains_key(&parent_now)
                        && !promotable_roots.contains(&parent_now))
                {
                    continue;
                }
                graph_changed |= self.insert_node(child_key, Some(parent_now), false);
                added_edges += 1;
            }
        }

        for identity in promotable_roots {
            let changed = self.insert_node(identity, None, true);
            graph_changed |= changed;
            promoted_roots += usize::from(changed);
        }

        if graph_changed {
            self.cache.clear();
        }
        self.previous = Some(current);
        self.previous_root_validations = current_root_validations;
        Observation {
            added_edges,
            promoted_roots,
            baseline_only,
        }
    }

    fn insert_node(
        &mut self,
        identity: ProcessKey,
        parent: Option<ProcessKey>,
        is_agent_root: bool,
    ) -> bool {
        if let Some(node) = self.nodes.get_mut(&identity) {
            let before = node.clone();
            if is_agent_root {
                node.is_agent_root = true;
            } else if node.parent.is_none() {
                node.parent = parent;
            } else if node.parent != parent {
                node.parent = None;
            }
            return node.parent != before.parent || node.is_agent_root != before.is_agent_root;
        }
        self.nodes.insert(
            identity,
            Node {
                parent,
                is_agent_root,
            },
        );
        self.insertion_order.push_back(identity);
        while self.nodes.len() > self.node_capacity {
            if let Some(expired) = self.insertion_order.pop_front() {
                self.nodes.remove(&expired);
                self.cache.remove(&expired);
            }
        }
        true
    }

    fn classify_identity(&self, mut identity: ProcessKey) -> Ownership {
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
}

#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub added_edges: usize,
    pub promoted_roots: usize,
    pub baseline_only: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeRefresh {
    pub observation: Observation,
    pub snapshot: Duration,
    pub root_validation: Duration,
    pub feed_update: Duration,
    pub total: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeClassification {
    pub classification: Classification,
    pub refresh: NativeRefresh,
    pub supplied_snapshot_lookup: Duration,
    pub total: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots() -> AgentRootPaths {
        AgentRootPaths::from_existing_executables([std::env::current_exe().unwrap()]).unwrap()
    }

    fn tracker(capacity: usize) -> AncestryTracker {
        AncestryTracker::new(roots(), capacity, 16_384).unwrap()
    }

    fn snapshot(rows: &[(u32, u32, u64)]) -> ProcessSnapshot {
        ProcessSnapshot::new(
            64,
            rows.iter()
                .map(|&(pid, parent_pid, sequence)| SnapshotProcess {
                    pid,
                    parent_pid,
                    sequence,
                }),
        )
        .unwrap()
    }

    fn root(pid: u32, sequence: u64) -> ValidatedRoot {
        ValidatedRoot {
            identity: ProcessKey { pid, sequence },
            normalized_path: normalize_path(&std::env::current_exe().unwrap()),
        }
    }

    #[test]
    fn child_is_admitted_only_after_parent_was_present() {
        let mut tracker = tracker(8);
        tracker.advance(snapshot(&[(10, 1, 900)]), vec![root(10, 900)]);
        tracker.advance(snapshot(&[(10, 1, 900), (20, 10, 1)]), vec![root(10, 900)]);
        assert_eq!(tracker.classify_current(20).ownership, Ownership::Agent);
    }

    #[test]
    fn nonmonotonic_sequence_values_make_no_chronology_claim() {
        let mut tracker = tracker(8);
        tracker.advance(snapshot(&[(10, 1, 9_000)]), vec![root(10, 9_000)]);
        tracker.advance(
            snapshot(&[(10, 1, 9_000), (20, 10, 1)]),
            vec![root(10, 9_000)],
        );
        assert_eq!(tracker.classify_current(20).ownership, Ownership::Agent);
    }

    #[test]
    fn existing_unobserved_ancestry_stays_unknown() {
        let mut tracker = tracker(8);
        tracker.advance(
            snapshot(&[(10, 1, 100), (20, 10, 200)]),
            vec![root(10, 100)],
        );
        tracker.advance(
            snapshot(&[(10, 1, 100), (20, 10, 200)]),
            vec![root(10, 100)],
        );
        assert_eq!(tracker.classify_current(20).ownership, Ownership::Unknown);
    }

    #[test]
    fn parent_and_child_first_seen_together_leave_child_unknown() {
        let mut tracker = tracker(8);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(
            snapshot(&[(10, 1, 100), (20, 10, 200), (30, 20, 300)]),
            vec![],
        );
        assert_eq!(tracker.classify_current(30).ownership, Ownership::Unknown);
    }

    #[test]
    fn observation_gap_prevents_comparison_across_the_gap() {
        let mut tracker = tracker(8);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.invalidate_continuity();
        tracker
            .observe(snapshot(&[(10, 1, 100), (20, 10, 200)]))
            .unwrap();
        assert_eq!(tracker.classify_current(20).ownership, Ownership::Unknown);
    }

    #[test]
    fn pid_reuse_after_a_gap_does_not_hit_the_old_cache() {
        let mut tracker = tracker(8);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(
            snapshot(&[(10, 1, 100), (20, 10, 200)]),
            vec![root(10, 100)],
        );
        assert_eq!(tracker.classify_current(20).ownership, Ownership::Agent);
        assert!(tracker.classify_current(20).cache_hit);
        tracker.invalidate_continuity();
        tracker
            .observe(snapshot(&[(10, 1, 100), (20, 10, 999)]))
            .unwrap();
        let reused = tracker.classify_current(20);
        assert_eq!(reused.ownership, Ownership::Unknown);
        assert!(!reused.cache_hit);
    }

    #[test]
    fn missing_or_inaccessible_parent_is_unknown() {
        let mut tracker = tracker(8);
        tracker.observe(snapshot(&[(30, 20, 300)])).unwrap();
        tracker
            .observe(snapshot(&[(30, 20, 300), (40, 30, 400)]))
            .unwrap();
        assert_eq!(tracker.classify_current(40).ownership, Ownership::Unknown);
    }

    #[test]
    fn root_requires_the_same_image_binding_twice() {
        let mut tracker = tracker(8);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![]);
        assert_eq!(tracker.classify_current(10).ownership, Ownership::Unknown);
    }

    #[test]
    fn retained_edges_survive_ancestor_exit() {
        let mut tracker = tracker(8);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(
            snapshot(&[(10, 1, 100), (20, 10, 200)]),
            vec![root(10, 100)],
        );
        tracker
            .observe(snapshot(&[(20, 10, 200), (30, 20, 300)]))
            .unwrap();
        tracker.observe(snapshot(&[(30, 20, 300)])).unwrap();
        assert_eq!(tracker.classify_current(30).ownership, Ownership::Agent);
    }

    #[test]
    fn retained_nodes_and_cached_answers_are_bounded() {
        let mut tracker = tracker(2);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(
            snapshot(&[(10, 1, 100), (20, 10, 200)]),
            vec![root(10, 100)],
        );
        tracker
            .observe(snapshot(&[(10, 1, 100), (20, 10, 200), (30, 20, 300)]))
            .unwrap();
        assert_eq!(tracker.retained_nodes(), 2);
        let _ = tracker.classify_current(30);
        assert!(tracker.cached_answers() <= tracker.retained_nodes());
    }

    #[test]
    fn configured_generic_shell_path_is_rejected() {
        let windows = std::env::var_os("WINDIR").unwrap();
        let cmd = PathBuf::from(windows).join("System32/cmd.exe");
        assert!(AgentRootPaths::from_existing_executables([cmd]).is_err());
    }

    #[test]
    fn public_observe_enforces_tracker_cap_and_breaks_continuity() {
        let mut tracker = AncestryTracker::new(roots(), 8, 2).unwrap();
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);
        tracker.advance(snapshot(&[(10, 1, 100)]), vec![root(10, 100)]);

        let oversized = ProcessSnapshot::new(
            3,
            [
                SnapshotProcess {
                    pid: 10,
                    parent_pid: 1,
                    sequence: 100,
                },
                SnapshotProcess {
                    pid: 20,
                    parent_pid: 10,
                    sequence: 200,
                },
                SnapshotProcess {
                    pid: 30,
                    parent_pid: 20,
                    sequence: 300,
                },
            ],
        )
        .unwrap();
        assert_eq!(
            tracker.observe(oversized).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );

        let baseline = tracker
            .observe(snapshot(&[(10, 1, 100), (20, 10, 200)]))
            .unwrap();
        assert!(baseline.baseline_only);
        assert_eq!(tracker.classify_current(20).ownership, Ownership::Unknown);
    }

    #[test]
    fn native_self_root_needs_two_live_image_validations() {
        let mut tracker = tracker(8);
        tracker.enroll_root_pid(std::process::id()).unwrap();
        for _ in 0..2 {
            if let Err(error) = tracker.refresh_native() {
                if error.kind() == io::ErrorKind::Unsupported {
                    return;
                }
                panic!("native sequence observation failed: {error}");
            }
        }
        assert_eq!(
            tracker.classify_current(std::process::id()).ownership,
            Ownership::Agent
        );
    }
}
