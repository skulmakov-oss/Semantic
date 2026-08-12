use std::{fs, path::Path};

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

#[test]
fn ssf_03_contract_freezes_selected_and_deferred_families() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = read(root, "docs/spec/foundation_stdlib_v0.md");
    let evidence = read(
        root,
        "docs/roadmap/stable_foundation/standard_library_v0_evidence.md",
    );

    for required in [
        "semantic.foundation.std/0.1",
        "not published stable",
        "canonical language-owned",
        "`std.core`",
        "`std.quad`",
        "`std.math`",
        "`std.text`",
        "`std.seq`",
        "`std.map`",
        "`std.option`",
        "`std.result`",
        "`std.serde`",
        "`std.rand`",
        "`std.*` source spellings remain reserved",
        "`std.math` selects only `sqrt` and `abs`",
        "`std.serde` selects no API",
        "`print` is deliberately excluded",
        "xorshift64/13-7-17",
        "no observable map-order promise",
        "neither `N` nor `S` is normalized",
        "qtruth_impl(T, T) = S",
        "sign-extend modulo 2^64",
        "SSF-04 entry conditions",
    ] {
        assert!(
            contract.contains(required),
            "contract is missing {required}"
        );
    }

    for required in [
        "Positive evidence",
        "Negative evidence",
        "Compatibility baseline",
        "crates/sm-front",
        "crates/sm-ir",
        "crates/sm-verify",
        "crates/sm-vm",
        "crates/semantic-core-quad",
        "0115b161f27d12aa877562b440a1cc5a84a05bcb",
    ] {
        assert!(
            evidence.contains(required),
            "evidence is missing {required}"
        );
    }
}

#[test]
fn ssf_03_contract_points_at_real_implementation_and_test_authority() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let front = read(root, "crates/sm-front/src/typecheck.rs");
    for builtin in [
        "len",
        "is_empty",
        "contains",
        "push",
        "prepend",
        "pop",
        "map_empty",
        "map_contains",
        "map_get",
        "map_set",
        "to_text",
        "random_seed",
        "random_next_i32",
    ] {
        assert!(
            front.contains(&format!("\"{builtin}\"")),
            "frontend is missing selected builtin {builtin}"
        );
    }

    let builtin_sigs = read(root, "crates/sm-front/src/lib.rs");
    for builtin in [
        "qtruth_and",
        "qtruth_or",
        "qtruth_not",
        "qtruth_impl",
        "sqrt",
        "abs",
    ] {
        assert!(
            builtin_sigs.contains(&format!("\"{builtin}\"")),
            "frontend is missing selected builtin {builtin}"
        );
    }

    let vm = read(root, "crates/sm-vm/src/semcode_vm.rs");
    for step in ["x ^= x << 13", "x ^= x >> 7", "x ^= x << 17"] {
        assert!(
            vm.contains(step),
            "VM PRNG algorithm drifted: missing {step}"
        );
    }

    for relative in [
        "tests/pcc3_text_core_gate.rs",
        "tests/pcc6_option_acceptance.rs",
        "tests/pcc6_result_acceptance.rs",
        "tests/pcc6_option_result_diagnostics.rs",
        "tests/pcc7_sequence_acceptance.rs",
        "tests/pcc7_map_acceptance.rs",
        "tests/pcc7_collections_diagnostics.rs",
        "tests/pcc8_stdlib_acceptance.rs",
        "tests/pcc8_stdlib_diagnostics.rs",
        "tests/snake_benchmark_gap_matrix.rs",
        "tests/canonical_examples.rs",
    ] {
        assert!(root.join(relative).is_file(), "missing evidence {relative}");
    }
}

#[test]
fn canonical_example_uses_every_selected_language_owned_family() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let example = read(root, "examples/canonical/stdlib_v0_helpers/src/main.sm");

    for required in [
        "assert(",
        "qtruth_and(",
        "qtruth_or(",
        "qtruth_not(",
        "qtruth_impl(",
        "to_text(",
        "sqrt(",
        "abs(",
        "len(",
        "map_empty(",
        "Option::Some(",
        "Result::Ok(",
        "random_seed(",
        "random_next_i32(",
    ] {
        assert!(example.contains(required), "example is missing {required}");
    }

    assert!(
        !example.contains("print("),
        "SSF-03 canonical example must remain host-effect free"
    );
}
