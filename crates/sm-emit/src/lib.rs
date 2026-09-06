#![cfg_attr(not(feature = "std"), no_std)]

// Facade re-export: sm-emit does not own this vocabulary, sm-format does.
// A glob keeps the two in sync automatically instead of hand-curating an
// allowlist that silently drifts behind sm-format's canonical exports.
#[cfg(feature = "std")]
pub use sm_format::semcode_format::*;
#[cfg(feature = "std")]
pub use sm_ir::{
    compile_program_to_semcode, compile_program_to_semcode_with_options,
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, CompileProfile, OptLevel,
};

#[cfg(feature = "std")]
pub mod hello_real_semcode;

#[cfg(feature = "std")]
pub mod hello_observation_bytes;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use sm_ir::{compile_program_to_ir, PathComponent};

    fn function_code<'a>(bytes: &'a [u8], target: &str) -> &'a [u8] {
        let mut cursor = 8usize;
        while cursor < bytes.len() {
            let name_len = read_u16_le(bytes, &mut cursor).expect("name length") as usize;
            let name = std::str::from_utf8(&bytes[cursor..cursor + name_len]).expect("utf8 name");
            cursor += name_len;
            let code_len = read_u32_le(bytes, &mut cursor).expect("code length") as usize;
            if name == target {
                return &bytes[cursor..cursor + code_len];
            }
            cursor += code_len;
        }
        panic!("function '{target}' not found");
    }

    fn skip_string_table(code: &[u8]) -> usize {
        let mut cursor = 0usize;
        let count = read_u16_le(code, &mut cursor).expect("string count") as usize;
        for _ in 0..count {
            let len = read_u16_le(code, &mut cursor).expect("string length") as usize;
            cursor += len;
        }
        cursor
    }

    // #1725 (FA-04-019): OWN0's wire `root` field is now the *index* of the
    // lowered-local key in this same table, not a raw frontend SymbolId, so
    // asserting its value requires decoding the actual strings to find that
    // index - unlike `skip_string_table`, which only needs their lengths.
    fn read_string_table(code: &[u8]) -> (Vec<String>, usize) {
        let mut cursor = 0usize;
        let count = read_u16_le(code, &mut cursor).expect("string count") as usize;
        let mut strings = Vec::with_capacity(count);
        for _ in 0..count {
            let len = read_u16_le(code, &mut cursor).expect("string length") as usize;
            let s = std::str::from_utf8(&code[cursor..cursor + len])
                .expect("utf8 string")
                .to_string();
            cursor += len;
            strings.push(s);
        }
        (strings, cursor)
    }

    fn skip_optional_ownership_section(code: &[u8], mut cursor: usize, header_rev: u16) -> usize {
        if !code[cursor..].starts_with(&OWNERSHIP_SECTION_TAG) {
            return cursor;
        }
        cursor += OWNERSHIP_SECTION_TAG.len();
        let event_count = read_u16_le(code, &mut cursor).expect("event count") as usize;
        for _ in 0..event_count {
            let kind = read_u8(code, &mut cursor).expect("event kind");
            // #1726 Checkpoint D2a / #1891 Checkpoint W2D: at/above the
            // anchor revision, Borrow carries an activation-mode prefix and
            // Write carries an execution-mode prefix, both immediately after
            // `kind` and before `root` - this generic skip-only helper does
            // not care which mode value it is, only how many bytes to
            // consume to reach `root` correctly.
            if header_rev >= SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION {
                if kind == OWNERSHIP_EVENT_KIND_BORROW {
                    let mode = read_u8(code, &mut cursor).expect("activation mode");
                    if mode == ACTIVATION_MODE_STORE_VAR_SITE {
                        let _ = read_u32_le(code, &mut cursor).expect("executable anchor");
                    }
                } else if kind == OWNERSHIP_EVENT_KIND_WRITE {
                    let _mode = read_u8(code, &mut cursor).expect("write execution mode");
                    let _ = read_u32_le(code, &mut cursor).expect("write executable anchor");
                }
            }
            let _root = read_u32_le(code, &mut cursor).expect("root");
            let component_count = read_u16_le(code, &mut cursor).expect("component count") as usize;
            for _ in 0..component_count {
                let component_kind = read_u8(code, &mut cursor).expect("component kind");
                match component_kind {
                    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX => {
                        let _ = read_u16_le(code, &mut cursor).expect("tuple index");
                    }
                    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL => {
                        let _ = read_u32_le(code, &mut cursor).expect("field symbol");
                    }
                    other => panic!("unexpected ownership path component kind: {other:#x}"),
                }
            }
        }
        cursor
    }

    #[test]
    fn sm_emit_smoke_compile_to_semcode() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("emit");
        // #1773 (FA-09-005): SEMCOD19 is now the floor for every compiled
        // artifact regardless of which opcodes it uses (was SEMCODE0).
        assert_eq!(&bytes[0..8], &MAGIC19);
    }

    #[test]
    fn sm_emit_promotes_header_and_encodes_ownership_events_deterministically() {
        let src = r#"
            fn pair() -> (i32, i32) = (1, 2);

            fn main() {
                let pair: (i32, i32) = pair();
                let (ref left, _): (i32, i32) = pair;
                let total: f64 = 0.0;
                total += 1.0;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("emit");
        let bytes_again = compile_program_to_semcode(src).expect("emit");

        assert_eq!(bytes, bytes_again);
        // #1726 Checkpoint D2a: a Tuple/Record Borrow event now always
        // carries a resolved ActivationSiteId (Checkpoint D1), so this
        // program's own artifact is promoted to SEMCOD20/rev21 -- was
        // SEMCOD19/rev20 before D2a (itself promoted from SEMCOD11/rev12 by
        // #1773). Verified semantically unchanged: same event count, same
        // roots, same path/component shape, same trailing Ret -- only the
        // Borrow event's own bytes gained the new activation prefix (and,
        // as of #1891 Checkpoint W2D, the Write event's own bytes gained the
        // analogous execution-mode prefix - this program's reassignment
        // would independently promote to rev21 on its own merits by then).
        assert_eq!(&bytes[0..8], &MAGIC20);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 21);
        assert_ne!(spec.capabilities & CAP_OWNERSHIP_PATHS, 0);

        let code = function_code(&bytes, "main");
        let mut cursor = skip_string_table(code);
        assert_eq!(&code[cursor..cursor + 4], &OWNERSHIP_SECTION_TAG);
        cursor += 4;
        assert_eq!(read_u16_le(code, &mut cursor).expect("event count"), 2);

        assert_eq!(
            read_u8(code, &mut cursor).expect("event kind"),
            OWNERSHIP_EVENT_KIND_BORROW
        );
        assert_eq!(
            read_u8(code, &mut cursor).expect("activation mode"),
            ACTIVATION_MODE_STORE_VAR_SITE
        );
        let _anchor = read_u32_le(code, &mut cursor).expect("executable anchor");
        let borrow_root = read_u32_le(code, &mut cursor).expect("root");
        assert_eq!(read_u16_le(code, &mut cursor).expect("component count"), 1);
        assert_eq!(
            read_u8(code, &mut cursor).expect("component kind"),
            OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX
        );
        assert_eq!(read_u16_le(code, &mut cursor).expect("component value"), 0);

        assert_eq!(
            read_u8(code, &mut cursor).expect("event kind"),
            OWNERSHIP_EVENT_KIND_WRITE
        );
        // #1891 Checkpoint W2D: `total += 1.0;` is a plain reassignment
        // (producer B) - its resolved WriteSiteId is a StoreVarSite, never a
        // MakeRecordSite.
        assert_eq!(
            read_u8(code, &mut cursor).expect("write execution mode"),
            WRITE_EXECUTION_MODE_STORE_VAR_SITE
        );
        let _write_anchor = read_u32_le(code, &mut cursor).expect("write executable anchor");
        let write_root = read_u32_le(code, &mut cursor).expect("root");
        assert_eq!(read_u16_le(code, &mut cursor).expect("component count"), 0);
        assert_ne!(borrow_root, write_root);

        assert!(code[cursor..].ends_with(&[Opcode::Ret.byte(), 0]));
    }

    #[test]
    fn sm_emit_promotes_record_field_borrow_transport_to_v12() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let DecisionContext { camera: ref seen_camera, quality: _ } = ctx;
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("ir");
        let main_ir = ir.iter().find(|func| func.name == "main").expect("main");
        let borrow = main_ir
            .ownership_events
            .first()
            .expect("record borrow ownership event");
        let field_symbol = match borrow.path.components.as_slice() {
            [PathComponent::Field(field)] => field.0,
            other => panic!("expected one field component, got {other:?}"),
        };

        let bytes = compile_program_to_semcode(src).expect("emit");
        let bytes_again = compile_program_to_semcode(src).expect("emit");

        assert_eq!(bytes, bytes_again);
        // #1726 Checkpoint D2a: same reasoning as the sibling tuple-borrow
        // test above -- a record-field Borrow event from a frozen producer
        // now always carries a resolved ActivationSiteId, promoting this
        // artifact to SEMCOD20/rev21. Verified semantically unchanged: same
        // root, same field symbol, same component shape.
        assert_eq!(&bytes[0..8], &MAGIC20);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 21);
        assert_ne!(spec.capabilities & CAP_OWNERSHIP_PATHS, 0);
        assert_ne!(spec.capabilities & CAP_OWNERSHIP_FIELD_PATHS, 0);

        let code = function_code(&bytes, "main");
        let (strings, mut cursor) = read_string_table(code);
        let root_index = strings
            .iter()
            .position(|s| s == &borrow.path.root)
            .expect("lowered-local root key interned in this function's string table")
            as u32;
        assert_eq!(&code[cursor..cursor + 4], &OWNERSHIP_SECTION_TAG);
        cursor += 4;
        assert_eq!(read_u16_le(code, &mut cursor).expect("event count"), 1);
        assert_eq!(
            read_u8(code, &mut cursor).expect("event kind"),
            OWNERSHIP_EVENT_KIND_BORROW
        );
        assert_eq!(
            read_u8(code, &mut cursor).expect("activation mode"),
            ACTIVATION_MODE_STORE_VAR_SITE
        );
        let _anchor = read_u32_le(code, &mut cursor).expect("executable anchor");
        assert_eq!(read_u32_le(code, &mut cursor).expect("root"), root_index);
        assert_eq!(read_u16_le(code, &mut cursor).expect("component count"), 1);
        assert_eq!(
            read_u8(code, &mut cursor).expect("component kind"),
            OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL
        );
        assert_eq!(
            read_u32_le(code, &mut cursor).expect("field symbol"),
            field_symbol
        );
    }

    #[test]
    fn sm_emit_promotes_record_field_write_transport_to_v12() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                assert(patched.camera == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("emit");
        let bytes_again = compile_program_to_semcode(src).expect("emit");

        assert_eq!(bytes, bytes_again);
        // #1891 Checkpoint W2D: this RecordUpdate's Write event now always
        // carries a resolved WriteSiteId (Checkpoint W2C), promoting this
        // artifact to SEMCOD20/rev21 - was SEMCOD19/rev20 before this
        // checkpoint (the SIG0 floor). Capability bits are unaffected by
        // this revision promotion; they remain a separate axis (item 10 of
        // the W2D brief).
        assert_eq!(&bytes[0..8], &MAGIC20);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 21);
        assert_ne!(spec.capabilities & CAP_OWNERSHIP_PATHS, 0);
        assert_ne!(spec.capabilities & CAP_OWNERSHIP_FIELD_PATHS, 0);

        let code = function_code(&bytes, "main");
        let mut cursor = skip_string_table(code);
        assert_eq!(&code[cursor..cursor + 4], &OWNERSHIP_SECTION_TAG);
        cursor += 4;
        assert_eq!(read_u16_le(code, &mut cursor).expect("event count"), 1);
        assert_eq!(
            read_u8(code, &mut cursor).expect("event kind"),
            OWNERSHIP_EVENT_KIND_WRITE
        );
        // This RecordUpdate's Write event resolves to producer C's
        // MakeRecordSite, never a StoreVarSite.
        assert_eq!(
            read_u8(code, &mut cursor).expect("write execution mode"),
            WRITE_EXECUTION_MODE_MAKE_RECORD_SITE
        );
        let _write_anchor = read_u32_le(code, &mut cursor).expect("write executable anchor");
        let _root = read_u32_le(code, &mut cursor).expect("root");
        assert_eq!(read_u16_le(code, &mut cursor).expect("component count"), 1);
        assert_eq!(
            read_u8(code, &mut cursor).expect("component kind"),
            OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL
        );
        let _field = read_u32_le(code, &mut cursor).expect("field symbol");
    }

    #[test]
    fn sm_emit_promotes_sequence_iterable_execution_to_v13() {
        let src = r#"
            fn main() {
                let items: Sequence(i32) = [1, 2, 3];
                let seen: bool = false;
                for item in items {
                    if item == 2 {
                        seen ||= true;
                    }
                }
                assert(seen == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("emit");
        let bytes_again = compile_program_to_semcode(src).expect("emit");

        assert_eq!(bytes, bytes_again);
        // #1891 Checkpoint W2D: `seen ||= true;` is a plain reassignment
        // (producer B), which now always carries a resolved WriteSiteId
        // (Checkpoint W2C) - promoting this artifact to SEMCOD20/rev21, same
        // as a resolved Borrow ActivationSiteId already did for other
        // programs (Checkpoint D2a). Was SEMCOD19/rev20 (the SIG0 floor)
        // before this checkpoint.
        assert_eq!(&bytes[0..8], &MAGIC20);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 21);
        assert_ne!(spec.capabilities & CAP_SEQUENCE_VALUES, 0);
        assert_ne!(spec.capabilities & CAP_SEQUENCE_ITERATION, 0);

        let code = function_code(&bytes, "main");
        let cursor = skip_optional_ownership_section(code, skip_string_table(code), spec.rev);
        assert!(code[cursor..].contains(&Opcode::SequenceLen.byte()));
        assert!(code[cursor..].contains(&Opcode::SequenceGet.byte()));
    }
}
