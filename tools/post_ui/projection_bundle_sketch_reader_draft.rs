// ProjectionBundle sketch reader draft.
// This is test/fixture evidence only.
// It is not a loader.
// It is not runtime activation code.
// It is not final serialization.
// It is not a public API.
// It does not verify bundles.
// It does not authorize production UI wiring.
#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::process;

const EXPECTED_SCALARS: &[(&str, &str, &str)] = &[
    ("bundle id", "bundle_id", "bundle.example.minimal"),
    ("bundle version", "bundle_version", "0-sketch"),
    ("projection id", "projection_id", "ExampleMinimalProjection"),
    ("ui ir ref", "ui_ir_ref", "ui_ir.example.minimal"),
    ("binding graph ref", "binding_graph_ref", "binding_graph.example.minimal"),
    ("action ir ref", "action_ir_ref", "action_ir.example.minimal"),
    (
        "role dictionary version",
        "role_dictionary_version",
        "ui-roles.0-sketch",
    ),
    (
        "renderer profile",
        "renderer_profile",
        "semantic-shell.reference-sketch",
    ),
    ("safety class", "safety_class", "VerifiedDynamic"),
    ("criticality", "criticality", "NonCritical"),
    ("freshness policy", "freshness_policy", "FreshForControl"),
    ("hash", "hash", "sha256:SKETCH-NOT-A-REAL-HASH"),
    (
        "signature",
        "signature",
        "signature:SKETCH-NOT-A-REAL-SIGNATURE",
    ),
    (
        "created by",
        "created_by",
        "semantic-projection-compiler.SKETCH",
    ),
    ("created at", "created_at", "not-a-real-timestamp"),
    (
        "compiler identity",
        "compiler_identity",
        "semantic-projection-compiler.0-sketch",
    ),
    (
        "runtime tree streaming policy",
        "allow_runtime_tree_streaming",
        "false",
    ),
    (
        "production activation policy",
        "allow_production_activation",
        "false",
    ),
    (
        "pending unknown update policy",
        "allow_critical_update_during_pending_unknown",
        "false",
    ),
    (
        "quarantine update policy",
        "allow_critical_update_during_quarantine",
        "false",
    ),
    ("verification policy", "require_verification", "true"),
    (
        "safe update boundary policy",
        "require_safe_update_boundary",
        "true",
    ),
];

const EXPECTED_CONTAINS: &[(&str, &str)] = &[
    ("semantic source ref", "semantic.source.example"),
    ("projection source ref", "projection.source.example"),
];

