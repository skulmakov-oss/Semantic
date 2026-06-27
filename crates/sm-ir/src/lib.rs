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
pub mod hello_ir;
#[cfg(feature = "std")]
pub mod hello_semcode;
#[cfg(feature = "std")]
#[doc(hidden)]
pub mod semcode_decode {
    pub use sm_format::semcode_decode::*;
}

#[cfg(feature = "std")]
pub mod semcode_format {
    pub use sm_format::semcode_format::*;
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
