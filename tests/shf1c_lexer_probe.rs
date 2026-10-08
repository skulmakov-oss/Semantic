//! SHF-1C (#2010): qualification of the `compiler_text_v0.md` §9.4 lexer probe.
//!
//! The byte scan happens entirely in Semantic
//! (`tests/fixtures/shf1c_lexer_probe/probe.sm`). This harness only loads the
//! fixed source, compiles it through the public compiler at O0 and O1, checks
//! the header identity, admits it with `sm-verify`, runs the verified `probe`
//! entry in `sm-vm`, and compares the sentinel. It never scans, classifies or
//! decodes the sample itself; the expected counts are asserted inside the
//! Semantic program.

use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_format::semcode_format::MAGIC23;
use sm_verify::{verify_semcode, verify_semcode_token};
use sm_vm::{run_verified_function_semcode_with_args, Value};

const PROBE: &str = include_str!("fixtures/shf1c_lexer_probe/probe.sm");

fn compile(opt: OptLevel) -> Vec<u8> {
    compile_program_to_semcode_with_options(PROBE, CompileProfile::RustLike, opt)
        .unwrap_or_else(|e| panic!("probe must compile at {opt:?}: {e}"))
}

fn run(bytes: &[u8]) -> Value {
    let token = verify_semcode_token(bytes).expect("probe must be admitted by sm-verify");
    let entry = token.require_entry("probe").expect("probe entry");
    run_verified_function_semcode_with_args(&entry, vec![])
        .unwrap_or_else(|e| panic!("probe must execute without a trap: {e}"))
}

#[test]
fn lexer_probe_scans_the_sample_in_semantic_at_o0_and_o1() {
    let mut results = Vec::new();
    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes = compile(opt);
        // Plain-u32 arithmetic/ordering is consumed from SHF-3A: SEMCOD23.
        assert_eq!(&bytes[0..8], &MAGIC23, "{opt:?}: probe artifact header");
        verify_semcode(&bytes).unwrap_or_else(|e| panic!("{opt:?}: verify: {e}"));
        let result = run(&bytes);
        assert_eq!(
            format!("{result:?}"),
            format!("{:?}", Value::U32(1)),
            "{opt:?}: sentinel"
        );
        results.push(format!("{result:?}"));
    }
    assert_eq!(
        results[0], results[1],
        "O0 and O1 must observe the same result"
    );
}

#[test]
fn lexer_probe_compilation_is_byte_identical_per_opt_level() {
    for opt in [OptLevel::O0, OptLevel::O1] {
        assert_eq!(
            compile(opt),
            compile(opt),
            "{opt:?}: SemCode must be byte-identical"
        );
    }
}
