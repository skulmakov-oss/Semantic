# Static UI IR Artifact V1

Status: normative format contract; UI-DNA2-3B crate-private qualification landed in `main`
Track: UI DNA v2 / UI-DNA2-3A contract and landed UI-DNA2-3B qualification
Scope: canonical in-memory artifact bytes only
Implementation status: crate-private pure in-memory encoder/verifier with two committed golden vectors and executable invalid-artifact qualification landed through PR #1511 at `ddf28436c1c4ab0a961c007e89c757deae87dcfe`; no loader or public codec API exists

## 1. Purpose and authority boundary

Static UI IR Artifact V1 is the deterministic serialized representation of a valid, canonical `StaticUiDocument`:

```text
validated StaticUiDocument
  -> deterministic canonicalization
  -> Static UI IR Artifact V1 bytes
```

The landed crate-private `StaticUiDocument::canonical_bytes()` implementation is internal qualification evidence for the byte layout described here. This specification freezes the normative intended contract. UI-DNA2-3B is merged through PR #1511 at `ddf28436c1c4ab0a961c007e89c757deae87dcfe` and constitutes landed conformance evidence: a crate-private pure in-memory verifier, two committed golden vectors, all 22 normative invalid-artifact rows, deterministic guaranteed-rejection mutations, exhaustive minimal-vector truncation, and exact canonical re-encoding equality. Exact-head and post-merge CI succeeded. This landed boundary does not claim a public artifact implementation, loader, runtime activation, Gate D activation, production promotion, or completion of the UI-DNA2 program.

The authority boundaries are strict:

```text
artifact bytes != Semantic truth
artifact possession != authority
well-formed bytes != valid Static UI IR
valid Static UI IR != admitted runtime state
verification != loading
loading != activation
compile success != runtime activation
artifact compatibility != production promotion
```

The artifact contains projected structure and provenance. It contains no verifier authority, capabilities, admission decisions, runtime commands, shell state, renderer commands, or effects.

## 2. Normative primitive encoding

Artifact V1 uses these primitive rules:

| Primitive | Encoding |
| --- | --- |
| Byte order | little-endian |
| `u32` | exactly 4 bytes |
| `u64` | exactly 8 bytes |
| Count | `u32` |
| Byte-string length | `u32` |
| Byte string | length followed by exactly that many bytes |
| String | a byte string whose payload is valid UTF-8 |
| Optional tag | exactly one byte |
| `None` | tag `0` with no payload |
| `Some` | tag `1` followed by the specified payload |

Every other optional tag is invalid. Truncated fields and trailing bytes are invalid. Offset, end-position, count, and length arithmetic must be checked; unchecked overflow, wrapping, or saturation is forbidden.

The `u32` count and byte-string length representation sets a format ceiling of `u32::MAX`. This representability ceiling is not a recommended operational size, and format representability does not grant permission for a host allocation. The landed verifier enforces caller-supplied input, count, child-count, and role-length resource limits before untrusted allocation; conforming decoders may enforce additional allocation, depth, or other resource quotas. Host quota rejection is a resource rejection and must remain distinct from malformed-artifact rejection.

## 3. Exact Artifact V1 layout

Every field below is emitted exactly once and in the stated order.

### Header

```text
magic_length:     u32_le
magic:            [u8; magic_length]
schema_version:   u32_le
contract_version: u32_le
document_id:      u64_le
revision:         u64_le
epoch:            u64_le
```

The exact magic payload is the 20 UTF-8/ASCII bytes:

```text
UI_DNA2_STATIC_IR_V1
```

The magic is length-prefixed by the same `u32` byte-string rule used elsewhere. It is a format discriminator, not a cryptographic identifier, digest, authenticity proof, or authority token.

The landed `SchemaVersion::CURRENT` and `ContractVersion::CURRENT` values are both unambiguously `1`. Artifact V1 therefore requires `schema_version == 1` and `contract_version == 1`.

`document_id` is a nonzero `StaticDocumentId`. `revision` and `epoch` are unrestricted `u64` values.

### Surface collection

```text
surface_count: u32_le

repeat surface_count times:
  surface_id:     u64_le
  root_node_id:   u64_le
  collection_key: u64_le
  source_ref:     optional SourceRef
```

`surface_id`, `root_node_id`, and `collection_key` represent nonzero identifiers or keys.

### Node collection

