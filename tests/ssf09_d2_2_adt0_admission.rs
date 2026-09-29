//! SSF-09 D2-2: SemCode V22 activation, strict ADT0 decoding and
//! descriptor-aware verifier admission (Contract 84217).
//!
//! The artifact's ADT0 table is the unique ADT authority: the decoder reads
//! it exactly as encoded and only validates it (it never injects, drops,
//! sorts or repairs a descriptor), and the verifier checks every ADT opcode
//! against that decoded table, resolving types by canonical name.

use sm_format::semcode_decode::{decode_semcode, decode_semcode_envelope, DecodeError};
use sm_format::semcode_format::{
    AdtDescriptor, AdtDescriptorTable, AdtVariantDescriptor, HEADER_V21, HEADER_V22, MAGIC21,
    MAGIC22,
};
use sm_front::parse_program;
use sm_ir::{
    adt_descriptor_table, compile_program_to_semcode, emit_ir_to_semcode_with_adt_descriptors,
    IrFunction, IrInstr,
};
use sm_verify::{verify_semcode, VerificationCode};
use sm_vm::run_verified_semcode;

const MAIN: &str = "fn main() {\n    return;\n}\n";

type RawDescriptor<'a> = (&'a [u8], &'a [(&'a [u8], u16)]);

const OPTION: RawDescriptor = (b"Option", &[(b"None", 0), (b"Some", 1)]);
const RESULT: RawDescriptor = (b"Result", &[(b"Ok", 1), (b"Err", 1)]);

fn push_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// An ADT0 section written byte for byte, so malformed tables can be built.
fn raw_section(descriptors: &[RawDescriptor]) -> Vec<u8> {
    let mut out = b"ADT0".to_vec();
    push_u16(&mut out, descriptors.len() as u16);
    for (name, variants) in descriptors {
        push_u16(&mut out, name.len() as u16);
        out.extend_from_slice(name);
        push_u16(&mut out, variants.len() as u16);
        for (variant, arity) in *variants {
            push_u16(&mut out, variant.len() as u16);
            out.extend_from_slice(variant);
            push_u16(&mut out, *arity);
        }
    }
    out
}

/// Length of the ADT0 section at the front of a compiled V22 artifact.
fn section_len(bytes: &[u8]) -> usize {
    let table = decode_semcode(bytes)
        .expect("decode")
        .adt_descriptors
        .expect("V22 artifact carries ADT0");
    table.encode_section().len()
}

/// The function envelopes of a compiled artifact (everything after ADT0).
fn functions_of(bytes: &[u8]) -> Vec<u8> {
    bytes[8 + section_len(bytes)..].to_vec()
}

fn main_functions() -> Vec<u8> {
    functions_of(&compile_program_to_semcode(MAIN).expect("compile"))
}

/// A V22 artifact with the given raw ADT0 bytes and ordinary functions.
fn v22_with_section(section: &[u8]) -> Vec<u8> {
    [&MAGIC22[..], section, &main_functions()].concat()
}

/// The legacy (V21) wire form of a compiled V22 artifact: no ADT0 section.
fn as_v21(bytes: &[u8]) -> Vec<u8> {
    [&MAGIC21[..], &functions_of(bytes)].concat()
}

fn decode_error(bytes: &[u8]) -> (usize, &'static str) {
    match decode_semcode(bytes) {
        Err(DecodeError::InvalidAdtDescriptorSection { offset, msg }) => (offset, msg),
        other => panic!("expected InvalidAdtDescriptorSection, got {other:?}"),
    }
}

fn verify_code(bytes: &[u8]) -> VerificationCode {
    let report = verify_semcode(bytes).expect_err("verifier must reject");
    report.diagnostics[0].code
}

fn rejected(section: &[u8], msg: &str) {
    let bytes = v22_with_section(section);
    assert_eq!(decode_error(&bytes).1, msg);
    assert_eq!(
        verify_code(&bytes),
        VerificationCode::InvalidAdtDescriptorSection
    );
}

