//! Exercises `build.rs`'s own identity-construction logic directly. A build
//! script is a separate compilation unit that `cargo test` never runs, so
//! this file pulls its source in verbatim via `#[path]` (no duplication) and
//! calls its `pub(crate)` helpers with synthetic inputs - proving the
//! fail-closed properties required by the PR #1616 compiler-identity repair:
//! same input -> same identity, checkout-path independence, no silent
//! omission on I/O failure, and no lossy/absolute-path fallback on the
//! identity's path labels.

#[path = "../build.rs"]
#[allow(dead_code)]
mod build_script;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&base).expect("mkdir");
    base
}

// --- 8.1-equivalent: same lossless identity for identical input ---

#[test]
fn hash_file_is_deterministic_for_identical_content_and_path() {
    let dir = mk_temp_dir("build_identity_same");
    fs::write(dir.join("a.rs"), b"fn main() {}").expect("write");

    let mut h1 = 0u64;
    build_script::hash_file(&dir, &dir.join("a.rs"), &mut h1).expect("hash 1");
    let mut h2 = 0u64;
    build_script::hash_file(&dir, &dir.join("a.rs"), &mut h2).expect("hash 2");

    assert_eq!(h1, h2, "identical path + content must hash identically");
}

#[test]
fn hash_file_changes_when_content_changes() {
    let dir = mk_temp_dir("build_identity_content_change");
    fs::write(dir.join("a.rs"), b"fn main() {}").expect("write");
    let mut h1 = 0u64;
    build_script::hash_file(&dir, &dir.join("a.rs"), &mut h1).expect("hash 1");

    fs::write(dir.join("a.rs"), b"fn main() { loop {} }").expect("rewrite");
    let mut h2 = 0u64;
    build_script::hash_file(&dir, &dir.join("a.rs"), &mut h2).expect("hash 2");

    assert_ne!(h1, h2, "changed content must change the identity");
}

// --- checkout-path independence ---

#[test]
fn hash_dir_is_independent_of_the_absolute_checkout_directory() {
    let root_a = mk_temp_dir("build_identity_root_a");
    let root_b = mk_temp_dir("build_identity_root_b");
    for root in [&root_a, &root_b] {
        fs::create_dir_all(root.join("crate_x/src/nested")).expect("mkdir");
        fs::write(root.join("crate_x/src/lib.rs"), b"pub fn f() {}").expect("write");
        fs::write(root.join("crate_x/src/nested/m.rs"), b"pub fn g() {}").expect("write");
    }

    let mut h_a = 0u64;
    build_script::hash_dir(&root_a, &root_a.join("crate_x/src"), &mut h_a).expect("hash a");
    let mut h_b = 0u64;
    build_script::hash_dir(&root_b, &root_b.join("crate_x/src"), &mut h_b).expect("hash b");

    assert_eq!(
        h_a, h_b,
        "identical relative source tree under two different absolute roots must hash identically"
    );
}

// --- path identity policy: root escape, UTF-8, backslash folding ---

#[test]
fn repository_relative_identity_path_rejects_a_path_outside_its_root() {
    let root = mk_temp_dir("build_identity_root_escape");
    let outside = mk_temp_dir("build_identity_outside");
    let file = outside.join("x.rs");
    fs::write(&file, b"x").expect("write");

    let result = build_script::repository_relative_identity_path(&root, &file);
    assert!(
        result.is_err(),
        "a path outside the identity root must fail closed, not fall back to the absolute path"
    );
}

#[test]
#[cfg(unix)]
fn repository_relative_identity_path_folds_backslash_only_on_windows() {
    use std::path::Path;

    let root = Path::new("/repo");
    let real_subdir = Path::new("/repo/a/b.rs");
    let literal_backslash_name = Path::new("/repo/a\\b.rs");

    let via_subdir =
        build_script::repository_relative_identity_path(root, real_subdir).expect("subdir path");
    let via_literal_backslash =
        build_script::repository_relative_identity_path(root, literal_backslash_name)
            .expect("literal-backslash path");

    assert_eq!(via_subdir, "a/b.rs");
    assert_eq!(
        via_literal_backslash, "a\\b.rs",
        "on Unix a literal backslash in a filename is an ordinary byte, not a separator"
    );
    assert_ne!(
        via_subdir, via_literal_backslash,
        "a real subdirectory and a single file with a literal backslash in its name must not collide"
    );
}

#[test]
#[cfg(unix)]
fn repository_relative_identity_path_rejects_non_utf8_paths() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    let root = Path::new("/repo");
    let non_utf8 =
        Path::new("/repo").join(OsStr::from_bytes(&[0x66, 0xFF, 0xFE, 0x2E, 0x72, 0x73]));

    let result = build_script::repository_relative_identity_path(root, &non_utf8);
    assert!(
        result.is_err(),
        "a non-UTF-8 identity path must fail closed, not fall back to a lossy rendering"
    );
}

