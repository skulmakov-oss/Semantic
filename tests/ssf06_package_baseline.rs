use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_workspace(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "semantic_ssf06_{label}_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&root).expect("create workspace");
    root
}

fn write_package(root: &Path, manifest: &str, source: &str) {
    std::fs::create_dir_all(root.join("src")).expect("create source root");
    std::fs::write(root.join("Semantic.package"), manifest).expect("write manifest");
    std::fs::write(root.join("src/main.sm"), source).expect("write source");
}

fn smc(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

fn inspect(root: &Path) -> Output {
    smc(&["package", "inspect", &root.to_string_lossy()])
}

#[test]
fn package_contract_freezes_scope_and_authority_boundaries() {
    let contract = include_str!("../docs/spec/package_baseline_v0.md");
    for required in [
        "semantic.foundation.package/0.1",
        "semantic.foundation.package.provenance/0.1",
        "smc package inspect <project-root>",
        "not** a cryptographic trust claim",
        "inventory only",
        "No registry",
        "SSF-07",
    ] {
        assert!(
            contract.contains(required),
            "missing contract text: {required}"
        );
    }
}

#[test]
fn local_graph_record_is_checkout_independent_and_deterministic() {
    let root = temp_workspace("deterministic");
    let app = root.join("app");
    let alpha = root.join("alpha");
    let zeta = root.join("zeta");
    write_package(
        &alpha,
        "format 1\npackage alpha\nmanifest_dir .\nmodule_root src\ncapability fs.read\n",
        "fn alpha_value(value: i32) -> i32 { return value; }\n",
    );
    write_package(
        &zeta,
        "format 1\npackage zeta\nmanifest_dir .\nmodule_root src\n",
        "fn zeta_value(value: i32) -> i32 { return value; }\n",
    );
    write_package(
        &app,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep z zeta ../zeta\ndep a alpha ../alpha\ncapability stdout.write\n",
        "Import \"a::main.sm\"\nImport \"z::main.sm\"\nfn main() { assert(alpha_value(1) == zeta_value(1)); return; }\n",
    );

    let first = inspect(&app);
    let second = inspect(&app);
    assert!(
        first.status.success(),
        "inspect failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    std::fs::write(
        alpha.join("src/main.sm"),
        "fn alpha_value(value: i32) -> i32 { return value; }\r\n",
    )
    .expect("rewrite with CRLF");
    assert_eq!(first.stdout, inspect(&app).stdout);
    let record: serde_json::Value = serde_json::from_slice(&first.stdout).expect("record JSON");
    assert_eq!(
        record["schema"],
        "semantic.foundation.package.provenance/0.1"
    );
    assert_eq!(record["fingerprint_algorithm"], "fnv1a64-non-cryptographic");
    assert_eq!(record["root_package"], "app");
    assert!(record["graph_fingerprint"]
        .as_str()
        .expect("graph fingerprint")
        .starts_with("fnv1a64:"));
    let packages = record["packages"].as_array().expect("packages");
    assert_eq!(
        packages
            .iter()
            .map(|package| package["name"].as_str().expect("name"))
            .collect::<Vec<_>>(),
        vec!["alpha", "app", "zeta"]
    );
    assert_eq!(packages[1]["capability_requests"][0], "stdout.write");
    assert!(!String::from_utf8_lossy(&first.stdout).contains(root.to_string_lossy().as_ref()));

    let app_arg = app.to_string_lossy();
    for command in ["check", "run"] {
        let output = smc(&[command, &app_arg]);
        assert!(
            output.status.success(),
            "{command} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn cycles_missing_dependencies_and_duplicate_names_reject_deterministically() {
    let root = temp_workspace("invalid_graphs");
    let a = root.join("a");
    let b = root.join("b");
    write_package(
        &a,
        "format 1\npackage a\nmanifest_dir .\nmodule_root src\ndep b b ../b\n",
        "fn main() { return; }\n",
    );
    write_package(
        &b,
        "format 1\npackage b\nmanifest_dir .\nmodule_root src\ndep a a ../a\n",
        "fn main() { return; }\n",
    );
    let cycle_first = inspect(&a);
    let cycle_second = inspect(&a);
    assert!(!cycle_first.status.success());
    assert_eq!(cycle_first.stderr, cycle_second.stderr);
    assert!(String::from_utf8_lossy(&cycle_first.stderr)
        .contains("package dependency cycle: a -> b -> a"));

    std::fs::write(
        a.join("Semantic.package"),
        "format 1\npackage a\nmanifest_dir .\nmodule_root src\ndep missing missing ../missing\n",
    )
    .expect("replace manifest");
    let missing = inspect(&a);
    assert!(!missing.status.success());
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("dependency alias 'missing' expected")
    );

    let first = root.join("first");
    let second = root.join("second");
    write_package(
        &first,
        "format 1\npackage shared\nmanifest_dir .\nmodule_root src\n",
        "fn main() { return; }\n",
    );
    write_package(
        &second,
        "format 1\npackage shared\nmanifest_dir .\nmodule_root src\n",
        "fn main() { return; }\n",
    );
    std::fs::write(
        a.join("Semantic.package"),
        "format 1\npackage a\nmanifest_dir .\nmodule_root src\ndep first shared ../first\ndep second shared ../second\n",
    )
    .expect("replace manifest");
    let duplicate = inspect(&a);
    assert!(!duplicate.status.success());
    assert!(
        String::from_utf8_lossy(&duplicate.stderr).contains("duplicate package identity 'shared'")
    );

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn pinned_identity_and_root_policy_fail_closed() {
    let root = temp_workspace("identity");
    let app = root.join("app");
    let dependency = root.join("dependency");
    write_package(
        &dependency,
        "format 1\npackage dependency\nmanifest_dir .\nmodule_root src\n",
        "fn value(input: i32) -> i32 { return input; }\n",
    );
    write_package(
        &app,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep dependency dependency ../dependency\n",
        "Import \"dependency::main.sm\"\nfn main() { assert(value(1) == 1); return; }\n",
    );
    let initial = inspect(&app);
    assert!(initial.status.success());
    let record: serde_json::Value = serde_json::from_slice(&initial.stdout).expect("record JSON");
    let dependency_record = record["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .find(|package| package["name"] == "dependency")
        .expect("dependency record");
    let manifest_fingerprint = dependency_record["manifest_fingerprint"]
        .as_str()
        .expect("manifest fingerprint");
    let content_fingerprint = dependency_record["content_fingerprint"]
        .as_str()
        .expect("content fingerprint");
    std::fs::write(
        app.join("Semantic.package"),
        format!(
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep dependency dependency ../dependency {manifest_fingerprint} {content_fingerprint}\n"
        ),
    )
    .expect("pin dependency");
    assert!(inspect(&app).status.success());
    assert!(smc(&["check", &app.to_string_lossy()]).status.success());

    std::fs::write(
        dependency.join("src/main.sm"),
        "fn value(input: i32) -> i32 { return input + 1; }\n",
    )
    .expect("change dependency content");
    let mismatch = inspect(&app);
    assert!(!mismatch.status.success());
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("content fingerprint mismatch"));
    let load_mismatch = smc(&["check", &app.to_string_lossy()]);
    assert!(!load_mismatch.status.success());
    assert!(String::from_utf8_lossy(&load_mismatch.stderr).contains("content fingerprint mismatch"));

    std::fs::write(
        app.join("Semantic.package"),
        "format 1\npackage app\nmanifest_dir ..\nmodule_root src\n",
    )
    .expect("escape manifest");
    let escape = inspect(&app);
    assert!(!escape.status.success());
    assert!(String::from_utf8_lossy(&escape.stderr).contains("must not escape its declared root"));

    std::fs::write(
        app.join("Semantic.package"),
        format!(
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep dependency dependency {}\n",
            dependency.to_string_lossy().replace('\\', "/")
        ),
    )
    .expect("absolute dependency manifest");
    let absolute = inspect(&app);
    assert!(!absolute.status.success());
    assert!(String::from_utf8_lossy(&absolute.stderr).contains("must be relative"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn package_content_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let root = temp_workspace("symlink");
    let package = root.join("app");
    write_package(
        &package,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\n",
        "fn main() { return; }\n",
    );
    std::fs::write(root.join("outside.sm"), "fn outside() { return; }\n").expect("outside");
    symlink(root.join("outside.sm"), package.join("src/linked.sm")).expect("create symlink");

    let output = inspect(&package);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink or reparse point"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn package_entry_symlink_cannot_escape_before_manifest_lookup() {
    use std::os::unix::fs::symlink;

    let root = temp_workspace("entry_symlink");
    let package = root.join("app");
    write_package(
        &package,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\n",
        "fn main() { return; }\n",
    );
    let entry = package.join("src/main.sm");
    std::fs::remove_file(&entry).expect("remove real entry");
    let outside = root.join("outside.sm");
    std::fs::write(&outside, "fn main() { return; }\n").expect("outside source");
    symlink(&outside, &entry).expect("create entry symlink");

    let output = smc(&["check", &package.to_string_lossy()]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink or reparse point"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn dependency_relative_import_cannot_escape_module_root() {
    let root = temp_workspace("dependency_import_escape");
    let app = root.join("app");
    let dependency = root.join("dependency");
    write_package(
        &dependency,
        "format 1\npackage dependency\nmanifest_dir .\nmodule_root src\n",
        "Import \"../../outside.sm\"\nfn value() -> i32 { return leaked(); }\n",
    );
    std::fs::write(
        root.join("outside.sm"),
        "fn leaked() -> i32 { return 1; }\n",
    )
    .expect("outside source");
    write_package(
        &app,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep dependency dependency ../dependency\n",
        "Import \"dependency::main.sm\"\nfn main() { assert(value() == 1); return; }\n",
    );

    let output = smc(&["check", &app.to_string_lossy()]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("relative package import '../../outside.sm' escapes package module_root"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn nested_manifest_cannot_rebind_package_authority() {
    let root = temp_workspace("nested_manifest");
    let app = root.join("app");
    let dependency = root.join("dependency");
    write_package(
        &dependency,
        "format 1\npackage dependency\nmanifest_dir .\nmodule_root src\n",
        "Import \"nested/helper.sm\"\nfn value() -> i32 { return helper(); }\n",
    );
    std::fs::create_dir_all(dependency.join("src/nested")).expect("nested source root");
    std::fs::write(
        dependency.join("src/nested/Semantic.package"),
        "format 1\npackage nested\nmanifest_dir .\nmodule_root .\n",
    )
    .expect("nested manifest");
    std::fs::write(
        dependency.join("src/nested/helper.sm"),
        "fn helper() -> i32 { return 1; }\n",
    )
    .expect("nested helper");
    write_package(
        &app,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep dependency dependency ../dependency\n",
        "Import \"dependency::main.sm\"\nfn main() { assert(value() == 1); return; }\n",
    );

    let inspect_output = inspect(&dependency);
    assert!(!inspect_output.status.success());
    assert!(String::from_utf8_lossy(&inspect_output.stderr).contains("nested package manifest"));

    let check_output = smc(&["check", &app.to_string_lossy()]);
    assert!(!check_output.status.success());
    assert!(String::from_utf8_lossy(&check_output.stderr).contains("nested package manifest"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn authority_manifest_is_allowed_at_package_root_module_root() {
    let root = temp_workspace("root_module_root");
    write_package(
        &root,
        "format 1\npackage flat\nmanifest_dir .\nmodule_root .\n",
        "fn main() { return; }\n",
    );

    let output = inspect(&root);
    assert!(
        output.status.success(),
        "inspect failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn package_import_rejects_unfingerprinted_module_extension() {
    let root = temp_workspace("module_extension");
    let app = root.join("app");
    let dependency = root.join("dependency");
    write_package(
        &dependency,
        "format 1\npackage dependency\nmanifest_dir .\nmodule_root src\n",
        "fn unused() { return; }\n",
    );
    std::fs::write(
        dependency.join("src/helper.txt"),
        "fn helper() -> i32 { return 1; }\n",
    )
    .expect("text extension module");
    write_package(
        &app,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep dependency dependency ../dependency\n",
        "Import \"dependency::helper.txt\"\nfn main() { assert(helper() == 1); return; }\n",
    );

    let output = smc(&["check", &app.to_string_lossy()]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("must use a .sm or .exo module extension")
    );

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn authority_manifest_symlink_is_rejected_before_read() {
    use std::os::unix::fs::symlink;

    let root = temp_workspace("manifest_symlink");
    let package = root.join("app");
    std::fs::create_dir_all(package.join("src")).expect("source root");
    std::fs::write(package.join("src/main.sm"), "fn main() { return; }\n").expect("entry source");
    let outside_manifest = root.join("outside.package");
    std::fs::write(
        &outside_manifest,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\n",
    )
    .expect("outside manifest");
    symlink(&outside_manifest, package.join("Semantic.package")).expect("manifest symlink");

    let output = smc(&["check", &package.to_string_lossy()]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink or reparse point"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn dependency_authority_manifest_symlink_is_rejected_by_inspect() {
    use std::os::unix::fs::symlink;

    let root = temp_workspace("dependency_manifest_symlink");
    let app = root.join("app");
    let dependency = root.join("dependency");
    let outside = root.join("outside");
    write_package(
        &app,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep dependency dependency ../dependency\n",
        "fn main() { return; }\n",
    );
    std::fs::create_dir_all(&dependency).expect("dependency root");
    write_package(
        &outside,
        "format 1\npackage dependency\nmanifest_dir .\nmodule_root src\n",
        "fn helper() { return; }\n",
    );
    symlink(
        outside.join("Semantic.package"),
        dependency.join("Semantic.package"),
    )
    .expect("dependency manifest symlink");

    let output = inspect(&app);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink or reparse point"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn capability_request_is_inventory_only_and_never_a_grant() {
    let root = temp_workspace("capability");
    write_package(
        &root,
        "format 1\npackage app\nmanifest_dir .\nmodule_root src\ncapability fs.write\n",
        "fn main() { fs_write_text(\"output.txt\", \"payload\"); return; }\n",
    );
    let root_arg = root.to_string_lossy();
    let output = smc(&["run", &root_arg, "--profile", "pure", "--root", &root_arg]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("capability FsWrite denied"));
    assert!(!root.join("output.txt").exists());

    std::fs::remove_dir_all(root).expect("cleanup");
}