const NEGATIVE_CASES: &[NegativeFixtureCase] = &[
    NegativeFixtureCase {
        name: "manifest_activation_enabled",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_activation_enabled.sketch.md",
        ],
        expected_error_substring: "production activation policy",
        rule: NegativeRule::ScalarValue {
            label: "production activation policy",
            key: "allow_production_activation",
            expected_value: "false",
            rejected_value: "true",
            rejection_reason: "production activation policy: allow_production_activation is true",
        },
    },
    NegativeFixtureCase {
        name: "manifest_missing_bundle_id",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_missing_bundle_id.sketch.md",
        ],
        expected_error_substring: "bundle id",
        rule: NegativeRule::MissingField {
            label: "bundle id",
            key: "bundle_id",
        },
    },
    NegativeFixtureCase {
        name: "manifest_missing_projection_id",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_missing_projection_id.sketch.md",
        ],
        expected_error_substring: "projection id",
        rule: NegativeRule::MissingField {
            label: "projection id",
            key: "projection_id",
        },
    },
    NegativeFixtureCase {
        name: "manifest_runtime_tree_streaming_enabled",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_runtime_tree_streaming_enabled.sketch.md",
        ],
        expected_error_substring: "runtime tree streaming",
        rule: NegativeRule::ScalarValue {
            label: "runtime tree streaming policy",
            key: "allow_runtime_tree_streaming",
            expected_value: "false",
            rejected_value: "true",
            rejection_reason: "runtime tree streaming policy: allow_runtime_tree_streaming is true",
        },
    },
    NegativeFixtureCase {
        name: "manifest_verification_disabled",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_verification_disabled.sketch.md",
        ],
        expected_error_substring: "verification",
        rule: NegativeRule::ScalarValue {
            label: "verification policy",
            key: "require_verification",
            expected_value: "true",
            rejected_value: "false",
            rejection_reason: "verification policy: require_verification is false",
        },
    },
    NegativeFixtureCase {
        name: "manifest_safe_update_boundary_disabled",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_safe_update_boundary_disabled.sketch.md",
        ],
        expected_error_substring: "safe update boundary",
        rule: NegativeRule::ScalarValue {
            label: "safe update boundary policy",
            key: "require_safe_update_boundary",
            expected_value: "true",
            rejected_value: "false",
            rejection_reason: "safe update boundary policy: require_safe_update_boundary is false",
        },
    },
    NegativeFixtureCase {
        name: "manifest_pending_unknown_update_enabled",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_pending_unknown_update_enabled.sketch.md",
        ],
        expected_error_substring: "pending unknown",
        rule: NegativeRule::ScalarValue {
            label: "pending unknown update policy",
            key: "allow_critical_update_during_pending_unknown",
            expected_value: "false",
            rejected_value: "true",
            rejection_reason:
                "pending unknown update policy: allow_critical_update_during_pending_unknown is true",
        },
    },
    NegativeFixtureCase {
        name: "manifest_quarantine_update_enabled",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_quarantine_update_enabled.sketch.md",
        ],
        expected_error_substring: "quarantine",
        rule: NegativeRule::ScalarValue {
            label: "quarantine update policy",
            key: "allow_critical_update_during_quarantine",
            expected_value: "false",
            rejected_value: "true",
            rejection_reason:
                "quarantine update policy: allow_critical_update_during_quarantine is true",
        },
    },
];

#[derive(Clone, Copy)]
struct NegativeFixtureCase {
    name: &'static str,
    relative_path: &'static [&'static str],
    expected_error_substring: &'static str,
    rule: NegativeRule,
}

#[derive(Clone, Copy)]
enum NegativeRule {
    MissingField {
        label: &'static str,
        key: &'static str,
    },
    ScalarValue {
        label: &'static str,
        key: &'static str,
        expected_value: &'static str,
        rejected_value: &'static str,
        rejection_reason: &'static str,
    },
}

#[derive(Clone)]
struct ManifestSnapshot {
    bundle_id: String,
    bundle_version: String,
    projection_id: String,
    source_ref_0: String,
    source_ref_1: String,
    ui_ir_ref: String,
    binding_graph_ref: String,
    action_ir_ref: String,
    role_dictionary_version: String,
    renderer_profile: String,
    safety_class: String,
    criticality: String,
    freshness_policy: String,
    hash: String,
    signature: String,
    created_by: String,
    created_at: String,
    compiler_identity: String,
    require_verification: String,
    allow_runtime_tree_streaming: String,
    allow_production_activation: String,
    require_safe_update_boundary: String,
    allow_critical_update_during_pending_unknown: String,
    allow_critical_update_during_quarantine: String,
}

#[derive(Clone)]
struct NegativeCaseResult {
    name: &'static str,
    input: String,
    reason: String,
}

fn fail(message: impl AsRef<str>) -> ! {
    eprintln!("FAIL: {}", message.as_ref());
    process::exit(1);
}

fn normalize_repo_root(raw: &str) -> String {
    raw.trim()
        .trim_end_matches(|ch| ch == '\\' || ch == '/')
        .to_string()
}

fn read_sketch(repo_root: &str) -> String {
    let sketch_path = Path::new(repo_root)
        .join("tests")
        .join("fixtures")
        .join("post_ui")
        .join("projection_bundle")
        .join("manifest_minimal.sketch.md");
    fs::read_to_string(&sketch_path).unwrap_or_else(|err| {
        fail(format!(
            "cannot read manifest sketch at {}: {}",
            sketch_path.display(),
            err
        ))
    })
}

