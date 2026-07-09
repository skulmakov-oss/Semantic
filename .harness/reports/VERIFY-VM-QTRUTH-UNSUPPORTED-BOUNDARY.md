# VERIFY-VM-QTRUTH-UNSUPPORTED-BOUNDARY

## Status
Completed

## Details
Added tests locking the transitional QTruth opcode boundary after #1455.

- sm-verify accepts structurally valid QTruth operand encodings.
- sm-verify rejects truncated QTruth operands.
- sm-vm rejects QTruth opcodes at load/disassembly boundaries as unsupported.
- No QTruth execution, lowering, IR, emitter, semantic-core-quad, opcode byte, hidden adapter, or lattice fallback changes were added.

## Verification
- cargo test -p sm-verify --quiet
- cargo test -p sm-vm --quiet
- cargo test -p sm-vm --features vm-profile --quiet
- git diff --check
- scripts/harness-check.ps1
