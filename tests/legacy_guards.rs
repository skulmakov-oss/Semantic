use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let prefix = normalize_repo_prefix(root);
    for rel in tracked_paths() {
        if is_under_prefix(&rel, &prefix) && rel.ends_with(".rs") {
            out.push(PathBuf::from(rel));
        }
    }
}

fn collect_files_with_extensions(root: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let prefix = normalize_repo_prefix(root);
    for rel in tracked_paths() {
        if !is_under_prefix(&rel, &prefix) {
            continue;
        }
        let p = Path::new(&rel);
        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
            if exts.contains(&ext) {
                out.push(PathBuf::from(format!("./{}", rel)));
            }
        }
    }
}

fn tracked_paths() -> BTreeSet<String> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .output()
        .expect("run git ls-files");
    assert!(output.status.success(), "git ls-files must succeed");
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).replace('\\', "/"))
        .collect()
}

fn normalize_repo_prefix(root: &Path) -> String {
    let rel = root.to_string_lossy().replace('\\', "/");
    let rel = rel.trim_start_matches("./");
    if rel == "." {
        String::new()
    } else {
        rel.trim_end_matches('/').to_string()
    }
}

fn is_under_prefix(path: &str, prefix: &str) -> bool {
    prefix.is_empty() || path == prefix || path.starts_with(&format!("{prefix}/"))
}

#[test]
fn no_path_adapter_back_to_root_src() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("crates"), &mut files);
    for f in files {
        let txt = match fs::read_to_string(&f) {
            Ok(s) => s,
            Err(_) => continue,
        };
        assert!(
            !txt.contains("#[path = \"../../../src/"),
            "forbidden legacy path adapter in {}",
            f.display()
        );
    }
}

#[test]
fn root_src_is_shim_and_bins_only_allowlist() {
    let mut rs_files = Vec::new();
    collect_rs_files(Path::new("src"), &mut rs_files);
    let mut bad = Vec::new();
    for f in rs_files {
        let rel = f.strip_prefix(".").unwrap_or(&f);
        let rel = rel.to_string_lossy().replace('\\', "/");
        let ok = rel == "src/lib.rs" || rel.starts_with("src/bin/");
        if !ok {
            bad.push(rel);
        }
    }
    assert!(
        bad.is_empty(),
        "root/src contains forbidden rust sources outside allowlist: {:?}",
        bad
    );
}

#[test]
fn root_src_contains_only_lib_and_bin_dir() {
    let mut top = BTreeSet::new();
    for path in tracked_paths() {
        if let Some(rest) = path.strip_prefix("src/") {
            if let Some((first, _)) = rest.split_once('/') {
                top.insert(first.to_string());
            } else {
                top.insert(rest.to_string());
            }
        }
    }
    let expected: BTreeSet<String> = ["lib.rs", "bin"].iter().map(|s| s.to_string()).collect();
    assert_eq!(top, expected, "root/src must contain only lib.rs and bin/");
}

#[test]
fn root_src_rust_inventory_is_explicit() {
    let mut rs_files = Vec::new();
    collect_rs_files(Path::new("src"), &mut rs_files);
    let found: BTreeSet<String> = rs_files
        .into_iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    let expected: BTreeSet<String> = [
        "src/lib.rs",
        "src/bin/smc.rs",
        "src/bin/svm.rs",
        "src/bin/ton618_core.rs",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        found, expected,
        "root/src rust inventory must stay inside the explicit cleanup allowlist"
    );
}

#[test]
fn root_src_bans_legacy_patterns() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("src"), &mut files);
    for f in files {
        let txt = match fs::read_to_string(&f) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let rel = f.to_string_lossy().replace('\\', "/");
        assert!(
            !txt.contains("legacy_"),
            "forbidden legacy marker in {}",
            rel
        );
        assert!(
            !txt.contains("#[path ="),
            "forbidden #[path = ...] in {}",
            rel
        );
        assert!(
            !txt.contains("include!("),
            "forbidden include!(...) in {}",
            rel
        );
        assert!(
            !txt.contains("mod legacy"),
            "forbidden legacy module declaration in {}",
            rel
        );
    }
}

#[test]
fn root_smc_is_thin_wrapper_over_smc_cli() {
    let root = fs::read_to_string("src/bin/smc.rs").expect("read root smc");
    let owner = fs::read_to_string("crates/smc-cli/src/app.rs").expect("read smc-cli app");

    assert!(
        root.contains("smc_cli::main_entry()"),
        "root smc must delegate to the canonical smc-cli owner"
    );
    assert!(
        !root.contains("match args[0].as_str()"),
        "root smc must not carry its own command dispatch"
    );
    assert!(
        owner.contains("pub fn main_entry()"),
        "smc-cli must own the exported CLI process entry"
    );
    assert!(
        owner.contains("pub fn run(args: Vec<String>)"),
        "smc-cli must own the command runner"
    );
}