fn adt(name: &str, variants: &[(&str, u16)]) -> AdtDescriptor {
    AdtDescriptor::new(
        name,
        variants
            .iter()
            .map(|(n, a)| AdtVariantDescriptor::new(n, *a).expect("variant"))
            .collect(),
    )
    .expect("descriptor")
}

// ---- positive decoder tests ------------------------------------------------

#[test]
fn p1_builtins_only_v22_artifact_is_admitted() {
    let bytes = compile_program_to_semcode(MAIN).expect("compile");
    let decoded = decode_semcode(&bytes).expect("decode");
    assert_eq!(decoded.header, HEADER_V22);
    assert_eq!(
        decoded.adt_descriptors,
        Some(AdtDescriptorTable::with_builtins(vec![]).expect("table"))
    );
    verify_semcode(&bytes).expect("admitted");
    run_verified_semcode(&bytes).expect("runs");
}

#[test]
fn p2_user_enum_v22_artifact_is_admitted_and_runs() {
    let src = "enum E {\n    A,\n    B(i32),\n}\n\
         fn pick(e: E) -> i32 {\n    match e {\n        E::A => {\n            return 0;\n        }\n        \
         E::B(x) => {\n            return x;\n        }\n    }\n    return 9;\n}\n\
         fn main() {\n    assert(pick(E::B(4)) == 4);\n    assert(pick(E::A) == 0);\n    return;\n}\n";
    let bytes = compile_program_to_semcode(src).expect("compile");
    verify_semcode(&bytes).expect("admitted");
    run_verified_semcode(&bytes).expect("runs");
}

#[test]
fn p3_declaration_order_does_not_change_the_artifact() {
    let z = "enum Z {\n    Z0,\n}\n";
    let a = "enum A {\n    A0(i32),\n}\n";
    let za = compile_program_to_semcode(&format!("{z}{a}{MAIN}")).expect("compile");
    let az = compile_program_to_semcode(&format!("{a}{z}{MAIN}")).expect("compile");
    assert_eq!(za, az);
    let names: Vec<String> = decode_semcode(&za)
        .expect("decode")
        .adt_descriptors
        .expect("table")
        .descriptors()
        .iter()
        .map(|d| d.name().to_string())
        .collect();
    assert_eq!(names, ["A", "Option", "Result", "Z"]);
}

#[test]
fn p4_non_ascii_names_sort_by_raw_utf8_bytes() {
    // 0x5A (Z) < 0x62 (b) < 0xC3 0x89 (E-acute).
    let table = AdtDescriptorTable::with_builtins(vec![
        adt("\u{c9}mile", &[("X", 0)]),
        adt("b", &[]),
        adt("Z", &[]),
    ])
    .expect("table");
    let bytes = v22_with_section(&table.encode_section());
    assert_eq!(
        decode_semcode(&bytes).expect("decode").adt_descriptors,
        Some(table)
    );
    // Locale order (b before Z) is not canonical.
    let locale = raw_section(&[
        OPTION,
        RESULT,
        (b"b", &[]),
        (b"Z", &[]),
        ("\u{c9}mile".as_bytes(), &[(b"X", 0)]),
    ]);
    rejected(
        &locale,
        "ADT0 descriptors are not in strictly ascending name-byte order",
    );
}

#[test]
fn p5_p6_exact_canonical_builtins_are_accepted() {
    let bytes = v22_with_section(&raw_section(&[OPTION, RESULT]));
    let table = decode_semcode(&bytes)
        .expect("decode")
        .adt_descriptors
        .expect("table");
    let shape = |name: &str| -> Vec<(String, u16)> {
        table
            .get(name)
            .expect("builtin")
            .variants()
            .iter()
            .map(|v| (v.name().to_string(), v.payload_arity()))
            .collect()
    };
    assert_eq!(shape("Option"), [("None".into(), 0), ("Some".into(), 1)]);
    assert_eq!(shape("Result"), [("Ok".into(), 1), ("Err".into(), 1)]);
    verify_semcode(&bytes).expect("admitted");
}

