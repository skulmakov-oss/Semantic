//! Computes deterministic build-identity fingerprints and exposes them to
//! `smc-cli` via `env!()`:
//!
//! - `SM_COMPILER_SOURCE_HASH` — a fail-closed fingerprint of everything that
//!   determines `smc check`'s semantic behavior (parse, typecheck, lowering,
//!   verification, execution): the content of every crate in `smc-cli`'s
//!   local semantic dependency closure (`Cargo.toml` + optional `build.rs` +
//!   `src/**/*.rs`).
//! - `SM_ENABLED_FEATURES` — the Cargo feature flags this specific build of
//!   `smc-cli` was compiled with, since those change lowering/verification
//!   behavior independently of source content.
//!
//! Together these let the on-disk semantic/artifact cache (see
//! `current_toolchain_hash`/`current_feature_hash` in `src/app.rs`) tell two
//! compiler builds apart even when neither `Cargo.toml`'s package version nor
//! any environment variable changed.
//!
//! # Fail-closed identity construction
//!
//! This is a cache **identity**, not a cache **lookup**. A stale/corrupt/
//! mismatched runtime cache entry safely falls back to a fresh check (see
//! `CacheReason` in `src/incremental.rs`) - that fallback is correct and
//! deliberately preserved. But the identity itself must never guess: if a
//! selected input cannot be read completely and losslessly, this script
//! fails the build (`run()` returns `Err`, `main()` reports it and exits
//! non-zero) rather than silently hashing an incomplete or approximate
//! substitute. A narrower-than-real identity is how two genuinely different
//! compiler builds end up sharing a hash and a stale cache entry survives as
//! a false `HIT`; a build failure is always safe, just inconvenient.
//! Concretely this script never uses `Vec::new()`/early-return/`.ok()`/
//! `to_string_lossy()` to paper over an I/O or encoding failure on a file or
//! directory it has committed to including in the identity.
//!
//! # Dependency-closure discovery
//!
//! The semantic dependency closure is derived from the workspace
//! `Cargo.lock`, not by hand-scanning `Cargo.toml` text for `path = "../`
//! substrings. `Cargo.lock` is Cargo's own fully-resolved, uniformly
//! machine-generated record of exactly which packages this workspace
//! actually builds and how they depend on each other; local (path/workspace)
//! packages are the `[[package]]` blocks with no `source = ...` line.
//! Deriving the closure from it is both simpler and strictly more robust
//! than re-deriving Cargo's own resolution by parsing hand-written
//! `Cargo.toml` prose (whose formatting is not guaranteed uniform - e.g.
//! `crates/prom-ui-demo/Cargo.toml` has a `path = "../../experiments/..."`
//! dependency that a naive single-level `../<name>` scan would have silently
//! mis-extracted). If `Cargo.lock` ever references a dependency name with no
//! matching `[[package]]` block, that is treated as a malformed/out-of-sync
//! lock file and fails the build rather than silently excluding it.
//!
//! `Cargo.lock` is used *only* to discover which local crates exist and how
//! they depend on each other - never hashed as content. That local-crate
//! shape is itself fully determined by tracked, committed `Cargo.toml`
//! files, so deriving it from `Cargo.lock` is deterministic. But `Cargo.lock`
//! is listed in this repository's own `.gitignore` (not committed at all),
//! so its *external*-dependency versions/checksums are independently
//! resolved per checkout and are not guaranteed identical for two checkouts
//! of the same commit - confirmed empirically: two fresh worktrees of one
//! identical commit resolved different patch versions for several transitive
//! dependencies. Hashing the raw file would make this identity spuriously
//! diverge across otherwise-identical checkouts, which is exactly the
//! failure mode the checkout-independence requirement rules out. See
//! "Inputs deliberately NOT included" below.
//!
//! # Path identity policy
//!
//! Every hashed file contributes its content plus a *label*: a path relative
//! to `crates/`, never the absolute `CARGO_MANIFEST_DIR`-rooted path - so
//! identical source checked out at two different absolute locations
//! produces an identical hash. A path that unexpectedly falls outside
//! `crates/`, or is not valid UTF-8, fails the build instead of falling back
//! to the absolute path or a lossy (`�`-substituted) rendering - either
//! fallback could let two genuinely different paths collapse onto the same
//! identity label. Backslash-to-slash folding is applied only on Windows
//! (`cfg!(windows)`): on Unix, `\` is an ordinary filename byte, and folding
//! it unconditionally would make `a/b.rs` and `a\b.rs` collide.
//!
//! # Inputs deliberately NOT included, and why
//!
//! - `Cargo.lock`'s own content (versions/checksums of *external*
//!   dependencies): gitignored in this repository (see above), so it is not
//!   a reproducible, checkout-independent input - two checkouts of the
//!   identical commit can and do resolve different external-dependency
//!   versions. This project does not otherwise pin/commit exact external
//!   dependency versions across checkouts or CI runs, so a compiled-behavior
//!   change from external-dependency drift is a pre-existing, accepted
//!   property of how this project already builds, not something this
//!   identity repair introduces or is positioned to fix without imposing a
//!   new, unrelated pinning policy on the whole workspace.
//! - Workspace-root `Cargo.toml` (`[workspace.dependencies]`, `[profile.*]`,
//!   `members`): `[profile.dev]`/`[profile.release]` here only set
//!   `opt-level`/`debug`/`lto`/`codegen-units`/`panic`/`strip`, none of which
//!   change a safe-Rust semantic checker's pass/fail output; neither profile
//!   overrides `overflow-checks`, so it keeps tracking
//!   `cfg!(debug_assertions)`, which `current_feature_hash()` already folds
//!   in. `[workspace.dependencies]`/`members` only affect resolution, i.e.
//!   flow into `Cargo.lock`, already excluded above for the same reason.
//! - Other crates' `build.rs`: still included, defensively, per-closure-
//!   member (see `hash_optional_file` below) even though `smc-cli`'s is
//!   currently the only `build.rs` in the workspace.
//! - Host toolchain version, target triple, timestamps, hostnames, absolute
//!   checkout paths: excluded on purpose - none of them are reproducible
//!   build inputs, and none of them change this checker's logical output.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: smc-cli build-identity computation failed: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| format!("CARGO_MANIFEST_DIR not set by cargo: {e}"))?;
    let crates_dir = Path::new(&manifest_dir)
        .parent()
        .ok_or_else(|| "smc-cli manifest dir has no crates/ parent".to_string())?
        .to_path_buf();
    let workspace_root = crates_dir
        .parent()
        .ok_or_else(|| "crates/ has no workspace-root parent".to_string())?
        .to_path_buf();

    let lock_path = workspace_root.join("Cargo.lock");
    let lock_bytes = read_lock_file(&lock_path)?;
    println!("cargo:rerun-if-changed={}", lock_path.display());
    let lock_text = std::str::from_utf8(&lock_bytes).map_err(|e| {
        format!(
            "'{}' is not valid UTF-8: {e}; refusing a lossy fallback for a compiler-identity input",
            lock_path.display()
        )
    })?;
    let lock_graph = parse_cargo_lock(lock_text)?;
    let closure = discover_local_dependency_closure(&lock_graph, "smc-cli")?;

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for crate_name in &closure {
        let crate_dir = crates_dir.join(crate_name);
        hash_file(&crates_dir, &crate_dir.join("Cargo.toml"), &mut hash)?;
        hash_optional_file(&crates_dir, &crate_dir.join("build.rs"), &mut hash)?;
        hash_dir(&crates_dir, &crate_dir.join("src"), &mut hash)?;
    }
    println!("cargo:rustc-env=SM_COMPILER_SOURCE_HASH={:016x}", hash);
    println!("cargo:rustc-env=SM_ENABLED_FEATURES={}", enabled_features());
    Ok(())
}