```text
node_count: u32_le

repeat node_count times:
  node_id:        u64_le
  role_name:      length-prefixed UTF-8 byte string
  collection_key: u64_le
  source_ref:     optional SourceRef
  child_count:    u32_le

  repeat child_count times:
    child_order:   u32_le
    child_node_id: u64_le

  accessibility_ref: optional u64
```

`node_id`, `collection_key`, `child_node_id`, and a present `accessibility_ref` represent nonzero identifiers or keys. `child_order` is the complete semantic `ChildOrder` value and may be zero.

### Optional `SourceRef`

```text
tag: u8

when tag == 1:
  source_id:  u64_le
  span_start: u32_le
  span_end:   u32_le
```

Tag `0` has no payload. `source_id` represents a nonzero `SourceId`. The underlying `SourceSpan` contract requires `span_start <= span_end`; inverted bounds are invalid.

### Optional accessibility reference

```text
tag: u8

when tag == 1:
  target_node_id: u64_le
```

Tag `0` has no payload. A present target represents a nonzero `StaticNodeId` and remains subject to Static UI IR accessibility-reference validation.

## 4. Canonicalization rules

Validation precedes canonical encoding. Given a valid `StaticUiDocument`, the exact landed canonical order is:

1. surfaces ascending by the tuple `(surface.collection_key, surface.surface_id)`;
2. nodes ascending by the tuple `(node.collection_key, node.node_id)`;
3. each node's child declarations ascending by `child.child_order` alone.

The child sort has no secondary key. This is deterministic for valid input because the Static UI IR validator rejects duplicate `ChildOrder` values within one parent before canonicalization. Implementations must not invent a child-ID tie-break or use storage position as a semantic tie-break.

Surface and node storage insertion order is not artifact identity. Child storage insertion order is not artifact identity after ordering by semantic `ChildOrder`; semantic child order is artifact identity. Document ID, schema version, contract version, revision, epoch, surface and node IDs, collection keys, role-name bytes, roots, child references and orders, accessibility references, and source provenance are artifact identity.

Canonicalization must not invent, discard, repair, infer, or rewrite semantic fields.

```text
canonicalization != semantic repair
canonicalization != validation bypass
normalization != acceptance of malformed input
```

## 5. Role Dictionary boundary

Role names are encoded as their exact UTF-8 bytes. Artifact V1 does not serialize a `RoleDictionary`, a dictionary version, aliases, or dictionary authority.

Possession of an encoded role name does not make that role valid. Artifact verification requires an explicitly selected `RoleDictionary` context. There is no implicit current/latest-role fallback, case folding, aliasing, or substitution for an unknown role. Dictionary acceptance occurs after byte-level decoding and is distinct from byte well-formedness. Cross-dictionary compatibility is not claimed by V1.

No dictionary version field is introduced by this contract.

## 6. Compatibility policy

Artifact V1 is fail-closed:

- the exact magic is required;
- the exact supported schema version is required;
- the exact supported contract version is required;
- an unknown version is deterministically rejected;
- a future version is not an implicit V1 extension;
- forward-field skipping is forbidden;
- trailing extension bytes are forbidden;
- implicit migration is forbidden;
- downgrade is forbidden;
- best-effort decoding is forbidden;
- a latest-version fallback is forbidden.

The schema version governs the structural schema of Static UI IR. The contract version governs the behavioral contract expected by consumers. In landed V1 both encoded values are `1`. Matching the explicitly selected supported versions permits verification to continue; it does not guarantee structural validity, role validity, canonicality, admission, loading, or activation.

A future compatibility bridge requires a separate approved contract. It cannot be inferred from Artifact V1.

## 7. Verification model

The landed crate-private verifier performs these normative stages in order:

1. input and resource preflight;
2. exact bounded byte decoding;
3. exact magic and version checks;
4. primitive and UTF-8 validation;
5. complete-consumption check;
6. construction of a candidate `StaticUiDocument`;
7. Static UI IR structural validation;
8. role validation using the explicitly selected `RoleDictionary` context;
9. canonical re-encoding;
10. exact byte-for-byte equality with the original input;
11. production of a verified-artifact result.

Stages may share implementation machinery, but they must preserve the failure-class distinctions and deterministic precedence specified below. A decoder must never create identifiers, defaults, missing payload, or partial documents from absent bytes.

Incoming noncanonical bytes are rejected. A verifier must not silently normalize them and accept semantic equivalence.

