# Projection Source Grammar v0

Status: PROPOSED NORMATIVE GRAMMAR CONTRACT
Track: UI DNA v2
Phase: UI-DNA2-WP2B
File posture: `.proj.sm`
Implementation status: NOT IMPLEMENTED
Parser authorization: NOT INCLUDED
Runtime activation: NOT AUTHORIZED

## 1. Ownership and Boundaries

crates/prom-ui::projection_source
    owns Projection Source tokens,
    textual grammar interpretation,
    source AST construction,
    source references,
    parser/source diagnostics,
    source normalization.

crates/prom-ui::projection_compile
    consumes validated Projection Source AST
    and emits Static UI IR.

crates/prom-ui::static_ir
    owns renderer-independent projected structure
    and does not depend on Projection Source syntax.

Required equations:
text grammar != Semantic grammar
parse success != source validation success
source validation success != Static UI IR validity
Static UI IR validity != bundle verification
bundle verification != activation
activation != production promotion

## 2. v0 Capability Boundary

The grammar maps only to the already landed structural AST:
ProjectionSourceDocument
ProjectionSourceSurface
ProjectionSourceNode
ProjectionSourceChild

v0 may express only:
syntax version;
document revision;
document epoch;
surface identity;
surface root node;
surface collection key;
node identity;
node role;
node collection key;
ordered child edge.

The following are reserved for later grammar versions or separate approved extensions:
state bindings;
evidence bindings;
ActionIntent routes;
action offers;
denial outlets;
recovery outlets;
task projection;
freshness/connectivity projection;
accessibility metadata;
focus policy;
criticality;
viewer-relative visibility;
layout hints;
renderer hints.

Do not create placeholder syntax for unimplemented features.
Unknown clauses must be rejected rather than silently ignored.

## 3. Concrete Syntax

```text
projection v0 {
    revision 0;
    epoch 0;

    surface 1 root 10 key 1;

    node 10 role root key 10 {
        child 11 order 0;
        child 12 order 1;
    }

    node 11 role numeric_readout key 11 {
    }

    node 12 role text key 12 {
    }
}
```

Important:
SourceId is supplied by the parser caller.
SourceId is not authored inside `.proj.sm`.

## 4. Document Grammar

document
    = "projection" "v0" "{"
      revision_decl
      epoch_decl
      declaration*
      "}";

revision_decl
    = "revision" decimal_u64 ";";

epoch_decl
    = "epoch" decimal_u64 ";";

declaration
    = surface_decl
    | node_decl;

surface_decl
    = "surface" nonzero_u64
      "root" nonzero_u64
      "key" nonzero_u64
      ";";

node_decl
    = "node" nonzero_u64
      "role" role_identifier
      "key" nonzero_u64
      "{"
      child_decl*
      "}";

child_decl
    = "child" nonzero_u64
      "order" decimal_u32
      ";";

empty node bodies are valid.

## 5. Lexical Rules

### 5.1 Encoding and line endings
LF occupies one byte.

CRLF occupies two bytes.

Bare CR is rejected as PSP_UNEXPECTED_CHAR.

UTF-8 BOM is rejected at byte offset 0 as PSP_UNEXPECTED_CHAR.

Non-ASCII UTF-8 is allowed only inside line-comment text.

Outside comments, non-ASCII input is rejected as PSP_UNEXPECTED_CHAR.

### 5.1.1 Source-size representability

The normative maximum accepted Projection Source input length is:

```text
MAX_PROJECTION_SOURCE_BYTES = u32::MAX = 4_294_967_295 bytes
```

This is a SourceSpan representability ceiling, not a recommended operational
file size. The input length is the UTF-8 encoded byte length of the caller's
`&str`, not a character, scalar, grapheme or line count.

The boundary is inclusive:

```text
len = 0
            -> representable
len = u32::MAX
            -> representable and accepted for lexical processing
len = u32::MAX + 1
            -> rejected before lexical processing
```

`len <= u32::MAX` means only that the input byte domain and its EOF position
can be represented by `SourceSpan` and may proceed to lexical processing. It
does not mean that the source conforms to Grammar v0, that parsing will
succeed, that the resulting AST will pass PS validation, or that the document
may be loaded or executed.