#[test]
fn p7_legacy_v21_artifacts_keep_their_legacy_decoding() {
    let legacy = as_v21(&compile_program_to_semcode(MAIN).expect("compile"));
    let decoded = decode_semcode(&legacy).expect("legacy decode");
    assert_eq!(decoded.header, HEADER_V21);
    // No descriptor table is synthesized for a legacy artifact.
    assert_eq!(decoded.adt_descriptors, None);
    verify_semcode(&legacy).expect("legacy non-ADT artifact still admitted");
    run_verified_semcode(&legacy).expect("legacy non-ADT artifact still runs");
}

#[test]
fn p8_decode_returns_the_artifacts_own_descriptor_table() {
    let src = format!("enum E {{\n    A,\n    B(i32, bool),\n}}\n{MAIN}");
    let expected = adt_descriptor_table(&parse_program(&src).expect("parse")).expect("table");
    let bytes = compile_program_to_semcode(&src).expect("compile");
    assert_eq!(
        decode_semcode(&bytes).expect("decode").adt_descriptors,
        Some(expected)
    );
}

#[test]
fn p9_encode_decode_roundtrip_is_exact() {
    let table = AdtDescriptorTable::with_builtins(vec![
        adt("E", &[("A", 0), ("B", 3)]),
        adt("W", &[("Max", u16::MAX)]),
    ])
    .expect("table");
    let bytes = [&MAGIC22[..], &table.encode_section()].concat();
    let decoded = decode_semcode(&bytes).expect("decode");
    assert_eq!(decoded.adt_descriptors, Some(table));
    assert!(decoded.functions.is_empty());
}

#[test]
fn p10_compiler_output_uses_semcod22() {
    let bytes = compile_program_to_semcode(MAIN).expect("compile");
    assert_eq!(bytes[..8], MAGIC22);
    // The compatibility projection delegates to the canonical decoder.
    let (header, functions) = decode_semcode_envelope(&bytes).expect("decode");
    let canonical = decode_semcode(&bytes).expect("decode");
    assert_eq!(header, canonical.header);
    assert_eq!(functions, canonical.functions);
}

// ---- negative decoder tests ------------------------------------------------

#[test]
fn n1_v22_without_adt0_is_rejected() {
    let bytes = [&MAGIC22[..], &main_functions()].concat();
    assert_eq!(decode_error(&bytes), (8, "missing ADT0 descriptor section"));
    assert_eq!(
        verify_code(&bytes),
        VerificationCode::InvalidAdtDescriptorSection
    );
    assert_eq!(
        decode_error(&MAGIC22).1,
        "missing ADT0 descriptor section",
        "header only"
    );
}

#[test]
fn n2_n3_n18_missing_builtins_are_rejected_not_repaired() {
    rejected(
        &raw_section(&[RESULT]),
        "ADT0 is missing the built-in Option descriptor",
    );
    rejected(
        &raw_section(&[OPTION]),
        "ADT0 is missing the built-in Result descriptor",
    );
    // N18: a table with only a user enum must not be completed with the
    // built-ins (that would be the with_builtins repair).
    rejected(
        &raw_section(&[(b"E", &[(b"A", 0)])]),
        "ADT0 is missing the built-in Option descriptor",
    );
    rejected(
        &raw_section(&[]),
        "ADT0 is missing the built-in Option descriptor",
    );
}

#[test]
fn n4_n5_n6_non_canonical_option_is_rejected() {
    let msg = "ADT0 Option descriptor is not the canonical [None/0, Some/1]";
    let options: [RawDescriptor; 4] = [
        (b"Option", &[(b"None", 0), (b"Some", 2)]),
        (b"Option", &[(b"Some", 1), (b"None", 0)]),
        (b"Option", &[(b"None", 0), (b"Just", 1)]),
        (b"Option", &[(b"None", 0), (b"Some", 1), (b"Extra", 0)]),
    ];
    for option in options {
        rejected(&raw_section(&[option, RESULT]), msg);
    }
}

