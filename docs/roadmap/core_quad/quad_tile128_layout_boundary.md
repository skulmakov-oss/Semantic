# QuadTile128 Layout Boundary

**Owner**: `semantic-core-quad`
**Issue**: #1417
**Related Policy**: #1413

## Core Layout
The canonical `QuadTile128` type defines the in-memory semantic storage layout:
- Defined as `#[repr(C, align(16))]`
- Fields: `t: u128` (truth plane) followed by `f: u128` (falsity plane)
- Asserted size: 32 bytes
- Asserted alignment: 16 bytes
- Asserted field offsets: `t` at 0, `f` at 16

## Stable Semantic Properties
The following semantic properties are frozen and tested:
- Lane count (128 lanes)
- Lane numbering (0-127)
- Plane meanings (`t` for truth, `f` for falsity)
- Field ordering
- Constructors (`new`, `from_planes`, `from_regs`) and accessors (`to_regs`, `true_plane`, `false_plane`, `try_get`, `try_set`, `get_unchecked`, `set_unchecked`)

## Explicit ABI Classification
- **In-Memory Layout**: The core CPU representation is now qualified strictly by static assertions.
- **Serialization**: Serialized-data roundtrip compatibility remains a separate concern and is not implied by this layout.
- **Safety**: No general byte-cast or `Pod`/`Zeroable` claims are made. Consumers must not perform unsafe byte-casts based merely on the `repr(C, align(16))` declaration.

## Core / Visual Boundary
- **Semantic Storage**: `QuadTile128` exists solely as semantic core storage.
- **Transport Layout**: The visual adapter crate strictly owns its own GPU transport representation.
- **Transport Architecture**: The GPU transport representation must use a portable word representation (e.g., arrays of `u32`) to avoid 128-bit alignment padding mismatches in graphics APIs.

## Deferred Visual Work
The following work is explicitly deferred to visual integration slices:
- `GpuQuadTile128` definition
- Helpers for splitting/joining `u128` into `[u32; 4]`
- WGSL shader mirrors
- Byte upload implementations
- `bytemuck` qualification

## Explicit Non-Changes
- Existing tile state semantics remain identical.
- Mask evaluation logic is unchanged.
- `QuadroReg32` conversion semantics and dependencies remain unaltered.
- No `bytemuck` or visual dependencies are added to `semantic-core-quad`.
