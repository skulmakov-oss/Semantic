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

const EXPECTED_CONTAINS: &[(&str, &str)] = &[
    ("bundle id", "bundle.example.minimal"),
    ("bundle version", "0-sketch"),
    ("projection id", "ExampleMinimalProjection"),
    ("semantic source ref", "semantic.source.example"),
    ("projection source ref", "projection.source.example"),
    ("ui ir ref", "ui_ir.example.minimal"),
    ("binding graph ref", "binding_graph.example.minimal"),
    ("action ir ref", "action_ir.example.minimal"),
    ("role dictionary version", "ui-roles.0-sketch"),
    ("renderer profile", "semantic-shell.reference-sketch"),
    ("safety class", "VerifiedDynamic"),
    ("criticality", "NonCritical"),
    ("freshness policy", "FreshForControl"),
    ("hash", "sha256:SKETCH-NOT-A-REAL-HASH"),
    ("signature", "signature:SKETCH-NOT-A-REAL-SIGNATURE"),
    ("created by", "semantic-projection-compiler.SKETCH"),
    ("created at", "not-a-real-timestamp"),
    ("compiler identity", "semantic-projection-compiler.0-sketch"),
    ("runtime tree streaming disabled", "allow_runtime_tree_streaming: false"),
    ("production activation disabled", "allow_production_activation: false"),
    (
        "pending unknown updates disabled",
        "allow_critical_update_during_pending_unknown: false",
    ),
    (
        "quarantine updates disabled",
        "allow_critical_update_during_quarantine: false",
    ),
    ("verification required", "require_verification: true"),
    ("safe update boundary required", "require_safe_update_boundary: true"),
];

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

fn expect_contains(content: &str, label: &str, needle: &str) {
    if !content.contains(needle) {
        fail(format!("missing required anchor {}: {}", label, needle));
    }
}

fn expect_scalar(content: &str, label: &str, key: &str, expected: &str) {
    let actual = extract_scalar(content, key).unwrap_or_else(|| {
        fail(format!("missing required field {}: {}", label, key));
    });

    if actual != expected {
        fail(format!(
            "field {} mismatch: expected {:?}, got {:?}",
            label, expected, actual
        ));
    }
}

fn main() {
    let repo_root = env::args()
        .nth(1)
        .unwrap_or_else(|| fail("missing repository root argument"));
    let repo_root = normalize_repo_root(&repo_root);
    let sketch = read_sketch(&repo_root);

    for (label, needle) in EXPECTED_CONTAINS {
        expect_contains(&sketch, label, needle);
    }

    for (label, key, expected) in EXPECTED_SCALARS {
        expect_scalar(&sketch, label, key, expected);
    }

    println!("PASS: ProjectionBundle sketch reader draft matched expected manifest anchors");
}
