# Projection Source Grammar v0

Status: NORMATIVE GRAMMAR V0; PARSER AND FRONTEND QUALIFICATION LANDED
Track: UI DNA v2
Phase coverage: UI-DNA2-WP2B + WP2C-P1 + WP2C-P2 + WP2C-P3 + WP2C-P4 + WP2C-P5
File posture: `.proj.sm`
Implementation status: CRATE-PRIVATE PARSER AND PURE IN-MEMORY FRONTEND LANDED
Public parser/frontend APIs and filesystem/runtime loaders: ABSENT
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

This document contains the landed P1 source-size representability contract,
the landed P2 clause-context diagnostic contract, and the landed P3
identifier-tokenization contract. The Grammar v0 normative contract landed in
#1500, its P1-P3 refinements landed through #1502, #1503 and #1506, the
crate-private parser/scanner and qualification landed in #1507, and the
crate-private pure in-memory parser-to-compiler frontend and qualification
landed in #1508.

```text
grammar specification != parser implementation
parser implementation != public parser API
frontend composition != public frontend API
pure in-memory compilation != filesystem loading
filesystem loading != runtime loading
runtime loading != activation
ledger rebaseline -> coordination record of landed evidence
ledger rebaseline != repository truth
issue state != Git tree state
```

The landed parser/scanner remains crate-private. The landed frontend is also
crate-private and operates only on caller-supplied in-memory source text. This
evidence creates no public parser or frontend API, filesystem loader, runtime
loader, semantic admission, activation or production-promotion authority.

Issue #1489 remains the coordination ledger for landed checkpoints and
explicit next-step authorization. It grants no architectural authority and
does not create landed repository evidence. This specification does not mark
`NEXT AUTHORIZED`.

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

The identifier shape is ASCII-only. A role identifier starts with one
lowercase ASCII letter and continues with lowercase ASCII letters, digits or
underscore. Reserved keyword spelling does not globally invalidate this
shape: at a `role_identifier` position, `root`, `surface`, `node` and `v0`
remain syntactically valid identifiers. `RoleDictionary`, not tokenization,
determines whether a syntactically valid role is known.

```text
reserved keyword spelling != invalid role identifier shape
```

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

At a numeric-owning parser position, a leading ASCII `+` or `-` immediately
followed by an ASCII decimal digit begins the signed numeric diagnostic
candidate defined in section 8. Signed forms remain invalid Grammar v0
numbers and report `PSP_INVALID_NUMBER`; this grouping does not admit signed
decimal syntax.

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

### Unknown clause versus unexpected token

This section defines the deterministic choice between
`PSP_UNKNOWN_CLAUSE` and `PSP_UNEXPECTED_TOKEN`. Classification depends only
on the current grammar position and the first failing token. It does not
depend on parser implementation strategy, later tokens, semantic intent,
future grammar extensions or renderer interpretation.

For this rule, a recognized keyword is one of the exact reserved words in
section 5.2. A valid unreserved identifier-shaped word is a complete token
matching `[a-z][a-z0-9_]*` that is not a recognized keyword.

A clause-list entry position is exactly one of these two contexts:

1. **Document declaration-list entry.** This position occurs after the
   mandatory revision and epoch declarations, after a completed surface or
   node declaration, and before the document closing brace. Legal next tokens
   are `surface`, `node` and `}`. A repeated top-level `revision` or `epoch`
   starter uses its dedicated duplicate diagnostic when applicable.
2. **Node child-list entry.** This position occurs after the node opening
   brace, after a completed child declaration, and before the node closing
   brace. Legal next tokens are `child` and `}`.

No other parser position is a clause-list entry position in Grammar v0.

```text
unknown clause classification
    requires a clause-list entry position
```

At a clause-list entry position, if the next complete token is a valid
unreserved identifier-shaped word, the parser reports
`PSP_UNKNOWN_CLAUSE`. Its span is the exact first unknown clause-name token,
excluding trivia, following arguments, the semicolon and the rest of the
source.

Consequently, `background;`, `background blue;` and
`background blue red;` all fail on the same `background` token with
`PSP_UNKNOWN_CLAUSE`. The parser does not need to establish whether the
remainder resembles a complete clause.

Normative clause-list examples are:

- `projection v0 { revision 0; epoch 0; background blue; }` reports
  `PSP_UNKNOWN_CLAUSE` on the exact `background` token;
- `projection v0 { revision 0; epoch 0; layout grid; }` reports
  `PSP_UNKNOWN_CLAUSE` on the exact `layout` token;
- `projection v0 { revision 0; epoch 0; node 1 role text key 1 { width 100; } }`
  reports `PSP_UNKNOWN_CLAUSE` on the exact `width` token;
- `projection v0 { revision 0; epoch 0; node 1 role text key 1 { bind state; } }`
  reports `PSP_UNKNOWN_CLAUSE` on the exact `bind` token.

```text
unknown clause name != known grammar keyword
unknown clause detection != future feature recognition
unknown clause detection != semantic interpretation
unknown clause detection != parser recovery
```

