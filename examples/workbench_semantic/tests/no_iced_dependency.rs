//! Dependency-boundary enforcement (owner directive 2026-08-02, "WORKBENCH
//! ICED SUBSTRATE MIGRATION"): `iced` is pinned as the private native UI
//! substrate behind `crates/prom-ui-iced-adapter`, and *only* that crate
//! (and lower platform crates it depends on) may import `iced` types.
//! `examples/workbench_semantic` must depend on the public, Iced-free Prom
//! UI component contract, and reach `iced` (if at all) *only* by going
//! through that one adapter crate -- never directly, and never via some
//! other path that skips it.
//!
//! The directive's required architecture is exactly
//! `workbench_semantic -> prom-ui-iced-adapter -> iced`, so "zero reachable
//! path to iced" is *not* the invariant to check here (that was the
//! pre-Iced-migration invariant). What must hold instead is: no edge from
//! `workbench_semantic` to `iced` exists that does not pass through
//! `prom-ui-iced-adapter`.
//!
//! This is a real reachability check against the actual resolved
//! dependency graph (`Cargo.lock`), not a hand-maintained allowlist that
//! could silently go stale -- if `workbench_semantic` ever gains a real
//! path to `iced` that bypasses the adapter (directly, or via some other
//! dependency quietly starting to depend on it), this test fails.
//!
//! A small, purpose-built `Cargo.lock` reader, not a general TOML parser
//! -- the same "hand-rolled parser for one fixed, well-understood format"
//! convention `crates/smc-cli/src/ui_frame_snapshot.rs` already uses for
//! its own constrained format.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

struct LockPackage {
    name: String,
    dependencies: Vec<String>,
}

/// Parses the subset of Cargo.lock's format this check needs: `[[package]]`
/// blocks, each with a `name = "..."` line and an optional
/// `dependencies = [...]` array of quoted strings (each either `"pkgname"`
/// or `"pkgname version"` -- only the name portion, the text before any
/// space, is needed for a reachability check).
fn parse_lockfile_packages(text: &str) -> Vec<LockPackage> {
    let mut packages = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() != "[[package]]" {
            continue;
        }
        let mut name = None;
        let mut dependencies = Vec::new();
        while let Some(&next) = lines.peek() {
            let trimmed = next.trim();
            if trimmed == "[[package]]" || (trimmed.starts_with('[') && trimmed != "[") {
                break;
            }
            let line = lines.next().unwrap();
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name = \"") {
                name = rest.strip_suffix('"').map(|s| s.to_string());
            } else if trimmed == "dependencies = [" {
                for dep_line in lines.by_ref() {
                    let dep_trimmed = dep_line.trim().trim_end_matches(',');
                    if dep_trimmed == "]" {
                        break;
                    }
                    if let Some(rest) = dep_trimmed.strip_prefix('"') {
                        if let Some(quoted) = rest.strip_suffix('"') {
                            let pkg_name = quoted.split(' ').next().unwrap_or(quoted);
                            dependencies.push(pkg_name.to_string());
                        }
                    }
                }
            }
        }
        if let Some(name) = name {
            packages.push(LockPackage { name, dependencies });
        }
    }
    packages
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("examples/workbench_semantic is two levels under the repo root")
        .to_path_buf()
}

const ADAPTER: &str = "prom-ui-iced-adapter";

