// SSF-08 Lane 2c (#1725 / FA-04-019): "OWN0 roots serialize frontend
// SymbolId but VM interprets them as function string-table indexes".
//
// #1724 already proved (tests/lexical_binding_identity_e2e.rs) that
// ordinary VM local transport (LoadVar/StoreVar) correctly distinguishes
// shadowed bindings. This file proves the same property holds for OWN0
// ownership metadata specifically: a Borrow/Write event attaches to the
// exact runtime local it was recorded against, even when frontend SymbolId
// numbering and the function's own string-table ordering deliberately
// diverge (the normal case since #1724's local-name mangling), and even
// under lexical shadowing where source spelling alone is ambiguous.

use sm_emit::compile_program_to_semcode;
use sm_ir::semcode_format::{
    read_u16_le, read_u32_le, read_u8, read_utf8, ACTIVATION_MODE_STORE_VAR_SITE,
    OWNERSHIP_EVENT_KIND_BORROW, OWNERSHIP_EVENT_KIND_WRITE, OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD,
    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL, OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX,
    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX, OWNERSHIP_SECTION_TAG,
};
use sm_runtime_core::RuntimeTrap;
use sm_vm::RuntimeError;

fn run_source(src: &str) -> Result<(), RuntimeError> {
    let bytes = compile_program_to_semcode(src).expect("compile");
    let token = sm_verify::verify_semcode_token(&bytes).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    sm_vm::run_verified_entry_semcode(&entry_token)
}

const SHADOWED_BOX_SOURCE: &str = r#"
    record Box {
        value: i32,
    }

    fn main() {
        let b: Box = Box { value: 1 };
        let Box { value: ref v } = b;
        if true {
            let mut b: Box = Box { value: 2 };
            b = Box { value: 99 };
        }
        assert(v == 1);
        return;
    }
"#;

#[test]
fn shadowed_binding_write_does_not_falsely_conflict_with_outer_borrow() {
    // Positive root mapping + shadowing: the outer `b` (a real StoreVar,
    // interned into the string table well before the inner shadow's own
    // StoreVar) is field-borrowed via `ref v`; the *inner*, shadowed `b` -
    // a completely different lowered-local key, per #1724 - is directly
    // reassigned (a whole-root write, the shape the VM's StoreVar-time
    // checker actually enforces) while the outer borrow is still live. If
    // OWN0 root identity were broken (both events coincidentally resolving
    // to the same, or an unrelated, string-table index), this would either
    // falsely reject the inner reassignment as a conflict on the outer's
    // `value` field, or silently fail to protect the outer borrow at all.
    run_source(SHADOWED_BOX_SOURCE)
        .expect("shadowed inner reassignment must not conflict with outer borrow");
}

#[test]
fn same_binding_reassignment_still_conflicts_with_its_own_borrow() {
    // Negative control for the test above: with the shadow removed, a
    // direct reassignment of the *same* borrowed binding (root-level write
    // overlapping a field-level borrow, the parent/child overlap shape)
    // must still be rejected - proving root-identity correctness isn't
    // achieved by making the checker too permissive to ever conflict.
    let src = r#"
        record Box {
            value: i32,
        }

        fn main() {
            let mut b: Box = Box { value: 1 };
            let Box { value: ref v } = b;
            b = Box { value: 99 };
            assert(v == 1);
            return;
        }
    "#;
    let err = run_source(src).expect_err("reassigning a borrowed binding must still conflict");
    assert!(
        matches!(err, RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)),
        "expected a borrow/write conflict trap, got {err:?}"
    );
}

fn read_string_table(code: &[u8]) -> (Vec<String>, usize) {
    let mut cursor = 0usize;
    let count = read_u16_le(code, &mut cursor).expect("string count") as usize;
    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let len = read_u16_le(code, &mut cursor).expect("string length") as usize;
        strings.push(
            read_utf8(code, &mut cursor, len)
                .expect("utf8 string")
                .to_string(),
        );
    }
    (strings, cursor)
}