`PSP_UNEXPECTED_TOKEN` applies when the current grammar position requires a
specific token or token class and the actual token cannot legally satisfy
that position, unless a more specific existing diagnostic applies. This
includes every non-clause-list position, a recognized keyword that is illegal
at a clause-list entry, any non-trivia token after the complete projection
document, and any token that does not qualify for `PSP_UNKNOWN_CLAUSE`.

```text
recognized keyword in wrong context
    -> PSP_UNEXPECTED_TOKEN
```

The exception is an applicable dedicated diagnostic. A more specific existing
PSP diagnostic takes precedence over both `PSP_UNKNOWN_CLAUSE` and generic
`PSP_UNEXPECTED_TOKEN`, including:

- unsupported version -> `PSP_UNSUPPORTED_VERSION`;
- invalid number -> `PSP_INVALID_NUMBER`;
- number overflow -> `PSP_NUMBER_OVERFLOW`;
- zero identity -> `PSP_ZERO_ID`;
- missing semicolon -> `PSP_MISSING_SEMICOLON`;
- unexpected EOF -> `PSP_UNEXPECTED_EOF`;
- duplicate revision declaration -> `PSP_DUP_REVISION`;
- duplicate epoch declaration -> `PSP_DUP_EPOCH`.

Normative fixed-position and wrong-context examples are:

- `background v0 {}` reports `PSP_UNEXPECTED_TOKEN` on `background` because
  `projection` is required;
- `projection v0 { background 0; }` reports `PSP_UNEXPECTED_TOKEN` on
  `background` because `revision` is required;
- `projection v0 { revision 0; epoch 0; surface 1 background 10 key 1; }`
  reports `PSP_UNEXPECTED_TOKEN` on `background` because `root` is required;
- `projection v0 { revision 0; epoch 0; node 1 role text bind val key 1 {} }`
  reports `PSP_UNEXPECTED_TOKEN` on `bind` because `key` is required;
- `projection v0 { revision 0; epoch 0; node 1 role text key 1 { child 2 width 0; } }`
  reports `PSP_UNEXPECTED_TOKEN` on `width` because `order` is required;
- `projection v0 { revision 0; epoch 0; } background blue;` reports
  `PSP_UNEXPECTED_TOKEN` on `background` because only trivia and EOF are
  legal after the document;
- `projection v0 { revision 0; epoch 0; child 2 order 0; }` reports
  `PSP_UNEXPECTED_TOKEN` on the wrong-context recognized keyword `child`;
- `projection v0 { revision 0; epoch 0; node 1 role text key 1 { surface 2 root 2 key 2; } }`
  reports `PSP_UNEXPECTED_TOKEN` on the wrong-context recognized keyword
  `surface`;
- `projection v0 { revision 0; epoch 0; node 1 role text key 1 { revision 2; } }`
  reports `PSP_UNEXPECTED_TOKEN` on the wrong-context recognized keyword
  `revision`.

After a revision declaration has already been accepted, another top-level
revision declaration starter reports `PSP_DUP_REVISION`. After an epoch
declaration has already been accepted, another top-level epoch declaration
starter reports `PSP_DUP_EPOCH`. These duplicate diagnostics apply only in
the top-level declaration sequence. For example, `revision` where `root` is
required inside `surface_decl` is `PSP_UNEXPECTED_TOKEN`, not
`PSP_DUP_REVISION`.

When a complete declaration requires a semicolon and the next token begins
before that semicolon, `PSP_MISSING_SEMICOLON` takes precedence. Its span is
the zero-width range at the next token start.

```text
unfinished current production
    != next clause-list entry
```

At a valid clause-list entry, `}` closes the current list. EOF before the
required closing brace reports `PSP_UNEXPECTED_EOF`; EOF and a missing closing
brace are never `PSP_UNKNOWN_CLAUSE`.

The normative decision table is:

| Parser position | Actual token | Result |
| --- | --- | --- |
| document declaration-list entry | `surface` or `node` | parse the legal declaration |
| document declaration-list entry | `}` | close the document |
| document declaration-list entry | valid unreserved identifier-shaped word | `PSP_UNKNOWN_CLAUSE` |
| document declaration-list entry | recognized keyword illegal in this context | `PSP_UNEXPECTED_TOKEN`, unless a dedicated duplicate diagnostic applies |
| node child-list entry | `child` | parse the legal child declaration |
| node child-list entry | `}` | close the node |
| node child-list entry | valid unreserved identifier-shaped word | `PSP_UNKNOWN_CLAUSE` |
| node child-list entry | recognized keyword illegal in this context | `PSP_UNEXPECTED_TOKEN` |
| fixed production position | any incompatible token | specific diagnostic or `PSP_UNEXPECTED_TOKEN` |
| after complete document | any non-trivia token | `PSP_UNEXPECTED_TOKEN` |
| any position before a required closing brace | EOF | `PSP_UNEXPECTED_EOF` |

Classification is determined from the current parser state and the first
failing token only. The parser does not scan to a later semicolon, inspect a
future clause version, guess intended syntax, skip or consume the failing
clause, recover, or emit multiple diagnostics.

