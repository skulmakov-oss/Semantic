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

    fn skip_optional_ownership_section(code: &[u8], mut cursor: usize) -> usize {
        if !code[cursor..].starts_with(&OWNERSHIP_SECTION_TAG) {
            return cursor;
        }
        cursor += OWNERSHIP_SECTION_TAG.len();
        let event_count = read_u16_le(code, &mut cursor).expect("event count") as usize;
        for _ in 0..event_count {
            let _kind = read_u8(code, &mut cursor).expect("event kind");
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
        // #1773 (FA-09-005): SEMCOD19/rev20 is now the floor (was
        // SEMCOD11/rev12).
        assert_eq!(&bytes[0..8], &MAGIC19);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 20);
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
        assert_eq!(&bytes[0..8], &MAGIC19);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 20);
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
        assert_eq!(&bytes[0..8], &MAGIC19);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 20);
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
        assert_eq!(&bytes[0..8], &MAGIC19);
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let spec = header_spec_from_magic(&magic).expect("known header");
        assert_eq!(spec.rev, 20);
        assert_ne!(spec.capabilities & CAP_SEQUENCE_VALUES, 0);
        assert_ne!(spec.capabilities & CAP_SEQUENCE_ITERATION, 0);

        let code = function_code(&bytes, "main");
        let cursor = skip_optional_ownership_section(code, skip_string_table(code));
        assert!(code[cursor..].contains(&Opcode::SequenceLen.byte()));
        assert!(code[cursor..].contains(&Opcode::SequenceGet.byte()));
    }
}
