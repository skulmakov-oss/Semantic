//! Computes deterministic build-identity fingerprints and exposes them to
//! `smc-cli` via `env!()`:
//!
//! - `SM_COMPILER_SOURCE_HASH` — content of every crate that determines
//!   `smc check`'s semantic behavior (parse, typecheck, lowering,
//!   verification, execution), discovered as `smc-cli`'s full transitive
//!   local-path dependency closure rather than a hand-maintained list, so a
//!   newly-added semantic dependency is picked up automatically instead of
//!   silently falling outside the fingerprint.
//! - `SM_ENABLED_FEATURES` — the Cargo feature flags this specific build of
//!   `smc-cli` was compiled with (e.g. `profile-rust`, `debug-symbols`),
//!   since those change lowering/verification behavior independently of
//!   source content.
//!
//! Together these let the on-disk semantic/artifact cache (see
//! `current_toolchain_hash`/`current_feature_hash` in `src/app.rs`) tell two
//! compiler builds apart even when neither `Cargo.toml`'s package version
//! nor any environment variable changed. A `smc-cli` rebuilt from edited
//! (including uncommitted) source in any dependency in the closure, or
//! rebuilt with different features, gets different hashes than the build
//! that produced an existing cache entry, so that entry is correctly
//! treated as stale.
//!
//! Deliberately content-based rather than git-commit-based: the defect this
//! repairs was observed after a local rebuild from *uncommitted* source
//! edits, where the git HEAD commit does not change at all.
//!
//! Paths are hashed relative to `crates/`, not as the absolute
//! `CARGO_MANIFEST_DIR`-rooted path, so identical source checked out at two
//! different absolute locations produces the identical hash (required:
//! build identity must be independent of the checkout directory).

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let crates_dir = Path::new(&manifest_dir)
        .parent()
        .expect("smc-cli manifest dir has a crates/ parent")
        .to_path_buf();

    let closure = discover_local_dependency_closure(&crates_dir, "smc-cli");

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for crate_name in &closure {
        hash_dir(
            &crates_dir,
            &crates_dir.join(crate_name).join("src"),
            &mut hash,
        );
    }
    println!("cargo:rustc-env=SM_COMPILER_SOURCE_HASH={:016x}", hash);

    println!("cargo:rustc-env=SM_ENABLED_FEATURES={}", enabled_features());
}

/// Every local-path crate reachable from `entry_crate`'s `Cargo.toml`,
/// transitively, including `entry_crate` itself. Discovered from the
/// manifests themselves (rather than hand-listed) so this stays correct as
/// the dependency graph evolves - a crate stops being silently excluded from
/// the fingerprint just because someone forgot to update a constant list.
fn discover_local_dependency_closure(crates_dir: &Path, entry_crate: &str) -> BTreeSet<String> {
    let mut visited = BTreeSet::new();
    let mut stack = vec![entry_crate.to_string()];
    while let Some(name) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let manifest_path = crates_dir.join(&name).join("Cargo.toml");
        println!("cargo:rerun-if-changed={}", manifest_path.display());
        for dep in local_path_dependencies(&manifest_path) {
            stack.push(dep);
        }
    }
    visited
}

/// Extracts crate directory names from `path = "../name"` entries in a
/// `Cargo.toml`. Deliberately a plain line scan rather than a full TOML
/// parser (no new dependency needed): every path-dependency in this
/// workspace is written as a single-line inline table, e.g.
/// `sm-ir = { path = "../sm-ir", default-features = false }`.
fn local_path_dependencies(manifest_path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(manifest_path) else {
        return Vec::new();
    };
    let marker = "path = \"../";
    let mut deps = Vec::new();
    for line in text.lines() {
        let Some(start) = line.find(marker) else {
            continue;
        };
        let rest = &line[start + marker.len()..];
        if let Some(end) = rest.find('"') {
            deps.push(rest[..end].to_string());
        }
    }
    deps
}

/// Recursively hashes every `.rs` file's crates-relative path (ordering
/// matters, hence the sort) and content, and registers each with
/// `cargo:rerun-if-changed` so editing any of them re-runs this script.
/// Missing directories are skipped rather than treated as an error, since a
/// stripped-down checkout should still build.
fn hash_dir(crates_dir: &Path, dir: &Path, hash: &mut u64) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            hash_dir(crates_dir, &path, hash);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(bytes) = fs::read(&path) {
                let relative = path.strip_prefix(crates_dir).unwrap_or(&path);
                fnv1a64_update(
                    hash,
                    relative.to_string_lossy().replace('\\', "/").as_bytes(),
                );
                fnv1a64_update(hash, &bytes);
            }
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn fnv1a64_update(hash: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *hash ^= *b as u64;
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

/// The Cargo feature flags this build of `smc-cli` itself was compiled
/// with, sorted for determinism. Cargo sets `CARGO_FEATURE_<NAME>=1` for
/// every active feature of the crate whose build script is running.
fn enabled_features() -> String {
    let mut features: Vec<String> = std::env::vars()
        .filter_map(|(k, _)| k.strip_prefix("CARGO_FEATURE_").map(str::to_string))
        .collect();
    features.sort();
    features.join(",")
}