#[test]
fn n7_n8_non_canonical_result_is_rejected() {
    let msg = "ADT0 Result descriptor is not the canonical [Ok/1, Err/1]";
    let results: [RawDescriptor; 2] = [
        (b"Result", &[(b"Ok", 1), (b"Err", 0)]),
        (b"Result", &[(b"Err", 1), (b"Ok", 1)]),
    ];
    for result in results {
        rejected(&raw_section(&[OPTION, result]), msg);
    }
}

#[test]
fn n9_n10_order_and_duplicates_are_rejected_without_sorting() {
    rejected(
        &raw_section(&[RESULT, OPTION]),
        "ADT0 descriptors are not in strictly ascending name-byte order",
    );
    rejected(
        &raw_section(&[OPTION, OPTION, RESULT]),
        "ADT0 repeats a descriptor name",
    );
}

#[test]
fn n11_duplicate_variant_name_is_rejected() {
    rejected(
        &raw_section(&[(b"E", &[(b"A", 0), (b"A", 1)]), OPTION, RESULT]),
        "ADT0 descriptor repeats a variant name",
    );
}

#[test]
fn n12_n13_invalid_utf8_names_are_rejected() {
    rejected(
        &raw_section(&[(b"\xff", &[]), OPTION, RESULT]),
        "invalid utf8 in ADT0 descriptor name",
    );
    rejected(
        &raw_section(&[(b"E", &[(b"\xc3", 0)]), OPTION, RESULT]),
        "invalid utf8 in ADT0 variant name",
    );
}

#[test]
fn empty_names_are_rejected() {
    rejected(
        &raw_section(&[(b"", &[]), OPTION, RESULT]),
        "empty ADT0 descriptor name",
    );
    rejected(
        &raw_section(&[(b"E", &[(b"", 0)]), OPTION, RESULT]),
        "empty ADT0 variant name",
    );
}

#[test]
fn n14_n15_n16_truncation_and_impossible_bounds_are_rejected() {
    let full = raw_section(&[(b"Enum", &[(b"Variant", 1)]), OPTION, RESULT]);
    // Every strict prefix of the section is malformed (no functions follow).
    for cut in 0..full.len() {
        let bytes = [&MAGIC22[..], &full[..cut]].concat();
        assert!(
            matches!(
                decode_semcode(&bytes),
                Err(DecodeError::InvalidAdtDescriptorSection { .. })
            ),
            "prefix of {cut} bytes must be rejected"
        );
    }
    // N14: a name length that runs past the input.
    let mut long_name = b"ADT0".to_vec();
    push_u16(&mut long_name, 1);
    push_u16(&mut long_name, u16::MAX);
    long_name.extend_from_slice(b"Option");
    assert_eq!(
        decode_error(&[&MAGIC22[..], &long_name].concat()).1,
        "truncated ADT0 name"
    );
    // N15: a variant record cut before its arity.
    let mut cut_variant = b"ADT0".to_vec();
    push_u16(&mut cut_variant, 1);
    push_u16(&mut cut_variant, 1);
    cut_variant.extend_from_slice(b"E");
    push_u16(&mut cut_variant, 1);
    push_u16(&mut cut_variant, 1);
    cut_variant.extend_from_slice(b"A");
    assert_eq!(
        decode_error(&[&MAGIC22[..], &cut_variant].concat()).1,
        "truncated ADT0 variant payload arity"
    );
    // N16: a descriptor count far beyond the bytes present.
    let mut huge_count = b"ADT0".to_vec();
    push_u16(&mut huge_count, u16::MAX);
    assert_eq!(
        decode_error(&[&MAGIC22[..], &huge_count].concat()).1,
        "truncated ADT0 descriptor name length"
    );
}

#[test]
fn n17_duplicate_adt0_section_is_rejected() {
    let section = raw_section(&[OPTION, RESULT]);
    let bytes = [&MAGIC22[..], &section, &section, &main_functions()].concat();
    assert_eq!(
        decode_error(&bytes),
        (8 + section.len(), "duplicate ADT0 descriptor section")
    );
    assert_eq!(
        verify_code(&bytes),
        VerificationCode::InvalidAdtDescriptorSection
    );
}

