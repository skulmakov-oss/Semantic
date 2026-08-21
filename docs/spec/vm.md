# Semantic VM Specification

Status: draft v0
Owner crate: `sm-vm`
Shared runtime vocabulary owner: `sm-runtime-core`

## Purpose

The Semantic VM executes verified SemCode inside a deterministic, isolated
runtime.

The VM is responsible for:

- register-oriented execution
- call-frame management
- deterministic runtime evaluation
- runtime quota enforcement
- safe execution failure reporting

The VM is not responsible for:

- owning the SemCode binary contract
- replacing verifier admission
- owning capability policy semantics
- owning semantic state or rule scheduling

## Standard Execution Rule

The canonical trusted execution route is verifier-first and token-first:
SemCode bytes must pass `sm-verify` admission and produce a verified token
before trusted VM execution.

Current public verified entrypoints include:

- token-first canonical execution:
  - `run_verified_entry_semcode*`
- verified compatibility helpers:
  - `run_verified_semcode`
  - `run_verified_semcode_with_config`
  - `run_verified_semcode_with_entry`
  - `run_verified_semcode_with_entry_and_config`

Lower-level helpers may still exist for tests or narrow integration points, but
they must not redefine the public contract or be described as the canonical
trusted route.

## Runtime Value Model

Current runtime values:

- `Quad`
- `Bool`
- `I32`
- `F64`
- `U32`
- `Fx`
- `Unit`

Presence in the runtime value model does not by itself imply that every
source-level numeric family is already complete through the full public
language pipeline.

## Symbol Runtime Rule

The VM hot path uses `SymbolId` rather than string-keyed locals.

Current rule:

- function-local string tables are converted into runtime `SymbolId` values
  during SemCode loading
- frame locals are keyed by `SymbolId`
- debug naming remains a separate concern

## Execution Contexts

The VM consumes `ExecutionConfig`, which binds:

- `ExecutionContext`
- `RuntimeQuotas`
- trace enablement

Current execution contexts:

- `PureCompute`
- `VerifiedLocal`
- `RuleExecution`
- `KernelBound`

Context rule:

- context selects the runtime quota baseline
- context does not weaken verifier admission or SemCode safety checks

## Trap And Error Model

Current public runtime error families include:

- `BadHeader`
- `UnsupportedBytecodeVersion`
- `BadFormat`
- `UnknownFunction`
- `InvalidJumpAddress`
- `TypeMismatchRuntime`
- `StackUnderflow`
- `StackOverflow`
- `QuotaExceeded`
- `VerifierRejected`
- `BorrowWriteConflict`
- `UnknownVariable`
- `InvalidStringId`

Contract rule:

- verifier rejection must be surfaced distinctly from runtime execution failure
- quota exhaustion must preserve the exceeded quota kind, limit, and usage
- malformed or unsupported bytecode must not be treated as a successful run

`MAP_GET` evaluates its default register only on a key miss. Failure to
read the default register is a runtime error. `MAP_GET` must never
convert a register-read failure into `Value::Unit`.

## Determinism Rule

The VM must behave deterministically for the same:

- verified SemCode input
- execution config
- entry function

The VM must not silently reinterpret:

- unsupported header versions
- malformed jump targets
- malformed function layouts

## Function And Frame Model

Current frame model includes:

- program counter
- register vector
- `SymbolId` local map
- frame-local borrowed tuple and direct record-field paths
- function identity
- optional return destination

Register storage capacity and register initialization are independent:
a register slot may physically exist without ever having been defined by
an instruction. `Value::Unit` is an ordinary Semantic value, never a
stand-in for "nothing was written here." A read of an undefined register
deterministically fails; it must never be treated as `Value::Unit` by
default. This runtime defense is independent of and does not replace
verifier dataflow proof of definite assignment.

Current function-bytecode model includes:

- function name
- string table
- runtime symbol ids
- optional debug symbols
- tuple and direct record-field ownership path metadata admitted from `OWN0`
- instruction stream
- instruction start offset

## Runtime Ownership Slice

Current supported runtime ownership slice is:

- tuple `AccessPath`
- direct record field `AccessPath`
- frame-local borrow lifetime
- runtime write rejection on overlapping borrowed tuple and direct record field
  paths

Current overlap cases that reject:

- exact path equality
- borrowed parent, written child
- borrowed child, written parent

Current allowed case:

- sibling tuple paths
- sibling direct record fields

Current ownership conflict surface:

- runtime overlap rejection uses `BorrowWriteConflict`

Unsupported ownership behavior remains outside the VM contract here and is
specified explicitly in `runtime_ownership.md`.

## Effect Opcode Boundary

The current VM instruction set includes effect-oriented opcodes such as:

- `GateRead`
- `GateWrite`
- `PulseEmit`

Current core rule:

- effectful instructions are counted against the effect-call quota
- effectful instructions do not define capability policy semantics by
  themselves

Richer ABI and capability binding is outside this core contract PR and must not
be smuggled into the VM execution contract by implication.

## Compatibility Rule

The VM must reject unsupported SemCode versions with a clear migration hint.

It must not:

- silently coerce one supported `SEMCODE*` family into another
- accept malformed bytecode by best effort
- treat verifier rejection as a normal runtime success path