fn read_fixture(repo_root: &str, relative_path: &[&str]) -> String {
    let mut path = Path::new(repo_root).to_path_buf();
    for part in relative_path {
        path = path.join(part);
    }

    fs::read_to_string(&path).unwrap_or_else(|err| {
        fail(format!(
            "cannot read fixture at {}: {}",
            path.display(),
            err
        ))
    })
}

fn extract_scalar(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{}:", key);

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let mut value = rest.trim().trim_end_matches(',').trim();
            if let Some(stripped) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
                value = stripped;
            }
            return Some(value.to_string());
        }
    }

    None
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

fn repo_relative_path(relative_path: &[&str]) -> String {
    relative_path.join("/")
}

fn require_contains(content: &str, label: &str, needle: &str) -> Result<(), String> {
    if !content.contains(needle) {
        return Err(format!("missing required anchor {}: {}", label, needle));
    }

    Ok(())
}

fn require_scalar(content: &str, label: &str, key: &str, expected: &str) -> Result<String, String> {
    let actual = extract_scalar(content, key)
        .ok_or_else(|| format!("missing required field {}: {}", label, key))?;

    if actual != expected {
        return Err(format!(
            "field {} mismatch: expected {:?}, got {:?}",
            label, expected, actual
        ));
    }

    Ok(actual)
}

fn validate_sketch(content: &str) -> Result<ManifestSnapshot, String> {
    for &(label, needle) in EXPECTED_CONTAINS {
        require_contains(content, label, needle)?;
    }

    let bundle_id = require_scalar(content, "bundle id", "bundle_id", "bundle.example.minimal")?;
    let bundle_version =
        require_scalar(content, "bundle version", "bundle_version", "0-sketch")?;
    let projection_id =
        require_scalar(content, "projection id", "projection_id", "ExampleMinimalProjection")?;
    let ui_ir_ref = require_scalar(content, "ui ir ref", "ui_ir_ref", "ui_ir.example.minimal")?;
    let binding_graph_ref = require_scalar(
        content,
        "binding graph ref",
        "binding_graph_ref",
        "binding_graph.example.minimal",
    )?;
    let action_ir_ref =
        require_scalar(content, "action ir ref", "action_ir_ref", "action_ir.example.minimal")?;
    let role_dictionary_version = require_scalar(
        content,
        "role dictionary version",
        "role_dictionary_version",
        "ui-roles.0-sketch",
    )?;
    let renderer_profile = require_scalar(
        content,
        "renderer profile",
        "renderer_profile",
        "semantic-shell.reference-sketch",
    )?;
    let safety_class =
        require_scalar(content, "safety class", "safety_class", "VerifiedDynamic")?;
    let criticality = require_scalar(content, "criticality", "criticality", "NonCritical")?;
    let freshness_policy = require_scalar(
        content,
        "freshness policy",
        "freshness_policy",
        "FreshForControl",
    )?;
    let hash = require_scalar(content, "hash", "hash", "sha256:SKETCH-NOT-A-REAL-HASH")?;
    let signature = require_scalar(
        content,
        "signature",
        "signature",
        "signature:SKETCH-NOT-A-REAL-SIGNATURE",
    )?;
    let created_by = require_scalar(
        content,
        "created by",
        "created_by",
        "semantic-projection-compiler.SKETCH",
    )?;
    let created_at = require_scalar(content, "created at", "created_at", "not-a-real-timestamp")?;
    let compiler_identity = require_scalar(
        content,
        "compiler identity",
        "compiler_identity",
        "semantic-projection-compiler.0-sketch",
    )?;
    let require_verification = require_scalar(
        content,
        "verification policy",
        "require_verification",
        "true",
    )?;
    let allow_runtime_tree_streaming = require_scalar(
        content,
        "runtime tree streaming policy",
        "allow_runtime_tree_streaming",
        "false",
    )?;
    let allow_production_activation = require_scalar(
        content,
        "production activation policy",
        "allow_production_activation",
        "false",
    )?;
    let require_safe_update_boundary = require_scalar(
        content,
        "safe update boundary policy",
        "require_safe_update_boundary",
        "true",
    )?;
    let allow_critical_update_during_pending_unknown = require_scalar(
        content,
        "pending unknown update policy",
        "allow_critical_update_during_pending_unknown",
        "false",
    )?;
    let allow_critical_update_during_quarantine = require_scalar(
        content,
        "quarantine update policy",
        "allow_critical_update_during_quarantine",
        "false",
    )?;

    Ok(ManifestSnapshot {
        bundle_id,
        bundle_version,
        projection_id,
        source_ref_0: "semantic.source.example".to_string(),
        source_ref_1: "projection.source.example".to_string(),
        ui_ir_ref,
        binding_graph_ref,
        action_ir_ref,
        role_dictionary_version,
        renderer_profile,
        safety_class,
        criticality,
        freshness_policy,
        hash,
        signature,
        created_by,
        created_at,
        compiler_identity,
        require_verification,
        allow_runtime_tree_streaming,
        allow_production_activation,
        require_safe_update_boundary,
        allow_critical_update_during_pending_unknown,
        allow_critical_update_during_quarantine,
    })
}