```text
decode + normalize + accept = forbidden
decode + validate + exact canonical equality = required
```

A verified-artifact result is still not a loaded, admitted, activated, or executable runtime object.

## 8. Invalid-artifact matrix

| Failure class | Required result | Must not do |
| --- | --- | --- |
| truncated primitive | reject | synthesize zero/default |
| truncated string/body | reject | return partial document |
| trailing bytes | reject | ignore extension |
| wrong magic | reject | guess format |
| unsupported schema version | reject | migrate implicitly |
| unsupported contract version | reject | downgrade |
| count/length arithmetic overflow | reject | wrap or saturate |
| host quota exceeded | resource rejection | classify as malformed bytes |
| invalid UTF-8 role | reject | lossy decode |
| invalid option tag | reject | interpret as `Some` |
| invalid `SourceSpan` | reject | clamp span |
| invalid nonzero identifier | reject | replace identifier |
| unknown role | validation failure | substitute role |
| duplicate IDs or keys | existing Static UI IR diagnostic | deduplicate |
| missing root or child | existing Static UI IR diagnostic | remove reference |
| duplicate child/order | existing Static UI IR diagnostic | reorder silently |
| cycle | existing Static UI IR diagnostic | cut edge |
| multiple parents/shared surface ownership | existing Static UI IR diagnostic | clone node |
| unreachable node | existing Static UI IR diagnostic | discard node |
| noncanonical surface/node order | reject as noncanonical | normalize and accept |
| noncanonical child order | reject as noncanonical | reorder and accept |
| canonical re-encoding mismatch | reject | accept semantic equivalence |

Failure precedence is deterministic at the stage level:

```text
resource preflight
  -> byte-domain decoding
  -> header/version
  -> complete-consumption
  -> representation construction
  -> Static UI IR validation
  -> role validation
  -> canonical equality
```

Within byte-domain decoding, bounded field extraction and primitive/UTF-8 checks follow wire order and stop deterministically at the first undecodable field. Resource-policy refusal remains distinct from malformed bytes. This contract freezes stage precedence but does not define a Rust enum, public diagnostic type, diagnostic string, or new diagnostic code.

## 9. Determinism and identity

The same valid document under the same explicitly selected dictionary and version context produces identical Artifact V1 bytes. Permutations of surface, node, or child storage do not alter bytes after canonicalization when semantic child orders remain unchanged. Changing semantic child order changes bytes. Changing provenance, revision, or epoch changes bytes.

Artifact bytes are not a hash, signature, MAC, capability, authenticity proof, integrity proof, or admission proof. Cryptographic digesting and signing are outside Artifact V1.

## 10. Explicit non-goals

The following remain outside the landed crate-private UI-DNA2-3B qualification boundary:

- encoder refactor;
- public artifact API;
- filesystem format registration;
- file extension decision;
- MIME type;
- compression;
- encryption;
- hashing or signing;
- memory mapping;
- streaming decoder;
- zero-copy decoder;
- `ProjectionBundle` format;
- Binding Graph serialization;
- Action IR serialization;
- patch serialization;
- runtime loading;
- shell-player integration;
- renderer integration;
- admission;
- capability resolution;
- Gate D activation;
- production promotion.

UI-DNA2-3B is limited to the landed crate-private pure in-memory verifier and executable qualification described above. Verification does not imply loading or activation and does not authorize a filesystem or runtime loader, public codec API, runtime loading, Gate D, or production promotion. No next implementation slice is authorized by this status reconciliation.

```text
public Artifact V1 codec API = absent
filesystem Artifact V1 loader = absent
runtime Artifact V1 loader = absent
runtime loading = unauthorized
Gate D = closed
production promotion = unauthorized
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```

## 11. Acceptance criteria

This contract is complete only when:

1. every byte field emitted by the landed internal qualification encoding is accounted for;
2. exact primitive widths and little-endian order are specified;
3. exact canonical sort tuples are stated;
4. `SourceRef` and accessibility option layouts are specified;
5. compatibility is fail-closed;
6. malformed, structurally or semantically invalid, resource-rejected, and noncanonical artifacts are distinguished;
7. verification is separated from loading and activation;
8. Role Dictionary ownership and explicit context selection are stated;
9. the landed crate-private `canonical_bytes()` and verifier are not claimed as a public artifact API;
10. UI-DNA2 program completion, loading, activation, Gate D, and production promotion remain separately unauthorized.