```text
first failing token determines diagnostic coordinate
unknown clause classification does not consume the clause
unknown clause classification does not produce partial AST
unknown clause classification does not enter PS validation
```

This P2 rule applies only after identifier-candidate classification has
produced either a recognized keyword or a complete valid unreserved
identifier-shaped word. Malformed identifier candidates are classified by the
P3 contract below. When the parser context owns identifier classification,
the dedicated `PSP_INVALID_IDENTIFIER` diagnostic outranks P2 generic
classification. P2 does not reclassify a P3 failure.

```text
P2 clause-context disambiguation != P3 identifier tokenization
```

The P1 source-size preflight remains earlier than lexical classification and
this P2 choice. An oversized source returns `SourceTooLarge`, performs no
lexical scan and produces no PSP diagnostic.

P2 specification alone was not parser or lexer implementation, public API or
parser authorization. The crate-private parser/scanner landed in #1507 and
implements the P1-P3 contracts without publishing a parser or token API. The
crate-private pure in-memory frontend landed in #1508 and composes parsing with
the existing Semantic validation and Static UI IR compilation boundary.
Diagnostic classification is not Semantic validation, capability, admission
or runtime activation. Landed qualification does not mean publication, runtime
loading or production promotion. Gate D remains closed.

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

### Identifier tokenization and malformed identifier diagnostics

This section defines the normative WP2C-P3 diagnostic boundary. It defines
candidate coordinates and context-sensitive diagnostic selection, not a
public token type or a concrete scanner implementation.

#### Identifier vocabulary and candidate boundary

A valid identifier remains exactly:

```text
valid_identifier = [a-z][a-z0-9_]*
```

An unquoted identifier candidate is the maximal ASCII sequence described by:

```text
candidate_start = [A-Za-z0-9_]
candidate_continue = [A-Za-z0-9_.:-]
unquoted_identifier_candidate = [A-Za-z0-9_][A-Za-z0-9_.:-]*
```

The candidate ends before space, horizontal tab, LF, CRLF, `{`, `}`, `;`, the
`//` comment opener, EOF, or a byte not included in `candidate_continue`.
Candidate scanning is maximal: `root-name`, `root.name`, `root::name`, `Root`,
`9root`, `_root` and `root-` are each one candidate. They are not split into
smaller parser-level tokens to choose a diagnostic.

```text
malformed identifier punctuation != token recovery boundary
```

A valid identifier starts with one lowercase ASCII letter, continues only
with lowercase ASCII letters, digits or underscore, and contains no hyphen,
dot, colon, namespace separator, uppercase ASCII or non-ASCII scalar. Reserved
keyword spelling remains valid identifier shape at a `role_identifier`
position.

The remaining vocabulary is context-sensitive:

- a quoted role candidate is defined only at a `role_identifier` position;
- a numeric candidate is an unquoted candidate interpreted at a numeric
  grammar position;
- a recognized keyword is an exact keyword from section 5.2;
- `{`, `}` and `;` are structural delimiters;
- `//` opens line-comment trivia;
- any other character is governed by the existing unexpected-character
  boundary.

#### Quoted role candidate

At a `role_identifier` position only, an ASCII `"` starts a malformed quoted
role candidate. It extends through the next `"` on the same logical line, has
no escape syntax, and is always invalid in Grammar v0. A closed candidate such
as `"root"` reports `PSP_INVALID_IDENTIFIER` over both quotes and all enclosed
bytes.

If no closing quote occurs on the same logical line, the candidate extends
from the opening quote to the first byte of the line ending, excluding that
line ending, or to EOF. It reports `PSP_INVALID_IDENTIFIER` over that complete
malformed candidate. This rule introduces no string syntax, escape processing
or string token. Outside a `role_identifier` position, `"` remains governed by
the existing unexpected-character or unexpected-token rules. If a forbidden
non-ASCII scalar occurs inside a quoted role candidate, lexical
`PSP_UNEXPECTED_CHAR` precedence still reports that scalar rather than folding
it into `PSP_INVALID_IDENTIFIER`.

#### Signed numeric, standalone plus and standalone minus boundaries

Only when the current parser position requires a decimal number, an ASCII `+`
or `-` immediately followed by an ASCII decimal digit begins a conceptual
signed numeric candidate:

```text
signed_numeric_candidate
    = sign + maximal unsigned digit-start numeric candidate

sign
    = + or -
```

The underlying unsigned digit-start numeric candidate begins with an ASCII
digit and consumes the maximal number-like ASCII sequence used by the existing
invalid-number rules. It includes ASCII letters, digits, underscore, dot and
colon, but ends before `+` or `-`. This numeric diagnostic boundary is distinct
from the unquoted identifier-candidate boundary: `root-name` remains one
identifier candidate, while `1-1` contains a complete numeric token `1`
followed by `-`.