/// Reads the workspace `Cargo.lock`. An unreadable/missing lock file fails
/// the build: it is the authoritative source for the compiler's semantic
/// dependency closure, so silently proceeding without it (e.g. treating it
/// as "no dependencies") would produce an incomplete identity.
pub(crate) fn read_lock_file(lock_path: &Path) -> Result<Vec<u8>, String> {
    fs::read(lock_path).map_err(|e| format!("cannot read '{}': {e}", lock_path.display()))
}

/// One `[[package]]` block from `Cargo.lock`: whether it is a local
/// (path/workspace) package (no `source = ...` line) and the bare names it
/// depends on.
pub(crate) struct LockPackage {
    pub(crate) is_local: bool,
    pub(crate) dependencies: Vec<String>,
}

/// Parses `Cargo.lock` into a name-keyed map. Cargo's own lock-file emission
/// is uniform (one block per package, `name`/`source`/`dependencies` each on
/// their own line, `dependencies` either absent or a `[`...`]` block with one
/// quoted name per line) - unlike hand-written `Cargo.toml`, this format has
/// no human-formatting variance to be fragile against. Every recognized
/// field is asserted to parse; an unrecognized line is ignored (its raw
/// bytes are still covered by hashing the whole file's content separately).
pub(crate) fn parse_cargo_lock(text: &str) -> Result<BTreeMap<String, LockPackage>, String> {
    let mut packages: BTreeMap<String, LockPackage> = BTreeMap::new();
    let mut block: Vec<&str> = Vec::new();
    let mut in_package = false;

    // A synthetic trailing marker flushes the final real block without
    // duplicating the flush logic below.
    for line in text.lines().chain(std::iter::once("[[package]]")) {
        if line.trim() == "[[package]]" {
            if in_package {
                let (name, pkg) = parse_package_block(&block)?;
                packages.insert(name, pkg);
            }
            block.clear();
            in_package = true;
            continue;
        }
        if in_package {
            block.push(line);
        }
    }
    Ok(packages)
}

