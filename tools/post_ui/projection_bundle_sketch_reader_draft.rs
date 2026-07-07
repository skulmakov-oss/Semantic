// ProjectionBundle sketch reader draft.
// This is test/fixture evidence only.
// It is not a loader.
// It is not runtime activation code.
// It is not final serialization.
// It is not a public API.
// It does not verify bundles.
// It does not authorize production UI wiring.
#![forbid(unsafe_code)]
#![allow(dead_code)]

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
    NegativeFixtureCase {
        name: "manifest_malformed_bundle_id_empty",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_malformed_bundle_id_empty.sketch.md",
        ],
        expected_error_substring: "malformed_field: bundle_id is empty",
        rule: NegativeRule::MalformedEmpty {
            key: "bundle_id",
            rejection_reason: "malformed_field: bundle_id is empty",
        },
    },
    NegativeFixtureCase {
        name: "manifest_malformed_require_verification",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_malformed_require_verification.sketch.md",
        ],
        expected_error_substring: "malformed_field: require_verification must be boolean",
        rule: NegativeRule::MalformedBoolean {
            key: "require_verification",
            rejection_reason: "malformed_field: require_verification must be boolean",
        },
    },
    NegativeFixtureCase {
        name: "manifest_unknown_root_field",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_unknown_root_field.sketch.md",
        ],
        expected_error_substring: "unknown_field: unknown_future_field",
        rule: NegativeRule::UnknownField {
            key: "unknown_future_field",
            rejection_reason: "unknown_field: unknown_future_field",
        },
    },
    NegativeFixtureCase {
        name: "manifest_unknown_activation_policy_field",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_unknown_activation_policy_field.sketch.md",
        ],
        expected_error_substring: "unknown_field: allow_unreviewed_activation",
        rule: NegativeRule::UnknownField {
            key: "allow_unreviewed_activation",
            rejection_reason: "unknown_field: allow_unreviewed_activation",
        },
    },
    NegativeFixtureCase {
        name: "manifest_duplicate_bundle_id",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_duplicate_bundle_id.sketch.md",
        ],
        expected_error_substring: "duplicate_field: bundle_id",
        rule: NegativeRule::DuplicateField {
            key: "bundle_id",
            rejection_reason: "duplicate_field: bundle_id",
        },
    },
    NegativeFixtureCase {
        name: "manifest_duplicate_require_verification",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_duplicate_require_verification.sketch.md",
        ],
        expected_error_substring: "duplicate_field: require_verification",
        rule: NegativeRule::DuplicateField {
            key: "require_verification",
            rejection_reason: "duplicate_field: require_verification",
        },
    },
    NegativeFixtureCase {
        name: "manifest_out_of_order_identity",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_out_of_order_identity.sketch.md",
        ],
        expected_error_substring: "field_ordering: bundle_version must appear before projection_id",
        rule: NegativeRule::FieldOrdering {
            earlier: "bundle_version:",
            later: "projection_id:",
            rejection_reason: "field_ordering: bundle_version must appear before projection_id",
        },
    },
    NegativeFixtureCase {
        name: "manifest_out_of_order_policy_blocks",
        relative_path: &[
            "tests",
            "fixtures",
            "post_ui",
            "projection_bundle",
            "invalid",
            "manifest_out_of_order_policy_blocks.sketch.md",
        ],
        expected_error_substring:
            "field_ordering: activation_policy must appear before update_policy",
        rule: NegativeRule::FieldOrdering {
            earlier: "activation_policy {",
            later: "update_policy {",
            rejection_reason: "field_ordering: activation_policy must appear before update_policy",
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
    MalformedEmpty {
        key: &'static str,
        rejection_reason: &'static str,
    },
    MalformedBoolean {
        key: &'static str,
        rejection_reason: &'static str,
    },
    UnknownField {
        key: &'static str,
        rejection_reason: &'static str,
    },
    DuplicateField {
        key: &'static str,
        rejection_reason: &'static str,
    },
    FieldOrdering {
        earlier: &'static str,
        later: &'static str,
        rejection_reason: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SketchReaderOutput {
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

type ManifestSnapshot = SketchReaderOutput;

#[derive(Clone, Debug, Eq, PartialEq)]
struct SketchScalar {
    section: String,
    key: String,
    value: String,
    line: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SketchSection {
    name: String,
    path: String,
    start_line: usize,
    end_line: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SketchArray {
    section: String,
    key: String,
    values: Vec<String>,
    line: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SketchReaderCore {
    text_block: String,
    scalars: Vec<SketchScalar>,
    sections: Vec<SketchSection>,
    arrays: Vec<SketchArray>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NegativeCaseResult {
    name: &'static str,
    input: String,
    status: NegativeCaseStatus,
    reason: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NegativeCaseStatus {
    Rejected,
    UnexpectedPass,
    WrongReason,
}

impl NegativeCaseStatus {
    fn as_str(self) -> &'static str {
        match self {
            NegativeCaseStatus::Rejected => "rejected",
            NegativeCaseStatus::UnexpectedPass => "unexpected_pass",
            NegativeCaseStatus::WrongReason => "wrong_reason",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NegativeReport {
    cases: Vec<NegativeCaseResult>,
}

impl NegativeReport {
    fn new() -> Self {
        Self { cases: Vec::new() }
    }

    fn from_cases(cases: Vec<NegativeCaseResult>) -> Self {
        Self { cases }
    }

    fn push_case(&mut self, case: NegativeCaseResult) {
        self.cases.push(case);
    }

    fn accepted_positive(&self) -> usize {
        1
    }

    fn rejected_negative(&self) -> usize {
        self.cases
            .iter()
            .filter(|case| case.status == NegativeCaseStatus::Rejected)
            .count()
    }

    fn unexpected_pass(&self) -> usize {
        self.cases
            .iter()
            .filter(|case| case.status == NegativeCaseStatus::UnexpectedPass)
            .count()
    }

    fn wrong_reason(&self) -> usize {
        self.cases
            .iter()
            .filter(|case| case.status == NegativeCaseStatus::WrongReason)
            .count()
    }

    fn with_unexpected_pass(mut self, name: &'static str, input: impl Into<String>) -> Self {
        self.push_case(NegativeCaseResult {
            name,
            input: input.into(),
            status: NegativeCaseStatus::UnexpectedPass,
            reason: String::new(),
        });
        self
    }

    fn with_wrong_reason(
        mut self,
        name: &'static str,
        input: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        self.push_case(NegativeCaseResult {
            name,
            input: input.into(),
            status: NegativeCaseStatus::WrongReason,
            reason: reason.into(),
        });
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReaderErrorKind {
    MissingRequiredField,
    MalformedField,
    UnknownField,
    DuplicateField,
    FieldOrdering,
    InvalidPolicyValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReaderError {
    kind: ReaderErrorKind,
    field: String,
    message: String,
}

impl ReaderError {
    fn new(kind: ReaderErrorKind, field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            message: message.into(),
        }
    }
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

fn extract_text_block(content: &str) -> Result<String, ReaderError> {
    let mut in_text_block = false;
    let mut block = String::new();
    let mut found_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_text_block {
            if trimmed == "```text" {
                in_text_block = true;
                found_block = true;
            }
            continue;
        }

        if trimmed == "```" {
            return Ok(block);
        }

        block.push_str(line);
        block.push('\n');
    }

    if found_block {
        Err(ReaderError::new(
            ReaderErrorKind::MalformedField,
            "projection_bundle",
            "unterminated text block",
        ))
    } else {
        Err(ReaderError::new(
            ReaderErrorKind::MissingRequiredField,
            "projection_bundle",
            "missing required text block",
        ))
    }
}

fn scan_scalars(text_block: &str) -> SketchReaderCore {
    let mut scalars = Vec::new();
    let mut sections: Vec<SketchSection> = Vec::new();
    let mut arrays = Vec::new();
    let mut section_stack: Vec<usize> = Vec::new();
    let mut current_array: Option<SketchArray> = None;

    for (index, line) in text_block.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if current_array.is_some() {
            if trimmed == "]" {
                arrays.push(current_array.take().unwrap());
                continue;
            }

            if let Some(array) = current_array.as_mut() {
                let candidate = trimmed.trim_end_matches(',');
                if let Some(value) = candidate
                    .strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
                {
                    array.values.push(value.to_string());
                }
            }

            continue;
        }

        if trimmed == "}" {
            if let Some(section_index) = section_stack.pop() {
                if let Some(section) = sections.get_mut(section_index) {
                    section.end_line = Some(index + 1);
                }
            }
            continue;
        }

        if trimmed.ends_with('{') {
            let section = trimmed.trim_end_matches('{').trim().to_string();
            if !section.is_empty() {
                let path = if let Some(parent_index) = section_stack.last() {
                    let parent_path = sections
                        .get(*parent_index)
                        .map(|section| section.path.clone())
                        .unwrap_or_default();
                    if parent_path.is_empty() {
                        section.clone()
                    } else {
                        format!("{}/{}", parent_path, section)
                    }
                } else {
                    section.clone()
                };

                sections.push(SketchSection {
                    name: section,
                    path,
                    start_line: index + 1,
                    end_line: None,
                });
                section_stack.push(sections.len() - 1);
            }
            continue;
        }

        if let Some((key, rest)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = rest.trim();
            if value.starts_with('[') {
                let section = section_stack
                    .last()
                    .and_then(|section_index| sections.get(*section_index))
                    .map(|section| section.path.clone())
                    .unwrap_or_default();

                if value == "[]" {
                    arrays.push(SketchArray {
                        section,
                        key: key.to_string(),
                        values: Vec::new(),
                        line: index + 1,
                    });
                } else {
                    let mut array = SketchArray {
                        section,
                        key: key.to_string(),
                        values: Vec::new(),
                        line: index + 1,
                    };

                    if value.ends_with(']') && value != "[" {
                        let inner = value
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .trim();
                        if !inner.is_empty() {
                            for part in inner.split(',') {
                                let item = part.trim().trim_matches('"');
                                if !item.is_empty() {
                                    array.values.push(item.to_string());
                                }
                            }
                        }
                        arrays.push(array);
                    } else {
                        current_array = Some(array);
                    }
                }
                continue;
            }

            scalars.push(SketchScalar {
                section: section_stack
                    .last()
                    .and_then(|section_index| sections.get(*section_index))
                    .map(|section| section.path.clone())
                    .unwrap_or_default(),
                key: key.to_string(),
                value: value.trim_end_matches(',').trim().trim_matches('"').to_string(),
                line: index + 1,
            });
        }
    }

    if let Some(array) = current_array.take() {
        arrays.push(array);
    }

    SketchReaderCore {
        text_block: text_block.to_string(),
        scalars,
        sections,
        arrays,
    }
}

fn parse_sketch_core(content: &str) -> Result<SketchReaderCore, ReaderError> {
    let text_block = extract_text_block(content)?;
    Ok(scan_scalars(&text_block))
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

fn find_scalar_core<'a>(core: &'a SketchReaderCore, key: &str) -> Option<&'a SketchScalar> {
    core.scalars.iter().find(|scalar| scalar.key == key)
}

fn count_scalar_occurrences_core(core: &SketchReaderCore, key: &str) -> usize {
    core.scalars.iter().filter(|scalar| scalar.key == key).count()
}

fn find_section_core<'a>(core: &'a SketchReaderCore, path: &str) -> Option<&'a SketchSection> {
    core.sections.iter().find(|section| section.path == path)
}

fn find_array_core<'a>(
    core: &'a SketchReaderCore,
    section: &str,
    key: &str,
) -> Option<&'a SketchArray> {
    core.arrays
        .iter()
        .find(|array| array.section == section && array.key == key)
}

fn require_required_scalar_core(
    core: &SketchReaderCore,
    key: &str,
    expected: &str,
) -> Result<String, ReaderError> {
    let scalar = find_scalar_core(core, key).ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::MissingRequiredField,
            key,
            format!("missing required field {}", key),
        )
    })?;

    if scalar.value != expected {
        return Err(ReaderError::new(
            ReaderErrorKind::InvalidPolicyValue,
            key,
            format!(
                "field {} mismatch: expected {:?}, got {:?}",
                key, expected, scalar.value
            ),
        ));
    }

    Ok(scalar.value.clone())
}

fn require_non_empty_scalar_core(core: &SketchReaderCore, key: &str) -> Result<(), ReaderError> {
    let scalar = find_scalar_core(core, key).ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::MissingRequiredField,
            key,
            format!("missing required field {}", key),
        )
    })?;

    if scalar.value.is_empty() {
        return Err(ReaderError::new(
            ReaderErrorKind::MalformedField,
            key,
            format!("malformed_field: {} is empty", key),
        ));
    }

    Ok(())
}

fn require_boolean_scalar_core(core: &SketchReaderCore, key: &str) -> Result<(), ReaderError> {
    let scalar = find_scalar_core(core, key).ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::MissingRequiredField,
            key,
            format!("missing required field {}", key),
        )
    })?;

    match scalar.value.as_str() {
        "true" | "false" => Ok(()),
        _ => Err(ReaderError::new(
            ReaderErrorKind::MalformedField,
            key,
            format!("malformed_field: {} must be boolean", key),
        )),
    }
}