A signed numeric candidate ends before space, horizontal tab, LF, CRLF, `{`,
`}`, `;`, the exact `//` comment opener, EOF, `+`, `-`, or another byte outside
the underlying digit-start number-like candidate boundary. It reports
`PSP_INVALID_NUMBER`. Its span includes the sign and the complete following
maximal digit-start numeric candidate, excluding surrounding trivia and the
terminating delimiter.

Thus `-1;` groups complete `-1`, and `+1 // comment` groups complete `+1`
before the delimiter or comment trivia. In `-1+1`, the signed candidate is
complete `-1`; it immediately reports `PSP_INVALID_NUMBER`, and fail-fast
parsing does not inspect the trailing `+1`.

Normative signed candidates include:

| Input at a numeric position | Result | Exact span |
| --- | --- | --- |
| `-1` | `PSP_INVALID_NUMBER` | complete `-1` |
| `+1` | `PSP_INVALID_NUMBER` | complete `+1` |
| `-0` | `PSP_INVALID_NUMBER` | complete `-0` |
| `+0` | `PSP_INVALID_NUMBER` | complete `+0` |
| `-01` | `PSP_INVALID_NUMBER` | complete `-01` |
| `+01` | `PSP_INVALID_NUMBER` | complete `+01` |
| `-1.0` | `PSP_INVALID_NUMBER` | complete `-1.0` |
| `+1_000` | `PSP_INVALID_NUMBER` | complete `+1_000` |
| `-1e3` | `PSP_INVALID_NUMBER` | complete `-1e3` |
| `+0x10` | `PSP_INVALID_NUMBER` | complete `+0x10` |

Signed form classification precedes overflow, leading-zero and non-zero-ID
interpretation. Signed zero is therefore not `PSP_ZERO_ID`, and signed
overflow text is not `PSP_NUMBER_OVERFLOW`; every recognized signed numeric
candidate reports `PSP_INVALID_NUMBER`.

The signed-candidate grouping is owned by a numeric parser context and does
not create a general signed token. Outside a numeric-owning position, `-root`
starts with `PSP_UNEXPECTED_CHAR` on the one-byte `-`. A sign followed by
digits after the complete document does not become a signed numeric candidate.

When it is not consumed by the numeric-context signed-candidate rule, ASCII
`+` is a conceptual one-byte incompatible punctuation token for deterministic
diagnostic selection. It is not an addition operator, expression syntax,
public token enum or lexer API. Its exact span is:

```text
[plus_start, plus_start + 1)
```

At a parser position where `+` is incompatible and no dedicated diagnostic
takes precedence, it reports `PSP_UNEXPECTED_TOKEN`. ASCII `-` does not gain
this standalone punctuation boundary. Unless it is internal to an identifier
candidate or begins a signed numeric candidate at a numeric-owning position,
`-` remains `PSP_UNEXPECTED_CHAR` over its one-byte span.

#### Context-sensitive classification

At `node <id> role <role_identifier> key <key> { ... }`, a candidate matching
`[a-z][a-z0-9_]*` is syntactically valid. A known valid identifier is later
accepted by `RoleDictionary`; an unknown valid identifier such as `button`
parses successfully and later reports `PS_UNKNOWN_ROLE`. A malformed unquoted
candidate or quoted role candidate reports `PSP_INVALID_IDENTIFIER` over the
complete candidate. A forbidden non-ASCII scalar reports
`PSP_UNEXPECTED_CHAR` over that scalar.

At either landed P2 clause-list entry, classification is ordered as follows:

1. a legal recognized keyword parses the declaration or closes the list;
2. a recognized keyword illegal in that list reports `PSP_UNEXPECTED_TOKEN`,
   unless a dedicated duplicate diagnostic applies;
3. a valid unreserved identifier-shaped word reports `PSP_UNKNOWN_CLAUSE`;
4. a malformed unquoted identifier candidate reports
   `PSP_INVALID_IDENTIFIER`;
5. a decimal-only numeric candidate reports `PSP_UNEXPECTED_TOKEN` because it
   is neither an identifier-shaped word nor a malformed identifier candidate.

```text
P2 unknown clause requires a valid complete identifier-shaped word
malformed identifier candidate != unknown clause
PSP_INVALID_IDENTIFIER outranks P2 generic classification when applicable
```

Thus `background blue;` at a clause-list entry reports
`PSP_UNKNOWN_CLAUSE` on `background`, while `Background blue;`,
`_background blue;`, `9background blue;`, `background-color blue;`,
`background.color blue;` and `background::color blue;` each report
`PSP_INVALID_IDENTIFIER` on the complete first candidate. The landed P2 rules
for wrong-context keywords remain unchanged: `child` at a document
declaration-list entry and `surface` at a node child-list entry report
`PSP_UNEXPECTED_TOKEN`.

At a position requiring a specific keyword (`projection`, `revision`, `epoch`,
`surface`, `root`, `key`, `node`, `role`, `child` or `order`), a complete ASCII
candidate that is not that exact keyword reports `PSP_UNEXPECTED_TOKEN`.
Consequently, `Projection v0 {}` reports `PSP_UNEXPECTED_TOKEN` on
`Projection`, and `projec-tion v0 {}` reports `PSP_UNEXPECTED_TOKEN` on
`projec-tion`.