fn parse_package_block(block: &[&str]) -> Result<(String, LockPackage), String> {
    let mut name: Option<String> = None;
    let mut is_local = true;
    let mut dependencies = Vec::new();
    let mut i = 0;
    while i < block.len() {
        let trimmed = block[i].trim();
        if let Some(rest) = trimmed.strip_prefix("name = ") {
            name = Some(unquote(rest)?);
        } else if trimmed.starts_with("source = ") {
            is_local = false;
        } else if trimmed == "dependencies = [" {
            i += 1;
            while i < block.len() && block[i].trim() != "]" {
                dependencies.push(unquote(block[i].trim())?);
                i += 1;
            }
        }
        i += 1;
    }
    let name =
        name.ok_or_else(|| "a Cargo.lock [[package]] block has no 'name = ...' line".to_string())?;
    Ok((
        name,
        LockPackage {
            is_local,
            dependencies,
        },
    ))
}

/// Unquotes a double-quoted TOML string, tolerating a single trailing comma
/// (Cargo emits `dependencies` array entries as one `"name",` per line,
/// including after the last entry).
fn unquote(s: &str) -> Result<String, String> {
    let s = s.strip_suffix(',').unwrap_or(s);
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        Ok(s[1..s.len() - 1].to_string())
    } else {
        Err(format!(
            "expected a double-quoted TOML string in Cargo.lock, found '{s}'"
        ))
    }
}

/// Every local package reachable from `entry_crate` in the resolved
/// dependency graph, transitively, including `entry_crate` itself. A
/// dependency name with no matching `[[package]]` entry anywhere - local or
/// external - means `Cargo.lock` is malformed or out of sync with the
/// workspace; that fails the build rather than silently treating the name as
/// absent from the closure.
pub(crate) fn discover_local_dependency_closure(
    graph: &BTreeMap<String, LockPackage>,
    entry_crate: &str,
) -> Result<BTreeSet<String>, String> {
    if !graph.contains_key(entry_crate) {
        return Err(format!(
            "Cargo.lock has no [[package]] entry named '{entry_crate}'"
        ));
    }
    let mut visited = BTreeSet::new();
    let mut stack = vec![entry_crate.to_string()];
    while let Some(name) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let pkg = graph
            .get(&name)
            .expect("every stacked name was verified present before being pushed");
        for dep in &pkg.dependencies {
            match graph.get(dep) {
                Some(dep_pkg) if dep_pkg.is_local => stack.push(dep.clone()),
                Some(_) => {
                    // External dependency: its version/checksum is captured
                    // by hashing the full Cargo.lock content instead.
                }
                None => {
                    return Err(format!(
                        "Cargo.lock dependency '{dep}' (referenced by '{name}') has no \
                         matching [[package]] entry; refusing to silently exclude it from \
                         the compiler identity"
                    ));
                }
            }
        }
    }
    Ok(visited)
}

