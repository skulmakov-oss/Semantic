# Semantic Error Codes

Semantic diagnostic codes reference.
CLI source: `smc explain <code>` and `smc explain --list`.

## How to use

- View a specific code:
  - `smc explain E0201`
- List all codes:
  - `smc explain --list`

## Catalog

- `E0000`: Retired generic frontend placeholder (SSF-09 #1580): no producer emits it; frontend failures carry E0001-E0009 or E0201.
- `E0001`: Unexpected character in source input.
- `E0002`: Expected logical operator `&&`.
- `E0003`: Expected logical operator `||`.
- `E0004`: Unterminated string literal.
- `E0005`: Frontend syntax error: the grammar parser rejected the input at the reported token.
- `E0006`: Frontend policy violation: the active parser profile does not admit this construct.
- `E0007`: Ambiguous or conflicting source surface: both grammars claim the input.
- `E0008`: No source surface claim: neither grammar establishes evidence for the input.
- `E0009`: Executable bundle composition failed before type checking.
- `E0101`: Bad indentation level (INDENT/DEDENT mismatch).
- `E0200`: Expected Logos declaration (System/Entity/Law).
- `E0201`: Type mismatch. Example: expected QVec/Bool, found other type.
- `E0202`: Expected '=' in a Logos declaration.
- `E0203`: Expected ')' closing a Logos parameter list.
- `E0210`: Malformed Entity declaration header.
- `E0211`: Expected `:` after Entity name.
- `E0212`: Expected newline after Entity header.
- `E0213`: Expected INDENT for Entity body.
- `E0214`: Expected Entity field declaration.
- `E0215`: Entity field must start with `state` or `prop`.
- `E0216`: Expected `:` in Entity field declaration.
- `E0217`: Expected a quoted Law name.
- `E0220`: Duplicate Entity declaration.
- `E0221`: Duplicate Law name within one module.
- `E0222`: Law body is empty.
- `E0223`: Shadowing is forbidden inside a Law scope.
- `E0224`: Empty When condition.
- `E0225`: Empty When body/effect.
- `E0226`: Expected ']' after a Law priority.
- `E0227`: Expected ':' after a Law header.
- `E0228`: Expected newline after a Law header.
- `E0229`: Expected INDENT for a Law body.
- `E0230`: Expected `When` clause in Law body.
- `E0231`: Empty When condition.
- `E0232`: Expected '->' after a When condition.
- `E0233`: Empty When effect.
- `E0234`: Expected type annotation.
- `E0235`: Unexpected line break in a Logos expression.
- `E0236`: Expected identifier in a Logos declaration.
- `E0237`: Expected identifier or number in a Logos expression.
- `E0238`: Cyclic import detected.
- `E0239`: Import resolution/read/parse failure.
- `E0240`: Import re-export is not supported in v0.1.
- `E0241`: Duplicate import alias within one module.
- `E0242`: Public re-export collision.
- `E0243`: Symbol re-export cycle detected.
- `E0244`: Selected import symbol not found in the dependency exports.
- `E0245`: Invalid selected import: duplicate alias, wildcard/select conflict, or kind mismatch.
- `R0001`: Runtime trap: assertion failed.
- `R0002`: Runtime trap: write path overlaps an active borrow.
- `R0003`: Runtime trap: division by zero.
- `R0004`: Runtime trap: arithmetic overflow.
- `R0010`: Runtime quota exceeded.
- `R0020`: Runtime capability denied by the host capability policy.
- `R0030`: Verifier rejected the artifact at the execution entry; the verifier findings are its structured cause.
- `V0001`: Verifier rejection: bad header (BadHeader).
- `V0002`: Verifier rejection: unsupported version (UnsupportedVersion).
- `V0003`: Verifier rejection: truncated function (TruncatedFunction).
- `V0004`: Verifier rejection: invalid function name (InvalidFunctionName).
- `V0005`: Verifier rejection: duplicate function (DuplicateFunction).
- `V0006`: Verifier rejection: invalid string table (InvalidStringTable).
- `V0007`: Verifier rejection: invalid debug section (InvalidDebugSection).
- `V0008`: Verifier rejection: invalid ownership section (InvalidOwnershipSection).
- `V0009`: Verifier rejection: unknown opcode (UnknownOpcode).
- `V0010`: Verifier rejection: operand out of bounds (OperandOutOfBounds).
- `V0011`: Verifier rejection: invalid jump target (InvalidJumpTarget).
- `V0012`: Verifier rejection: invalid string reference (InvalidStringReference).
- `V0013`: Verifier rejection: invalid register reference (InvalidRegisterReference).
- `V0014`: Verifier rejection: unknown call target (UnknownCallTarget).
- `V0015`: Verifier rejection: resource limit exceeded (ResourceLimitExceeded).
- `V0016`: Verifier rejection: capability violation (CapabilityViolation).
- `V0017`: Verifier rejection: ambiguous instruction framing (AmbiguousInstructionFraming).
- `V0018`: Verifier rejection: opcode requires newer header (OpcodeRequiresNewerHeader).
- `V0019`: Verifier rejection: reachable function fallthrough (ReachableFunctionFallthrough).
- `V0020`: Verifier rejection: invalid signature section (InvalidSignatureSection).
- `V0021`: Verifier rejection: call argument count mismatch (CallArgumentCountMismatch).
- `V0022`: Verifier rejection: undefined register read (UndefinedRegisterRead).
- `V0023`: Verifier rejection: analysis state limit exceeded (AnalysisStateLimitExceeded).
- `V0024`: Verifier rejection: analysis work limit exceeded (AnalysisWorkLimitExceeded).
- `V0025`: Verifier rejection: invalid ownership anchor (InvalidOwnershipAnchor).
- `V0026`: Verifier rejection: invalid ADT descriptor section (InvalidAdtDescriptorSection).
- `V0027`: Verifier rejection: ADT requires descriptor header (AdtRequiresDescriptorHeader).
- `V0028`: Verifier rejection: unknown ADT type (UnknownAdtType).
- `V0029`: Verifier rejection: invalid ADT discriminant (InvalidAdtDiscriminant).
- `V0030`: Verifier rejection: ADT variant name mismatch (AdtVariantNameMismatch).
- `V0031`: Verifier rejection: ADT payload arity mismatch (AdtPayloadArityMismatch).
- `V0032`: Verifier rejection: ADT payload index out of range (AdtPayloadIndexOutOfRange).
- `W0240`: Dead law branch detected: When condition is always false.
- `W0241`: Constant folding candidate detected for `fx.*` call with literals (catalogued; not currently emitted, PB-03 #1678).
- `W0250`: Law name style warning (expected `UpperCamelCase`).
- `W0251`: Large Law block warning (too many `When` clauses).
- `W0252`: Unused Entity field warning (`state/prop` not referenced).
- `W0253`: Magic number warning (consider named constant).

## Maintenance

When adding new codes:

1. Update the catalog in `crates/ton618-core/src/diagnostics.rs` (`diagnostic_catalog`).
2. Update this document.