```text
identifier-shape diagnostic depends on an identifier-owning parser context
wrong fixed keyword -> PSP_UNEXPECTED_TOKEN
```

At a numeric position, classification remains numeric. A digit-start candidate
containing any non-decimal candidate character reports `PSP_INVALID_NUMBER`
over the complete candidate. This includes `9root`, `1_000`, `1.0`, `1e3` and
`0x10`. Decimal overflow remains `PSP_NUMBER_OVERFLOW`; zero in a non-zero
identity position remains `PSP_ZERO_ID`; and a leading zero other than literal
`0` remains `PSP_INVALID_NUMBER`. An ASCII candidate beginning with a letter
or underscore where a number is required reports `PSP_UNEXPECTED_TOKEN`, not
`PSP_INVALID_IDENTIFIER`. Before those unsigned interpretations, an eligible
signed numeric candidate reports `PSP_INVALID_NUMBER` over its complete
sign-plus-candidate span.

After an unsigned numeric field is accepted, the next expected grammar symbol
determines ownership of following input. Adjacent-plus classification applies
only after a numeric field whose next required grammar symbol is not `;`.
Adjacent-minus classification has the same limitation.

#### Semicolon-required parser state

The conceptual parser state
`semicolon_required_after_complete_declaration` begins when all mandatory
fields of a semicolon-terminated Grammar v0 declaration have been accepted but
the terminating `;` has not yet been consumed. The semicolon-terminated
declarations are exactly:

```text
revision_decl
epoch_decl
surface_decl
child_decl
```

Their terminal forms are:

```text
revision <revision_value> ;
epoch <epoch_value> ;
surface <surface_id> root <root_id> key <surface_key> ;
child <child_id> order <child_order> ;
```

This state does not apply before the final field is parsed, inside a numeric or
identifier candidate, to a node declaration ending with `}`, or while another
keyword or delimiter other than `;` is required.

Once the parser enters `semicolon_required_after_complete_declaration`, it
inspects the next non-trivia source coordinate before assigning a lexical or
token class to the following source unit:

```text
next non-trivia source unit is ;
    -> consume ;
    -> complete declaration

next non-trivia source unit is not ;
    -> PSP_MISSING_SEMICOLON
    -> [next_non_trivia_start, next_non_trivia_start)

complete declaration awaiting ; owns the next coordinate
semicolon-required state precedes classification of the following source unit
```

The missing-semicolon span is always zero-width at the first byte of the next
non-trivia source unit. It retains the caller-supplied `SourceId`, half-open
UTF-8 byte offsets and `u32` endpoints. Trivia is excluded, and the following
unit is neither consumed nor classified. EOF preserves the existing Grammar v0
EOF rule; this state-specific missing-semicolon decision applies when actual
following non-trivia input exists.

The final numeric fields that enter this state are the revision value, epoch
value, surface key and child order. Therefore the no-whitespace forms below all
accept their first number and then report `PSP_MISSING_SEMICOLON`:

| Input | Result | Exact span |
| --- | --- | --- |
| `revision 1+1;` | `PSP_MISSING_SEMICOLON` | `[plus_start, plus_start)` |
| `epoch 0+1;` | `PSP_MISSING_SEMICOLON` | `[plus_start, plus_start)` |
| `surface 1 root 1 key 1+1;` | `PSP_MISSING_SEMICOLON` | `[plus_start, plus_start)` |
| `child 2 order 1+1;` | `PSP_MISSING_SEMICOLON` | `[plus_start, plus_start)` |
| `revision 1-1;` | `PSP_MISSING_SEMICOLON` | `[minus_start, minus_start)` |
| `epoch 0-1;` | `PSP_MISSING_SEMICOLON` | `[minus_start, minus_start)` |
| `surface 1 root 1 key 1-1;` | `PSP_MISSING_SEMICOLON` | `[minus_start, minus_start)` |
| `child 2 order 1-1;` | `PSP_MISSING_SEMICOLON` | `[minus_start, minus_start)` |

Whitespace does not change ownership. `surface 1 root 1 key 1 +1;` and
`surface 1 root 1 key 1 -1;` report at the zero-width start of `+` and `-`,
respectively. `surface 1 root 1 key 1 1+1;` reports at
`[second_number_start, second_number_start)`, not at the internal `+`, because
the parser stops before classifying `1+1`. `surface 1 root 1 key 1 Root;`
likewise reports at `[Root_start, Root_start)`.

When the numeric field has not yet been accepted, numeric diagnostics remain
authoritative. Thus `revision +1;`, `revision -1;`, `epoch +1;`,
`surface 1 root 1 key +1;` and `child 2 order -1;` report
`PSP_INVALID_NUMBER` over the complete signed candidate. In contrast, after a
terminal field has been accepted, the parser is no longer in a new numeric
position:

```text
sign at start of required numeric field
    -> signed numeric candidate
    -> PSP_INVALID_NUMBER

sign after accepted final numeric field
    -> semicolon required
    -> PSP_MISSING_SEMICOLON
```

