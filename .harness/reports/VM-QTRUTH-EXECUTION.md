# VM-QTRUTH-EXECUTION: execute QTruth opcodes through semantic-core-quad truth maps

## Execution Evidence

- **QTruth Execution Enabled**: The `pre_scan` stage in `crates/sm-vm/src/semcode_vm.rs` no longer rejects `QTruthAnd`, `QTruthOr`, `QTruthNot`, and `QTruthImpl` (opcodes `0x17`-`0x1A`) as `BadFormat`. Instead, it advances the instruction pointer appropriately.
- **Semantic Mapping**: Execution of these opcodes inside `run_semcode` is routed to newly added bridge methods `quad_truth_and`, `quad_truth_or`, `quad_truth_not`, and `quad_truth_implies` which wrap calls to the proper Belnap implementations from `semantic-core-quad` (`map_and`, `map_or`, `map_not`, `map_implies`).
- **Disassembly Supported**: Handled `QTruth` variants explicitly within `disassemble_function` under cases `Q_TRUTH_AND`, `Q_TRUTH_OR`, `Q_TRUTH_NOT`, and `Q_TRUTH_IMPL`.
- **Legacy Behavior Preserved**: Pre-existing bitwise opcodes (`QAnd`, `QOr`, `QNot`, `QImpl`) are wholly untouched and continue using their respective lattice functions (`lattice_meet`, `lattice_join`, etc.).
- **Validation Added**: Added `vm_executes_qtruth_opcodes_correctly` to test that the raw VM correctly executes the QTruth opcodes and properly stores the terminal observations for comparison.

All required validation checks succeed without touching out-of-scope layers like `sm-emit` or `sm-ir`.