// ---- verifier admission ----------------------------------------------------

/// A one-function artifact: `main` runs `body` then returns.
fn artifact(table: &AdtDescriptorTable, body: Vec<IrInstr>) -> Vec<u8> {
    let mut instrs = body;
    instrs.push(IrInstr::Ret { src: None });
    let main = IrFunction {
        name: "main".to_string(),
        instrs,
        ownership_events: vec![],
        params: vec![],
    };
    emit_ir_to_semcode_with_adt_descriptors(&[main], table, false).expect("emit")
}

fn e_table() -> AdtDescriptorTable {
    AdtDescriptorTable::with_builtins(vec![adt("E", &[("A", 0), ("B", 1)])]).expect("table")
}

fn make_adt(adt_name: &str, variant_name: &str, tag: u16, items: Vec<u16>) -> Vec<IrInstr> {
    vec![
        IrInstr::LoadI32 { dst: 0, val: 7 },
        IrInstr::MakeAdt {
            dst: 1,
            adt_name: adt_name.to_string(),
            variant_name: variant_name.to_string(),
            tag,
            items,
        },
    ]
}

#[test]
fn valid_make_adt_against_the_table_is_admitted() {
    let bytes = artifact(&e_table(), make_adt("E", "B", 1, vec![0]));
    verify_semcode(&bytes).expect("admitted");
    run_verified_semcode(&bytes).expect("runs");
}

#[test]
fn v1_unknown_adt_type_is_rejected() {
    let bytes = artifact(&e_table(), make_adt("F", "B", 1, vec![0]));
    assert_eq!(verify_code(&bytes), VerificationCode::UnknownAdtType);
    let mut tag = make_adt("E", "B", 1, vec![0]);
    tag.push(IrInstr::AdtTag {
        dst: 2,
        src: 1,
        adt_name: "G".to_string(),
    });
    assert_eq!(
        verify_code(&artifact(&e_table(), tag)),
        VerificationCode::UnknownAdtType
    );
}

#[test]
fn v2_out_of_range_discriminant_is_rejected() {
    let bytes = artifact(&e_table(), make_adt("E", "B", 2, vec![0]));
    assert_eq!(
        verify_code(&bytes),
        VerificationCode::InvalidAdtDiscriminant
    );
}

#[test]
fn v3_variant_name_mismatch_is_rejected() {
    let bytes = artifact(&e_table(), make_adt("E", "A", 1, vec![0]));
    assert_eq!(
        verify_code(&bytes),
        VerificationCode::AdtVariantNameMismatch
    );
}

#[test]
fn v4_payload_arity_mismatch_is_rejected() {
    let bytes = artifact(&e_table(), make_adt("E", "B", 1, vec![0, 0]));
    assert_eq!(
        verify_code(&bytes),
        VerificationCode::AdtPayloadArityMismatch
    );
    let bytes = artifact(&e_table(), make_adt("E", "A", 0, vec![0]));
    assert_eq!(
        verify_code(&bytes),
        VerificationCode::AdtPayloadArityMismatch
    );
}

#[test]
fn v5_payload_index_out_of_range_is_rejected() {
    let get = |index| {
        let mut body = make_adt("E", "B", 1, vec![0]);
        body.push(IrInstr::AdtGet {
            dst: 2,
            src: 1,
            adt_name: "E".to_string(),
            index,
        });
        artifact(&e_table(), body)
    };
    verify_semcode(&get(0)).expect("index below the largest arity is admitted");
    assert_eq!(
        verify_code(&get(1)),
        VerificationCode::AdtPayloadIndexOutOfRange
    );
}

#[test]
fn v6_adt_opcode_under_a_legacy_header_is_rejected() {
    let bytes = artifact(&e_table(), make_adt("E", "B", 1, vec![0]));
    verify_semcode(&bytes).expect("V22 form admitted");
    let legacy = as_v21(&bytes);
    assert_eq!(
        decode_semcode(&legacy).expect("decode").adt_descriptors,
        None
    );
    assert_eq!(
        verify_code(&legacy),
        VerificationCode::AdtRequiresDescriptorHeader
    );
}