Completed non-terminal numeric fields retain the adjacent sign behavior. The
numeric-field ownership matrix is:

| Numeric field | Next required symbol after accepted value | `1+1` result |
| --- | --- | --- |
| revision value | `;` | `PSP_MISSING_SEMICOLON` at zero-width `+` start |
| epoch value | `;` | `PSP_MISSING_SEMICOLON` at zero-width `+` start |
| surface ID | `root` | `PSP_UNEXPECTED_TOKEN` on one-byte `+` |
| surface root ID | `key` | `PSP_UNEXPECTED_TOKEN` on one-byte `+` |
| surface key | `;` | `PSP_MISSING_SEMICOLON` at zero-width `+` start |
| node ID | `role` | `PSP_UNEXPECTED_TOKEN` on one-byte `+` |
| node key | `{` | `PSP_UNEXPECTED_TOKEN` on one-byte `+` |
| child ID | `order` | `PSP_UNEXPECTED_TOKEN` on one-byte `+` |
| child order | `;` | `PSP_MISSING_SEMICOLON` at zero-width `+` start |

For minus forms in the same positions, a semicolon-terminal field reports
`PSP_MISSING_SEMICOLON`, while a non-terminal field reports
`PSP_UNEXPECTED_CHAR` on the one-byte `-`. For example,
`surface 1+1 root 10 key 1;`, `surface 1 root 10+1 key 1;`,
`node 1+1 role root key 1 {}`, `node 1 role root key 1+1 {}` and
`child 2+1 order 0;` report `PSP_UNEXPECTED_TOKEN` on `+`. Their minus
counterparts report `PSP_UNEXPECTED_CHAR` on `-`.

The parser stops at that first failure and does not inspect trailing digits.
At a field start, `child 2 order +;` still reports `PSP_UNEXPECTED_TOKEN` on
the one-byte `+`, and `child 2 order -;` still reports
`PSP_UNEXPECTED_CHAR` on the one-byte `-`.

After the complete projection document, every non-trivia candidate reports
`PSP_UNEXPECTED_TOKEN`, including `Background`, `root-name` and `9root`, unless
lexical processing first encounters a character owned by
`PSP_UNEXPECTED_CHAR`. Other fixed production positions likewise use an
applicable dedicated diagnostic or `PSP_UNEXPECTED_TOKEN`, unless that position
specifically owns an identifier or numeric token class.

The normative decision table is:

| Parser context | Candidate | Result |
| --- | --- | --- |
| role identifier | valid `[a-z][a-z0-9_]*` | accept syntactically |
| role identifier | malformed unquoted candidate | `PSP_INVALID_IDENTIFIER` |
| role identifier | quoted role candidate | `PSP_INVALID_IDENTIFIER` |
| role identifier | forbidden non-ASCII scalar | `PSP_UNEXPECTED_CHAR` |
| clause-list entry | valid unreserved identifier | `PSP_UNKNOWN_CLAUSE` |
| clause-list entry | recognized keyword | landed P2 legal/wrong-context rule |
| clause-list entry | malformed identifier candidate | `PSP_INVALID_IDENTIFIER` |
| clause-list entry | decimal-only numeric candidate | `PSP_UNEXPECTED_TOKEN` |
| fixed keyword position | non-matching candidate | `PSP_UNEXPECTED_TOKEN` |
| numeric position | sign immediately followed by ASCII digit | `PSP_INVALID_NUMBER` over the complete signed numeric candidate |
| numeric position | digit-start candidate with non-decimal content | `PSP_INVALID_NUMBER` |
| numeric position | ASCII word candidate beginning with letter or underscore | `PSP_UNEXPECTED_TOKEN` |
| numeric position | standalone `+` | `PSP_UNEXPECTED_TOKEN` over the one-byte plus token |
| numeric position | standalone `-` | `PSP_UNEXPECTED_CHAR` over the one-byte minus character |
| after complete document | any candidate | `PSP_UNEXPECTED_TOKEN` |
| any context | forbidden non-ASCII scalar | `PSP_UNEXPECTED_CHAR` |

The same candidate may receive a different diagnostic because the current
parser position owns a different token class:

```text
9root at role_identifier -> PSP_INVALID_IDENTIFIER
9root at numeric position -> PSP_INVALID_NUMBER
9root at clause-list entry -> PSP_INVALID_IDENTIFIER
9root after complete document -> PSP_UNEXPECTED_TOKEN
```

#### Unexpected-character and precedence boundary

`PSP_UNEXPECTED_CHAR` retains ownership of a UTF-8 BOM at byte zero, bare CR,
forbidden non-ASCII scalars outside comments, and ASCII characters not
recognized as part of the current candidate, delimiter, comment or existing
token boundary. An invalid ASCII character spans one byte; a forbidden
non-ASCII scalar spans all UTF-8 bytes of that scalar. A forbidden scalar is
never folded into `PSP_INVALID_IDENTIFIER`.