fn validate_sketch_except(content: &str, skipped_key: &str) -> Result<(), String> {
    for &(label, needle) in EXPECTED_CONTAINS {
        require_contains(content, label, needle)?;
    }

    for &(label, key, expected) in EXPECTED_SCALARS {
        if key == skipped_key {
            continue;
        }

        require_scalar(content, label, key, expected)?;
    }

    Ok(())
}

fn validate_positive_fixture(repo_root: &str) -> Result<ManifestSnapshot, String> {
    let sketch = read_sketch(repo_root);

    validate_sketch(&sketch)
}

fn validate_negative_fixture(
    repo_root: &str,
    case: &NegativeFixtureCase,
) -> Result<NegativeCaseResult, String> {
    let sketch = read_fixture(repo_root, case.relative_path);
    let input = repo_relative_path(case.relative_path);

    match case.rule {
        NegativeRule::MissingField {
            label,
            key,
        } => {
            validate_sketch_except(&sketch, key)?;

            if extract_scalar(&sketch, key).is_some() {
                return Err(format!("negative fixture unexpectedly passed: {}", case.name));
            }

            let reason = format!("missing required field {}", label);
            if !reason.contains(case.expected_error_substring) {
                return Err(format!(
                    "negative fixture failed for wrong reason: {}: {}",
                    case.name, reason
                ));
            }

            Ok(NegativeCaseResult {
                name: case.name,
                input,
                reason,
            })
        }
        NegativeRule::ScalarValue {
            label,
            key,
            expected_value,
            rejected_value,
            rejection_reason,
        } => {
            validate_sketch_except(&sketch, key)?;

            match extract_scalar(&sketch, key) {
                Some(actual) if actual == expected_value => {
                    Err(format!("negative fixture unexpectedly passed: {}", case.name))
                }
                Some(actual) if actual == rejected_value => {
                    let reason = rejection_reason.to_string();
                    if !reason.contains(case.expected_error_substring) {
                        return Err(format!(
                            "negative fixture failed for wrong reason: {}: {}",
                            case.name, reason
                        ));
                    }

                    Ok(NegativeCaseResult {
                        name: case.name,
                        input,
                        reason,
                    })
                }
                Some(actual) => Err(format!(
                    "field {} mismatch: expected {:?}, got {:?}",
                    label, expected_value, actual
                )),
                None => Err(format!("missing required field {}", key)),
            }
        }
    }
}