#[test]
fn v7_types_resolve_by_name_not_by_table_position() {
    // Z sits at position 3; position 0 is A, which has no variants. Any
    // position-based resolution (e.g. by string id) picks the wrong type.
    let table = AdtDescriptorTable::with_builtins(vec![adt("Z", &[("Z0", 1)]), adt("A", &[])])
        .expect("table");
    assert_eq!(
        table.descriptors().iter().position(|d| d.name() == "Z"),
        Some(3)
    );
    let bytes = artifact(&table, make_adt("Z", "Z0", 0, vec![0]));
    verify_semcode(&bytes).expect("resolved by name");
    run_verified_semcode(&bytes).expect("runs");
}

#[test]
fn v8_illegally_reordered_table_is_rejected_before_verification() {
    let bytes = artifact(&e_table(), make_adt("E", "B", 1, vec![0]));
    let functions = functions_of(&bytes);
    let reordered = raw_section(&[OPTION, (b"E", &[(b"A", 0), (b"B", 1)]), RESULT]);
    let bad = [&MAGIC22[..], &reordered, &functions].concat();
    assert_eq!(
        decode_error(&bad).1,
        "ADT0 descriptors are not in strictly ascending name-byte order"
    );
    assert_eq!(
        verify_code(&bad),
        VerificationCode::InvalidAdtDescriptorSection
    );
}

// ---- legacy no-sniff law ----------------------------------------------------

#[test]
fn legacy_v21_never_sniffs_an_adt0_section() {
    // A structurally valid canonical ADT0 section after a V21 header is legacy
    // function bytes, never a descriptor table: "AD" is read as a function
    // name length (0x4441) and the legacy grammar rejects it.
    let section = raw_section(&[OPTION, RESULT]);
    let legacy = [&MAGIC21[..], &section, &main_functions()].concat();
    match decode_semcode(&legacy) {
        Err(DecodeError::InvalidFunctionName {
            offset: 10,
            msg: "function name too long",
        }) => {}
        other => panic!("V21 bytes must be read by the legacy grammar, got {other:?}"),
    }
    assert_eq!(verify_code(&legacy), VerificationCode::InvalidFunctionName);
    // Byte for byte the same body under V22 is a valid artifact, so only the
    // header revision decides whether ADT0 exists.
    let v22 = [&MAGIC22[..], &section, &main_functions()].concat();
    assert!(decode_semcode(&v22).expect("V22").adt_descriptors.is_some());
    verify_semcode(&v22).expect("V22 admitted");
}

// ---- IR emission descriptor authority ----------------------------------------

fn main_fn(body: Vec<IrInstr>) -> IrFunction {
    IrFunction {
        name: "main".to_string(),
        instrs: [body, vec![IrInstr::Ret { src: None }]].concat(),
        ownership_events: vec![],
        params: vec![],
    }
}

fn builtins_only() -> AdtDescriptorTable {
    AdtDescriptorTable::with_builtins(vec![]).expect("table")
}

#[test]
fn bare_ir_emission_without_adts_is_verifiable_v22() {
    let body = vec![IrInstr::LoadI32 { dst: 0, val: 7 }];
    let bytes = sm_ir::emit_ir_to_semcode(&[main_fn(body)], false).expect("emit");
    assert_eq!(bytes[..8], MAGIC22);
    assert_eq!(
        decode_semcode(&bytes).expect("decode").adt_descriptors,
        Some(builtins_only())
    );
    verify_semcode(&bytes).expect("admitted");
}

