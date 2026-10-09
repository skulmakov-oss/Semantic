# Compiler Fixed-width Little-endian Integer Contract v0

Contract ID: `semantic.compiler.endian/0.1`

Scope: SHF-2B U1 of #1910, additive to `semantic.compiler.bytes/0.1` and
`semantic.compiler.u32/0.1`. This is a current-toolchain contract, not a change
to `v1.2.0` or `semantic.foundation.std/0.1`.

## Source surface

These six reserved builtins use ordinary positional calls. Arguments have the
exact plain, unmeasured types below; there is no coercion or new `u8`/`u16` type.

| Builtin | Signature |
|---|---|
| `write_u16_le` | `(value: u32) -> Option(Bytes)` |
| `write_u32_le` | `(value: u32) -> Bytes` |
| `write_i32_le` | `(value: i32) -> Bytes` |
| `read_u16_le` | `(bytes: Bytes, offset: u32) -> Option(u32)` |
| `read_u32_le` | `(bytes: Bytes, offset: u32) -> Option(u32)` |
| `read_i32_le` | `(bytes: Bytes, offset: u32) -> Option(i32)` |

Writes construct a new buffer of exactly two or four octets. They do not append
or mutate a buffer. Use the existing `bytes_extend` to concatenate results.
`write_u16_le` returns `None` for values above `65535u32`, without truncation,
wrapping or saturation. Every `u32` and `i32` value has an exact four-octet encoding.

Reads use a zero-based byte offset, with no alignment requirement. For width
`W` (two or four), the complete half-open range `[offset, offset + W)` must
exist. If checked `u32` addition overflows, a host index cannot represent the
range, or the range is outside the buffer, the result is `None`. Empty,
truncated and out-of-bounds reads never produce fabricated zero values or a
partial integer. Trailing bytes are ignored. Buffers remain immutable.

## Representation and execution

The least significant octet is first, independently of the host's byte order.
Signed values use canonical 32-bit two's complement; there is no signed-to-
unsigned numeric coercion in the source API. All 256 octets are data, including
`00`, `7F`, `80`, `FF` and invalid UTF-8 sequences.

| Input | Exact octets | Decoded value |
|---|---|---|
| u16 `0x1234u32` | `34 12` | `4660u32` |
| u32 `0x01020304u32` | `04 03 02 01` | `16909060u32` |
| i32 `-1` | `FF FF FF FF` | `-1` |
| i32 minimum (`-2147483647 - 1`) | `00 00 00 80` | `-2147483648` |
| i32 maximum (`2147483647`) | `FF FF FF 7F` | `2147483647` |

The compiler uses the existing `Call` instruction. No opcode, value-family tag,
ADT descriptor or section layout is added. Trusted execution remains source →
frontend/typecheck → IR/emission → SemCode → `sm-verify` verified token → `sm-vm`.
Runtime arity/family checks remain required even for hand-built admitted bytecode.
O0 and O1 must have identical values, octets, failure results and runtime traps;
repeated builds at each level must produce identical artifacts.

## Admission and compatibility

`SEMCOD25` (epoch 0, revision 26) inherits `HEADER_V24` and adds only
`CAP_BYTES_ENDIAN` (`1 << 29`). An endian builtin requires both this bit and
`CAP_BYTES_VALUES`. The producer selects this header only for emitted endian
calls; core Bytes-only programs keep `SEMCOD24`, u32 arithmetic keeps
`SEMCOD23`, and other programs keep their existing floor.
Bare IR may contain an internal function with an endian spelling. Internal
function resolution takes precedence over builtin resolution in the producer,
verifier and VM; such a call alone does not request endian authority.

This separate envelope is required because `SEMCOD24` admits only the six frozen
core Bytes builtins. Adding executable endian names to its existing capability
would change its admitted vocabulary. Older headers and capability bits keep
their meaning. Relabeling an endian artifact as `SEMCOD24` or older is rejected
before a verified token is issued, including bare calls with no Bytes parameter
or other Bytes-producing instruction.

## Ownership and qualification boundary

The host primitives perform only pure fixed-width integer transformations. They
make no decisions about opcodes, registers, functions, sections, IR, type analysis
or compiler lowering. Semantic-written SemCode construction belongs to SHF-9.
Binary filesystem I/O remains SHF-2C. Big-endian/u64 helpers, conversions, bitwise
operators, mutable buffers, and release/C0 changes are outside this contract.

Qualification uses `tests/shf2b_endian_qualification.rs` and the Semantic golden
program `tests/fixtures/shf2b/endian_golden.sm`. Required evidence includes the
vectors above, unsigned/signed boundaries, exact/truncated/trailing/nonzero-offset
reads, overflow offsets, invalid u16 values, immutability, runtime/source type
rejection, downgrade rejection, O0/O1 parity and deterministic artifacts.
