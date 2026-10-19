//! S2043 CP06: the single canonical registry behind
//! `smc language describe --format json` (`semantic.language-surface` v1,
//! `docs/spec/language_surface_v1.md`).
//!
//! Every entry states, for one source feature, its release maturity and the
//! outcome of each pipeline stage under the default parser profile at both
//! `O0` and `O1`. The entry also carries the probe program that demonstrates
//! those outcomes and, for executable features, a negative probe with the
//! stage that must reject it. `tests/s2043_language_surface.rs` runs every
//! probe through the real `check -> compile -> verify -> run` pipeline of
//! the same build and fails when a declared status disagrees with what the
//! pipeline does, so this table cannot drift from the compiler unnoticed.
//! It is data only: no host effect, no capability, no execution here.

/// `schema_id` of the published document.
pub const LANGUAGE_SURFACE_SCHEMA_ID: &str = "semantic.language-surface";
/// `schema_version` of the published document.
pub const LANGUAGE_SURFACE_SCHEMA_VERSION: u32 = 1;

/// Release maturity, independent of pipeline stage status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseState {
    PublishedStable,
    FoundationIncluded,
    CurrentMainExperimental,
    DraftOnly,
    Unsupported,
    Unknown,
}

impl ReleaseState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PublishedStable => "published-stable",
            Self::FoundationIncluded => "foundation-included",
            Self::CurrentMainExperimental => "current-main-experimental",
            Self::DraftOnly => "draft-only",
            Self::Unsupported => "unsupported",
            Self::Unknown => "unknown",
        }
    }
}

/// One pipeline stage, in pipeline order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Lex,
    Parse,
    Typecheck,
    Lower,
    Emit,
    Verify,
    Run,
}

impl Stage {
    pub const ALL: [Stage; 7] = [
        Stage::Lex,
        Stage::Parse,
        Stage::Typecheck,
        Stage::Lower,
        Stage::Emit,
        Stage::Verify,
        Stage::Run,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lex => "lex",
            Self::Parse => "parse",
            Self::Typecheck => "typecheck",
            Self::Lower => "lower",
            Self::Emit => "emit",
            Self::Verify => "verify",
            Self::Run => "run",
        }
    }
}

/// The outcome of one stage for a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    /// The stage accepts the feature, proven by the feature's probe.
    Qualified,
    /// The stage deterministically rejects the feature (the first failing
    /// stage), proven by the feature's probe.
    Rejected,
    /// Not proven either way.
    NotQualified,
    /// Not reached: an earlier stage rejects the feature.
    NotApplicable,
}

impl StageStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Qualified => "qualified",
            Self::Rejected => "rejected",
            Self::NotQualified => "not-qualified",
            Self::NotApplicable => "not-applicable",
        }
    }
}

/// One registry entry. `stages` is indexed in [`Stage::ALL`] order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageFeature {
    pub id: &'static str,
    pub release_state: ReleaseState,
    pub stages: [StageStatus; 7],
    /// Program whose pipeline outcome is exactly `stages`.
    pub probe: &'static str,
    /// For executable features: a program using the feature wrongly, and
    /// the stage that must reject it.
    pub negative_probe: Option<(&'static str, Stage)>,
    /// The suite that checks this entry against the running pipeline.
    pub evidence_suite: &'static str,
}

impl LanguageFeature {
    pub const fn stage(&self, stage: Stage) -> StageStatus {
        self.stages[stage as usize]
    }
}

const EVIDENCE: &str =
    "tests/s2043_language_surface.rs::declared_stages_match_the_running_pipeline";

use StageStatus::{NotApplicable as NA, Qualified as Q, Rejected as R};

const EXECUTABLE: [StageStatus; 7] = [Q, Q, Q, Q, Q, Q, Q];
const LEX_REJECTED: [StageStatus; 7] = [R, NA, NA, NA, NA, NA, NA];
const PARSE_REJECTED: [StageStatus; 7] = [Q, R, NA, NA, NA, NA, NA];
const TYPECHECK_REJECTED: [StageStatus; 7] = [Q, Q, R, NA, NA, NA, NA];
const LOWER_REJECTED: [StageStatus; 7] = [Q, Q, Q, R, NA, NA, NA];

