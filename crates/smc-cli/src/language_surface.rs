//! S2043 CP06: `smc language describe --format json`, the deterministic
//! projection of `sm_front::language_surface` (the one canonical registry)
//! into the `semantic.language-surface` v1 document
//! (`docs/spec/language_surface_v1.md`). This module owns only the wire
//! shape; it states no feature status of its own.

use serde::Serialize;
use sm_front::language_surface::{
    LanguageFeature, ReleaseState, SourceSurface, Stage, StageStatus, LANGUAGE_FEATURES,
    LANGUAGE_SURFACE_SCHEMA_ID, LANGUAGE_SURFACE_SCHEMA_VERSION,
};

const USAGE: &str = "usage: smc language describe --format json";

#[derive(Serialize)]
struct Document {
    schema_id: &'static str,
    schema_version: u32,
    compiler_identity: CompilerIdentity,
    profile: Profile,
    features: Vec<Feature>,
    semcode: Semcode,
    runtime: Runtime,
}

#[derive(Serialize)]
struct CompilerIdentity {
    /// No trustworthy Git provenance is compiled in: always `null`.
    source_commit: Option<&'static str>,
    /// The non-cryptographic build fingerprint of the compiler sources
    /// (`SM_COMPILER_SOURCE_HASH`), never presented as a commit or digest.
    source_fingerprint: Option<&'static str>,
    build_id: Option<&'static str>,
}

#[derive(Serialize)]
struct Profile {
    source: String,
    compilation: &'static str,
}

#[derive(Serialize)]
struct Feature {
    id: &'static str,
    release_state: &'static str,
    stage_status: StageStatusJson,
    evidence: Evidence,
}

#[derive(Serialize)]
struct StageStatusJson {
    lex: &'static str,
    parse: &'static str,
    typecheck: &'static str,
    lower: &'static str,
    emit: &'static str,
    verify: &'static str,
    run: &'static str,
}

#[derive(Serialize)]
struct Evidence {
    suite: &'static str,
}

#[derive(Serialize)]
struct Semcode {
    floor: &'static str,
    promotions: Vec<&'static str>,
}

#[derive(Serialize)]
struct Runtime {
    /// Describing the language runs nothing, so no envelope applies.
    envelope: Option<&'static str>,
    /// Introspection never grants a capability.
    capability_grants: Option<&'static str>,
}

/// The Cargo build feature (as reported in `SM_ENABLED_FEATURES`) that must
/// be compiled in for a surface's registry statuses to describe this binary.
fn required_build_feature(surface: SourceSurface) -> &'static str {
    match surface {
        SourceSurface::RustLike => "PROFILE_RUST",
        SourceSurface::Logos => "PROFILE_LOGOS",
    }
}

/// Projects one registry entry for a build with `enabled_features`. A
/// feature whose surface is not compiled into this binary is reported fail
/// closed (`unknown` / `not-qualified`): its registry statuses were
/// qualified on a build that has it.
fn feature(entry: &LanguageFeature, enabled_features: &str) -> Feature {
    let built = enabled_features
        .split(',')
        .map(str::trim)
        .any(|f| f == required_build_feature(entry.surface));
    let status = |stage| {
        if built {
            entry.stage(stage).as_str()
        } else {
            StageStatus::NotQualified.as_str()
        }
    };
    Feature {
        id: entry.id,
        release_state: if built {
            entry.release_state.as_str()
        } else {
            ReleaseState::Unknown.as_str()
        },
        stage_status: StageStatusJson {
            lex: status(Stage::Lex),
            parse: status(Stage::Parse),
            typecheck: status(Stage::Typecheck),
            lower: status(Stage::Lower),
            emit: status(Stage::Emit),
            verify: status(Stage::Verify),
            run: status(Stage::Run),
        },
        evidence: Evidence {
            suite: entry.evidence_suite,
        },
    }
}

fn magic(bytes: &'static [u8; 8]) -> &'static str {
    core::str::from_utf8(bytes).unwrap_or("")
}

/// The complete `semantic.language-surface` v1 document, with a trailing
/// newline. Identical for every call of the same build.
pub(crate) fn describe_json() -> Result<String, String> {
    // The registry is the one ordering authority (sorted by id, enforced by
    // tests/s2043_language_surface.rs); the document keeps its order.
    let enabled = env!("SM_ENABLED_FEATURES");
    let features: Vec<Feature> = LANGUAGE_FEATURES
        .iter()
        .map(|entry| feature(entry, enabled))
        .collect();
    let document = Document {
        schema_id: LANGUAGE_SURFACE_SCHEMA_ID,
        schema_version: LANGUAGE_SURFACE_SCHEMA_VERSION,
        compiler_identity: CompilerIdentity {
            source_commit: None,
            source_fingerprint: Some(env!("SM_COMPILER_SOURCE_HASH")),
            build_id: None,
        },
        profile: Profile {
            source: crate::app::cli_profile().identity,
            compilation: "O0+O1",
        },
        features,
        semcode: Semcode {
            floor: crate::compatibility::CANONICAL_SEMCODE_FORMAT,
            promotions: vec![
                magic(&sm_format::semcode_format::MAGIC23),
                magic(&sm_format::semcode_format::MAGIC24),
                magic(&sm_format::semcode_format::MAGIC25),
            ],
        },
        runtime: Runtime {
            envelope: None,
            capability_grants: None,
        },
    };
    let mut text = serde_json::to_string_pretty(&document).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text)
}

/// `smc language describe --format json`. Any other argument list is a
/// deterministic usage error; nothing is written to stdout in that case.
pub(crate) fn cmd_language(args: &[String]) -> Result<(), String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["describe", "--format", "json"] => {
            print!("{}", describe_json()?);
            Ok(())
        }
        ["describe", "--format", other] => Err(format!(
            "unsupported language describe format '{other}'\n{USAGE}"
        )),
        _ => Err(USAGE.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn features_of_a_surface_missing_from_the_build_fail_closed() {
        let logos = LANGUAGE_FEATURES
            .iter()
            .find(|f| f.surface == SourceSurface::Logos)
            .expect("a Logos entry");
        let rust = LANGUAGE_FEATURES
            .iter()
            .find(|f| f.surface == SourceSurface::RustLike)
            .expect("a RustLike entry");

        let without_logos = "STD, PROFILE_RUST";
        let doc = feature(logos, without_logos);
        assert_eq!(doc.release_state, "unknown");
        assert_eq!(doc.stage_status.lex, "not-qualified");
        assert_eq!(doc.stage_status.typecheck, "not-qualified");
        assert_eq!(doc.stage_status.lower, "not-qualified");
        assert_eq!(
            feature(rust, without_logos).release_state,
            rust.release_state.as_str()
        );

        let without_rust = "STD,PROFILE_LOGOS";
        let doc = feature(rust, without_rust);
        assert_eq!(doc.release_state, "unknown");
        assert_eq!(doc.stage_status.run, "not-qualified");

        let full = "DEBUG_SYMBOLS,PROFILE_LOGOS,PROFILE_RUST,STD";
        assert_eq!(feature(logos, full).stage_status.parse, "qualified");
        assert_eq!(
            feature(rust, full).stage_status.run,
            rust.stage(Stage::Run).as_str()
        );
    }
}
