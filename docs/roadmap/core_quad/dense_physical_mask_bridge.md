# Core Quad Dense and Physical Mask Bridge

## Ownership
* **Owner:** `semantic-core-quad`

## Representations
* **Dense lane mask (`QuadLaneMask32` / `QuadMask32`):** Represents logical lane selection (lane `i` maps to bit `i`). Valid bits are `0` through `31`.
* **Physical packed mask (`QuadPhysicalMask32`):** Represents physical selection within packed two-bit quadits (lane `i` maps to bit `i * 2`).

## Validation Invariant
* `QuadPhysicalMask32` valid bits are restricted to the LSBs of each quadit (even bits from `0` to `62`).
* An explicit constructor `try_from_bits` validates that all odd bits are exactly `0` (`bits & !LSB_MASK_32 == 0`), rejecting invalid bits with a `QuadMaskError::InvalidPhysicalBits`.

## Conversion Direction
* **Dense to Physical:** `QuadMask32::to_physical(self) -> QuadPhysicalMask32` exactly spreads bit `i` to bit `i * 2`.
* **Physical to Dense:** `QuadPhysicalMask32::try_to_lane(self) -> Result<QuadMask32, QuadMaskError>` exactly compresses bit `i * 2` to bit `i`.

## Compatibility
* **`QuadMask32` Status:** The existing semantic meaning remains stable. Bit `i` continues to refer to logical lane `i`. It now has a semantic alias `QuadLaneMask32`.
* **Raw API Naming Rule:** Conversion to and from raw integers correctly respects the internal invariants and errors on failure, utilizing names like `bits()`, `raw()`, and `try_from_bits()`.
* **Scope boundaries:**
  * 128-lane physical masks (`QuadPhysicalMask128`) are explicitly **not** introduced here.
  * This bridge fulfills the mask splitting required by #1408 under the compatibility policy #1413.
  * Delta API and Tile Layout changes are out of scope.
