//! Computes a deterministic content fingerprint over the source of every
//! crate that determines `smc check`'s semantic behavior (parse, typecheck,
//! lowering, verification, execution) and exposes it to the crate as
//! `SM_COMPILER_SOURCE_HASH` via `env!()`.
//!
//! This exists so the on-disk semantic cache (see `current_toolchain_hash`
//! in `src/app.rs`) can tell two compiler builds apart even when neither
//! `Cargo.toml`'s package version nor any environment variable changed. A
//! `smc-cli` rebuilt from edited (including uncommitted) source in these
//! crates gets a different hash than the build that produced an existing
//! cache entry, so that entry is correctly treated as stale.
//!
//! Deliberately content-based rather than git-commit-based: the defect this
//! repairs was observed after a local rebuild from *uncommitted* source
//! edits, where the git HEAD commit does not change at all.

use std::fs;
use std::path::Path;

/// Crates whose source content determines what a semantic-cache entry means.
/// Kept in sync manually; adding a new crate to the check/compile/verify/run
/// pipeline should add it here too.
const SEMANTIC_CRATES: &[&str] = &[
    "sm-front",
    "sm-ir",
    "sm-sema",
    "sm-emit",
    "sm-verify",
    "sm-vm",
    "smc-cli",
];

fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let crates_dir = Path::new(&manifest_dir)
        .parent()
        .expect("smc-cli manifest dir has a crates/ parent");

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for crate_name in SEMANTIC_CRATES {
        let src_dir = crates_dir.join(crate_name).join("src");
        hash_dir(&src_dir, &mut hash);
    }

    println!("cargo:rustc-env=SM_COMPILER_SOURCE_HASH={:016x}", hash);
}

/// Recursively hashes every `.rs` file's path (relative ordering matters,
/// hence the sort) and content, and registers each with
/// `cargo:rerun-if-changed` so editing any of them re-runs this script.
/// Missing directories are skipped rather than treated as an error, since a
/// stripped-down checkout should still build.
fn hash_dir(dir: &Path, hash: &mut u64) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            hash_dir(&path, hash);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(bytes) = fs::read(&path) {
                fnv1a64_update(hash, path.to_string_lossy().as_bytes());
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