#[test]
fn bare_ir_emission_with_option_and_result_is_verifiable_v22() {
    let body = [
        make_adt("Option", "Some", 1, vec![0]),
        make_adt("Result", "Err", 1, vec![0])[1..].to_vec(),
        vec![
            IrInstr::AdtTag {
                dst: 2,
                src: 1,
                adt_name: "Result".to_string(),
            },
            IrInstr::AdtGet {
                dst: 3,
                src: 1,
                adt_name: "Result".to_string(),
                index: 0,
            },
        ],
    ]
    .concat();
    let bytes = sm_ir::emit_ir_to_semcode(&[main_fn(body)], false).expect("emit");
    assert_eq!(
        decode_semcode(&bytes).expect("decode").adt_descriptors,
        Some(builtins_only())
    );
    verify_semcode(&bytes).expect("admitted");
    run_verified_semcode(&bytes).expect("runs");
}

#[test]
fn bare_ir_emission_rejects_user_adts_without_descriptor_authority() {
    // Every ADT instruction form that names a type is gated.
    let bodies = [
        make_adt("E", "B", 1, vec![0]),
        vec![IrInstr::AdtTag {
            dst: 1,
            src: 0,
            adt_name: "E".to_string(),
        }],
        vec![IrInstr::AdtGet {
            dst: 1,
            src: 0,
            adt_name: "E".to_string(),
            index: 0,
        }],
    ];
    for body in bodies {
        let err = sm_ir::emit_ir_to_semcode(&[main_fn(body)], false)
            .expect_err("no artifact without descriptor authority");
        assert!(
            err.message.contains("ADT type 'E'")
                && err
                    .message
                    .contains("emit_ir_to_semcode_with_adt_descriptors"),
            "{}",
            err.message
        );
    }
}

#[test]
fn descriptor_aware_ir_emission_admits_user_adts() {
    // The same IR the bare emitter refuses, through the public facade.
    let body = make_adt("E", "B", 1, vec![0]);
    let bytes = semantic_language::frontend::emit::emit_ir_to_semcode_with_adt_descriptors(
        &[main_fn(body)],
        &e_table(),
        false,
    )
    .expect("emit");
    assert_eq!(
        decode_semcode(&bytes).expect("decode").adt_descriptors,
        Some(e_table())
    );
    verify_semcode(&bytes).expect("admitted");
    run_verified_semcode(&bytes).expect("runs");
}

#[test]
fn facade_consumers_can_name_and_match_descriptor_errors() {
    use semantic_language::semcode_format::{
        AdtDescriptor, AdtDescriptorError, AdtDescriptorTable, AdtVariantDescriptor,
    };
    fn table(variant: &str) -> Result<AdtDescriptorTable, AdtDescriptorError> {
        let e = AdtDescriptor::new("E", vec![AdtVariantDescriptor::new(variant, 0)?])?;
        AdtDescriptorTable::with_builtins(vec![e])
    }
    assert!(table("A").is_ok());
    assert!(matches!(table(""), Err(AdtDescriptorError::EmptyName)));
}

// ---- documented compatibility migration --------------------------------------

#[test]
fn legacy_adt_admission_is_withdrawn_and_recompilation_is_the_migration() {
    // docs/spec/semcode.md "Backward Compatibility Rule": D2-2 withdraws
    // verifier admission of descriptor-dependent ADT opcodes under legacy
    // headers; non-ADT legacy artifacts keep their contract.
    let src = format!("enum E {{\n    A,\n    B(i32),\n}}\n{MAIN}")
        .replace("    return;", "    let e: E = E::B(1);\n    return;");
    let current = compile_program_to_semcode(&src).expect("compile");
    verify_semcode(&current).expect("recompiled V22 artifact is admitted");

    // The same program in its legacy wire form: decodable, never given a
    // descriptor table, and rejected at its first ADT opcode.
    let legacy = as_v21(&current);
    assert_eq!(
        decode_semcode(&legacy)
            .expect("still decodable")
            .adt_descriptors,
        None
    );
    assert_eq!(
        verify_code(&legacy),
        VerificationCode::AdtRequiresDescriptorHeader
    );

    // A legacy artifact without ADT opcodes is unaffected.
    verify_semcode(&as_v21(&compile_program_to_semcode(MAIN).expect("compile")))
        .expect("legacy non-ADT artifact still admitted");
}