#[test]
fn shadowed_binding_borrow_and_write_decode_to_distinct_string_table_roots() {
    // Direct wire-level companion to the VM-behavioral test above:
    // decodes the actual compiled `main` function's OWN0 section and
    // proves the borrow event's root and the write event's root resolve
    // to two *different* string-table entries - both ending in the source
    // spelling `_b`, confirming they're genuinely the outer and inner
    // shadowed bindings, not merely two unrelated locals that happen to
    // differ. This is the property #1725 establishes; the VM-behavioral
    // test above only shows one *consequence* of it.
    let bytes = compile_program_to_semcode(SHADOWED_BOX_SOURCE).expect("compile");
    let code_start = find_function_code_start(&bytes, "main");
    let code = &bytes[code_start..];
    let (strings, string_table_end) = read_string_table(code);

    let mut cursor = string_table_end;
    if code[cursor..].starts_with(b"DBG0") {
        cursor += 4;
        let count = read_u16_le(code, &mut cursor).expect("debug count") as usize;
        cursor += count * (4 + 4 + 2);
    }
    assert_eq!(&code[cursor..cursor + 4], OWNERSHIP_SECTION_TAG);
    cursor += 4;
    let event_count = read_u16_le(code, &mut cursor).expect("event count");
    assert_eq!(
        event_count, 2,
        "expected exactly the outer borrow and the inner reassignment"
    );

    let mut roots = Vec::with_capacity(2);
    for _ in 0..event_count {
        let kind = read_u8(code, &mut cursor).expect("event kind");
        if kind == OWNERSHIP_EVENT_KIND_BORROW {
            // #1726 Checkpoint D2a: this program's field Borrow now always
            // carries a resolved ActivationSiteId, promoting the artifact to
            // SEMCOD20/rev21, so every Borrow event carries an activation
            // tag (+ a 4-byte executable anchor for StoreVarSite) before its
            // own root.
            let mode = read_u8(code, &mut cursor).expect("activation mode");
            if mode == ACTIVATION_MODE_STORE_VAR_SITE {
                let _anchor = read_u32_le(code, &mut cursor).expect("executable anchor");
            }
        } else if kind == OWNERSHIP_EVENT_KIND_WRITE {
            // #1891 Checkpoint W2D: the shadowed reassignment's Write event
            // now carries its own execution-mode tag + a 4-byte executable
            // anchor before its root, at the identical rev21 gate as Borrow's
            // prefix above (this program is already promoted to rev21 by its
            // Borrow event, independent of this Write event's own promotion
            // rule).
            let _mode = read_u8(code, &mut cursor).expect("write execution mode");
            let _anchor = read_u32_le(code, &mut cursor).expect("write executable anchor");
        }
        let root = read_u32_le(code, &mut cursor).expect("root") as usize;
        let component_count = read_u16_le(code, &mut cursor).expect("component count");
        for _ in 0..component_count {
            let kind = read_u8(code, &mut cursor).expect("component kind");
            if kind == OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX {
                let _ = read_u16_le(code, &mut cursor).expect("tuple index");
            } else if kind == OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX {
                let _ = read_u32_le(code, &mut cursor).expect("sequence index");
            } else if kind == OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL {
                let _ = read_u32_le(code, &mut cursor).expect("field symbol");
            } else if kind == OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD {
                let _ = read_u32_le(code, &mut cursor).expect("adt variant");
                let _ = read_u16_le(code, &mut cursor).expect("adt index");
            } else {
                panic!("unexpected component kind {kind}");
            }
        }
        roots.push(root);
    }

    let root_names: Vec<&str> = roots
        .iter()
        .map(|&idx| {
            strings
                .get(idx)
                .map(String::as_str)
                .expect("root index in bounds")
        })
        .collect();
    assert_ne!(
        root_names[0], root_names[1],
        "outer borrow and inner reassignment must resolve to distinct string-table entries, got {root_names:?}"
    );
    for name in &root_names {
        assert!(
            name.ends_with("_b"),
            "expected both roots to be lowered keys for source spelling 'b', got {name:?}"
        );
    }
}

fn find_function_code_start(bytes: &[u8], target: &str) -> usize {
    let mut cursor = 8usize;
    while cursor < bytes.len() {
        let name_len = read_u16_le(bytes, &mut cursor).expect("function name len") as usize;
        let name = read_utf8(bytes, &mut cursor, name_len)
            .expect("function name")
            .to_string();
        let code_len = read_u32_le(bytes, &mut cursor).expect("function code len") as usize;
        let code_start = cursor;
        if name == target {
            return code_start;
        }
        cursor = code_start + code_len;
    }
    panic!("function '{target}' not found");
}

fn ownership_section_offset_within_code(code: &[u8]) -> usize {
    let mut cursor = 0usize;
    let string_count = read_u16_le(code, &mut cursor).expect("string count") as usize;
    for _ in 0..string_count {
        let len = read_u16_le(code, &mut cursor).expect("string len") as usize;
        cursor += len;
    }
    if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == b"DBG0" {
        cursor += 4;
        let count = read_u16_le(code, &mut cursor).expect("debug count") as usize;
        cursor += count * (4 + 4 + 2);
    }
    assert_eq!(
        &code[cursor..cursor + 4],
        OWNERSHIP_SECTION_TAG,
        "expected an OWN0 section immediately after the string/debug tables"
    );
    cursor
}

