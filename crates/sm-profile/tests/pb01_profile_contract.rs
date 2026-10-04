//! PB-01 (#1618-#1632) `sm-profile` contract regressions. Training/line
//! normalization regressions live with their TON618 owner
//! (TON618 compatibility binary, `alias_compat::tests`); canonical-admission
//! regressions live in `sm-front`.

use sm_profile::*;

fn foundation_json() -> serde_json::Value {
    serde_json::from_str(&ParserProfile::foundation_default().to_json().unwrap()).unwrap()
}

fn load(v: &serde_json::Value) -> Result<ParserProfile, ProfileIoError> {
    ParserProfile::from_json(&v.to_string())
}

fn invalid(r: Result<ParserProfile, ProfileIoError>) -> ProfileError {
    match r {
        Err(ProfileIoError::Invalid(e)) => e,
        other => panic!("expected semantic rejection, got {other:?}"),
    }
}

fn is_json_err(r: Result<ParserProfile, ProfileIoError>) -> bool {
    matches!(r, Err(ProfileIoError::Json(_)))
}

// #1618
#[test]
fn default_is_the_single_foundation_baseline() {
    let d = ParserProfile::default();
    assert_eq!(d, ParserProfile::foundation_default());
    assert_eq!(d.identity, "semantic.foundation");
    assert_eq!(d.version, ProfileVersion::new(1, 0));
    assert_eq!(d.abi, AbiProfile::GateSurface);
    assert_eq!(d.compatibility, CompatibilityMode::LegacySupport);
    assert_eq!(d.features, FeaturePolicy::foundation());
    assert!(d.validate_for_canonical_source().is_ok());

    let core = ParserProfile::core();
    assert_eq!(core.identity, "semantic.core");
    assert_eq!(core.compatibility, CompatibilityMode::Strict);
    assert!(!core.features.allow_f64_math);
    assert!(!core.features.allow_logos_surface);
    assert!(!core.features.allow_schema_surface);
    assert!(core.validate_for_canonical_source().is_ok());
}

// #1632 / #1629
#[test]
fn add_alias_has_one_conflict_rule_and_one_validity_rule() {
    let mut p = ParserProfile::core();
    p.add_alias("AND", "&").unwrap();
    p.add_alias("AND", "&").unwrap(); // idempotent
    assert_eq!(
        p.add_alias("AND", "|"),
        Err(AliasError::Conflict {
            raw: "AND".into(),
            existing: "&".into(),
            proposed: "|".into()
        })
    );
    assert_eq!(p.aliases.get("AND").map(String::as_str), Some("&"));
    for raw in ["", "rm -rf", "a.b", "T", "&"] {
        assert!(
            matches!(p.add_alias(raw, "T"), Err(AliasError::InvalidRaw { .. })),
            "{raw:?}"
        );
    }
    for target in ["", "rm -rf", "&&", "true", "x"] {
        assert!(
            matches!(
                p.add_alias("X", target),
                Err(AliasError::InvalidCanonical { .. })
            ),
            "{target:?}"
        );
    }
    assert_eq!(p.aliases.len(), 1);
}

// #1623
#[test]
fn json_version_admission_is_exactly_1_0() {
    assert!(load(&foundation_json()).is_ok());
    for (major, minor) in [(2, 0), (0, 9), (99, 7), (1, 1)] {
        let mut v = foundation_json();
        v["version"] = serde_json::json!({"major": major, "minor": minor});
        assert_eq!(
            invalid(load(&v)),
            ProfileError::UnsupportedVersion(ProfileVersion::new(major, minor))
        );
    }
}

// #1631
#[test]
fn json_unknown_fields_fail_closed_at_every_level() {
    for path in [
        &[][..],
        &["features"][..],
        &["capabilities"][..],
        &["version"][..],
    ] {
        let mut v = foundation_json();
        let mut obj = &mut v;
        for k in path {
            obj = &mut obj[*k];
        }
        obj["bogus"] = serde_json::json!(true);
        assert!(is_json_err(load(&v)), "unknown field under {path:?}");
    }
}