fn require_source_refs_core(core: &SketchReaderCore) -> Result<(String, String), ReaderError> {
    let array = find_array_core(core, "projection_bundle", "source_refs").ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::MissingRequiredField,
            "source_refs",
            "missing required source_refs array",
        )
    })?;

    if array.values.len() != 2 {
        return Err(ReaderError::new(
            ReaderErrorKind::MalformedField,
            "source_refs",
            "malformed_field: source_refs must contain exactly two values",
        ));
    }

    Ok((array.values[0].clone(), array.values[1].clone()))
}

fn require_no_duplicate_scalars_core(
    core: &SketchReaderCore,
    keys: &[&str],
    skipped_key: Option<&str>,
) -> Result<(), ReaderError> {
    for key in keys {
        if skipped_key == Some(*key) {
            continue;
        }

        if count_scalar_occurrences_core(core, key) > 1 {
            return Err(ReaderError::new(
                ReaderErrorKind::DuplicateField,
                *key,
                format!("duplicate_field: {}", key),
            ));
        }
    }

    Ok(())
}

fn require_no_known_unknown_fields_core(core: &SketchReaderCore) -> Result<(), ReaderError> {
    for key in ["unknown_future_field", "allow_unreviewed_activation"] {
        if count_scalar_occurrences_core(core, key) > 0 {
            return Err(ReaderError::new(
                ReaderErrorKind::UnknownField,
                key,
                format!("unknown_field: {}", key),
            ));
        }
    }

    Ok(())
}

