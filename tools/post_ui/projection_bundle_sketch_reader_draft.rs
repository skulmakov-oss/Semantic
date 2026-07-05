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

fn require_contains(content: &str, label: &str, needle: &str) -> Result<(), String> {
    if !content.contains(needle) {
        return Err(format!("missing required anchor {}: {}", label, needle));
    }

    Ok(())
}

fn require_scalar(content: &str, label: &str, key: &str, expected: &str) -> Result<(), String> {
    let actual = extract_scalar(content, key)
        .ok_or_else(|| format!("missing required field {}: {}", label, key))?;

    if actual != expected {
        return Err(format!(
            "field {} mismatch: expected {:?}, got {:?}",
            label, expected, actual
        ));
    }

    Ok(())
}

fn validate_sketch(content: &str) -> Result<(), String> {
    for &(label, needle) in EXPECTED_CONTAINS {
        require_contains(content, label, needle)?;
    }

    for &(label, key, expected) in EXPECTED_SCALARS {
        require_scalar(content, label, key, expected)?;
    }

    Ok(())
}

fn validate_positive_fixture(repo_root: &str) -> Result<(), String> {
    let sketch = read_sketch(repo_root);

    validate_sketch(&sketch)
}

fn validate_negative_fixture(repo_root: &str, case: &NegativeFixtureCase) -> Result<(), String> {
    let sketch = read_fixture(repo_root, case.relative_path);

    match case.rule {
        NegativeRule::MissingField {
            label,
            key,
        } => {
            if extract_scalar(&sketch, key).is_some() {
                return Err(format!("negative fixture unexpectedly passed: {}", case.name));
            }

            Err(format!("missing required field {}", label))
        }
        NegativeRule::ScalarValue {
            label,
            key,
            expected_value,
            rejected_value,
            rejection_reason,
        } => match extract_scalar(&sketch, key) {
            Some(actual) if actual == rejected_value => Err(rejection_reason.to_string()),
            Some(actual) if actual == expected_value => {
                Err(format!("negative fixture unexpectedly passed: {}", case.name))
            }
            Some(actual) => Err(format!(
                "field {} mismatch: expected {:?}, got {:?}",
                label, expected_value, actual
            )),
            None => Err(format!("missing required field {}", key)),
        },
    }
}

fn main() {
    let repo_root = env::args()
        .nth(1)
        .unwrap_or_else(|| fail("missing repository root argument"));
    let repo_root = normalize_repo_root(&repo_root);
    if let Err(reason) = validate_positive_fixture(&repo_root) {
        fail(format!("positive fixture failed: {}", reason));
    }

    for case in NEGATIVE_CASES {
        match validate_negative_fixture(&repo_root, case) {
            Ok(_) => fail(format!("negative fixture unexpectedly passed: {}", case.name)),
            Err(reason) => {
                if !reason.contains(case.expected_error_substring) {
                    fail(format!(
                        "negative fixture failed for wrong reason: {}: {}",
                        case.name, reason
                    ));
                }
            }
        }
    }

    println!(
        "PASS: ProjectionBundle sketch reader draft accepted positive and rejected negative manifest anchors"
    );
}