#[test]
fn workbench_semantic_only_reaches_iced_through_the_adapter() {
    let lock_path = repo_root().join("Cargo.lock");
    let text = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|e| panic!("must be able to read {}: {e}", lock_path.display()));
    let packages = parse_lockfile_packages(&text);
    assert!(
        !packages.is_empty(),
        "the hand-rolled Cargo.lock parser found zero [[package]] blocks -- \
         it has drifted from the real lockfile format, fix the parser \
         before trusting this test's result"
    );

    // Union dependency-name sets across all versions of the same package
    // name -- an intentionally conservative over-approximation for a
    // boundary check whose job is "never silently miss a real path to
    // iced," not "be maximally precise about which exact version."
    let mut by_name: HashMap<&str, HashSet<&str>> = HashMap::new();
    for pkg in &packages {
        by_name
            .entry(pkg.name.as_str())
            .or_default()
            .extend(pkg.dependencies.iter().map(String::as_str));
    }

    assert!(
        by_name.contains_key("iced"),
        "expected the real `iced` package to be present in Cargo.lock at all \
         (pinned by prom-ui-iced-adapter) -- if this fails, the pin itself \
         was removed, which this test cannot distinguish from a parser bug"
    );
    assert!(
        by_name.contains_key("workbench_semantic"),
        "expected the real `workbench_semantic` package to be present in \
         Cargo.lock -- if this fails, the parser or the package name has \
         drifted"
    );
    assert!(
        !by_name.get("workbench_semantic").unwrap().is_empty(),
        "workbench_semantic parsed with zero dependencies -- the lockfile \
         parser has regressed (this exact bug happened before: a trailing \
         comma after each quoted dependency name defeated a naive \
         strip_suffix('\"') check), fix the parser before trusting this \
         test's result"
    );
    assert!(
        by_name
            .get(ADAPTER)
            .is_some_and(|deps| deps.contains("iced")),
        "expected prom-ui-iced-adapter to directly depend on iced in \
         Cargo.lock -- if this fails, either the pin moved or the parser \
         regressed"
    );

    // Sanity: the adapter must actually be reachable from
    // workbench_semantic, or the boundary below would pass vacuously
    // without ever exercising the real required architecture
    // (workbench_semantic -> prom-ui-iced-adapter -> iced).
    assert!(
        reachable(&by_name, "workbench_semantic", ADAPTER),
        "workbench_semantic has no dependency path to prom-ui-iced-adapter \
         at all -- the required architecture \
         (workbench_semantic -> prom-ui-iced-adapter -> iced) is not wired up"
    );

    // Real BFS over the real resolved dependency graph, starting from
    // workbench_semantic, but deliberately NOT expanding past the adapter
    // crate -- it is the one authorized gateway to iced, so its own
    // dependencies are not "workbench_semantic's problem." Anything this
    // BFS still finds a path to `iced` through is, by construction, a real
    // bypass of the adapter boundary.
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back("workbench_semantic");
    visited.insert("workbench_semantic");
    let mut path_to_iced: Option<Vec<&str>> = None;
    let mut parent: HashMap<&str, &str> = HashMap::new();

    while let Some(current) = queue.pop_front() {
        if current == ADAPTER {
            // Boundary reached via an authorized route -- don't cross it.
            continue;
        }
        let Some(deps) = by_name.get(current) else {
            continue;
        };
        for &dep in deps {
            if dep == "iced" {
                // Reconstruct the real path for a genuinely useful failure
                // message, not just "somewhere, somehow."
                let mut path = vec!["iced", current];
                let mut node = current;
                while let Some(&p) = parent.get(node) {
                    path.push(p);
                    node = p;
                }
                path.reverse();
                path_to_iced = Some(path);
                break;
            }
            if visited.insert(dep) {
                parent.insert(dep, current);
                queue.push_back(dep);
            }
        }
        if path_to_iced.is_some() {
            break;
        }
    }

    assert!(
        path_to_iced.is_none(),
        "workbench_semantic has a real dependency path to iced that bypasses \
         prom-ui-iced-adapter, which the private-adapter boundary forbids: {}",
        path_to_iced.unwrap_or_default().join(" -> ")
    );
}

/// Plain BFS reachability, no `iced`-specific short-circuiting -- used only
/// for the sanity check that the adapter itself is on the real graph.
fn reachable(by_name: &HashMap<&str, HashSet<&str>>, from: &str, to: &str) -> bool {
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back(from);
    visited.insert(from);
    while let Some(current) = queue.pop_front() {
        if current == to {
            return true;
        }
        let Some(deps) = by_name.get(current) else {
            continue;
        };
        for &dep in deps {
            if visited.insert(dep) {
                queue.push_back(dep);
            }
        }
    }
    false
}

#[test]
fn workbench_semantic_manifest_does_not_list_iced_directly() {
    let manifest_path = repo_root().join("examples/workbench_semantic/Cargo.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("must be able to read {}: {e}", manifest_path.display()));
    assert!(
        !text.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("iced ") || trimmed.starts_with("iced=") || trimmed == "iced"
        }),
        "examples/workbench_semantic/Cargo.toml must never list `iced` as a \
         direct dependency -- only prom-ui-iced-adapter may"
    );
}