```text
source-size acceptance != syntax validity
source-size acceptance != parse success
source-size acceptance != semantic validity
parse success != semantic-validation success
semantic-validation success != runtime activation
```

An oversized input is any input for which `source_text.len() > u32::MAX`.
The future API classifies this input-domain failure as
`ProjectionSourceInputError::SourceTooLarge`, carrying the caller-supplied
`SourceId`, the actual UTF-8 byte length, and the maximum accepted length
`4_294_967_295`. This task specifies the classification but does not
implement the Rust type.

`SourceTooLarge` is not a `DiagnosticCode`, is not a `PSP_` diagnostic, is not
a `PS_` diagnostic, and is not a syntax or semantic-validation failure.
Every PSP diagnostic requires a representable `SourceSpan`; an oversized
input has no representable complete input domain or EOF position. Therefore
oversized rejection produces no SourceSpan, tokens, AST or PS validation.

The source-size check is performed before inspection of BOM, bare CR, invalid
ASCII, forbidden non-ASCII input, identifier shape, number shape, version,
clauses or EOF. Thus an oversized malformed source returns `SourceTooLarge`,
not a PSP diagnostic. Parser left-to-right first-failure semantics begin only
after this source-input preflight succeeds.

Once preflight succeeds, every derived parser offset must use checked
conversion equivalent to `u32::try_from(offset)`. Offset arithmetic for token
lengths, UTF-8 widths, delimiters, CRLF, semicolons, braces and EOF must be
checked; wrapping, saturating, clamping, truncating or lossy fallback is
forbidden. A correct scanner must never derive an offset beyond the accepted
input length.

For a source of exactly `u32::MAX` bytes, the first byte index is `0`, the
last byte index is `u32::MAX - 1`, and the EOF byte position is `u32::MAX`.
Therefore an EOF span is `[u32::MAX, u32::MAX)`, and a token ending at EOF may
use `[start, u32::MAX)`.

After source-size preflight succeeds, every valid parser position from zero
through `input_byte_length` inclusive is representable as `u32`. This covers
surface declaration starts and ends, node declaration starts and ends, token
starts and ends, zero-width missing-token positions, duplicate-declaration
keyword spans, offending-character and offending-token spans, the EOF
position, and `PSP_UNEXPECTED_EOF` spans.

```text
accepted source
    -> every parser position is in 0..=input_byte_length
accepted source
    -> every parser position is in 0..=u32::MAX
accepted source
    -> every parser-produced `SourceSpan` endpoint is representable as u32
```

Representable does not mean automatically valid, and it does not permit
unchecked casts. Checked offset conversion and checked arithmetic remain
required. Existing span shapes remain unchanged: surface declarations include
their terminating semicolon, node declarations include their matching closing
brace, token spans exclude surrounding trivia, missing-token spans may be
zero-width, and EOF uses `[input_byte_length, input_byte_length)`.

An operational host or loader may impose a smaller limit for memory, quota,
transport or sandbox reasons. Such a host resource rejection is outside this
grammar contract and is distinct from syntax failure, PS validation failure,
and SourceSpan representability.

### 5.1.2 Architecture portability

The normative representability ceiling is `u32::MAX` bytes on every target.
On platforms where `usize` can represent values greater than `u32::MAX`, an
`&str` whose byte length exceeds `u32::MAX` must be rejected as
`ProjectionSourceInputError::SourceTooLarge` before lexical processing. On platforms where `usize` cannot represent values greater than `u32::MAX`, every
constructible `&str` is within the `SourceSpan` representability ceiling.
The grammar limit does not change with pointer width or platform `usize` width.
This is a representability rule; it does not claim that a 32-bit target can
practically allocate an input of exactly `u32::MAX` bytes.

```text
normative maximum != usize::MAX
normative maximum != platform-dependent maximum
32-bit platform behavior != different grammar contract
64-bit platform capability != wider SourceSpan
```

### 5.1.3 Qualification and authorization posture

This documentation defines only the P1 normative source-size contract. The
contract does not self-authorize publication. P1 requires strict review and a
separately authorized publication task before it can be treated as landed
evidence.

