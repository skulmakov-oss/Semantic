# FORMAT-QTRUTH-OPCODE-SLOTS

## Status
Completed

## Details
Added explicit `QTruthAnd`, `QTruthOr`, `QTruthNot`, and `QTruthImpl` opcode slots to the `Opcode` enum in `sm-format`. Placed at `0x17`-`0x1A` as reserved in #1452. Added `sm-format` unit tests to mathematically enforce backward compatibility with the existing opcode layout. No existing code was touched.