/// The registry, sorted by `id` (ascending, byte order) and unique.
pub const LANGUAGE_FEATURES: &[LanguageFeature] = &[
    LanguageFeature {
        id: "arithmetic.u32.checked",
        release_state: ReleaseState::CurrentMainExperimental,
        stages: EXECUTABLE,
        probe: "fn main() {\n    let a: u32 = 7u32;\n    assert(a / 2u32 == 3u32);\n    assert(a % 2u32 == 1u32);\n    assert(a > 2u32);\n    return;\n}\n",
        negative_probe: Some((
            "fn main() {\n    let a: u32 = 1u32;\n    let b: i32 = 1;\n    let c: u32 = a + b;\n    let _ = c;\n    return;\n}\n",
            Stage::Typecheck,
        )),
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "control.function.return_completeness",
        release_state: ReleaseState::CurrentMainExperimental,
        stages: EXECUTABLE,
        probe: "fn pick(flag: bool) -> i32 {\n    if flag {\n        return 1;\n    } else {\n        return 2;\n    }\n}\nfn main() {\n    assert(pick(false) == 2);\n    return;\n}\n",
        negative_probe: Some((
            "fn pick(flag: bool) -> i32 {\n    if flag {\n        return 1;\n    } else {\n        let z: i32 = 0;\n        let _ = z;\n    }\n}\nfn main() {\n    assert(pick(true) == 1);\n    return;\n}\n",
            Stage::Typecheck,
        )),
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "control.if_let.expression",
        release_state: ReleaseState::CurrentMainExperimental,
        stages: EXECUTABLE,
        probe: "fn main() {\n    let o: Option(i32) = Option::Some(4);\n    let v: i32 = if let Option::Some(x) = o { x } else { 0 };\n    assert(v == 4);\n    return;\n}\n",
        negative_probe: Some((
            "fn main() {\n    let o: Option(i32) = Option::Some(4);\n    let v: i32 = if let Option::Some(x) = o { x } else { true };\n    let _ = v;\n    return;\n}\n",
            Stage::Typecheck,
        )),
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "control.let_else.adt_pattern",
        release_state: ReleaseState::Unsupported,
        stages: PARSE_REJECTED,
        probe: "fn f(o: Option(i32)) -> i32 {\n    let Option::Some(x) = o else return 0;\n    return x;\n}\nfn main() {\n    return;\n}\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "control.match.integer_range_pattern",
        release_state: ReleaseState::Unsupported,
        stages: LOWER_REJECTED,
        probe: "fn f(x: i32) -> i32 {\n    match x {\n        1..=3 => { return 1; }\n        _ => { return 0; }\n    }\n}\nfn main() {\n    assert(f(2) == 1);\n    return;\n}\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "control.match.or_pattern",
        release_state: ReleaseState::Unsupported,
        stages: TYPECHECK_REJECTED,
        probe: "fn f(x: i32) -> i32 {\n    match x {\n        1 | 2 => { return 1; }\n        _ => { return 0; }\n    }\n}\nfn main() {\n    return;\n}\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "item.const.top_level",
        release_state: ReleaseState::Unsupported,
        stages: PARSE_REJECTED,
        probe: "const LIMIT: i32 = 3;\nfn main() {\n    return;\n}\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "logic.bool.and_or",
        release_state: ReleaseState::FoundationIncluded,
        stages: EXECUTABLE,
        probe: "fn main() {\n    let a: bool = true;\n    let b: bool = false;\n    assert((a && !b) == true);\n    assert((a || b) == true);\n    return;\n}\n",
        negative_probe: Some((
            "fn main() {\n    let a: bool = true;\n    let c: bool = a && 1;\n    let _ = c;\n    return;\n}\n",
            Stage::Typecheck,
        )),
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "logic.quad.and_or",
        release_state: ReleaseState::CurrentMainExperimental,
        stages: EXECUTABLE,
        probe: "fn main() {\n    let a: quad = T;\n    let b: quad = F;\n    assert((a && b) == N);\n    assert((a || b) == S);\n    return;\n}\n",
        negative_probe: Some((
            "fn main() {\n    let a: quad = T;\n    let c: quad = a && true;\n    let _ = c;\n    return;\n}\n",
            Stage::Typecheck,
        )),
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "logic.quad.implicit_bool_condition",
        release_state: ReleaseState::Unsupported,
        stages: TYPECHECK_REJECTED,
        probe: "fn main() {\n    let q: quad = T;\n    if q {\n        return;\n    } else {\n        return;\n    }\n}\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "operator.coalesce",
        release_state: ReleaseState::Unsupported,
        stages: LEX_REJECTED,
        probe: "fn main() {\n    let o: Option(i32) = Option::Some(1);\n    let v: i32 = o ?? 0;\n    let _ = v;\n    return;\n}\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "operator.try",
        release_state: ReleaseState::Unsupported,
        stages: LEX_REJECTED,
        probe: "fn f() -> Result(i32, i32) {\n    let r: Result(i32, i32) = Result::Ok(1);\n    let v: i32 = r?;\n    return Result::Ok(v);\n}\nfn main() {\n    return;\n}\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
    LanguageFeature {
        id: "surface.logos.declarative",
        release_state: ReleaseState::CurrentMainExperimental,
        stages: LOWER_REJECTED,
        probe: "Entity Sensor:\n    state val: quad\n\nLaw \"CheckSignal\" [priority 10]:\n    When Sensor.val == T -> Log.emit(\"Signal OK\")\n",
        negative_probe: None,
        evidence_suite: EVIDENCE,
    },
];
