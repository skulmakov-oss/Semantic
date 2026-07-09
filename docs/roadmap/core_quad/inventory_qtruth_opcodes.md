# QTruth Opcodes: Inventory & Implementation Roadmap

## 1. Current Lattice Opcode Status
The VM currently executes four legacy scalar quad opcodes: `QAnd`, `QOr`, `QNot`, and `QImpl`.
As established by previous architectural audits (#1440, #1442) and the VM lattice bridge (#1444, #1446), these opcodes operate strictly on **Lattice semantics** (bitwise operations over the 2-bit state encoding mapping to mathematical meet and join). They do NOT utilize Belnap logic truth mappings.

## 2. Proposed QTruth Opcode Names
To introduce genuine truth-table semantics into the VM (evaluating the 4-state domain across orthogonal truth and falsity planes), we will introduce four distinct opcodes corresponding to the `semantic-core-quad` `map_*` methods:
- `QTruthAnd` (Backed by `map_and`)
- `QTruthOr` (Backed by `map_or`)
- `QTruthNot` (Backed by `map_not`)
- `QTruthImpl` (Backed by `map_implies`)

## 3. Affected Crates
Introducing these opcodes touches the full language and runtime contour. The affected crates are:
- `sm-format`: Adding new opcode values to the canonical byte format representation.
- `sm-ir`: Adding IR instructions representing the truth operations.
- `sm-emit` / lowering: Compiling source level constructs/IR down to the new `QTruth*` opcodes.
- `sm-verify`: Validation rules and instruction length checking for the new opcodes.
- `sm-vm`: The actual execution block within the virtual machine loop.

## 4. Required Implementation Sequence
To prevent semantic leakage or breaking the harness, the implementation must proceed in isolated slices, strictly in this order:
1. **Format & IR (`sm-format`, `sm-ir`)**: Register the new opcodes in the binary format enumerations and IR node structures. No logic changes.
2. **Verifier (`sm-verify`)**: Add admission rules for the new opcodes so the runtime can successfully parse and validate modules containing them.
3. **Execution (`sm-vm`)**: Add the execution paths for `QTruth*` in the VM instruction loop, wiring them directly to the `map_*` methods in `semantic-core-quad`.
4. **Emitter & Lowering (`sm-emit`)**: Wire the frontend compiler/lowering layers to emit the new opcodes when truth semantics are explicitly requested by the user code.
5. **Fixtures & Tests**: Generate updated golden SemCode fixtures and assert correct end-to-end execution.

## 5. Compatibility Risks
- **Opcode ID Shifting**: Adding opcodes to `sm-format` might shift subsequent opcode IDs if not carefully appended to the end of the current opcode range or carefully slotted into reserved spaces. This would break all existing compiled SemCode modules.
- **Naming Confusion**: The similarity in names (`QAnd` vs `QTruthAnd`) might lead to incorrect lowering if the emitter is updated indiscriminately.

## 6. Tests Required Before Implementation
- Format snapshot tests proving existing opcodes retain their current byte values.
- Verifier tests asserting rejection of malformed or unexpected opcode arguments for the new types.
- VM tests explicitly comparing `QTruth*` behavior against the `semantic-core-quad` truth mappings, alongside tests confirming legacy `QAnd` etc. remain unchanged.

## 7. Explicit Out-of-Scope List
- Renaming or deprecating existing `QAnd`, `QOr`, `QNot`, `QImpl` opcodes.
- Modifying any logic inside `semantic-core-quad`.
- Introducing equivalence checking (`EQUIV`) or NAND/NOR opcodes.
- Any "hidden adapters" or falsity plane inversions to make lattice opcodes behave like truth opcodes.