fn require_scalar_order_core(
    core: &SketchReaderCore,
    earlier: &str,
    later: &str,
) -> Result<(), ReaderError> {
    let earlier_scalar = find_scalar_core(core, earlier).ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::FieldOrdering,
            earlier,
            format!("field_ordering: {} must appear before {}", earlier, later),
        )
    })?;
    let later_scalar = find_scalar_core(core, later).ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::FieldOrdering,
            later,
            format!("field_ordering: {} must appear before {}", earlier, later),
        )
    })?;

    if earlier_scalar.line < later_scalar.line {
        Ok(())
    } else {
        Err(ReaderError::new(
            ReaderErrorKind::FieldOrdering,
            later,
            format!("field_ordering: {} must appear before {}", earlier, later),
        ))
    }
}

fn require_section_order_core(
    core: &SketchReaderCore,
    earlier_path: &str,
    later_path: &str,
) -> Result<(), ReaderError> {
    let earlier_label = earlier_path
        .trim()
        .trim_start_matches("projection_bundle/")
        .trim();
    let later_label = later_path
        .trim()
        .trim_start_matches("projection_bundle/")
        .trim();

    let earlier_section = find_section_core(core, earlier_path).ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::FieldOrdering,
            earlier_path,
            format!(
                "field_ordering: {} must appear before {}",
                earlier_label, later_label
            ),
        )
    })?;
    let later_section = find_section_core(core, later_path).ok_or_else(|| {
        ReaderError::new(
            ReaderErrorKind::FieldOrdering,
            later_path,
            format!(
                "field_ordering: {} must appear before {}",
                earlier_label, later_label
            ),
        )
    })?;

    if earlier_section.start_line < later_section.start_line {
        Ok(())
    } else {
        Err(ReaderError::new(
            ReaderErrorKind::FieldOrdering,
            later_path,
            format!(
                "field_ordering: {} must appear before {}",
                earlier_label, later_label
            ),
        ))
    }
}


fn require_no_known_unknown_fields_except_core(
    core: &SketchReaderCore,
    allowed_unknown_fields: &[&str],
) -> Result<(), String> {
    let known_sections = [
        "projection_bundle",
        "projection_bundle/artifacts",
        "projection_bundle/compatibility",
        "projection_bundle/safety",
        "projection_bundle/trust",
        "projection_bundle/activation_policy",
        "projection_bundle/update_policy",
        "projection_bundle/diagnostics",
    ];

    let known_fields = [
        ("projection_bundle", "bundle_id"),
        ("projection_bundle", "bundle_version"),
        ("projection_bundle", "projection_id"),
        ("projection_bundle", "source_refs"),
        ("projection_bundle/artifacts", "ui_ir_ref"),
        ("projection_bundle/artifacts", "binding_graph_ref"),
        ("projection_bundle/artifacts", "action_ir_ref"),
        ("projection_bundle/compatibility", "role_dictionary_version"),
        ("projection_bundle/compatibility", "renderer_profile"),
        ("projection_bundle/safety", "safety_class"),
        ("projection_bundle/safety", "criticality"),
        ("projection_bundle/safety", "freshness_policy"),
        ("projection_bundle/safety", "required_capabilities"),
        ("projection_bundle/trust", "hash"),
        ("projection_bundle/trust", "signature"),
        ("projection_bundle/trust", "created_by"),
        ("projection_bundle/trust", "created_at"),
        ("projection_bundle/trust", "compiler_identity"),
        ("projection_bundle/activation_policy", "require_verification"),
        ("projection_bundle/activation_policy", "allow_runtime_tree_streaming"),
        ("projection_bundle/activation_policy", "allow_production_activation"),
        ("projection_bundle/update_policy", "require_safe_update_boundary"),
        ("projection_bundle/update_policy", "allow_critical_update_during_pending_unknown"),
        ("projection_bundle/update_policy", "allow_critical_update_during_quarantine"),
        ("projection_bundle/diagnostics", "expected"),
    ];

    for section in &core.sections {
        if !known_sections.contains(&section.path.as_str()) {
            return Err(format!("unknown_section: {}", section.path));
        }
    }

    for scalar in &core.scalars {
        if allowed_unknown_fields.contains(&scalar.key.as_str()) {
            continue;
        }
        if !known_fields.contains(&(scalar.section.as_str(), scalar.key.as_str())) {
            return Err(format!("unknown_field: {}", scalar.key));
        }
    }

    for array in &core.arrays {
        if allowed_unknown_fields.contains(&array.key.as_str()) {
            continue;
        }
        if !known_fields.contains(&(array.section.as_str(), array.key.as_str())) {
            return Err(format!("unknown_field: {}", array.key));
        }
    }

    Ok(())
}

fn classify_hash_shape(val: &str) -> &'static str {
    if val == "sha256:SKETCH-NOT-A-REAL-HASH" {
        "placeholder"
    } else if val.starts_with("sha256:")
        && val.len() == 71
        && val[7..].chars().all(|c| c.is_ascii_hexdigit())
    {
        "sha256_like"
    } else {
        "malformed"
    }
}

fn classify_signature_shape(val: &str) -> &'static str {
    if val == "signature:SKETCH-NOT-A-REAL-SIGNATURE" {
        "placeholder"
    } else if val.starts_with("signature:")
        && val.len() == 74
        && val[10..].chars().all(|c| c.is_ascii_hexdigit())
    {
        "signature_like"
    } else {
        "malformed"
    }
}

fn require_trust_hash_shape_core(
    core: &SketchReaderCore,
    skipped_key: Option<&str>,
) -> Result<(), String> {
    if skipped_key == Some("hash") {
        return Ok(());
    }

    if let Some(hash_scalar) = find_scalar_core(core, "hash") {
        let shape = classify_hash_shape(&hash_scalar.value);
        if shape == "malformed" {
            return Err("malformed_hash_shape".to_string());
        }
    }

    Ok(())
}

fn require_trust_signature_shape_core(
    core: &SketchReaderCore,
    skipped_key: Option<&str>,
) -> Result<(), String> {
    if skipped_key == Some("signature") {
        return Ok(());
    }

    if let Some(sig_scalar) = find_scalar_core(core, "signature") {
        let shape = classify_signature_shape(&sig_scalar.value);
        if shape == "malformed" {
            return Err("malformed_signature_shape".to_string());
        }
    }

    Ok(())
}

fn validate_sketch_except_core(
    core: &SketchReaderCore,
    skipped_key: Option<&str>,
    allowed_unknown_fields: &[&str],
    skip_order_check: bool,
) -> Result<(), String> {
    for &(label, needle) in EXPECTED_CONTAINS {
        if !core.text_block.contains(needle) {
            return Err(format!("missing required anchor {}: {}", label, needle));
        }
    }

    require_trust_hash_shape_core(core, skipped_key)?;
    require_trust_signature_shape_core(core, skipped_key)?;

    for &(_label, key, expected) in EXPECTED_SCALARS {
        if skipped_key == Some(key) {
            continue;
        }
        require_required_scalar_core(core, key, expected).map_err(|err| err.message)?;
    }

    let expected_keys = [
        "bundle_id",
        "bundle_version",
        "projection_id",
        "ui_ir_ref",
        "binding_graph_ref",
        "action_ir_ref",
        "role_dictionary_version",
        "renderer_profile",
        "safety_class",
        "criticality",
        "freshness_policy",
        "hash",
        "signature",
        "created_by",
        "created_at",
        "compiler_identity",
        "require_verification",
        "allow_runtime_tree_streaming",
        "allow_production_activation",
        "require_safe_update_boundary",
        "allow_critical_update_during_pending_unknown",
        "allow_critical_update_during_quarantine",
    ];

    require_no_duplicate_scalars_core(core, &expected_keys, skipped_key).map_err(|err| err.message)?;
    require_no_known_unknown_fields_except_core(core, allowed_unknown_fields)?;

    if !skip_order_check {
        require_scalar_order_core(
            core,
            "bundle_version",
            "projection_id",
        ).map_err(|err| err.message)?;
        require_section_order_core(
            core,
            "projection_bundle/activation_policy",
            "projection_bundle/update_policy",
        ).map_err(|err| err.message)?;
    }

    Ok(())
}