#[test]
fn corrupted_own0_root_index_rejects_deterministically_on_load() {
    // Fail-closed negative (#1725 required coverage): an OWN0 root that
    // cannot be resolved against the function's own string table - here,
    // deliberately corrupted to an out-of-bounds index - must be rejected
    // deterministically at load time. Before this fix, the VM's remap
    // silently fell back to treating the raw wire number as if it were
    // already a valid *global* runtime SymbolId (`.unwrap_or(SymbolId(..))`);
    // that fallback is exactly what this test proves is gone.
    let src = r#"
        record Box {
            value: i32,
        }

        fn main() {
            let b: Box = Box { value: 1 };
            let Box { value: ref v } = b;
            assert(v == 1);
            return;
        }
    "#;
    let mut bytes = compile_program_to_semcode(src).expect("compile");
    let code_start = find_function_code_start(&bytes, "main");
    let own0_offset = ownership_section_offset_within_code(&bytes[code_start..]);

    // #1726 Checkpoint D2a: this program's field Borrow now always carries a
    // resolved ActivationSiteId, promoting the artifact to SEMCOD20/rev21.
    // OWN0 layout: TAG(4) + event_count:u16(2) + kind:u8(1) +
    // activation_mode:u8(1) + executable_anchor:u32(4) + root:u32(4) + ...
    let root_offset = code_start + own0_offset + 4 + 2 + 1 + 1 + 4;
    let corrupted_root: u32 = 0xFFFF_FFFF;
    bytes[root_offset..root_offset + 4].copy_from_slice(&corrupted_root.to_le_bytes());

    let err = sm_vm::run_verified_entry_semcode(
        &sm_verify::verify_semcode_token(&bytes)
            .expect("token admission")
            .require_entry("main")
            .expect("entry resolution"),
    )
    .expect_err("a corrupted OWN0 root index must be rejected, never silently accepted");
    assert!(
        matches!(err, RuntimeError::BadFormat(_)),
        "expected a deterministic BadFormat rejection, got {err:?}"
    );
}

// #1725 review follow-up: the fix left `PathComponent::Field`'s and
// `AdtPayload::variant`'s raw frontend `SymbolId` unresolved (see the doc
// comments on `PathComponent` in both crates/sm-runtime-core/src/lib.rs and
// crates/sm-ir/src/legacy_lowering.rs). This is a *narrower, already-sound*
// property than root identity, not the same defect: `access_paths_overlap`
// compares these values purely structurally, root-gated first
// (`lhs.root == rhs.root`, then component-prefix equality) - never through
// `RuntimeSymbolTable`/`vm.symbols`, verified by exhaustive inspection of
// every consumer. The pair below proves this exactly: two *unrelated*
// record types sharing a field spelling (so their raw `SymbolId`s for
// `value` are the identical number, since `SymbolId` is spelling-interned
// program-wide) must not be confused by the checker, because the check is
// root-gated first - while a genuine same-root, same-field conflict must
// still be caught. Both use `x = x with { field: v }` (self-update
// reassignment), the shape that actually round-trips through a field-level
// `Write` `AccessPath` the VM's StoreVar-time checker evaluates - a
// `with`-update into a *new* variable never reaches that check at all (see
// `Stmt::Assign`'s StoreVar-time `ensure_write_path_allowed`, which only
// fires when the reassigned symbol already exists as a local).

#[test]
fn field_symbol_collision_across_unrelated_record_types_does_not_false_conflict() {
    let src = r#"
        record Meter {
            value: f64,
        }
        record Counter {
            value: i32,
        }

        fn main() {
            let m: Meter = Meter { value: 1.0 };
            let Meter { value: ref mv } = m;
            let mut c: Counter = Counter { value: 2 };
            c = c with { value: 99 };
            assert(mv == 1.0);
            return;
        }
    "#;
    run_source(src).expect(
        "an unrelated record type's same-spelled field write must not conflict with the outer borrow",
    );
}

#[test]
fn field_symbol_same_root_same_field_still_conflicts() {
    // Negative control for the test above: with the same field on the same
    // root (no unrelated type involved), the self-update reassignment must
    // still be rejected - proving the checker's root-gating isn't hiding a
    // genuine field-level conflict, only correctly ignoring an unrelated one.
    let src = r#"
        record Meter {
            value: f64,
        }

        fn main() {
            let mut m: Meter = Meter { value: 1.0 };
            let Meter { value: ref mv } = m;
            m = m with { value: 99.0 };
            assert(mv == 1.0);
            return;
        }
    "#;
    let err =
        run_source(src).expect_err("same-root same-field self-update reassignment must conflict");
    assert!(
        matches!(err, RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)),
        "expected a borrow/write conflict trap, got {err:?}"
    );
}