```text
P1 contract definition != P1 review completion
P1 review completion != publication authorization
P1 publication != P2 authorization
P1 publication != P3 authorization
P1 publication != parser authorization
```

WP2C-P2 is unresolved and unauthorized. WP2C-P3 is unresolved and
unauthorized. The Projection Source parser and lexer are not implemented and unauthorized. Issue #1489 remains the coordination ledger for these decisions;
this contract does not mark `NEXT AUTHORIZED`.

## 5.5 Source span construction

All SourceSpan values use UTF-8 byte offsets.

All spans are half-open:
[start, end)

Leading and trailing trivia are excluded from declaration spans unless the
trivia occurs inside the declaration's own delimiters.

### Surface declaration span

For:
```text
surface 1 root 10 key 1;
```

define:
start = first byte of the `surface` keyword
end = byte immediately after the terminating semicolon

Therefore:
the terminating semicolon is included;
leading whitespace and comments are excluded;
whitespace and comments following the semicolon are excluded.

### Node declaration span

For:
```text
node 10 role root key 10 {
    child 11 order 0;
}
```

define:
start = first byte of the `node` keyword
end = byte immediately after the matching closing brace

Therefore:
the opening brace is included;
the closing brace is included;
whitespace and comments inside the braces are included;
leading trivia before `node` is excluded;
trailing trivia after the closing brace is excluded.

### Token spans

When token-level spans are produced:
start = first byte of the token
end = byte immediately after the final token byte

Surrounding trivia is excluded.

### Child provenance

ProjectionSourceChild has no SourceRef field.
The parser must not fabricate child-level provenance fields.

### 5.2 Keywords
Keywords are ASCII lowercase and case-sensitive:
projection
v0
revision
epoch
surface
root
key
node
role
child
order

Reject: Projection, V0, NODE, Root, etc.

### 5.3 Identifiers
Role identifiers: `[a-z][a-z0-9_]*`
Role identifiers are resolved against `RoleDictionary`.

The grammar does not permit:
quoted role names;
hyphens;
dots;
namespace separators;
Unicode role identifiers;
arbitrary renderer names.

### 5.4 Numbers
Use decimal ASCII integers only.

Allow: 0, 1, 42, 18446744073709551615
Reject: -1, +1, 01, 0x10, 1_000, 1.0, 1e3

Rules:
revision and epoch may be zero;
surface IDs must be non-zero u64;
node IDs must be non-zero u64;
collection keys must be non-zero u64;
child order is u32 and may be zero.

Leading zeroes are forbidden except for the literal `0`.

### 5.5 Whitespace
Allow: space, horizontal tab, LF, CRLF.
Whitespace is insignificant between tokens.
Do not permit whitespace inside tokens.

### 5.6 Comments
v0 supports only line comments: `// comment until line ending`
No block comments.
No nested comments.
Comments are trivia and do not enter the AST.

## 6. Ordering Semantics

## 6. Ordering Semantics

revision must be the first non-trivia declaration;
epoch must be the second non-trivia declaration;
each must appear exactly once.

Failures:
wrong token where revision is expected -> PSP_UNEXPECTED_TOKEN
EOF where revision is expected -> PSP_UNEXPECTED_EOF
wrong token where epoch is expected -> PSP_UNEXPECTED_TOKEN
EOF where epoch is expected -> PSP_UNEXPECTED_EOF
later revision declaration -> PSP_DUP_REVISION
later epoch declaration -> PSP_DUP_EPOCH

surface and node declarations may otherwise be interleaved;
source declaration order is not canonical structure order;
child order is explicit through the `order` field;
textual child declaration order does not replace ChildOrder;
normalization remains owned by ProjectionSourceDocument::normalized.

Required distinction:
source order = provenance
ChildOrder = semantic child position
normalized storage order = deterministic AST normalization

The parser must not invent child order from declaration position.

After the matching closing brace of the projection document, only trivia is
allowed until EOF.

Any additional token produces PSP_UNEXPECTED_TOKEN.

A second `projection` block is therefore rejected.

Empty input produces PSP_UNEXPECTED_EOF at [0, 0).

Normalization establishes deterministic structural ordering.

Inputs that differ only in declaration storage order may produce the same
provenance-independent structural inventory.