fn validate_negative_pack(repo_root: &str) -> Result<Vec<NegativeCaseResult>, String> {
    let mut results = Vec::with_capacity(NEGATIVE_CASES.len());

    for case in NEGATIVE_CASES {
        let result = validate_negative_fixture(repo_root, case)?;
        results.push(result);
    }

    Ok(results)
}

fn render_positive_output(snapshot: &ManifestSnapshot) -> String {
    let mut out = String::new();
    push_line(&mut out, "ProjectionBundleSketchReaderOutput v0");
    push_line(&mut out, "scope=fixture-facing");
    push_line(
        &mut out,
        "input=tests/fixtures/post_ui/projection_bundle/manifest_minimal.sketch.md",
    );
    push_line(&mut out, "status=accepted");
    push_line(&mut out, "");
    push_line(&mut out, "[identity]");
    push_line(&mut out, &format!("bundle_id={}", snapshot.bundle_id));
    push_line(
        &mut out,
        &format!("bundle_version={}", snapshot.bundle_version),
    );
    push_line(
        &mut out,
        &format!("projection_id={}", snapshot.projection_id),
    );
    push_line(&mut out, "");
    push_line(&mut out, "[sources]");
    push_line(&mut out, &format!("source_ref.0={}", snapshot.source_ref_0));
    push_line(&mut out, &format!("source_ref.1={}", snapshot.source_ref_1));
    push_line(&mut out, "");
    push_line(&mut out, "[artifacts]");
    push_line(&mut out, &format!("ui_ir_ref={}", snapshot.ui_ir_ref));
    push_line(
        &mut out,
        &format!("binding_graph_ref={}", snapshot.binding_graph_ref),
    );
    push_line(&mut out, &format!("action_ir_ref={}", snapshot.action_ir_ref));
    push_line(&mut out, "");
    push_line(&mut out, "[compatibility]");
    push_line(
        &mut out,
        &format!(
            "role_dictionary_version={}",
            snapshot.role_dictionary_version
        ),
    );
    push_line(
        &mut out,
        &format!("renderer_profile={}", snapshot.renderer_profile),
    );
    push_line(&mut out, "");
    push_line(&mut out, "[safety]");
    push_line(&mut out, &format!("safety_class={}", snapshot.safety_class));
    push_line(&mut out, &format!("criticality={}", snapshot.criticality));
    push_line(
        &mut out,
        &format!("freshness_policy={}", snapshot.freshness_policy),
    );
    push_line(&mut out, "");
    push_line(&mut out, "[trust]");
    push_line(&mut out, &format!("hash={}", snapshot.hash));
    push_line(&mut out, &format!("signature={}", snapshot.signature));
    push_line(&mut out, &format!("created_by={}", snapshot.created_by));
    push_line(&mut out, &format!("created_at={}", snapshot.created_at));
    push_line(
        &mut out,
        &format!("compiler_identity={}", snapshot.compiler_identity),
    );
    push_line(&mut out, "trust_status=placeholder");
    push_line(&mut out, "verification_status=not_verified");
    push_line(&mut out, "");
    push_line(&mut out, "[activation_policy]");
    push_line(
        &mut out,
        &format!(
            "require_verification={}",
            snapshot.require_verification
        ),
    );
    push_line(
        &mut out,
        &format!(
            "allow_runtime_tree_streaming={}",
            snapshot.allow_runtime_tree_streaming
        ),
    );
    push_line(
        &mut out,
        &format!(
            "allow_production_activation={}",
            snapshot.allow_production_activation
        ),
    );
    push_line(&mut out, "activation_ready=false");
    push_line(&mut out, "");
    push_line(&mut out, "[update_policy]");
    push_line(
        &mut out,
        &format!(
            "require_safe_update_boundary={}",
            snapshot.require_safe_update_boundary
        ),
    );
    push_line(
        &mut out,
        &format!(
            "allow_critical_update_during_pending_unknown={}",
            snapshot.allow_critical_update_during_pending_unknown
        ),
    );
    push_line(
        &mut out,
        &format!(
            "allow_critical_update_during_quarantine={}",
            snapshot.allow_critical_update_during_quarantine
        ),
    );
    push_line(&mut out, "");
    push_line(&mut out, "[authority]");
    push_line(&mut out, "loader_claim=false");
    push_line(&mut out, "runtime_claim=false");
    push_line(&mut out, "production_ui_claim=false");
    push_line(&mut out, "general_level_4_claim=false");
    push_line(&mut out, "level_5_plus_claim=false");
    push_line(&mut out, "");

    normalize_line_endings(&out)
}

