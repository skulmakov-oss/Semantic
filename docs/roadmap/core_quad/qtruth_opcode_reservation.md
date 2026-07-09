# QTruth Opcode Reservation

## 1. Goal
Record the format decision detailing exactly where and how the new explicit `QTruth` opcodes will be mapped in the `sm-format` byte enumeration to ensure full backward compatibility.

## 2. Backward Compatibility Policy
The `sm-format` canonical byte values are permanently frozen for all existing opcodes. Modifying the numeric value of an existing opcode constitutes a breaking change to the language and runtime, invalidating previously compiled modules.
- **Rule 1: Byte-Value Freeze:** No existing opcode may change its assigned numeric ID.
- **Rule 2: Append-Only unless Reserved:** New opcodes must either append to the highest available contiguous integer range or utilize explicitly empty blocks of unused slots that group logically with their domain.

## 3. Byte Layout Analysis
Within `crates/sm-format/src/local_format.rs`, logical and quad operations occupy the `0x10` through `0x16` range:
- `QAnd` = 0x10
- `QOr` = 0x11
- `QNot` = 0x12
- `QImpl` = 0x13
- `BoolAnd` = 0x14
- `BoolOr` = 0x15
- `BoolNot` = 0x16

The contiguous byte slots immediately following this block (`0x17`, `0x18`, `0x19`, `0x1a`) are currently unused. Comparisons (`Cmp*`) begin at `0x20`.

## 4. Slot Reservations
To group the new truth-table semantic operations with the existing logical/boolean operations without disrupting the layout, we reserve the unused `0x17` through `0x1A` block.

The formal reservations are:
- `QTruthAnd` = 0x17
- `QTruthOr`  = 0x18
- `QTruthNot` = 0x19
- `QTruthImpl` = 0x1A

This allocation strictly adheres to the append/reserve policy and protects all existing compiled binaries.
