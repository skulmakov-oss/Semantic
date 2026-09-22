# Canonical Diagnostic Carrier Contract

Status: ARCHITECTURE / CONTRACT DECISION — IMPLEMENTATION NOT AUTHORIZED
Repository: `skulmakov-oss/Semantic`
Authoritative Baseline: `5c8167994af43aedafbaf825e5327f6226a1d932`
Parent Issue: `#1580 — SSF-09 Diagnostics and editor baseline`
Authority References:
- `docs/roadmap/stable_foundation/ssf09_diagnostic_authority_decision.md` (Decisions B/C/D and Position B addendum via PR #1948)
- `docs/architecture/dependency_boundary_rules.md`
- `docs/architecture/module_ownership_map.md`

---

## 1. Decision Baseline & Governance Context

This document establishes the normative architectural contract for the canonical internal diagnostic carrier of the Semantic compiler and execution stack.

Following the diagnostic authority baseline established in `docs/roadmap/stable_foundation/ssf09_diagnostic_authority_decision.md`:
- **Decision B**: Diagnostic identity authority belongs to producer stages; external diagnostic representations must project from an internal canonical carrier rather than exposing ad-hoc producer types.
- **Decision C**: Canonical file identity authority is decoupled from host filesystem paths and platform encodings.
- **Decision D**: Source range authority requires genuine UTF-8 byte offsets `[start, end)`.
- **Carrier Ownership Decision (Position B)**: Frozen in PR #1948, establishing that the canonical internal carrier must reside in a dedicated leaf crate within the `Shared Foundation / Contract Leaf` tier.

This contract specifies the semantic model, type boundaries, invariant laws, and migration DAG for that carrier.

**IMPLEMENTATION STATUS**:
```text
CONTRACT: FROZEN
IMPLEMENTATION: NOT AUTHORIZED
```
Under the current `.harness/current.task.yaml` envelope (`dependency_changes: false`), the crate does not exist, no source files under `crates/sm-diagnostic/**` may be written, and root Cargo workspace membership must not be modified. Crate creation and dependency authorization require a separate, explicit repository-owner governance action.

---

## 2. Owner, Tier, and Crate Identity

- **Crate Name**: `sm-diagnostic`
- **Filesystem Path**: `crates/sm-diagnostic`
- **Architectural Tier**: `Shared Foundation / Contract Leaf` (precedent: `sm-format`)
- **Status**: Name and path are contract-frozen; implementation is pending.

`sm-diagnostic` is a pure contract leaf. It is neither a Construction crate nor an Execution crate. It owns only the data definitions and core invariants of internal diagnostic carriers.

---

## 3. Dependency Law

`sm-diagnostic` must remain strictly dependency-foundational:

```text
                  sm-diagnostic
             (Contract Leaf / Tier 0)
                 ▲     ▲     ▲
                 │     │     │
         Construction Execution Host/Tooling
```

1. **Forbidden Internal Dependencies**: `sm-diagnostic` MUST NOT depend on:
   - `sm-front`
   - `sm-sema`
   - `sm-ir`
   - `sm-emit`
   - `sm-verify`
   - `sm-runtime-core`
   - `sm-vm`
   - `smc-cli`
   - `prom-*`
   - legacy compatibility perimeter crates
2. **Workspace Dependency Expectation**: For Phase `C0`, `sm-diagnostic` has **zero internal workspace dependencies**.
3. **Environment & External Dependencies**:
   - Must be `#![no_std]` with `extern crate alloc`.
   - Allowed allocations: `alloc::string::String`, `alloc::vec::Vec`, `alloc::boxed::Box`, `alloc::borrow::Cow`.
   - **No `std::fs`**, **no `std::path`**, **no `std::net`**, **no OS strings**.
   - **No `serde`** dependency (no `Serialize`/`Deserialize` derives).
   - **No LSP types**, no wire protocols, no JSON schema types.

---

## 4. Layer Separation

The compilation and diagnostic pipeline is governed by three decoupled, independent layers:

```text
stage-native diagnostic
        ↓ producer-owned lossless projection
canonical internal diagnostic carrier (sm-diagnostic)
        ↓ explicit downstream projection
versioned external machine schema
```

1. **Stage-Native Diagnostics**: Types native to compiler phases (`FrontendError`, `SemanticDiagnostic`, `VerificationDiagnostic`, `RejectReport`, `RuntimeTrap`). They serve immediate compiler phase requirements.
2. **Canonical Internal Carrier**: Universal neutral representation in `sm-diagnostic`. Preserves semantics losslessly across stages without pulling in stage implementations.
3. **Versioned External Machine Schema**: Wire/JSON/LSP representation owned by downstream tooling.

**Anti-pattern rejection**:
$$\text{Stage Enum} \neq \text{Canonical Carrier} \neq \text{JSON Schema} \neq \text{LSP Diagnostic}$$
The internal carrier must never derive external wire serialization directly.

---

## 5. Canonical Semantic Model

The canonical carrier defines nine structural fields:

```text
code:              1..1 (DiagnosticCode)
severity:          1..1 (DiagnosticSeverity)
family:            1..1 (DiagnosticFamily)
message:           1..1 (DiagnosticMessage)
source_context:    0..1 (Option<SourceContext>)
related_locations: 0..N (Vec<RelatedLocation>)
notes:             0..N (Vec<DiagnosticNote>)
fix_proposal:      0..1 (Option<FixProposal>)
cause:             0..1 (Option<Box<DiagnosticCause>>)
```

### 5.1 DiagnosticCode
- **Concept**: Opaque textual identifier owned strictly by the producer stage (e.g. `"E0101"`, `"S0404"`).
- **Invariants**:
  - Valid UTF-8.
  - Not empty (`!ident.is_empty()`).
  - Not whitespace-only (`!ident.trim().is_empty()`).
  - Byte-exact and case-sensitive.
  - Producer-owned: never trimmed, normalized, case-folded, or inferred by carrier or downstream adapters.
  - No synthetic regex restriction (e.g. `^E\d+$` is forbidden; verifier and runtime may use different taxonomies).
- **Illustrative Signature**:
  ```rust
  pub struct DiagnosticCode(Cow<'static, str>);
  ```

### 5.2 Severity
- **Concept**: Two-state severity vocabulary representing compiler and verifier findings.
- **Invariants**:
  ```rust
  pub enum DiagnosticSeverity {
      Error,
      Warning,
  }
  ```
  - **No `Info` or `Hint`**: Current workspace evidence proves only `Error` and `Warning`. Expanding vocabulary requires separate contract justification.
  - **Producer Authority**: The producer stage determines severity. The carrier preserves it; downstream adapters must never infer severity from error codes, prefixes, or message strings.
  - For producers lacking native severity (e.g., `sm-front`, `sm-verify`), adaptation is blocked until producer-level repair occurs.

### 5.3 Family
- **Concept**: Identifies the originating canonical compiler or runtime stage.
  $$\text{family} \equiv \text{originating canonical diagnostic stage}$$
- **Invariants**:
  ```rust
  pub enum DiagnosticFamily {
      Frontend,
      Semantic,
      Verification,
      Runtime,
  }
  ```
  - `family` is NOT a surface grammar (e.g., `Syntax` is an internal `Frontend` concern, not a family).
  - `family` is NOT a policy category (e.g., `HostPolicy` is not a family).
  - `family` is NOT a crate rendering or CLI display category.
  - `Runtime` applies only to runtime execution failures explicitly admitted as canonical diagnostics.

### 5.4 SourceId
- **Concept**: An opaque internal/session token identifying an input source.
- **Invariants**:
  ```rust
  pub struct SourceId(u64);
  ```
  - Inner storage is private; raw direct construction (e.g. `SourceId(42)`) by downstream consumers is prohibited.
  - Minting and mapping authority belongs exclusively to a future canonical source registry/context.
  - Carries no host filesystem path, URI, or module path semantics.
  - Completely unrelated to legacy `FileId` representations in the non-canonical compatibility perimeter.
  - A rootless standalone input may possess an internal `SourceId` while having no external package identity.
  - Stdin/synthetic/virtual input sources remain explicitly unfrozen per Decision C; no `Virtual` or `Synthetic` variants exist.

### 5.5 ByteOffset and SourceRange
- **Concept**: Exact zero-based UTF-8 byte span within a source.
- **Invariants**:
  ```rust
  pub struct ByteOffset(u64);

  pub struct SourceRange {
      start: ByteOffset,
      end: ByteOffset,
  }
  ```
  - `ByteOffset` inner storage is private; construction from machine-sized offsets must be checked (`u64::try_from(value)`). No truncating casts (`as u64`) are canonical behavior.
  - `SourceRange::new(start, end)` enforces `start.0 <= end.0`, returning `None` if inverted.
  - A zero-width range `[n, n)` is valid only when producer authority explicitly proves a genuine zero-width anchor.
  - Legacy `FrontendError.pos` does NOT prove a zero-width range `[pos, pos)`.
  - Byte offsets in artifacts, bytecode instruction offsets, and VM program counters are NOT canonical source ranges.

### 5.6 SourceContext
- **Concept**: Canonical location binding a `SourceId` to an optional `SourceRange`.
- **Invariants**:
  ```rust
  pub struct SourceContext {
      pub source: SourceId,
      pub range: Option<SourceRange>,
  }
  ```
  - Expresses:
    1. Known source with known byte range (`range: Some(...)`).
    2. Known source with absent byte range (`range: None`).
    3. Entire source context absent (`source_context = None` on `Diagnostic`).
  - **No `point_offset`**: Legacy points are not canonical coordinates.
  - Line and column coordinates are derived display presentations, never canonical carrier coordinates.

### 5.7 Primary Message
- **Concept**: Human-readable semantic diagnostic text (`DiagnosticMessage`).
- **Invariants**:
  - Must not be empty or whitespace-only.
  - Contains semantic diagnostic text only; must not contain presentation formatting (ANSI escapes, caret blocks, path prefixes, line/column prefixes).
  - Message text does not constitute diagnostic identity and cannot be used to infer code, family, or severity.

### 5.8 Related Locations
- **Concept**: 0..N secondary source locations relevant to the diagnosis (e.g. prior declaration, conflicting definition, opening delimiter).
- **Invariants**:
  ```rust
  pub struct RelatedLocation {
      pub context: SourceContext,
      pub message: Option<String>,
  }
  ```
  - Each related location carries its own `SourceContext`.
  - Cross-file evidence is naturally supported through distinct `SourceId` values.
  - Preserves producer emission order; no canonical sorting.

### 5.9 Diagnostic Notes
- **Concept**: 0..N structured supplemental explanations or contextual hints.
- **Invariants**:
  ```rust
  pub struct DiagnosticNote {
      pub message: String,
  }
  ```
  - Notes are distinct from primary severity and distinct from `RelatedLocation`.
  - In this minimal model, notes do not carry source anchors.

### 5.10 Fix Proposals
- **Concept**: 0..1 optional structured proposal providing human/procedural guidance.
- **Invariants**:
  ```rust
  pub struct FixProposal {
      pub message: String,
  }
  ```
  - **Cardinality**: Strictly 0..1.
  - **No Machine Claims**: Current repository evidence provides no machine-applicable diff or replacement support. This model asserts no replacement text, no `TextEdit`, no `WorkspaceEdit`, and no machine-safety guarantees.
  - It is a typed guidance field distinct from ordinary notes.

### 5.11 Structured Cause
- **Concept**: 0..1 causal link preserving nested lower-level diagnostics structurally.
- **Invariants**:
  ```rust
  pub enum DiagnosticCause {
      Diagnostic(Box<Diagnostic>),
      Report(Vec<Diagnostic>),
  }
  ```
  - Specifically designed to represent causal nesting such as `RuntimeError::VerifierRejected(RejectReport)`.
  - **Strict Structure**: No `DiagnosticCause::Message(String)` escape hatch is permitted. Flattening lower-level structured diagnostics into a text string is an architectural contract violation.
  - Preserves exact child diagnostic count, producer ordering, and child context.

---

## 6. Ordering, Equality, and Deduplication Laws

1. **Ordering & Determinism**:
   - Producer observable emission order is preserved unconditionally across primary diagnostics, related locations, notes, and report causes.
   - The carrier performs no canonical sorting.
   - **Determinism Law**: Identical producer semantic input and identical proven context must produce structurally equal canonical diagnostics with deterministic observable collection order.
   - Rust in-memory representations are not an external byte ABI; in-memory byte stability is not asserted.
2. **Equality vs Identity**:
   - **Semantic Diagnostic Identity**: Defined by `(code, severity, family)` in conjunction with proven `SourceContext`.
   - **Structural Equality**: `PartialEq` / `Eq` covers all fields including notes, proposals, and nested causes.
3. **Deduplication**:
   - **Canonical Deduplication: NONE**.
   - The carrier leaf guarantees lossless multiplicity preservation. Emitted diagnostics are never discarded, deduplicated, or folded by the carrier.
   - Deduplication is exclusively a downstream presentation/consumer concern.

---

## 7. Producer Authority Gates & Migration DAG

Before any compiler stage is adapted into `sm-diagnostic`, its native authority must be verified or repaired.

```text
[ C0: sm-diagnostic primitive types crate ]
       │
       ├───► [ C1A: sm-front native authority repair ] ────► [ C1B: frontend adapter ]
       │
       ├───► [ C2: sm-sema -> sm-diagnostic adapter ]
       │
       ├───► [ C3A: sm-verify native authority repair ] ───► [ C3B: verifier adapter ]
       │
       ├───► [ C4: runtime explicit diagnostic admission ]
       │
       ▼
[ C5: Canonical human renderer / emitter ]
       │
       ▼
[ C6: Versioned external machine schema ]
```

### Stage Gates

- **Phase C0**: Create `sm-diagnostic` leaf crate containing only the pure models and checked constructors. No stage adapters.
- **Phase C1A (`sm-front` repair)**:
  - Must establish structured frontend diagnostic code authority.
  - Must establish explicit frontend severity authority.
  - Must establish genuine optional byte-range authority (`[start, end)`).
  - *Gate*: `sm-front -> sm-diagnostic` adaptation (**C1B**) is **BLOCKED** until C1A lands.
- **Phase C2 (`sm-sema` adapter)**:
  - May adapt directly using existing structured code (`&'static str`), `DiagLevel`, and semantic message.
  - Only proven source context may adapt; legacy `SourceMark` / default coordinates must not be fabricated into canonical locations.
- **Phase C3A (`sm-verify` repair)**:
  - Must establish producer-owned stable textual code tokens (cannot rely on enum `Debug` formatting).
  - Must establish producer-owned severity mapping.
  - *Gate*: `VerificationDiagnostic / RejectReport -> sm-diagnostic` adaptation (**C3B**) is **BLOCKED** until C3A lands.
- **Phase C4 (`runtime` admission)**:
  - Must explicitly admit which runtime failures project into canonical diagnostics.
  - Runtime trap codes and severities must be producer-owned.
  - Does not assume all `RuntimeTrap` variants become diagnostics (SSF-08 authority remains intact).
- **Phase C5**: Canonical human-readable renderer / CLI emitter.
- **Phase C6**: Versioned external machine schema (JSON, LSP).

---

## 8. Non-Fabrication Invariants

> **The Non-Fabrication Law**:
> If required semantic information is absent at the producer authority, adaptation is blocked or the optional canonical field remains absent. The carrier never invents semantic information to make an adapter convenient.

Explicitly forbidden behaviors:
1. Synthesizing placeholder codes (e.g. `E0000`, `UNKNOWN`, `SYNTAX`).
2. Guessing severity or family from error message text.
3. Converting legacy point position `FrontendError.pos` into a zero-width span `[pos, pos)`.
4. Converting `VerificationDiagnostic.offset` into a `SourceRange` (it is an artifact offset, not a source range).
5. Converting VM Program Counter into a `SourceRange`.
6. Using enum `Debug` string formatting as a stable diagnostic code.
7. Synthesizing empty or dummy `FixProposal` values.
8. Flattening `RejectReport` into a single text string.
9. Synthesizing `SourceId(0)` or `"<input>"` as an external canonical file identity.

---

## 9. Public API Contract & Qualification

When implemented, `sm-diagnostic` will be a workspace public Rust crate.

- It makes no crates.io release or stability claims.
- Its public API surface must be registered in and guarded by `tests/public_api_contracts.rs`.
- In accordance with #1700 governance lessons, public API snapshot guards must cover all public source items across modules, not merely top-level `lib.rs` re-exports.

---

## 10. Illustrative Rust Sketch

```rust
#![no_std]
extern crate alloc;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode(Cow<'static, str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidDiagnosticCodeError;

impl DiagnosticCode {
    pub fn try_from_static(ident: &'static str) -> Result<Self, InvalidDiagnosticCodeError> {
        if ident.is_empty() || ident.trim().is_empty() {
            Err(InvalidDiagnosticCodeError)
        } else {
            Ok(Self(Cow::Borrowed(ident)))
        }
    }

    pub fn try_from_string(ident: String) -> Result<Self, InvalidDiagnosticCodeError> {
        if ident.is_empty() || ident.trim().is_empty() {
            Err(InvalidDiagnosticCodeError)
        } else {
            Ok(Self(Cow::Owned(ident)))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticFamily {
    Frontend,
    Semantic,
    Verification,
    Runtime,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(u64);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffsetOverflow;

impl ByteOffset {
    pub fn try_from_usize(value: usize) -> Result<Self, OffsetOverflow> {
        u64::try_from(value).map(Self).map_err(|_| OffsetOverflow)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRange {
    start: ByteOffset,
    end: ByteOffset,
}

impl SourceRange {
    pub fn new(start: ByteOffset, end: ByteOffset) -> Option<Self> {
        if start.0 <= end.0 {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub fn start(&self) -> ByteOffset {
        self.start
    }

    pub fn end(&self) -> ByteOffset {
        self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start.0 == self.end.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceContext {
    pub source: SourceId,
    pub range: Option<SourceRange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticMessage(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyMessageError;

impl DiagnosticMessage {
    pub fn new(msg: impl Into<String>) -> Result<Self, EmptyMessageError> {
        let s = msg.into();
        if s.trim().is_empty() {
            Err(EmptyMessageError)
        } else {
            Ok(Self(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RelatedLocation {
    pub context: SourceContext,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticNote {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FixProposal {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticCause {
    Diagnostic(Box<Diagnostic>),
    Report(Vec<Diagnostic>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub family: DiagnosticFamily,
    pub message: DiagnosticMessage,
    pub source_context: Option<SourceContext>,
    pub related_locations: Vec<RelatedLocation>,
    pub notes: Vec<DiagnosticNote>,
    pub fix_proposal: Option<FixProposal>,
    pub cause: Option<Box<DiagnosticCause>>,
}
```