They do not necessarily produce equal ProjectionSourceDocument values.

They do not necessarily produce equal StaticUiDocument values or equal
canonical bytes when provenance is retained.

A provenance-independent structural inventory includes only:
revision;
epoch;
surface IDs;
surface roots;
surface keys;
node IDs;
node roles;
node keys;
child IDs;
ChildOrder values;
normalized surface ordering;
normalized node ordering;
normalized child ordering.

It explicitly excludes:
SourceId;
SourceSpan;
SourceRef;
source byte offsets;
comments;
whitespace;
textual declaration position.

Required equations:
same structure != same provenance
normalization != provenance erasure
structural equivalence != PartialEq

## 7. Parse versus Semantic Validation

### Parser-owned failures

Before parser-owned failures are considered, the future parser performs the
source-size representability preflight defined in section 5.1.1. An oversized
input returns the source-input error `SourceTooLarge` with no SourceSpan and
does not enter PSP diagnostic ordering.

The future parser owns:
invalid UTF-8 input contract;
unexpected character;
unexpected token;
unexpected end of file;
missing keyword;
missing delimiter;
missing semicolon;
invalid identifier shape;
invalid decimal literal;
integer overflow;
zero where a non-zero identity is required;
duplicate revision declaration;
duplicate epoch declaration;
unsupported syntax version;
unknown clause.

### Existing Projection Source validation owns
Do not duplicate these as parser authority:
duplicate surface ID;
duplicate surface key;
duplicate node ID;
duplicate node key;
missing root node;
missing child;
duplicate child;
duplicate child order;
duplicate surface root;
root used as child;
multiple parents;
cycle;
unreachable node;
shared across surfaces;
unknown role.

Unknown role must parse as a syntactically valid role identifier and then fail through existing `PS_UNKNOWN_ROLE` validation.

Required equation:
syntactically valid role token != known role

## 8. Parser Diagnostic Contract

Reserve a parser-specific diagnostic namespace:
`PSP_`

Document at least:
PSP_UNEXPECTED_CHAR
PSP_UNEXPECTED_TOKEN
PSP_UNEXPECTED_EOF
PSP_MISSING_SEMICOLON
PSP_INVALID_IDENTIFIER
PSP_INVALID_NUMBER
PSP_NUMBER_OVERFLOW
PSP_ZERO_ID
PSP_DUP_REVISION
PSP_DUP_EPOCH
PSP_UNSUPPORTED_VERSION
PSP_UNKNOWN_CLAUSE

Projection Source parser v0 is fail-fast.

For parser-invalid input it returns exactly one deterministic PSP diagnostic.

Parsing proceeds from left to right in UTF-8 byte order.

The parser stops at the first parser-owned failure.

It performs no recovery.

It performs no synchronization.

It performs no speculative reinterpretation.

It skips no unexpected tokens.

It returns no partial ProjectionSourceDocument.

Projection Source semantic validation runs only after the complete document
parses successfully.

Required equations:
parse failure -> exactly one PSP diagnostic
parse failure -> no AST
parse failure -> no PS validation

parse success -> AST
parse success -> PS validation may still fail

Every parser diagnostic must contain:
diagnostic code;
caller-supplied SourceId;
SourceSpan identifying the failure.

### Diagnostic span rules

#### Offending character or token
For:
PSP_UNEXPECTED_CHAR
PSP_UNEXPECTED_TOKEN
PSP_INVALID_IDENTIFIER
PSP_INVALID_NUMBER
PSP_NUMBER_OVERFLOW
PSP_ZERO_ID
PSP_UNSUPPORTED_VERSION
PSP_UNKNOWN_CLAUSE

the diagnostic span is the exact offending character or token span.

For `PSP_UNEXPECTED_CHAR`:
an invalid ASCII character spans one byte;
a forbidden non-ASCII scalar spans all UTF-8 bytes of that scalar.

Raw invalid UTF-8 is outside the future `&str` parser contract and is not represented as a parser diagnostic.

#### Duplicate revision or epoch
For:
PSP_DUP_REVISION
PSP_DUP_EPOCH

the diagnostic span is the keyword token of the second declaration.