// #1629 / #1620 via JSON
#[test]
fn json_aliases_share_the_alias_contract() {
    let mut v = foundation_json();
    v["aliases"] = serde_json::json!({"main": "!!!"});
    assert!(matches!(
        invalid(load(&v)),
        ProfileError::Alias(AliasError::InvalidCanonical { .. })
    ));

    let dup = foundation_json()
        .to_string()
        .replace("\"aliases\":{}", "\"aliases\":{\"A\":\"T\",\"A\":\"F\"}");
    assert!(dup.contains("\"A\":\"F\""));
    assert!(matches!(
        ParserProfile::from_json(&dup),
        Err(ProfileIoError::Json(_))
    ));

    let mut v = foundation_json();
    v["aliases"] = serde_json::json!({"AND": "&"});
    assert_eq!(
        load(&v).unwrap().aliases.get("AND").map(String::as_str),
        Some("&")
    );
}

#[test]
fn file_load_uses_the_same_admission_as_json() {
    let dir = std::env::temp_dir().join(format!("pb01_profile_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("p.json");
    let mut v = foundation_json();
    v["version"]["major"] = serde_json::json!(2);
    std::fs::write(&path, v.to_string()).unwrap();
    assert!(matches!(
        ParserProfile::load_from_file(&path),
        Err(ProfileIoError::Invalid(ProfileError::UnsupportedVersion(_)))
    ));
    let mut ok = ParserProfile::core();
    ok.add_alias("AND", "&").unwrap();
    ok.save_to_file(&path).unwrap();
    assert_eq!(ParserProfile::load_from_file(&path).unwrap(), ok);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn bypassing_add_alias_is_caught_at_serialization() {
    let mut p = ParserProfile::core();
    p.aliases.insert("fn".into(), "rm -rf".into());
    assert!(matches!(p.to_json(), Err(ProfileIoError::Invalid(_))));
}

// #1624-#1628: reserved fields and aliases are rejected at canonical admission
#[test]
fn canonical_admission_rejects_reserved_policy_and_aliases() {
    let base = ParserProfile::foundation_default;
    let cases: [(ParserProfile, ProfileError); 7] = [
        (
            ParserProfile {
                abi: AbiProfile::Core,
                ..base()
            },
            ProfileError::ReservedField("abi"),
        ),
        (
            ParserProfile {
                features: FeaturePolicy {
                    allow_debug_symbols: false,
                    ..FeaturePolicy::foundation()
                },
                ..base()
            },
            ProfileError::ReservedField("features.allow_debug_symbols"),
        ),
        (
            ParserProfile {
                features: FeaturePolicy {
                    allow_gate_surface: false,
                    ..FeaturePolicy::foundation()
                },
                ..base()
            },
            ProfileError::ReservedField("features.allow_gate_surface"),
        ),
        (
            ParserProfile {
                capabilities: CapabilityExpectations {
                    require_debug_symbols: true,
                    ..CapabilityExpectations::permissive()
                },
                ..base()
            },
            ProfileError::ReservedField("capabilities.require_debug_symbols"),
        ),
        (
            ParserProfile {
                capabilities: CapabilityExpectations {
                    require_f64_math: true,
                    ..CapabilityExpectations::permissive()
                },
                ..base()
            },
            ProfileError::ReservedField("capabilities.require_f64_math"),
        ),
        (
            ParserProfile {
                capabilities: CapabilityExpectations {
                    require_gate_surface: true,
                    ..CapabilityExpectations::permissive()
                },
                ..base()
            },
            ProfileError::ReservedField("capabilities.require_gate_surface"),
        ),
        (
            {
                let mut p = base();
                p.add_alias("TRUE", "T").unwrap();
                p
            },
            ProfileError::AliasesNotCanonical,
        ),
    ];
    for (profile, expected) in cases {
        assert!(profile.validate().is_ok(), "structurally valid");
        assert_eq!(profile.validate_for_canonical_source(), Err(expected));
    }
}