Where a numeric field is currently required, the numeric-context
signed-candidate rule groups an eligible sign before standalone sign
classification. Recognition does not make the signed form valid: it
deterministically selects `PSP_INVALID_NUMBER`. After a non-terminal numeric
field is accepted, standalone `+` is the one-byte incompatible punctuation
token defined above, while standalone `-` remains a one-byte
`PSP_UNEXPECTED_CHAR`. After a semicolon-terminal field is accepted, neither
sign is classified because the semicolon-required state owns its coordinate.

For `node 1 role røot key 1 {}`, lexical processing reports
`PSP_UNEXPECTED_CHAR` on the exact UTF-8 bytes of `ø`. It does not accept `r`
as a complete role and continue.

Effective precedence is:

1. `SourceTooLarge` input preflight;
2. the current parser-state structural obligation: when a completed
   semicolon-terminated declaration awaits `;`, test the next non-trivia source
   unit for `;` before assigning that unit a class; actual non-`;` input selects
   `PSP_MISSING_SEMICOLON`;
3. lexical boundaries and `PSP_UNEXPECTED_CHAR` conditions at coordinates not
   already owned by the semicolon-required state;
4. numeric-context signed-candidate recognition where a numeric field is
   currently required, selecting `PSP_INVALID_NUMBER` rather than accepting
   signed syntax;
5. other parser-state dedicated diagnostics: `PSP_UNEXPECTED_EOF`,
   `PSP_DUP_REVISION`, `PSP_DUP_EPOCH`, `PSP_UNSUPPORTED_VERSION`,
   `PSP_INVALID_NUMBER`, `PSP_NUMBER_OVERFLOW`, `PSP_ZERO_ID`, and
   `PSP_INVALID_IDENTIFIER`;
6. landed P2 clause-context selection between `PSP_UNKNOWN_CLAUSE` and
   `PSP_UNEXPECTED_TOKEN`;
7. semantic `PS_` validation after a complete parse.

```text
parser-state structural obligation may precede lexical classification
```

This is specific to the completed-declaration semicolon obligation. It does not
globally make parser diagnostics precede every lexical error.

Signed candidate recognition is diagnostic grouping, not acceptance. A
recognized signed candidate always reaches the dedicated
`PSP_INVALID_NUMBER` branch before overflow or zero-ID interpretation.

When a semicolon is still required, the next source unit does not begin a new
clause-list entry or token-classification context. For
`surface 1 root 1 key 1 Background blue;`, the parser reports
`PSP_MISSING_SEMICOLON` with zero-width span
`[Background_start, Background_start)`, not `PSP_INVALID_IDENTIFIER`,
`PSP_UNKNOWN_CLAUSE` or `PSP_UNEXPECTED_TOKEN`.

The same precedence applies to `surface 1 root 1 key 1 +1;`,
`surface 1 root 1 key 1 -1;` and `surface 1 root 1 key 1 1+1;`. The surface
fields are complete, so the parser reports `PSP_MISSING_SEMICOLON` at the
zero-width start of `+`, `-` or the second `1`, respectively. None of those
following units begins a new numeric position.

```text
unfinished production != clause-list entry
```

#### Identifier diagnostic spans and fail-fast behavior

For `PSP_INVALID_IDENTIFIER`, an unquoted candidate spans the complete maximal
candidate. A closed quoted role candidate spans from its opening quote through
its closing quote. An unterminated quoted role candidate spans from its opening
quote to the line-ending position or EOF. Surrounding trivia is excluded.

For `PSP_INVALID_NUMBER` on a signed numeric candidate, the span covers the
sign byte and the complete following maximal digit-start numeric candidate.
For `PSP_UNEXPECTED_TOKEN` on standalone `+`, the span is the exact one-byte
range `[plus_start, plus_start + 1)`. For `PSP_UNEXPECTED_CHAR` on standalone
`-`, the span is the exact one-byte range
`[minus_start, minus_start + 1)`.

Every span retains the caller-supplied `SourceId`, half-open UTF-8 byte offsets
and representable `u32` endpoints. No diagnostic fabricates line/column
positions, character indices or partial-AST provenance.

P3 preserves fail-fast parsing. The first parser-owned failure produces
exactly one PSP diagnostic, no AST and no PS validation. The parser performs no
recovery, synchronization, token skipping, malformed-candidate
reinterpretation, later-semicolon scan or multiple-diagnostic emission. A
malformed identifier candidate is diagnosed as one maximal candidate; it is
not split and parsing does not continue.

#### P3 non-goals and authority boundary