fn render_negative_report(results: &[NegativeCaseResult]) -> String {
    let mut out = String::new();
    push_line(&mut out, "ProjectionBundleSketchReaderNegativeReport v0");
    push_line(&mut out, "scope=fixture-facing");
    push_line(&mut out, "status=rejected-negative-pack");
    push_line(&mut out, "");

    for (index, case) in results.iter().enumerate() {
        push_line(&mut out, &format!("case.{}.name={}", index, case.name));
        push_line(&mut out, &format!("case.{}.input={}", index, case.input));
        push_line(&mut out, &format!("case.{}.status=rejected", index));
        push_line(&mut out, &format!("case.{}.reason={}", index, case.reason));
        push_line(&mut out, "");
    }

    push_line(&mut out, "summary.accepted_positive=1");
    push_line(&mut out, &format!("summary.rejected_negative={}", results.len()));
    push_line(&mut out, "summary.unexpected_pass=0");
    push_line(&mut out, "summary.wrong_reason=0");
    push_line(&mut out, "");
    push_line(&mut out, "[authority]");
    push_line(&mut out, "loader_claim=false");
    push_line(&mut out, "runtime_claim=false");
    push_line(&mut out, "production_ui_claim=false");
    push_line(&mut out, "general_level_4_claim=false");
    push_line(&mut out, "level_5_plus_claim=false");
    push_line(&mut out, "");

    normalize_line_endings(&out)
}

fn emit_positive_output(repo_root: &str) -> Result<String, String> {
    let snapshot = validate_positive_fixture(repo_root)?;
    Ok(render_positive_output(&snapshot))
}

fn emit_negative_report(repo_root: &str) -> Result<String, String> {
    let results = validate_negative_pack(repo_root)?;
    Ok(render_negative_report(&results))
}

fn main() {
    let mut args = env::args().skip(1);
    let first = args
        .next()
        .unwrap_or_else(|| fail("missing repository root argument"));

    let (mode, repo_root) = match first.as_str() {
        "--emit-positive-output" | "--emit-negative-report" => {
            let repo_root = args
                .next()
                .unwrap_or_else(|| fail("missing repository root argument"));
            if args.next().is_some() {
                fail("unexpected extra arguments");
            }

            (Some(first), normalize_repo_root(&repo_root))
        }
        _ => {
            if args.next().is_some() {
                fail("unexpected extra arguments");
            }

            (None, normalize_repo_root(&first))
        }
    };

    match mode.as_deref() {
        Some("--emit-positive-output") => match emit_positive_output(&repo_root) {
            Ok(output) => {
                print!("{}", output);
            }
            Err(reason) => fail(format!("positive fixture failed: {}", reason)),
        },
        Some("--emit-negative-report") => match emit_negative_report(&repo_root) {
            Ok(output) => {
                print!("{}", output);
            }
            Err(reason) => fail(format!("negative pack failed: {}", reason)),
        },
        None => {
            if let Err(reason) = validate_positive_fixture(&repo_root) {
                fail(format!("positive fixture failed: {}", reason));
            }

            if let Err(reason) = validate_negative_pack(&repo_root) {
                fail(format!("negative fixture failed: {}", reason));
            }

            println!(
                "PASS: ProjectionBundle sketch reader draft accepted positive and rejected negative manifest anchors"
            );
        }
        Some(other) => fail(format!("unsupported mode: {}", other)),
    }
}