#### Missing token before another token
For a missing token detected before an actual next token:
span = [next_token_start, next_token_start)

Use `PSP_MISSING_SEMICOLON` for a missing semicolon.
Use `PSP_UNEXPECTED_TOKEN` for another missing required token when the next token is incompatible.

#### Unexpected end of input
For `PSP_UNEXPECTED_EOF`:
span = [input_byte_length, input_byte_length)

#### Leading-zero numbers
Classify:
00
01
00042

as `PSP_INVALID_NUMBER`. The span covers the complete numeric token.

## 9. Source Reference Mapping

ProjectionSourceSurface.source
    spans the complete surface declaration.

ProjectionSourceNode.source
    spans the complete node declaration.

ProjectionSourceToken
    may span the exact originating token when used by the parser.

Every SourceRef uses the SourceId supplied by the caller.

v0 parser must not fabricate child-edge provenance fields
that do not exist in the current AST.

## 10. Role Dictionary Boundary

Accepted built-in identifiers as evidence:
danger_action
evidence_panel
fragment
numeric_readout
recovery_outlet
root
surface
text

grammar accepts syntactically valid role identifiers;
RoleDictionary determines whether the identifier is known;
renderer widget names are not roles;
role lookup grants no authority.

Reject examples such as `button`, `wgpu_surface`, `renderer_button` through semantic validation, not lexical rejection.

## 11. Forbidden Language Features

Explicitly forbid in grammar v0:
expressions;
conditions;
loops;
functions;
macros;
imports;
includes;
dependencies;
variables;
string interpolation;
business rules;
capability checks;
admission rules;
effect execution;
network access;
file access;
absolute pixels;
colors;
fonts;
themes;
CSS-like properties;
renderer/backend selection;
unsafe blocks;
embedded Rust;
embedded Semantic statements;
inline projection inside ordinary `.sm`.

Projection Source v0 is a structural presentation-intent source,
not a general-purpose programming language.

## 12. Valid Normative Examples

### 1. Minimal one-node surface
```text
projection v0 {
    revision 0;
    epoch 0;
    surface 1 root 1 key 1;
    node 1 role root key 1 {}
}
```
AST inventory: revision: 0, epoch: 0, surface IDs: 1, node IDs: 1, roles: root, keys: 1, child IDs: none, ChildOrder values: none.

### 2. Parent with ordered children
```text
projection v0 {
    revision 1;
    epoch 2;
    surface 1 root 10 key 1;
    node 10 role root key 10 {
        child 11 order 0;
        child 12 order 1;
    }
    node 11 role text key 11 {}
    node 12 role text key 12 {}
}
```
AST inventory: revision: 1, epoch: 2, surface IDs: 1, node IDs: 10, 11, 12, roles: root, text, text, keys: 1, 10, 11, 12, child IDs: 11, 12, ChildOrder values: 0, 1.

### 3. Multiple surfaces with interleaved declarations
```text
projection v0 {
    revision 2;
    epoch 1;
    surface 1 root 10 key 1;
    node 10 role root key 10 {}
    surface 2 root 20 key 2;
    node 20 role root key 20 {}
}
```
AST inventory: revision: 2, epoch: 1, surface IDs: 1, 2, node IDs: 10, 20, roles: root, root, keys: 1, 2, 10, 20, child IDs: none, ChildOrder values: none.

### 4. Comments and CRLF/whitespace-equivalent form
```text
projection v0 {
    revision 0; // The revision
    epoch 0;

    surface 1 root 10 key 1;

    // Nodes out of ID order
    node 11 role text key 11 {}
    node 10 role root key 10 {
        child 11 order 0;
    }
}
```
This yields the same provenance-independent normalized structural inventory as a document with nodes declared in order `10` then `11`.
AST inventory: revision: 0, epoch: 0, surface IDs: 1, node IDs: 10, 11, roles: root, text, keys: 1, 10, 11, child IDs: 11, ChildOrder values: 0.

## 13. Invalid Normative Examples

### Lexical & Parser Diagnostics (Rejected by Parser)

For all parser-invalid examples:
diagnostics returned: exactly one PSP diagnostic
AST returned: no
semantic validation reached: no