#[test]
fn legacy_compatibility_perimeter_is_explicit_and_narrow() {
    let explicit_shims = ["src/bin/ton618_core.rs", "crates/ton618-core/src/lib.rs"];

    for path in explicit_shims {
        let txt = fs::read_to_string(path).expect("read compatibility shim");
        assert!(
            txt.contains("compatibility"),
            "legacy shim {} must declare explicit compatibility status",
            path
        );
    }

    let mut ton618_named = Vec::new();
    collect_rs_files(Path::new("src"), &mut ton618_named);
    collect_rs_files(Path::new("crates/ton618-core/src"), &mut ton618_named);
    let mut matches = Vec::new();
    for file in ton618_named {
        let rel = file.to_string_lossy().replace('\\', "/");
        if rel.contains("ton618") {
            matches.push(rel);
        }
    }
    matches.sort();
    matches.dedup();

    assert!(
        matches.iter().all(|rel| {
            rel == "src/bin/ton618_core.rs" || rel.starts_with("crates/ton618-core/src/")
        }),
        "legacy ton618 naming must remain inside the explicit compatibility perimeter only: {:?}",
        matches
    );
    assert!(
        matches.iter().any(|rel| rel == "src/bin/ton618_core.rs"),
        "legacy compatibility bin must remain explicit"
    );
}

#[test]
fn ton618_named_path_inventory_is_explicit() {
    let mut matches = BTreeSet::new();
    for rel in tracked_paths() {
        if rel.contains("ton618") {
            matches.insert(format!("./{}", rel));
        }
    }

    let expected: BTreeSet<String> = [
        "./crates/ton618-core/Cargo.toml",
        "./crates/ton618-core/src/arena.rs",
        "./crates/ton618-core/src/diagnostics.rs",
        "./crates/ton618-core/src/ids.rs",
        "./crates/ton618-core/src/lib.rs",
        "./crates/ton618-core/src/sigtable.rs",
        "./crates/ton618-core/src/source.rs",
        "./docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md",
        "./src/bin/ton618_core.rs",
        "./ton618_legacy/.gitignore",
        "./ton618_legacy/LICENSE-APACHE",
        "./ton618_legacy/LICENSE-MIT",
        "./ton618_legacy/README.md",
        "./ton618_legacy/benches/bank_soa.rs",
        "./ton618_legacy/benches/simd_ops.rs",
        "./ton618_legacy/Cargo.toml",
        "./ton618_legacy/clippy.toml",
        "./ton618_legacy/rustfmt.toml",
        "./ton618_legacy/src/bank.rs",
        "./ton618_legacy/src/bench/mod.rs",
        "./ton618_legacy/src/delta.rs",
        "./ton618_legacy/src/lib.rs",
        "./ton618_legacy/src/masks.rs",
        "./ton618_legacy/src/prelude.rs",
        "./ton618_legacy/src/reg.rs",
        "./ton618_legacy/src/simd/arm_neon.rs",
        "./ton618_legacy/src/simd/mod.rs",
        "./ton618_legacy/src/simd/x86_avx2.rs",
        "./ton618_legacy/tests/correctness.rs",
        "./ton618_legacy/tests/no_msb_leakage.rs",
        "./ton618_legacy/tests/simd_equivalence.rs",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(
        matches, expected,
        "TON618-named filesystem perimeter must stay inside the explicit allowlist"
    );
}

#[test]
fn ton618_content_inventory_is_explicit() {
    let mut files = Vec::new();
    collect_files_with_extensions(Path::new("."), &["rs", "md", "toml", "lock"], &mut files);

    let mut matches = BTreeSet::new();
    for file in files {
        let rel = file.to_string_lossy().replace('\\', "/");
        let txt = match fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if txt.contains("ton618_core") || txt.contains("ton618-core") {
            matches.insert(rel);
        }
    }

    let expected: BTreeSet<String> = [
        "./Cargo.toml",
        "./README.md",
        "./crates/sm-front/Cargo.toml",
        "./crates/sm-front/src/lexer.rs",
        "./crates/sm-front/src/parser.rs",
        "./crates/sm-front/src/types.rs",
        "./crates/sm-sema/Cargo.toml",
        "./crates/sm-sema/src/alloc_core.rs",
        "./crates/sm-sema/src/lib.rs",
        "./crates/sm-sema/src/std_adapters.rs",
        "./crates/smc-cli/Cargo.toml",
        "./crates/smc-cli/src/app.rs",
        "./crates/smc-cli/src/lib.rs",
        "./crates/ton618-core/Cargo.toml",
        "./crates/ton618-core/src/lib.rs",
        "./docs/NAMING.md",
        "./docs/NO_STD.md",
        "./docs/architecture/dependency_boundary_rules.md",
        "./docs/architecture/module_ownership_map.md",
        "./docs/legacy-map.md",
        "./docs/roadmap/language_maturity/root_legacy_cleanup_full_scope.md",
        "./docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md",
        "./docs/roadmap/pcc/ctf_no_std_qualification_audit.md",
        "./docs/roadmap/roadmap_pulsar.md",
        "./docs/roadmap/m_tail_closeout.md",
        "./docs/roadmap/tail_t5_legacy_perimeter_check.md",
        "./src/bin/ton618_core.rs",
        "./src/lib.rs",
        "./tests/legacy_guards.rs",
        "./ton618_legacy/Cargo.toml",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(
        matches, expected,
        "files that mention TON618 compatibility names must stay inside the explicit inventory"
    );
}

#[test]
fn legacy_support_directory_is_removed() {
    assert!(
        tracked_paths()
            .iter()
            .all(|path| !path.starts_with("src/bin/support/")),
        "legacy support directory must not contain tracked compatibility helpers"
    );
}
