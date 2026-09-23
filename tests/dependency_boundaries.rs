use serde_json::Value;
use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn assert_no_path_dep(path: &str, forbidden: &[&str]) {
    let src = read(path);
    for dep in forbidden {
        let needle = format!("path = \"../{dep}\"");
        assert!(
            !src.contains(&needle),
            "{path} must not depend on forbidden crate {dep}"
        );
    }
}

#[test]
fn construction_crates_do_not_depend_on_vm_or_prometheus_layers() {
    let construction = [
        "crates/sm-front/Cargo.toml",
        "crates/sm-sema/Cargo.toml",
        "crates/sm-ir/Cargo.toml",
        "crates/sm-emit/Cargo.toml",
        "crates/sm-profile/Cargo.toml",
    ];
    let forbidden = [
        "sm-vm",
        "prom-abi",
        "prom-cap",
        "prom-gates",
        "prom-runtime",
        "prom-state",
        "prom-rules",
        "prom-audit",
    ];

    for path in construction {
        assert_no_path_dep(path, &forbidden);
    }
}

#[test]
fn execution_crates_do_not_depend_on_frontend_or_sema_layers() {
    let execution = [
        "crates/sm-verify/Cargo.toml",
        "crates/sm-runtime-core/Cargo.toml",
        "crates/sm-vm/Cargo.toml",
    ];
    let forbidden = ["sm-front", "sm-sema"];

    for path in execution {
        assert_no_path_dep(path, &forbidden);
    }
}

#[test]
fn integration_crates_do_not_depend_on_compiler_layers() {
    let integration = [
        "crates/prom-abi/Cargo.toml",
        "crates/prom-cap/Cargo.toml",
        "crates/prom-gates/Cargo.toml",
        "crates/prom-runtime/Cargo.toml",
        "crates/prom-state/Cargo.toml",
        "crates/prom-rules/Cargo.toml",
        "crates/prom-audit/Cargo.toml",
    ];
    let forbidden = ["sm-front", "sm-sema", "sm-ir", "sm-emit"];

    for path in integration {
        assert_no_path_dep(path, &forbidden);
    }
}

fn assert_cargo_metadata_isolation() {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .expect("failed to execute cargo metadata");
    assert!(output.status.success(), "cargo metadata execution failed");

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata output is not valid json");
    let packages = json["packages"]
        .as_array()
        .expect("cargo metadata missing packages array");

    assert!(
        packages.len() >= 30,
        "expected at least 30 workspace packages in cargo metadata, found {}",
        packages.len()
    );

    let sm_diag_pkg = packages
        .iter()
        .find(|p| p["name"].as_str() == Some("sm-diagnostic"))
        .expect("sm-diagnostic package not found in cargo metadata");

    let sm_diag_deps = sm_diag_pkg["dependencies"]
        .as_array()
        .expect("sm-diagnostic dependencies not an array");
    assert!(
        sm_diag_deps.is_empty(),
        "crates/sm-diagnostic must have zero dependencies in C0, found: {sm_diag_deps:?}"
    );

    let sm_diag_id = sm_diag_pkg["id"].as_str().unwrap_or_default();
    let workspace_members = json["workspace_members"]
        .as_array()
        .expect("cargo metadata missing workspace_members");
    assert!(
        workspace_members
            .iter()
            .any(|m| m.as_str() == Some(sm_diag_id)),
        "sm-diagnostic id must be present in workspace_members in cargo metadata"
    );

    for pkg in packages {
        let pkg_name = pkg["name"].as_str().unwrap_or_default();
        if pkg_name == "sm-diagnostic" {
            continue;
        }
        if let Some(deps) = pkg["dependencies"].as_array() {
            for dep in deps {
                let dep_name = dep["name"].as_str().unwrap_or_default();
                let dep_rename = dep["rename"].as_str();
                let dep_path = dep["path"].as_str().unwrap_or_default();

                assert_ne!(
                    dep_name, "sm-diagnostic",
                    "package {pkg_name} depends on sm-diagnostic in cargo metadata"
                );
                if let Some(rename) = dep_rename {
                    assert_ne!(
                        rename, "sm-diagnostic",
                        "package {pkg_name} renames dependency to sm-diagnostic in cargo metadata"
                    );
                }
                let norm_path = dep_path.replace('\\', "/");
                assert!(
                    !norm_path.ends_with("crates/sm-diagnostic")
                        && !norm_path.contains("/crates/sm-diagnostic/"),
                    "package {pkg_name} depends on path referencing sm-diagnostic in cargo metadata: {dep_path}"
                );
            }
        }
    }
}

fn assert_sm_diagnostic_manifest_zero_dependencies(manifest: &str) {
    let mut current_section = "";
    let mut has_package_section = false;
    let mut has_dependencies_section = false;

    for (line_idx, line) in manifest.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check for section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section = trimmed[1..trimmed.len() - 1].trim();
            match section {
                "package" => {
                    has_package_section = true;
                    current_section = "package";
                }
                "dependencies" => {
                    has_dependencies_section = true;
                    current_section = "dependencies";
                }
                other => {
                    panic!(
                        "crates/sm-diagnostic must not declare section [{other}] in C0: line {}",
                        line_idx + 1
                    );
                }
            }
            continue;
        }

        // Any non-header line before [package] is an illegal unsectioned key in C0
        if current_section.is_empty() {
            panic!(
                "crates/sm-diagnostic has unsectioned key-value before [package] in C0: line {}: {trimmed}",
                line_idx + 1
            );
        }

        // Dotted dependency keys, target-specific keys, or path dependencies anywhere
        if trimmed.contains("dependencies")
            || trimmed.starts_with("target.")
            || trimmed.contains("path =")
        {
            panic!(
                "crates/sm-diagnostic must not declare dependencies or targets in C0: line {}: {trimmed}",
                line_idx + 1
            );
        }

        // Within [dependencies], no dependencies are permitted
        if current_section == "dependencies" {
            panic!(
                "crates/sm-diagnostic [dependencies] must be empty in C0: line {}: {trimmed}",
                line_idx + 1
            );
        }
    }

    assert!(
        has_package_section,
        "crates/sm-diagnostic must contain [package] section"
    );
    assert!(
        has_dependencies_section,
        "crates/sm-diagnostic must contain [dependencies] section"
    );
}

fn assert_root_manifest_sm_diagnostic_isolation(root_manifest: &str) {
    let mut in_workspace_members = false;
    for (line_idx, line) in root_manifest.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("members = [") {
            in_workspace_members = true;
            continue;
        }
        if in_workspace_members && trimmed.starts_with(']') {
            in_workspace_members = false;
            continue;
        }

        if trimmed.contains("sm-diagnostic") {
            assert!(
                in_workspace_members,
                "root Cargo.toml line {}: sm-diagnostic is only authorized in [workspace].members, found: {trimmed}",
                line_idx + 1
            );
        }
    }
}

#[test]
fn sm_diagnostic_c0_has_zero_dependencies_and_is_isolated() {
    // Layer A: Authoritative semantic dependency graph inspection via Cargo metadata
    assert_cargo_metadata_isolation();

    // Layer B: Narrow governance assertions on crate manifests
    let sm_diag_manifest = read("crates/sm-diagnostic/Cargo.toml");
    assert_sm_diagnostic_manifest_zero_dependencies(&sm_diag_manifest);

    let root_manifest = read("Cargo.toml");
    assert_root_manifest_sm_diagnostic_isolation(&root_manifest);
}
