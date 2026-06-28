# Wave 2: SemCode Format Authority Split Plan

## 1. Goal
Map current SemCode format/decode ownership and all consumers. Identify the exact technical debt that causes `sm-vm` and `sm-verify` to depend on the Construction Layer (`sm-ir`), and formulate a plan to move the SemCode binary format authority to a neutral layer.

## 2. Evidence of Current Ownership

### String Matches for Format Constructs
Scanning the codebase for `semcode_format|semcode_decode|semcode_encode|Opcode|MAGIC|HEADER|CAP_` yields:
- `sm-ir/src/local_format.rs`: 243 matches (Actual format definitions, magic bytes, caps, ops)
- `sm-ir/src/semcode_decode.rs`: 11 matches (Decoding logic)
- `sm-ir/src/legacy_lowering.rs`: 94 matches (Encoding/emission logic)
- `sm-verify/src/lib.rs`: 190 matches (Consumption of decode/format)
- `sm-vm/src/semcode_vm.rs`: 274 matches (Consumption of decode/format)
- `sm-emit/src/lib.rs`: 39 matches (Public re-exports)
- `smc-cli/src/app.rs`: 55 matches (Disassembly and emission)

### Dependency Trees
```text
sm-vm -> sm-ir
sm-verify -> sm-ir
sm-emit -> sm-ir
```
All of the core execution and verification tools currently pull in `sm-ir`.

### Public Re-exports
`sm-emit/src/lib.rs` explicitly re-exports everything from `sm_ir::semcode_format`:
```rust
pub use sm_ir::semcode_format::{
    header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8,
    supported_headers, write_f64_le, write_i32_le, write_u16_le, write_u32_le, Opcode,
    SemcodeFormatError, SemcodeHeaderSpec, CAP_CLOCK_READ, ...
};
```

## 3. Classification of Usages

- **sm-vm**: `execution-required`. The VM cannot load or execute SemCode without `decode_semcode_envelope`.
- **sm-verify**: `verification-required`. The verifier needs to unpack the envelope to analyze the structures.
- **sm-emit**: `emit-required`. Packages IR into binary SemCode via legacy lowering logic currently stuck inside `sm-ir`.
- **smc-cli**: `disasm/tooling-only`. Uses the format knowledge to print human-readable disassembly.
- **sm-ir**: `test-only` & format authority owner (currently misplaced).

## 4. Target Ownership Structure

The SemCode binary format / decode authority currently lives in the Construction Layer. It must be moved out into a clean, neutral binary contract layer.

```text
crates/sm-format
  - Owns: `semcode_format.rs`, `semcode_decode.rs`, SemCode header/version constants, Opcode, CAP_* bits, read/write little-endian helpers, SemcodeFormatError, SemcodeHeaderSpec

crates/sm-ir
  - Owns: IR model, lowering, optimization, legacy lowering while not fully split

crates/sm-emit
  - Owns: IR/Semantic program -> SemCode bytes, emission API (may use sm-format)

crates/sm-verify
  - Consumes: sm-format decode/format types

crates/sm-vm
  - Consumes: sm-format decode/format types, sm-verify verified entry types

crates/smc-cli
  - Consumes: sm-format for disasm/tooling
```

## 5. Gradual Migration Ladder (Phase 2)

We will proceed with a strict façade-first approach to ensure zero breakage during the transition:

- **PR 2A**: Create `sm-format` as a neutral façade crate. (Temporarily re-exports from `sm-ir`).
- **PR 2B**: Migrate `sm-verify` consumers to use the `sm-format` path.
- **PR 2C**: Migrate `sm-vm` consumers to use the `sm-format` path.
- **PR 2D**: Migrate `smc-cli` / `sm-emit` public references to the `sm-format` path.
- **PR 2E**: Physically move `semcode_format.rs` and `semcode_decode.rs` from `sm-ir` to `sm-format`.
- **PR 2F**: Remove the transitional `sm-format -> sm-ir` dependency.
- **PR 2G**: Remove the `sm-vm -> sm-ir` direct dependency.
- **PR 2H**: Add static dependency guards ensuring `sm-vm` must not depend on `sm-ir`.
