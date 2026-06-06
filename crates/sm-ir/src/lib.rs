#![allow(
    clippy::too_many_arguments,
    clippy::redundant_closure,
    clippy::default_constructed_unit_structs,
    clippy::type_complexity,
    clippy::op_ref,
    clippy::useless_conversion
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(feature = "std")]
mod frontend {
    pub use sm_front::{
        build_adt_table, build_fn_table, build_record_table, builtin_sig,
        canonicalize_declared_type, parse_logos_program_with_profile, parse_program_with_profile,
        reorder_call_args, resolve_symbol_name, type_check_function_with_table, type_check_program,
        AdtTable, AstArena, BinaryOp, BlockExpr, CompileProfile, Expr, ExprId, FnTable,
        FrontendError, Function, LogosProgram, MatchExpr, OptLevel, QuadVal, RecordTable, ScopeEnv,
        Stmt, StmtId, SymbolId, Type, UnaryOp,
    };
    pub use sm_profile::ParserProfile;
}

#[cfg(feature = "std")]
#[allow(dead_code)]
mod local_format;

#[cfg(feature = "std")]
pub mod hello_ir;
#[cfg(feature = "std")]
pub mod hello_semcode;
#[cfg(feature = "std")]
#[doc(hidden)]
pub mod semcode_decode;

#[cfg(feature = "std")]
pub mod semcode_format {
    pub use crate::local_format::{
        header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8,
        read_utf8, supported_headers, write_f64_le, write_i32_le, write_u16_le, write_u32_le,
        Opcode, SemcodeFormatError, SemcodeHeaderSpec, CAP_CLOCK_READ, CAP_CLOSURE_VALUES,
        CAP_DEBUG_SYMBOLS, CAP_EVENT_POST, CAP_F64_MATH, CAP_FX_MATH, CAP_FX_VALUES,
        CAP_GATE_SURFACE, CAP_MAP_VALUES, CAP_OWNERSHIP_FIELD_PATHS, CAP_OWNERSHIP_PATHS, CAP_PRNG,
        CAP_SEQUENCE_ITERATION, CAP_SEQUENCE_VALUES, CAP_STATE_QUERY, CAP_STATE_UPDATE, CAP_STDOUT,
        CAP_TEXT_VALUES, HEADER_V0, HEADER_V1, HEADER_V10, HEADER_V11, HEADER_V12, HEADER_V13,
        HEADER_V14, HEADER_V15, HEADER_V16, HEADER_V2, HEADER_V3, HEADER_V4, HEADER_V5, HEADER_V6,
        HEADER_V7, HEADER_V8, HEADER_V9, MAGIC0, MAGIC1, MAGIC10, MAGIC11, MAGIC12, MAGIC13,
        MAGIC14, MAGIC15, MAGIC16, MAGIC2, MAGIC3, MAGIC4, MAGIC5, MAGIC6, MAGIC7, MAGIC8, MAGIC9,
        OWNERSHIP_EVENT_KIND_BORROW, OWNERSHIP_EVENT_KIND_WRITE,
        OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL, OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX,
        OWNERSHIP_SECTION_TAG,
    };
}

#[cfg(feature = "std")]
use frontend::*;

#[cfg(feature = "std")]
mod legacy_lowering;
#[cfg(feature = "std")]
pub mod passes;

#[cfg(feature = "std")]
pub use frontend::{CompileProfile, OptLevel};
#[cfg(feature = "std")]
pub use legacy_lowering::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn sm_ir_smoke_compile_program_to_ir() {
        let src = "fn main() { return; }";
        let ir = compile_program_to_ir(src).expect("ir compile");
        assert_eq!(ir.len(), 1);
        assert_eq!(ir[0].name, "main");
    }
}