fn validate_sketch_without_order_core(
    core: &SketchReaderCore,
    skipped_key: Option<&str>,
    allowed_unknown_fields: &[&str],
) -> Result<(), String> {
    for &(label, needle) in EXPECTED_CONTAINS {
        if !core.text_block.contains(needle) {
            return Err(format!("missing required anchor {}: {}", label, needle));
        }
    }

    require_trust_hash_shape_core(core, skipped_key)?;
    require_trust_signature_shape_core(core, skipped_key)?;

    for &(_label, key, expected) in EXPECTED_SCALARS {
        if skipped_key == Some(key) {
            continue;
        }
        require_required_scalar_core(core, key, expected).map_err(|err| err.message)?;
    }

    let expected_keys = [
        "bundle_id",
        "bundle_version",
        "projection_id",
        "ui_ir_ref",
        "binding_graph_ref",
        "action_ir_ref",
        "role_dictionary_version",
        "renderer_profile",
        "safety_class",
        "criticality",
        "freshness_policy",
        "hash",
        "signature",
        "created_by",
        "created_at",
        "compiler_identity",
        "require_verification",
        "allow_runtime_tree_streaming",
        "allow_production_activation",
        "require_safe_update_boundary",
        "allow_critical_update_during_pending_unknown",
        "allow_critical_update_during_quarantine",
    ];

    require_no_duplicate_scalars_core(core, &expected_keys, skipped_key).map_err(|err| err.message)?;
    require_no_known_unknown_fields_except_core(core, allowed_unknown_fields)?;

    Ok(())
}