// --- I/O failure -> build failure, never silent omission ---

#[test]
fn hash_file_fails_closed_when_the_selected_file_is_unreadable() {
    let dir = mk_temp_dir("build_identity_missing_file");
    let missing = dir.join("does_not_exist.rs");

    let mut hash = 0u64;
    let result = build_script::hash_file(&dir, &missing, &mut hash);
    assert!(
        result.is_err(),
        "an unreadable selected identity file must fail the build, not hash nothing"
    );
}

#[test]
fn hash_dir_fails_closed_when_the_expected_source_directory_is_missing() {
    let dir = mk_temp_dir("build_identity_missing_src");
    let missing_src = dir.join("src");

    let mut hash = 0u64;
    let result = build_script::hash_dir(&dir, &missing_src, &mut hash);
    assert!(
        result.is_err(),
        "an unreadable expected source directory must fail the build, not contribute nothing"
    );
}

#[test]
fn read_lock_file_fails_closed_when_cargo_lock_is_missing() {
    let dir = mk_temp_dir("build_identity_missing_lock");
    let missing_lock = dir.join("Cargo.lock");

    let result = build_script::read_lock_file(&missing_lock);
    assert!(
        result.is_err(),
        "a missing/unreadable Cargo.lock must fail the build, not be treated as an empty graph"
    );
}

// --- Cargo.lock parsing and dependency-closure completeness ---

#[test]
fn parse_cargo_lock_handles_trailing_commas_on_every_dependency_entry() {
    let text = r#"
# This file is automatically @generated by Cargo.
version = 4

[[package]]
name = "smc-cli"
version = "0.1.0"
dependencies = [
 "sm-ir",
 "serde",
]

[[package]]
name = "sm-ir"
version = "0.1.0"
dependencies = [
 "sm-format",
]

[[package]]
name = "sm-format"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    let graph = build_script::parse_cargo_lock(text).expect("parse");
    assert_eq!(graph.len(), 4);
    assert!(graph["smc-cli"].is_local);
    assert!(graph["sm-ir"].is_local);
    assert!(graph["sm-format"].is_local);
    assert!(!graph["serde"].is_local);
    assert_eq!(graph["smc-cli"].dependencies, vec!["sm-ir", "serde"]);
}

#[test]
fn parse_cargo_lock_rejects_a_package_block_with_no_name() {
    let text = "[[package]]\nversion = \"0.1.0\"\n";
    let result = build_script::parse_cargo_lock(text);
    assert!(
        result.is_err(),
        "a [[package]] block with no name = ... line must fail closed, not be silently skipped"
    );
}

fn local(deps: &[&str]) -> build_script::LockPackage {
    build_script::LockPackage {
        is_local: true,
        dependencies: deps.iter().map(|s| s.to_string()).collect(),
    }
}

fn external() -> build_script::LockPackage {
    build_script::LockPackage {
        is_local: false,
        dependencies: Vec::new(),
    }
}

#[test]
fn discover_local_dependency_closure_includes_every_reachable_local_crate() {
    let mut graph: BTreeMap<String, build_script::LockPackage> = BTreeMap::new();
    graph.insert("smc-cli".to_string(), local(&["sm-ir", "serde"]));
    graph.insert("sm-ir".to_string(), local(&["sm-format"]));
    graph.insert("sm-format".to_string(), local(&[]));
    graph.insert("serde".to_string(), external());

    let closure = build_script::discover_local_dependency_closure(&graph, "smc-cli")
        .expect("closure discovery");

    // A newly introduced local compiler crate (sm-format, reachable only
    // transitively through sm-ir) must not be silently excluded.
    assert!(closure.contains("smc-cli"));
    assert!(closure.contains("sm-ir"));
    assert!(closure.contains("sm-format"));
    assert!(
        !closure.contains("serde"),
        "external dependencies are not part of the per-crate source closure"
    );
}

#[test]
fn discover_local_dependency_closure_fails_closed_on_a_dangling_dependency_reference() {
    let mut graph: BTreeMap<String, build_script::LockPackage> = BTreeMap::new();
    graph.insert("smc-cli".to_string(), local(&["sm-ghost"]));
    // "sm-ghost" is referenced but has no [[package]] entry of its own.

    let result = build_script::discover_local_dependency_closure(&graph, "smc-cli");
    assert!(
        result.is_err(),
        "a dependency name with no matching package entry must fail the build, not be treated as absent"
    );
}

#[test]
fn discover_local_dependency_closure_fails_closed_when_entry_crate_is_missing() {
    let graph: BTreeMap<String, build_script::LockPackage> = BTreeMap::new();
    let result = build_script::discover_local_dependency_closure(&graph, "smc-cli");
    assert!(result.is_err());
}