P3 specification is not lexer implementation, parser implementation,
diagnostic Rust implementation or public API. It creates no token enum, string
syntax, escape processing, role authority, capability, admission, runtime
loading or activation. Tokenization does not grant authority. Gate D remains
closed and production promotion remains unauthorized.

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
15. unknown top-level clause: `projection v0 { revision 0; epoch 0; background blue; }` -> `PSP_UNKNOWN_CLAUSE` on the exact `background` token
16. missing semicolon before adjacent plus: `projection v0 { revision 0; epoch 0; node 1 role text key 1 { child 2 order 1+1; } }` -> `PSP_MISSING_SEMICOLON` with zero-width span `[plus_start, plus_start)` because `order` is the final field of `child_decl`
16a. signed negative decimal: `projection v0 { revision 0; epoch 0; node 1 role text key 1 { child 2 order -1; } }` -> `PSP_INVALID_NUMBER` on complete `-1`
16b. signed positive decimal: `projection v0 { revision 0; epoch 0; node 1 role text key 1 { child 2 order +1; } }` -> `PSP_INVALID_NUMBER` on complete `+1`
17. unknown node-body clause: `projection v0 { revision 0; epoch 0; node 1 role text key 1 { width 100; } }` -> `PSP_UNKNOWN_CLAUSE` on the exact `width` token
18. unknown word in fixed node header: `projection v0 { revision 0; epoch 0; node 1 role text bind val key 1 {} }` -> `PSP_UNEXPECTED_TOKEN` on the exact `bind` token because `key` is required
19. known keyword at top-level declaration-list entry: `projection v0 { revision 0; epoch 0; child 2 order 0; }` -> `PSP_UNEXPECTED_TOKEN` on the exact `child` token
20. known keyword at node child-list entry: `projection v0 { revision 0; epoch 0; node 1 role text key 1 { surface 2 root 2 key 2; } }` -> `PSP_UNEXPECTED_TOKEN` on the exact `surface` token
21. unknown word where `root` is required: `projection v0 { revision 0; epoch 0; surface 1 background 2 key 1; }` -> `PSP_UNEXPECTED_TOKEN` on the exact `background` token
22. unknown word after complete document: `projection v0 { revision 0; epoch 0; } background blue;` -> `PSP_UNEXPECTED_TOKEN` on the exact `background` token
23. missing semicolon before unknown word: `projection v0 { revision 0; epoch 0; surface 1 root 1 key 1 background blue; }` -> `PSP_MISSING_SEMICOLON` with zero-width span `[background_start, background_start)`
24. known keyword where `root` is required: `projection v0 { revision 0; epoch 0; surface 1 revision 2 key 1; }` -> `PSP_UNEXPECTED_TOKEN` on the exact `revision` token, not `PSP_DUP_REVISION`
25. uppercase role identifier: `projection v0 { revision 0; epoch 0; node 1 role Root key 1 {} }` -> `PSP_INVALID_IDENTIFIER` on the exact `Root` candidate
26. leading-digit role identifier: `projection v0 { revision 0; epoch 0; node 1 role 9root key 1 {} }` -> `PSP_INVALID_IDENTIFIER` on the exact `9root` candidate
27. leading-underscore role identifier: `projection v0 { revision 0; epoch 0; node 1 role _root key 1 {} }` -> `PSP_INVALID_IDENTIFIER` on the exact `_root` candidate
28. hyphenated role identifier: `projection v0 { revision 0; epoch 0; node 1 role root-name key 1 {} }` -> `PSP_INVALID_IDENTIFIER` on the exact `root-name` candidate
29. dotted role identifier: `projection v0 { revision 0; epoch 0; node 1 role root.name key 1 {} }` -> `PSP_INVALID_IDENTIFIER` on the exact `root.name` candidate
30. namespace-separated role identifier: `projection v0 { revision 0; epoch 0; node 1 role root::name key 1 {} }` -> `PSP_INVALID_IDENTIFIER` on the exact `root::name` candidate
31. quoted role identifier: `projection v0 { revision 0; epoch 0; node 1 role "root" key 1 {} }` -> `PSP_INVALID_IDENTIFIER` on the complete `"root"` candidate
32. malformed top-level clause candidate: `projection v0 { revision 0; epoch 0; Background blue; }` -> `PSP_INVALID_IDENTIFIER` on the exact `Background` candidate
33. malformed node-body clause candidate: `projection v0 { revision 0; epoch 0; node 1 role text key 1 { width-size 100; } }` -> `PSP_INVALID_IDENTIFIER` on the exact `width-size` candidate
34. malformed numeric candidate: `projection v0 { revision 0; epoch 0; surface 9root root 1 key 1; }` -> `PSP_INVALID_NUMBER` on the exact `9root` candidate
35. uppercase initial keyword: `Projection v0 {}` -> `PSP_UNEXPECTED_TOKEN` on the exact `Projection` candidate
36. malformed post-document candidate: `projection v0 { revision 0; epoch 0; } Root` -> `PSP_UNEXPECTED_TOKEN` on the exact `Root` candidate
37. missing semicolon before malformed candidate: `projection v0 { revision 0; epoch 0; surface 1 root 1 key 1 Background blue; }` -> `PSP_MISSING_SEMICOLON` at `[Background_start, Background_start)`

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

The original #1500 contract slice had these implementation non-goals:

no parser implementation in the #1500 contract slice;
no lexer implementation in the #1500 contract slice.

Those historical slice boundaries do not describe the current repository
state after #1507 and #1508. The following non-goals remain current:

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