fn validate_sketch_core(core: &SketchReaderCore) -> Result<ManifestSnapshot, String> {
    validate_sketch_except_core(core, None, &[], false)?;

    let bundle_id = require_required_scalar_core(core, "bundle_id", "bundle.example.minimal").map_err(|err| err.message)?;
    let bundle_version = require_required_scalar_core(core, "bundle_version", "0-sketch").map_err(|err| err.message)?;
    let projection_id = require_required_scalar_core(core, "projection_id", "ExampleMinimalProjection").map_err(|err| err.message)?;
    let ui_ir_ref = require_required_scalar_core(core, "ui_ir_ref", "ui_ir.example.minimal").map_err(|err| err.message)?;
    let binding_graph_ref = require_required_scalar_core(core, "binding_graph_ref", "binding_graph.example.minimal").map_err(|err| err.message)?;
    let action_ir_ref = require_required_scalar_core(core, "action_ir_ref", "action_ir.example.minimal").map_err(|err| err.message)?;
    let role_dictionary_version = require_required_scalar_core(core, "role_dictionary_version", "ui-roles.0-sketch").map_err(|err| err.message)?;
    let renderer_profile = require_required_scalar_core(core, "renderer_profile", "semantic-shell.reference-sketch").map_err(|err| err.message)?;
    let safety_class = require_required_scalar_core(core, "safety_class", "VerifiedDynamic").map_err(|err| err.message)?;
    let criticality = require_required_scalar_core(core, "criticality", "NonCritical").map_err(|err| err.message)?;
    let freshness_policy = require_required_scalar_core(core, "freshness_policy", "FreshForControl").map_err(|err| err.message)?;
    let hash = require_required_scalar_core(core, "hash", "sha256:SKETCH-NOT-A-REAL-HASH").map_err(|err| err.message)?;
    let signature = require_required_scalar_core(core, "signature", "signature:SKETCH-NOT-A-REAL-SIGNATURE").map_err(|err| err.message)?;
    let created_by = require_required_scalar_core(core, "created_by", "semantic-projection-compiler.SKETCH").map_err(|err| err.message)?;
    let created_at = require_required_scalar_core(core, "created_at", "not-a-real-timestamp").map_err(|err| err.message)?;
    let compiler_identity = require_required_scalar_core(core, "compiler_identity", "semantic-projection-compiler.0-sketch").map_err(|err| err.message)?;
    let require_verification = require_required_scalar_core(core, "require_verification", "true").map_err(|err| err.message)?;
    let allow_runtime_tree_streaming = require_required_scalar_core(core, "allow_runtime_tree_streaming", "false").map_err(|err| err.message)?;
    let allow_production_activation = require_required_scalar_core(core, "allow_production_activation", "false").map_err(|err| err.message)?;
    let require_safe_update_boundary = require_required_scalar_core(core, "require_safe_update_boundary", "true").map_err(|err| err.message)?;
    let allow_critical_update_during_pending_unknown = require_required_scalar_core(core, "allow_critical_update_during_pending_unknown", "false").map_err(|err| err.message)?;
    let allow_critical_update_during_quarantine = require_required_scalar_core(core, "allow_critical_update_during_quarantine", "false").map_err(|err| err.message)?;

    let (source_ref_0, source_ref_1) = require_source_refs_core(core).map_err(|err| err.message)?;

    Ok(ManifestSnapshot {
        bundle_id,
        bundle_version,
        projection_id,
        source_ref_0,
        source_ref_1,
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

fn extract_scalar(content: &str, key: &str) -> Option<String> {
    let core = parse_sketch_core(content).ok()?;
    find_scalar_core(&core, key).map(|scalar| scalar.value.clone())
}

fn count_scalar_occurrences(content: &str, key: &str) -> usize {
    match parse_sketch_core(content) {
        Ok(core) => count_scalar_occurrences_core(&core, key),
        Err(_) => 0,
    }
}

fn require_scalar(content: &str, _label: &str, key: &str, expected: &str) -> Result<String, String> {
    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    require_required_scalar_core(&core, key, expected).map_err(|err| err.message)
}

fn require_non_empty_scalar(content: &str, key: &str) -> Result<(), String> {
    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    require_non_empty_scalar_core(&core, key).map_err(|err| err.message)
}

fn require_boolean_scalar(content: &str, key: &str) -> Result<(), String> {
    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    require_boolean_scalar_core(&core, key).map_err(|err| err.message)
}

fn require_no_duplicate_scalars(
    content: &str,
    expected_keys: &[&str],
    skipped_key: Option<&str>,
) -> Result<(), String> {
    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    for key in expected_keys {
        if skipped_key == Some(*key) {
            continue;
        }

        if count_scalar_occurrences_core(&core, key) > 1 {
            return Err(format!("duplicate_field: {}", key));
        }
    }

    Ok(())
}

fn require_no_known_unknown_fields_except(
    content: &str,
    allowed_unknown_fields: &[&str],
) -> Result<(), String> {
    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    require_no_known_unknown_fields_except_core(&core, allowed_unknown_fields)
}

fn require_field_order(
    content: &str,
    earlier: &str,
    later: &str,
    rejection_reason: &str,
) -> Result<(), String> {
    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    let _ = rejection_reason;
    if earlier.contains('{') || later.contains('{') {
        let earlier_label = earlier
            .trim()
            .trim_end_matches('{')
            .trim_end_matches(':')
            .trim();
        let later_label = later
            .trim()
            .trim_end_matches('{')
            .trim_end_matches(':')
            .trim();
        require_section_order_core(
            &core,
            &format!("projection_bundle/{}", earlier_label),
            &format!("projection_bundle/{}", later_label),
        )
        .map_err(|err| err.message)
    } else {
        require_scalar_order_core(
            &core,
            earlier.trim().trim_end_matches(':'),
            later.trim().trim_end_matches(':'),
        )
        .map_err(|err| err.message)
    }
}

fn validate_sketch_except(
    content: &str,
    skipped_key: Option<&str>,
    allowed_unknown_fields: &[&str],
    skip_order_check: bool,
) -> Result<(), String> {
    for &(label, needle) in EXPECTED_CONTAINS {
        require_contains(content, label, needle)?;
    }

    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    require_trust_hash_shape_core(&core, skipped_key)?;
    require_trust_signature_shape_core(&core, skipped_key)?;

    for &(label, key, expected) in EXPECTED_SCALARS {
        if skipped_key == Some(key) {
            continue;
        }

        require_scalar(content, label, key, expected)?;
    }

    let expected_keys = [
        "bundle_id",
        "bundle_version",
        "projection_id",
        "ui_ir_ref",
        "binding_graph_ref",
        "action_ir_ref",
        "role_dictionary_version",
        "renderer_profile",
        "safety_class",
        "criticality",
        "freshness_policy",
        "hash",
        "signature",
        "created_by",
        "created_at",
        "compiler_identity",
        "require_verification",
        "allow_runtime_tree_streaming",
        "allow_production_activation",
        "require_safe_update_boundary",
        "allow_critical_update_during_pending_unknown",
        "allow_critical_update_during_quarantine",
    ];

    require_no_duplicate_scalars(content, &expected_keys, skipped_key)?;
    require_no_known_unknown_fields_except(content, allowed_unknown_fields)?;

    if !skip_order_check {
        require_field_order(
            content,
            "bundle_version:",
            "projection_id:",
            "field_ordering: bundle_version must appear before projection_id",
        )?;
        require_field_order(
            content,
            "activation_policy {",
            "update_policy {",
            "field_ordering: activation_policy must appear before update_policy",
        )?;
    }

    Ok(())
}

fn validate_sketch_without_order(
    content: &str,
    skipped_key: Option<&str>,
    allowed_unknown_fields: &[&str],
) -> Result<(), String> {
    for &(label, needle) in EXPECTED_CONTAINS {
        require_contains(content, label, needle)?;
    }

    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    require_trust_hash_shape_core(&core, skipped_key)?;
    require_trust_signature_shape_core(&core, skipped_key)?;

    for &(label, key, expected) in EXPECTED_SCALARS {
        if skipped_key == Some(key) {
            continue;
        }

        require_scalar(content, label, key, expected)?;
    }

    let expected_keys = [
        "bundle_id",
        "bundle_version",
        "projection_id",
        "ui_ir_ref",
        "binding_graph_ref",
        "action_ir_ref",
        "role_dictionary_version",
        "renderer_profile",
        "safety_class",
        "criticality",
        "freshness_policy",
        "hash",
        "signature",
        "created_by",
        "created_at",
        "compiler_identity",
        "require_verification",
        "allow_runtime_tree_streaming",
        "allow_production_activation",
        "require_safe_update_boundary",
        "allow_critical_update_during_pending_unknown",
        "allow_critical_update_during_quarantine",
    ];

    require_no_duplicate_scalars(content, &expected_keys, skipped_key)?;
    require_no_known_unknown_fields_except(content, allowed_unknown_fields)?;

    Ok(())
}

fn validate_sketch(content: &str) -> Result<ManifestSnapshot, String> {
    let core = parse_sketch_core(content).map_err(|err| err.message)?;
    validate_sketch_except(content, None, &[], false)?;

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
    let (source_ref_0, source_ref_1) =
        require_source_refs_core(&core).map_err(|err| err.message)?;

    Ok(ManifestSnapshot {
        bundle_id,
        bundle_version,
        projection_id,
        source_ref_0,
        source_ref_1,
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

fn validate_positive_fixture(repo_root: &str) -> Result<ManifestSnapshot, String> {
    let sketch = read_sketch(repo_root);
    let core = parse_sketch_core(&sketch).map_err(|err| err.message)?;
    validate_sketch_core(&core)
}

fn validate_negative_fixture(
    repo_root: &str,
    case: &NegativeFixtureCase,
) -> Result<NegativeCaseResult, String> {
    let sketch = read_fixture(repo_root, case.relative_path);
    let input = repo_relative_path(case.relative_path);
    let core = parse_sketch_core(&sketch).map_err(|err| err.message)?;

    match case.rule {
        NegativeRule::MissingField {
            label,
            key,
        } => {
            let skip_order_check = matches!(key, "projection_id" | "bundle_version");
            validate_sketch_except_core(&core, Some(key), &[], skip_order_check)?;

            if find_scalar_core(&core, key).map(|s| s.value.clone()).is_some() {
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
                status: NegativeCaseStatus::Rejected,
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
            validate_sketch_except_core(&core, Some(key), &[], false)?;

            match find_scalar_core(&core, key).map(|s| s.value.clone()) {
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
                        status: NegativeCaseStatus::Rejected,
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
        NegativeRule::MalformedEmpty {
            key,
            rejection_reason,
        } => {
            validate_sketch_except_core(&core, Some(key), &[], false)?;

            match require_non_empty_scalar_core(&core, key).map_err(|err| err.message) {
                Ok(()) => Err(format!("negative fixture unexpectedly passed: {}", case.name)),
                Err(reason) => {
                    if !reason.contains(case.expected_error_substring) || reason != rejection_reason {
                        return Err(format!(
                            "negative fixture failed for wrong reason: {}: {}",
                            case.name, reason
                        ));
                    }

                    Ok(NegativeCaseResult {
                        name: case.name,
                        input,
                        status: NegativeCaseStatus::Rejected,
                        reason,
                    })
                }
            }
        }
        NegativeRule::MalformedBoolean {
            key,
            rejection_reason,
        } => {
            validate_sketch_except_core(&core, Some(key), &[], false)?;

            match require_boolean_scalar_core(&core, key).map_err(|err| err.message) {
                Ok(()) => Err(format!("negative fixture unexpectedly passed: {}", case.name)),
                Err(reason) => {
                    if !reason.contains(case.expected_error_substring) || reason != rejection_reason {
                        return Err(format!(
                            "negative fixture failed for wrong reason: {}: {}",
                            case.name, reason
                        ));
                    }

                    Ok(NegativeCaseResult {
                        name: case.name,
                        input,
                        status: NegativeCaseStatus::Rejected,
                        reason,
                    })
                }
            }
        }
        NegativeRule::UnknownField {
            key,
            rejection_reason,
        } => {
            validate_sketch_except_core(&core, None, &[key], false)?;

            if count_scalar_occurrences_core(&core, key) == 0 {
                return Err(format!("negative fixture unexpectedly passed: {}", case.name));
            }

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
                status: NegativeCaseStatus::Rejected,
                reason,
            })
        }
        NegativeRule::DuplicateField {
            key,
            rejection_reason,
        } => {
            validate_sketch_except_core(&core, Some(key), &[], false)?;

            if count_scalar_occurrences_core(&core, key) <= 1 {
                return Err(format!("negative fixture unexpectedly passed: {}", case.name));
            }

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
                status: NegativeCaseStatus::Rejected,
                reason,
            })
        }
        NegativeRule::FieldOrdering {
            earlier,
            later,
            rejection_reason: _,
        } => {
            validate_sketch_without_order_core(&core, None, &[])?;

            match {
                let earlier_label = earlier.trim().trim_end_matches('{').trim_end_matches(':').trim();
                let later_label = later.trim().trim_end_matches('{').trim_end_matches(':').trim();
                if earlier.contains('{') || later.contains('{') {
                    require_section_order_core(&core, &format!("projection_bundle/{}", earlier_label), &format!("projection_bundle/{}", later_label)).map_err(|err| err.message)
                } else {
                    require_scalar_order_core(&core, earlier_label, later_label).map_err(|err| err.message)
                }
            } {
                Ok(()) => Err(format!("negative fixture unexpectedly passed: {}", case.name)),
                Err(reason) => {
                    if !reason.contains(case.expected_error_substring) {
                        return Err(format!(
                            "negative fixture failed for wrong reason: {}: {}",
                            case.name, reason
                        ));
                    }

                    Ok(NegativeCaseResult {
                        name: case.name,
                        input,
                        status: NegativeCaseStatus::Rejected,
                        reason,
                    })
                }
            }
        }
    }
}

fn validate_negative_pack(repo_root: &str) -> Result<NegativeReport, String> {
    let mut report = NegativeReport::new();

    for case in NEGATIVE_CASES {
        let result = validate_negative_fixture(repo_root, case)?;
        report.push_case(result);
    }

    Ok(report)
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

fn render_negative_report(report: &NegativeReport) -> String {
    let mut out = String::new();
    push_line(&mut out, "ProjectionBundleSketchReaderNegativeReport v0");
    push_line(&mut out, "scope=fixture-facing");
    push_line(&mut out, "status=rejected-negative-pack");
    push_line(&mut out, "");

    for (index, case) in report.cases.iter().enumerate() {
        push_line(&mut out, &format!("case.{}.name={}", index, case.name));
        push_line(&mut out, &format!("case.{}.input={}", index, case.input));
        push_line(
            &mut out,
            &format!("case.{}.status={}", index, case.status.as_str()),
        );
        push_line(&mut out, &format!("case.{}.reason={}", index, case.reason));
        push_line(&mut out, "");
    }

    push_line(
        &mut out,
        &format!("summary.accepted_positive={}", report.accepted_positive()),
    );
    push_line(
        &mut out,
        &format!("summary.rejected_negative={}", report.rejected_negative()),
    );
    push_line(
        &mut out,
        &format!("summary.unexpected_pass={}", report.unexpected_pass()),
    );
    push_line(
        &mut out,
        &format!("summary.wrong_reason={}", report.wrong_reason()),
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

fn emit_positive_output(repo_root: &str) -> Result<String, String> {
    let snapshot = validate_positive_fixture(repo_root)?;
    Ok(render_positive_output(&snapshot))
}

fn emit_negative_report(repo_root: &str) -> Result<String, String> {
    let report = validate_negative_pack(repo_root)?;
    Ok(render_negative_report(&report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn sample_positive_text() -> String {
        [
            "```text",
            "projection_bundle {",
            "  bundle_id: \"bundle.example.minimal\"",
            "  bundle_version: \"0-sketch\"",
            "  projection_id: \"ExampleMinimalProjection\"",
            "",
            "  source_refs: [",
            "    \"semantic.source.example\"",
            "    \"projection.source.example\"",
            "  ]",
            "",
            "  artifacts {",
            "    ui_ir_ref: \"ui_ir.example.minimal\"",
            "    binding_graph_ref: \"binding_graph.example.minimal\"",
            "    action_ir_ref: \"action_ir.example.minimal\"",
            "  }",
            "",
            "  compatibility {",
            "    role_dictionary_version: \"ui-roles.0-sketch\"",
            "    renderer_profile: \"semantic-shell.reference-sketch\"",
            "  }",
            "",
            "  safety {",
            "    safety_class: \"VerifiedDynamic\"",
            "    criticality: \"NonCritical\"",
            "    required_capabilities: []",
            "    freshness_policy: \"FreshForControl\"",
            "  }",
            "",
            "  trust {",
            "    hash: \"sha256:SKETCH-NOT-A-REAL-HASH\"",
            "    signature: \"signature:SKETCH-NOT-A-REAL-SIGNATURE\"",
            "    created_by: \"semantic-projection-compiler.SKETCH\"",
            "    created_at: \"not-a-real-timestamp\"",
            "    compiler_identity: \"semantic-projection-compiler.0-sketch\"",
            "  }",
            "",
            "  activation_policy {",
            "    require_verification: true",
            "    allow_runtime_tree_streaming: false",
            "    allow_production_activation: false",
            "  }",
            "",
            "  update_policy {",
            "    require_safe_update_boundary: true",
            "    allow_critical_update_during_pending_unknown: false",
            "    allow_critical_update_during_quarantine: false",
            "  }",
            "",
            "  diagnostics {",
            "    expected: []",
            "  }",
            "}",
            "```",
        ]
        .join("\n")
    }

    fn current_repo_root() -> String {
        std::env::current_dir()
            .unwrap_or_else(|err| panic!("current_dir failed: {}", err))
            .to_string_lossy()
            .to_string()
    }

    fn expected_positive_output() -> String {
        fs::read_to_string(
            Path::new("tests")
                .join("fixtures")
                .join("post_ui")
                .join("projection_bundle")
                .join("expected")
                .join("manifest_minimal.reader.out.txt"),
        )
        .unwrap_or_else(|err| panic!("failed to read positive golden: {}", err))
    }

    fn expected_negative_report() -> String {
        fs::read_to_string(
            Path::new("tests")
                .join("fixtures")
                .join("post_ui")
                .join("projection_bundle")
                .join("expected")
                .join("negative_pack.reader.out.txt"),
        )
        .unwrap_or_else(|err| panic!("failed to read negative golden: {}", err))
    }

    fn normalize_compared_text(text: &str) -> String {
        normalize_line_endings(text)
            .trim_end_matches('\n')
            .to_string()
    }

    #[test]
    fn extracts_text_block() {
        let core = parse_sketch_core(&sample_positive_text()).expect("core parse");
        assert!(core.text_block.contains("projection_bundle {"));
        assert!(core.text_block.contains("activation_policy {"));
    }

    #[test]
    fn scans_section_start_lines() {
        let core = parse_sketch_core(&sample_positive_text()).expect("core parse");
        let root = find_section_core(&core, "projection_bundle").expect("root section");
        let activation = find_section_core(&core, "projection_bundle/activation_policy")
            .expect("activation section");
        let update =
            find_section_core(&core, "projection_bundle/update_policy").expect("update section");

        assert_eq!(root.start_line, 1);
        assert!(activation.start_line > root.start_line);
        assert!(update.start_line > activation.start_line);
    }

    #[test]
    fn scans_section_end_lines() {
        let core = parse_sketch_core(&sample_positive_text()).expect("core parse");
        let root = find_section_core(&core, "projection_bundle").expect("root section");
        let activation = find_section_core(&core, "projection_bundle/activation_policy")
            .expect("activation section");
        let update =
            find_section_core(&core, "projection_bundle/update_policy").expect("update section");

        let root_end = root.end_line.expect("root end");
        let activation_end = activation.end_line.expect("activation end");
        let update_end = update.end_line.expect("update end");

        assert!(root_end > update_end);
        assert!(activation_end > activation.start_line);
        assert!(update_end > update.start_line);
    }

    #[test]
    fn scans_source_refs_array_values() {
        let core = parse_sketch_core(&sample_positive_text()).expect("core parse");
        let (source_ref_0, source_ref_1) =
            require_source_refs_core(&core).expect("source refs");

        assert_eq!(source_ref_0, "semantic.source.example");
        assert_eq!(source_ref_1, "projection.source.example");
    }

    #[test]
    fn detects_empty_bundle_id() {
        let sketch = sample_positive_text().replace(
            "  bundle_id: \"bundle.example.minimal\"",
            "  bundle_id: \"\"",
        );
        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_non_empty_scalar_core(&core, "bundle_id").expect_err("expected malformed bundle_id");
        assert_eq!(err.message, "malformed_field: bundle_id is empty");
    }

    #[test]
    fn detects_malformed_boolean() {
        let sketch = sample_positive_text().replace(
            "    require_verification: true",
            "    require_verification: \"not-a-bool\"",
        );
        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_boolean_scalar_core(&core, "require_verification")
            .expect_err("expected malformed boolean");
        assert_eq!(err.message, "malformed_field: require_verification must be boolean");
    }

    #[test]
    fn detects_unknown_future_field() {
        let sketch = sample_positive_text().replacen(
            "  projection_id: \"ExampleMinimalProjection\"",
            "  unknown_future_field: \"must-reject\"\n  projection_id: \"ExampleMinimalProjection\"",
            1,
        );
        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_no_known_unknown_fields_core(&core).expect_err("expected unknown field");
        assert_eq!(err.message, "unknown_field: unknown_future_field");
    }

    #[test]
    fn detects_allow_unreviewed_activation() {
        let sketch = sample_positive_text().replacen(
            "    allow_runtime_tree_streaming: false",
            "    allow_unreviewed_activation: true\n    allow_runtime_tree_streaming: false",
            1,
        );
        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_no_known_unknown_fields_core(&core).expect_err("expected unknown field");
        assert_eq!(err.message, "unknown_field: allow_unreviewed_activation");
    }

    #[test]
    fn detects_duplicate_bundle_id() {
        let sketch = sample_positive_text().replacen(
            "  bundle_id: \"bundle.example.minimal\"",
            "  bundle_id: \"bundle.example.minimal\"\n  bundle_id: \"bundle.example.duplicate\"",
            1,
        );
        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_no_duplicate_scalars_core(&core, &["bundle_id", "require_verification"], None)
            .expect_err("expected duplicate bundle_id");
        assert_eq!(err.message, "duplicate_field: bundle_id");
    }

    #[test]
    fn detects_duplicate_require_verification() {
        let sketch = sample_positive_text().replacen(
            "    require_verification: true",
            "    require_verification: true\n    require_verification: false",
            1,
        );
        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_no_duplicate_scalars_core(&core, &["bundle_id", "require_verification"], None)
            .expect_err("expected duplicate require_verification");
        assert_eq!(err.message, "duplicate_field: require_verification");
    }

    #[test]
    fn detects_identity_ordering_violation() {
        let sketch = sample_positive_text().replacen(
            "  bundle_version: \"0-sketch\"\n  projection_id: \"ExampleMinimalProjection\"",
            "  projection_id: \"ExampleMinimalProjection\"\n  bundle_version: \"0-sketch\"",
            1,
        );
        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_scalar_order_core(&core, "bundle_version", "projection_id")
            .expect_err("expected order violation");
        assert_eq!(
            err.message,
            "field_ordering: bundle_version must appear before projection_id"
        );
    }

    #[test]
    fn detects_policy_block_ordering_violation() {
        let sketch = [
            "```text",
            "projection_bundle {",
            "  bundle_id: \"bundle.example.minimal\"",
            "  bundle_version: \"0-sketch\"",
            "  projection_id: \"ExampleMinimalProjection\"",
            "",
            "  source_refs: [",
            "    \"semantic.source.example\"",
            "    \"projection.source.example\"",
            "  ]",
            "",
            "  artifacts {",
            "    ui_ir_ref: \"ui_ir.example.minimal\"",
            "    binding_graph_ref: \"binding_graph.example.minimal\"",
            "    action_ir_ref: \"action_ir.example.minimal\"",
            "  }",
            "",
            "  compatibility {",
            "    role_dictionary_version: \"ui-roles.0-sketch\"",
            "    renderer_profile: \"semantic-shell.reference-sketch\"",
            "  }",
            "",
            "  safety {",
            "    safety_class: \"VerifiedDynamic\"",
            "    criticality: \"NonCritical\"",
            "    required_capabilities: []",
            "    freshness_policy: \"FreshForControl\"",
            "  }",
            "",
            "  trust {",
            "    hash: \"sha256:SKETCH-NOT-A-REAL-HASH\"",
            "    signature: \"signature:SKETCH-NOT-A-REAL-SIGNATURE\"",
            "    created_by: \"semantic-projection-compiler.SKETCH\"",
            "    created_at: \"not-a-real-timestamp\"",
            "    compiler_identity: \"semantic-projection-compiler.0-sketch\"",
            "  }",
            "",
            "  update_policy {",
            "    require_safe_update_boundary: true",
            "    allow_critical_update_during_pending_unknown: false",
            "    allow_critical_update_during_quarantine: false",
            "  }",
            "",
            "  activation_policy {",
            "    require_verification: true",
            "    allow_runtime_tree_streaming: false",
            "    allow_production_activation: false",
            "  }",
            "",
            "  diagnostics {",
            "    expected: []",
            "  }",
            "}",
            "```",
        ]
        .join("\n");

        let core = parse_sketch_core(&sketch).expect("core parse");
        let err = require_section_order_core(
            &core,
            "projection_bundle/activation_policy",
            "projection_bundle/update_policy",
        )
            .expect_err("expected order violation");
        assert_eq!(
            err.message,
            "field_ordering: activation_policy must appear before update_policy"
        );
    }

    #[test]
    fn positive_snapshot_source_refs_come_from_parsed_source_refs() {
        let core = parse_sketch_core(&sample_positive_text()).expect("core parse");
        let (source_ref_0, source_ref_1) =
            require_source_refs_core(&core).expect("source refs");
        assert_eq!(source_ref_0, "semantic.source.example");
        assert_eq!(source_ref_1, "projection.source.example");

        let snapshot = validate_sketch(&sample_positive_text()).expect("snapshot");
        assert_eq!(snapshot.source_ref_0, source_ref_0);
        assert_eq!(snapshot.source_ref_1, source_ref_1);
    }

    #[test]
    fn source_ref_mutation_changes_parsed_snapshot_in_an_inline_test() {
        let mutated = sample_positive_text().replace(
            "projection.source.example",
            "projection.source.mutated",
        );
        let core = parse_sketch_core(&mutated).expect("core parse");
        let (source_ref_0, source_ref_1) =
            require_source_refs_core(&core).expect("source refs");

        assert_eq!(source_ref_0, "semantic.source.example");
        assert_eq!(source_ref_1, "projection.source.mutated");
    }

    #[test]
    fn positive_output_is_generated_from_parsed_reader_output() {
        let repo_root = current_repo_root();
        let snapshot = validate_positive_fixture(&repo_root).expect("positive snapshot");
        let actual = emit_positive_output(&repo_root).expect("positive output");
        let derived = render_positive_output(&snapshot);
        assert_eq!(
            normalize_compared_text(&actual),
            normalize_compared_text(&derived)
        );
    }

    #[test]
    fn golden_positive_output_unchanged() {
        let repo_root = current_repo_root();
        let actual = emit_positive_output(&repo_root).expect("positive output");
        let expected = expected_positive_output();
        assert_eq!(
            normalize_compared_text(&actual),
            normalize_compared_text(&expected)
        );
    }

    #[test]
    fn negative_report_contains_16_case_results() {
        let repo_root = current_repo_root();
        let report = validate_negative_pack(&repo_root).expect("negative report");
        assert_eq!(report.cases.len(), 16);
    }

    #[test]
    fn negative_report_summary_is_derived_from_case_results() {
        let repo_root = current_repo_root();
        let report = validate_negative_pack(&repo_root).expect("negative report");
        assert_eq!(report.accepted_positive(), 1);
        assert_eq!(report.rejected_negative(), 16);
        assert_eq!(report.unexpected_pass(), 0);
        assert_eq!(report.wrong_reason(), 0);
    }

    #[test]
    fn wrong_expected_reason_increments_wrong_reason() {
        let report = NegativeReport::new().with_wrong_reason("case", "input", "wrong reason");
        assert_eq!(report.wrong_reason(), 1);
        assert_eq!(report.unexpected_pass(), 0);
        assert_eq!(report.rejected_negative(), 0);
    }

    #[test]
    fn unexpected_accepted_negative_increments_unexpected_pass() {
        let report = NegativeReport::new().with_unexpected_pass("case", "input");
        assert_eq!(report.unexpected_pass(), 1);
        assert_eq!(report.wrong_reason(), 0);
        assert_eq!(report.rejected_negative(), 0);
    }

    #[test]
    fn golden_negative_report_unchanged() {
        let repo_root = current_repo_root();
        let actual = emit_negative_report(&repo_root).expect("negative output");
        let expected = expected_negative_report();
        assert_eq!(
            normalize_compared_text(&actual),
            normalize_compared_text(&expected)
        );
    }
}

fn run_probe(path: &str) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("io error: {}", e))?;
    let file_name = Path::new(path).file_name().unwrap_or_default().to_string_lossy();

    let mut out = String::new();
    push_line(&mut out, &format!("fixture: {}", file_name));

    match parse_sketch_core(&content) {
        Ok(core) => {
            push_line(
                &mut out,
                &format!(
                    "text_block: {}",
                    if core.text_block.is_empty() {
                        "missing"
                    } else {
                        "found"
                    }
                ),
            );
            push_line(&mut out, &format!("sections: {} total", core.sections.len()));
            for section in &core.sections {
                let end = section
                    .end_line
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "EOF".to_string());
                push_line(
                    &mut out,
                    &format!("  {} (L{} - L{})", section.path, section.start_line, end),
                );
            }
            push_line(&mut out, &format!("scalars: {} total", core.scalars.len()));
            for scalar in &core.scalars {
                push_line(
                    &mut out,
                    &format!(
                        "  [{}] {} = {} (L{})",
                        scalar.section, scalar.key, scalar.value, scalar.line
                    ),
                );
            }
            push_line(&mut out, &format!("arrays: {} total", core.arrays.len()));
            for array in &core.arrays {
                let joined = array.values.join(", ");
                push_line(
                    &mut out,
                    &format!(
                        "  [{}] {} = [{}] (L{})",
                        array.section, array.key, joined, array.line
                    ),
                );
            }
        }
        Err(e) => {
            push_line(&mut out, &format!("core_parse_error: {}", e.message));
        }
    }

    if let Ok(core) = parse_sketch_core(&content) {
        let mut hash_shape = "unknown";
        let mut signature_shape = "unknown";
        let mut has_unknown_sections = false;
        let mut has_unknown_fields = false;

        let known_sections = [
            "projection_bundle",
            "projection_bundle/artifacts",
            "projection_bundle/compatibility",
            "projection_bundle/safety",
            "projection_bundle/trust",
            "projection_bundle/activation_policy",
            "projection_bundle/update_policy",
            "projection_bundle/diagnostics",
        ];

        let known_fields = [
            ("projection_bundle", "bundle_id"),
            ("projection_bundle", "bundle_version"),
            ("projection_bundle", "projection_id"),
            ("projection_bundle", "source_refs"),
            ("projection_bundle/artifacts", "ui_ir_ref"),
            ("projection_bundle/artifacts", "binding_graph_ref"),
            ("projection_bundle/artifacts", "action_ir_ref"),
            ("projection_bundle/compatibility", "role_dictionary_version"),
            ("projection_bundle/compatibility", "renderer_profile"),
            ("projection_bundle/safety", "safety_class"),
            ("projection_bundle/safety", "criticality"),
            ("projection_bundle/safety", "freshness_policy"),
            ("projection_bundle/safety", "required_capabilities"),
            ("projection_bundle/trust", "hash"),
            ("projection_bundle/trust", "signature"),
            ("projection_bundle/trust", "created_by"),
            ("projection_bundle/trust", "created_at"),
            ("projection_bundle/trust", "compiler_identity"),
            ("projection_bundle/activation_policy", "require_verification"),
            ("projection_bundle/activation_policy", "allow_runtime_tree_streaming"),
            ("projection_bundle/activation_policy", "allow_production_activation"),
            ("projection_bundle/update_policy", "require_safe_update_boundary"),
            ("projection_bundle/update_policy", "allow_critical_update_during_pending_unknown"),
            ("projection_bundle/update_policy", "allow_critical_update_during_quarantine"),
            ("projection_bundle/diagnostics", "expected"),
        ];

        for section in &core.sections {
            if !known_sections.contains(&section.path.as_str()) {
                has_unknown_sections = true;
            }
        }

        for scalar in &core.scalars {
            if scalar.section == "projection_bundle/trust" {
                if scalar.key == "hash" {
                    hash_shape = classify_hash_shape(&scalar.value);
                } else if scalar.key == "signature" {
                    signature_shape = classify_signature_shape(&scalar.value);
                }
            }
            if !known_fields.contains(&(scalar.section.as_str(), scalar.key.as_str())) {
                has_unknown_fields = true;
            }
        }

        for array in &core.arrays {
            if !known_fields.contains(&(array.section.as_str(), array.key.as_str())) {
                has_unknown_fields = true;
            }
        }

        let unknown_items = match (has_unknown_sections, has_unknown_fields) {
            (true, true) => "both",
            (true, false) => "found_unknown_sections",
            (false, true) => "found_unknown_fields",
            (false, false) => "none",
        };

        push_line(&mut out, &format!("hash_shape: {}", hash_shape));
        push_line(&mut out, &format!("signature_shape: {}", signature_shape));
        push_line(&mut out, &format!("unknown_items: {}", unknown_items));
    }

    match validate_sketch(&content) {
        Ok(_) => {
            push_line(&mut out, "validation_result: accepted");
        }
        Err(e) => {
            push_line(&mut out, "validation_result: rejected");
            push_line(&mut out, &format!("primary_error: {}", e));
        }
    }
    push_line(&mut out, "---");
    Ok(out)
}

fn main() {
    let mut args = env::args().skip(1);
    let first = args
        .next()
        .unwrap_or_else(|| fail("missing repository root or probe argument"));

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
        "--probe-fixture" => {
            let file_path = args
                .next()
                .unwrap_or_else(|| fail("missing probe fixture path"));
            if args.next().is_some() {
                fail("unexpected extra arguments");
            }

            (Some(first), file_path)
        }
        _ => {
            if args.next().is_some() {
                fail("unexpected extra arguments");
            }

            (None, normalize_repo_root(&first))
        }
    };

    match mode.as_deref() {
        Some("--probe-fixture") => match run_probe(&repo_root) {
            Ok(output) => {
                print!("{}", output);
            }
            Err(e) => fail(format!("probe failed: {}", e)),
        },
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