1. unsupported version: `projection v1 { ... }` -> `PSP_UNSUPPORTED_VERSION`
2. missing revision: `projection v0 { epoch 0; }` -> `PSP_UNEXPECTED_TOKEN`
3. duplicate revision: `projection v0 { revision 0; revision 1; ... }` -> `PSP_DUP_REVISION`
4. missing epoch: `projection v0 { revision 0; surface ... }` -> `PSP_UNEXPECTED_TOKEN`
5. duplicate epoch: `projection v0 { revision 0; epoch 0; epoch 0; ... }` -> `PSP_DUP_EPOCH`
6. zero surface ID: `projection v0 { revision 0; epoch 0; surface 0 root 1 key 1; ... }` -> `PSP_ZERO_ID`
7. zero node ID: `projection v0 { ... node 0 role root key 0 {} }` -> `PSP_ZERO_ID`
8. u64 overflow: `projection v0 { revision 18446744073709551616; ... }` -> `PSP_NUMBER_OVERFLOW`
9. u32 child-order overflow: `projection v0 { ... { child 2 order 4294967296; } }` -> `PSP_NUMBER_OVERFLOW`
10. leading-zero number: `projection v0 { revision 01; ... }` -> `PSP_INVALID_NUMBER`
11. uppercase keyword: `Projection v0 { ... }` -> `PSP_UNEXPECTED_TOKEN`
12. invalid role identifier shape: `node 1 role "root" key 1 {}` -> `PSP_INVALID_IDENTIFIER`
13. missing semicolon: `projection v0 { revision 0 epoch 0; ... }` -> `PSP_MISSING_SEMICOLON`
14. missing closing brace: `projection v0 { revision 0; epoch 0; surface 1 root 1 key 1; node 1 role root key 1 {` -> `PSP_UNEXPECTED_EOF`
15. unknown clause: `projection v0 { revision 0; epoch 0; background blue; ... }` -> `PSP_UNKNOWN_CLAUSE`
16. forbidden expression: `projection v0 { revision 0; epoch 0; node 1 role text key 1 { child 2 order 1+1; } }` -> `PSP_UNEXPECTED_TOKEN`
17. inline binding syntax: `node 1 role text bind val key 1 {}` -> `PSP_UNEXPECTED_TOKEN`
18. renderer/layout property: `node 1 role text key 1 { width 100; }` -> `PSP_UNKNOWN_CLAUSE`

### Semantic Diagnostics (Parsed OK, Rejected by Validation)

For all semantically invalid examples:
parse result: success
semantic validation: one or more existing PS diagnostics

1. unknown role: `projection v0 { revision 0; epoch 0; surface 1 root 1 key 1; node 1 role button key 1 {} }` -> `PS_UNKNOWN_ROLE`
2. missing child node: `projection v0 { revision 0; epoch 0; surface 1 root 1 key 1; node 1 role root key 1 { child 2 order 0; } }` -> `PS_MISSING_CHILD`
3. cycle: `projection v0 { revision 0; epoch 0; surface 1 root 1 key 1; node 1 role root key 1 { child 1 order 0; } }` -> `PS_CYCLE`
4. duplicate child order: `projection v0 { revision 0; epoch 0; surface 1 root 1 key 1; node 1 role root key 1 { child 2 order 0; child 3 order 0; } node 2 role text key 2 {} node 3 role text key 3 {} }` -> `PS_DUP_CHILD_ORDER`

## 14. Forward Compatibility

v0 parsers reject unknown syntax;
unknown syntax is never ignored;
future syntax requires a new approved grammar version or explicit extension;
v0 documents remain parseable under the v0 contract;
grammar version does not imply Static UI IR schema version;
grammar version does not imply RoleDictionary version;
grammar version does not imply ProjectionBundle version.

Required distinction:
Projection Source grammar version
    != Static UI IR schema version
    != role dictionary version
    != bundle format version

## 15. Non-goals

no parser implementation;
no lexer implementation;
no formatter;
no language server;
no syntax highlighting;
no AST redesign;
no public API;
no new role;
no binding syntax;
no action syntax;
no ProjectionBundle format;
no runtime loader;
no shell application;
no Gate D;
no production promotion.
