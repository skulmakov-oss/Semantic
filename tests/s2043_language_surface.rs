//! S2043 CP06: `smc language describe --format json`
//! (`semantic.language-surface` v1, docs/spec/language_surface_v1.md).
//!
//! The document is a projection of one registry,
//! `sm_front::language_surface::LANGUAGE_FEATURES`. These tests keep that
//! registry honest against the running pipeline of this very build: every
//! declared stage status is re-derived by actually running the feature's probe
//! through `check -> compile (O0, O1) -> verify -> run-smc`, and every
//! executable feature's negative probe must be rejected at its declared stage.

use sm_front::language_surface::{
    ReleaseState, Stage, StageStatus, LANGUAGE_FEATURES, LANGUAGE_SURFACE_SCHEMA_ID,
    LANGUAGE_SURFACE_SCHEMA_VERSION,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn smc(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn describe() -> (String, serde_json::Value) {
    let (code, stdout, stderr) = smc(&["language", "describe", "--format", "json"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    let json = serde_json::from_str(&stdout).expect("describe output is JSON");
    (stdout, json)
}

/// The stage that rejects a `smc check --format json` diagnostic code.
fn check_code_stage(code: &str) -> Stage {
    match code {
        "E0001" | "E0002" | "E0003" | "E0004" | "E0101" => Stage::Lex,
        "E0005" | "E0006" | "E0007" | "E0008" => Stage::Parse,
        "E0201" => Stage::Typecheck,
        other => panic!("check code {other} has no stage mapping; extend it deliberately"),
    }
}

/// Runs `src` through the real pipeline and returns the observed status of
/// every stage, in `Stage::ALL` order.
fn observe(dir: &Path, name: &str, src: &str) -> [StageStatus; 7] {
    use StageStatus::{NotApplicable, Qualified, Rejected};
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(&path, src).unwrap();
    let path = path.to_str().unwrap();
    let mut out = [NotApplicable; 7];
    let reject = |out: &mut [StageStatus; 7], stage: Stage| {
        for s in Stage::ALL {
            if (s as usize) < (stage as usize) {
                out[s as usize] = Qualified;
            }
        }
        out[stage as usize] = Rejected;
        *out
    };

    let (code, stdout, stderr) = smc(&["check", path, "--format", "json"]);
    if code != 0 {
        let report: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("{name}: {e}: {stdout}{stderr}"));
        let diag_code = report["diagnostics"][0]["code"].as_str().unwrap_or("");
        return reject(&mut out, check_code_stage(diag_code));
    }

    let mut artifacts = Vec::new();
    let mut compiled = Vec::new();
    for level in ["O0", "O1"] {
        let smc_path = dir.join(format!("{name}.{level}.smc"));
        let (code, _, _) = smc(&[
            "compile",
            path,
            "-o",
            smc_path.to_str().unwrap(),
            "--opt-level",
            level,
        ]);
        compiled.push(code == 0);
        artifacts.push(smc_path);
    }
    assert_eq!(
        compiled[0], compiled[1],
        "{name}: O0/O1 disagree on lowering"
    );
    if !compiled[0] {
        return reject(&mut out, Stage::Lower);
    }

    for (stage, command) in [(Stage::Verify, "verify"), (Stage::Run, "run-smc")] {
        let results: Vec<bool> = artifacts
            .iter()
            .map(|a| smc(&[command, a.to_str().unwrap()]).0 == 0)
            .collect();
        assert_eq!(
            results[0], results[1],
            "{name}: O0/O1 disagree on {command}"
        );
        if !results[0] {
            return reject(&mut out, stage);
        }
    }
    [Qualified; 7]
}

#[test]
fn declared_stages_match_the_running_pipeline() {
    let dir = mk_temp_dir("s2043_cp06_pipeline");
    for (i, feature) in LANGUAGE_FEATURES.iter().enumerate() {
        let observed = observe(&dir, &format!("f{i}"), feature.probe);
        assert_eq!(
            observed, feature.stages,
            "{}: declared stages differ from the pipeline",
            feature.id
        );
        if let Some((negative, stage)) = feature.negative_probe {
            let observed = observe(&dir, &format!("f{i}_neg"), negative);
            let first_rejected = Stage::ALL
                .into_iter()
                .find(|s| observed[*s as usize] == StageStatus::Rejected);
            assert_eq!(
                first_rejected,
                Some(stage),
                "{}: negative probe not rejected at its declared stage",
                feature.id
            );
        }
    }
}

#[test]
fn registry_entries_are_well_formed() {
    let mut previous: Option<&str> = None;
    for feature in LANGUAGE_FEATURES {
        if let Some(previous) = previous {
            assert!(
                previous < feature.id,
                "ids must be sorted and unique: {}",
                feature.id
            );
        }
        previous = Some(feature.id);

        // Stage shape: qualified*, then at most one rejected, then
        // not-applicable for every later stage. `not-qualified` is never
        // used by a probe-backed entry.
        let rejected = feature
            .stages
            .iter()
            .position(|s| *s == StageStatus::Rejected);
        for (i, status) in feature.stages.iter().enumerate() {
            let expected = match rejected {
                Some(r) if i < r => StageStatus::Qualified,
                Some(r) if i == r => StageStatus::Rejected,
                Some(_) => StageStatus::NotApplicable,
                None => StageStatus::Qualified,
            };
            assert_eq!(*status, expected, "{}: stage {i}", feature.id);
        }

        // Release axis agrees with the stage axis: an unsupported feature is
        // rejected somewhere, and an executable one is never unsupported. (A
        // supported surface may still stop before lowering by design - the
        // inspection-only Logos profile.)
        let executable = rejected.is_none();
        if feature.release_state == ReleaseState::Unsupported {
            assert!(!executable, "{}: unsupported but executable", feature.id);
        }
        // A public `qualified` run claim needs a negative probe as well.
        assert_eq!(
            feature.negative_probe.is_some(),
            executable,
            "{}: executable features carry exactly one negative probe",
            feature.id
        );
    }
}

#[test]
fn document_is_the_registry_and_is_deterministic() {
    let (first, json) = describe();
    let (second, _) = describe();
    assert_eq!(first, second, "byte-identical across runs");
    assert!(first.ends_with("}\n"));

    assert_eq!(json["schema_id"], LANGUAGE_SURFACE_SCHEMA_ID);
    assert_eq!(json["schema_version"], LANGUAGE_SURFACE_SCHEMA_VERSION);
    let features = json["features"].as_array().expect("features");
    assert_eq!(features.len(), LANGUAGE_FEATURES.len());
    for (doc, entry) in features.iter().zip(LANGUAGE_FEATURES) {
        assert_eq!(doc["id"], entry.id);
        assert_eq!(doc["release_state"], entry.release_state.as_str());
        for stage in Stage::ALL {
            assert_eq!(
                doc["stage_status"][stage.as_str()],
                entry.stage(stage).as_str(),
                "{} {}",
                entry.id,
                stage.as_str()
            );
        }
        assert_eq!(doc["evidence"]["suite"], entry.evidence_suite);
        assert_eq!(
            doc.as_object().unwrap().len(),
            4,
            "{}: exact keys",
            entry.id
        );
    }

    let keys: Vec<&String> = json.as_object().unwrap().keys().collect();
    assert_eq!(keys.len(), 7, "{keys:?}");
}

#[test]
fn identity_and_runtime_fields_never_overclaim() {
    let (_, json) = describe();
    assert!(json["compiler_identity"]["source_commit"].is_null());
    assert!(json["compiler_identity"]["build_id"].is_null());
    let (code, version, _) = smc(&["version", "--json"]);
    assert_eq!(code, 0);
    let version: serde_json::Value = serde_json::from_str(&version).unwrap();
    assert_eq!(
        json["compiler_identity"]["source_fingerprint"],
        version["source_fingerprint"]
    );
    assert_eq!(json["profile"]["source"], "semantic.foundation");
    assert_eq!(json["profile"]["compilation"], "O0+O1");
    assert!(json["runtime"]["envelope"].is_null());
    assert!(json["runtime"]["capability_grants"].is_null());

    let magic = |bytes: [u8; 8]| String::from_utf8(bytes.to_vec()).unwrap();
    assert_eq!(
        json["semcode"]["floor"],
        magic(sm_format::semcode_format::MAGIC22)
    );
    assert_eq!(
        json["semcode"]["promotions"],
        serde_json::json!([
            magic(sm_format::semcode_format::MAGIC23),
            magic(sm_format::semcode_format::MAGIC24),
            magic(sm_format::semcode_format::MAGIC25),
        ])
    );
}

#[test]
fn malformed_queries_fail_closed_and_existing_commands_are_unchanged() {
    for args in [
        vec!["language"],
        vec!["language", "describe"],
        vec!["language", "describe", "--format"],
        vec!["language", "describe", "--format", "human"],
        vec!["language", "describe", "--format", "json", "extra"],
        vec!["language", "list"],
    ] {
        let (code, stdout, stderr) = smc(&args);
        assert_ne!(code, 0, "{args:?}");
        assert!(stdout.is_empty(), "{args:?}: {stdout}");
        assert!(
            stderr.contains("usage: smc language describe --format json"),
            "{args:?}: {stderr}"
        );
    }

    // `smc features` stays the build-flag report, `smc version --json` keeps
    // its keys: the language catalog is a separate command.
    let (code, stdout, _) = smc(&["features"]);
    assert_eq!(code, 0);
    assert!(stdout.starts_with("enabled: "), "{stdout}");
    assert!(stdout.contains("\ndisabled: "), "{stdout}");
    let (_, version, _) = smc(&["version", "--json"]);
    let version: serde_json::Value = serde_json::from_str(&version).unwrap();
    let mut keys: Vec<&str> = version
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "enabled_features",
            "runtime_profile",
            "schema_version",
            "semcode_format",
            "signing",
            "source_fingerprint",
            "source_hash",
            "stdlib_version",
            "toolchain_version",
            "verifier_profile",
        ]
    );
}
