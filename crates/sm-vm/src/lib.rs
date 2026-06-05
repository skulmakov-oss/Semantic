#![allow(clippy::clone_on_copy, clippy::needless_lifetimes)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[allow(unused_imports)]
mod semcode_format {
    pub use sm_emit::{
        read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8, Opcode,
        SemcodeFormatError, SemcodeHeaderSpec,
    };
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadVal {
    N,
    F,
    T,
    S,
}

#[cfg(feature = "std")]
mod semcode_vm;

#[cfg(feature = "std")]
pub use semcode_vm::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use sm_emit::compile_program_to_semcode;

    #[test]
    fn sm_vm_smoke_run() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_verified_semcode(&bytes).expect("run");
        let dis = disasm_semcode(&bytes).expect("disasm");
        assert!(dis.contains("main"));
    }
}