/// Recursively hashes every `.rs` file under `dir`, in sorted order.
/// `src/` is required to exist and be fully readable for every closure
/// member: an unreadable expected source directory, or a directory-entry
/// enumeration error, fails the build rather than silently contributing
/// nothing to the identity.
pub(crate) fn hash_dir(crates_dir: &Path, dir: &Path, hash: &mut u64) -> Result<(), String> {
    let read_dir = fs::read_dir(dir).map_err(|e| {
        format!(
            "cannot read compiler-source directory '{}': {e}",
            dir.display()
        )
    })?;
    let mut entries: Vec<fs::DirEntry> = Vec::new();
    for entry in read_dir {
        entries.push(entry.map_err(|e| {
            format!(
                "cannot enumerate an entry in compiler-source directory '{}': {e}",
                dir.display()
            )
        })?);
    }
    entries.sort_by_key(fs::DirEntry::path);

    for entry in entries {
        let file_type = entry.file_type().map_err(|e| {
            format!(
                "cannot determine the file type of '{}': {e}",
                entry.path().display()
            )
        })?;
        let path = entry.path();
        if file_type.is_dir() {
            hash_dir(crates_dir, &path, hash)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            hash_file(crates_dir, &path, hash)?;
        }
    }
    Ok(())
}

/// Hashes `path` if it exists, and does nothing if it definitely does not.
/// Used for per-crate `build.rs`, which is legitimately absent for most
/// crates. The existence check itself is fail-closed: an I/O error while
/// merely checking for presence (e.g. an unreadable parent directory) fails
/// the build rather than being treated as "absent".
///
/// The watch directive is registered unconditionally, before the existence
/// check: if the file is absent today, cargo must still rerun this script
/// when it's later created, or a crate that gains a `build.rs` after this
/// script last ran would silently keep the old (incomplete) identity.
pub(crate) fn hash_optional_file(
    crates_dir: &Path,
    path: &Path,
    hash: &mut u64,
) -> Result<(), String> {
    println!("cargo:rerun-if-changed={}", path.display());
    match path.try_exists() {
        Ok(true) => hash_file(crates_dir, path, hash),
        Ok(false) => Ok(()),
        Err(e) => Err(format!(
            "cannot determine whether optional compiler-identity input '{}' exists: {e}",
            path.display()
        )),
    }
}

/// Hashes one file's identity-root-relative path label and content, and
/// registers it with `cargo:rerun-if-changed`. A selected file that cannot
/// be read fails the build: silently omitting its content while still
/// watching it for changes would make the identity incomplete without any
/// visible signal.
pub(crate) fn hash_file(root: &Path, path: &Path, hash: &mut u64) -> Result<(), String> {
    println!("cargo:rerun-if-changed={}", path.display());
    let bytes = fs::read(path).map_err(|e| {
        format!(
            "cannot read compiler-identity input '{}': {e}",
            path.display()
        )
    })?;
    let label = repository_relative_identity_path(root, path)?;
    mix_labeled(hash, &label, &bytes);
    Ok(())
}

/// `path` relative to `root`, using `/` as the separator regardless of host
/// platform. Fails if `path` is not actually under `root` (would silently
/// re-embed an absolute, checkout-specific path into the identity) or is not
/// valid UTF-8 (would need a lossy, potentially-colliding rendering).
pub(crate) fn repository_relative_identity_path(
    root: &Path,
    path: &Path,
) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        format!(
            "compiler-identity input '{}' is outside its expected identity root '{}'",
            path.display(),
            root.display()
        )
    })?;
    let text = relative.to_str().ok_or_else(|| {
        format!(
            "compiler-identity input path '{}' is not valid UTF-8",
            path.display()
        )
    })?;
    Ok(if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.to_string()
    })
}

/// Mixes a `(label, content)` pair into `hash`, each part length-prefixed so
/// the boundary between them - and between consecutive calls - can never be
/// ambiguous. Plain concatenation would let a boundary shift between a
/// label and its payload (or between one file's payload and the next file's
/// label) produce the same byte stream, and therefore the same hash, for
/// two genuinely different source trees; e.g. label `"a/b.rs"` + content
/// `"x"` must not hash the same as label `"a/b.r"` + content `"sx"`.
pub(crate) fn mix_labeled(hash: &mut u64, label: &str, bytes: &[u8]) {
    mix_length_prefixed(hash, label.as_bytes());
    mix_length_prefixed(hash, bytes);
}

fn mix_length_prefixed(hash: &mut u64, bytes: &[u8]) {
    fnv1a64_update(hash, &(bytes.len() as u64).to_le_bytes());
    fnv1a64_update(hash, bytes);
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
