# VM / Core Quad Semantic Mismatch Audit

## Context

This document audits the fundamental semantic mismatch discovered during the attempt to route legacy VM quad opcodes (`QAnd`, `QOr`) through the new `semantic-core-quad` truth-table mapping operations. It identifies the divergence between bitwise/lattice operations and proper Belnap truth-table semantics.

## 1. Encoding

The base 4-state quad values share the same fundamental layout across both the legacy VM `QuadVal` and `semantic-core-quad::QuadState`:

```text
N = 00
F = 01
T = 10
S = 11
```

## 2. Legacy VM Behavior

The legacy `sm-vm` module implements its quad operations primarily through raw bitwise logic on the 2-bit values:

```text
QAnd  = raw bitwise AND (a & b)
QOr   = raw bitwise OR (a | b)
QNot  = T/F plane swap (a = swap_bits(a))
QImpl = QOr(QNot(lhs), rhs)
```

This makes `QAnd` and `QOr` behave computationally like lattice **meet** and **join** operators rather than logical truth-table mappings.

## 3. semantic-core-quad Behavior

The canonical `semantic-core-quad` SWAR implementation models strict truth-table logic based on truth and falsity planes:

```text
map_and:
  out_t = a_t & b_t
  out_f = a_f | b_f

map_or:
  out_t = a_t | b_t
  out_f = a_f & b_f

map_implies:
  NOT(A) join B (where join is bitwise OR)

map_not:
  T/F plane swap
```

## 4. sm-ir CrystalFold Behavior

The IR folding pass (`CrystalFold`) currently relies on legacy assumptions that align with lattice identities:

```text
QAnd:
  S behaves as identity
  N behaves as annihilator

QOr:
  N behaves as identity
  S behaves as annihilator
```

These identities are inherent properties of bitwise operations on the `00, 01, 10, 11` encodings, directly reflecting the legacy VM behavior.

---

## Required Tables (4x4 Matrix)

### Legacy QAnd (Bitwise AND)
| `a & b` | N (00) | F (01) | T (10) | S (11) |
|---------|--------|--------|--------|--------|
| **N (00)** | N | N | N | N |
| **F (01)** | N | F | N | F |
| **T (10)** | N | N | T | T |
| **S (11)** | N | F | T | S |

### semantic-core-quad map_and (Truth-Table AND)
| `map_and` | N (00) | F (01) | T (10) | S (11) |
|---------|--------|--------|--------|--------|
| **N (00)** | N | F | N | F |
| **F (01)** | F | F | F | F |
| **T (10)** | N | F | T | S |
| **S (11)** | F | F | S | S |

### Legacy QOr (Bitwise OR)
| `a \| b` | N (00) | F (01) | T (10) | S (11) |
|---------|--------|--------|--------|--------|
| **N (00)** | N | F | T | S |
| **F (01)** | F | F | S | S |
| **T (10)** | T | S | T | S |
| **S (11)** | S | S | S | S |

### semantic-core-quad map_or (Truth-Table OR)
| `map_or` | N (00) | F (01) | T (10) | S (11) |
|---------|--------|--------|--------|--------|
| **N (00)** | N | N | T | T |
| **F (01)** | N | F | T | S |
| **T (10)** | T | T | T | T |
| **S (11)** | T | S | T | S |

---

## Expected Key Mismatch Examples

The semantic mismatch becomes extremely clear in the following cases:

```text
F legacy_QAnd T = N
F core_map_and T = F

N legacy_QAnd F = N
N core_map_and F = F

F legacy_QOr T = S
F core_map_or T = T

S legacy_QOr N = S
S core_map_or N = T
```

---

## Required Conclusion & Path Forward

The mismatch between bitwise logic and truth-table mapping creates a rigid fork in the architecture. 

A. **Preserve legacy VM semantics**:
   `QAnd`/`QOr` remain lattice meet/join. `semantic-core-quad` `map_and`/`map_or` are **not** the VM backend for the current opcodes.

B. **Migrate VM to canonical map semantics**:
   This is a fundamental behavior change. It requires a broad compatibility decision and likely a SemCode/runtime versioning note.

C. **Split operations explicitly**:
   - **Lattice layer**: `meet` / `join` / `inverse`
   - **Truth-table layer**: `map_and` / `map_or` / `map_implies` / `map_not`

### Recommended Audit Verdict

**Recommended: C.**

**Reason:**
Current VM `QAnd`/`QOr` and `sm-ir` `CrystalFold` behave like lattice meet/join. 
The `semantic-core-quad` operations (`map_and`/`map_or`) are structurally truth-table maps. 
They represent two mathematically distinct sets of operations across a 4-state domain and should not be silently substituted. Both are useful, but they must be clearly disambiguated at the opcode and semantic layers.
