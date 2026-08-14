# Rust-like / Logos Coherence Decision — SSF-02

Status: accepted SSF-02 decision; not a stable-release claim
Decision: Model B — declarative Logos profile
Evidence base: `8ce27c48ef4cb4301da99c81286b11052748e10f`

## Decision

Rust-like Semantic remains the only Stable Foundation executable source
profile. Logos is a separate experimental declarative profile for describing
systems, entities, laws, and policy-shaped `When` fragments.

```text
Rust-like source
  -> semantic checks -> function IR -> SemCode -> verifier -> VM

Logos source
  -> Logos parser -> Logos semantic checks -> LogosIrLaw inspection projection
```

There is no arrow from `LogosIrLaw` to SemCode. Adding one would be a new
versioned architecture decision and qualification program, not an inference
from the current projection.

## Why Model B

| Criterion | Model A: executable lowering | Model B: declarative profile |
|---|---|---|
| Current evidence | no Logos function-IR/SemCode/verifier/VM path | parser, semantic checks, projection, fixtures, and honest rejection exist |
| Determinism | would require defining executable meaning for text fragments | existing parse/order/projection behavior is deterministic |
| Verifier authority | requires a new admitted artifact mapping | preserves the single existing verifier-first executable path |
| Implementation cost | compiler, type, effect, source-map, compatibility, and runtime work | documentation, CLI honesty, and boundary qualification only |
| Compatibility risk | high; could accidentally create a second language authority | low; freezes the actual current boundary |

Model A is rejected for Stable Foundation because current `When` condition and
effect bodies are stored as text, and `LogosIrLaw` contains only name, priority,
and `When` count. Treating that summary as executable would invent semantics
and bypass the frozen Rust-like contract.

## Capability Inventory

| Capability | Rust-like | Logos |
|---|---|---|
| Parse | yes | yes |
| Semantic analysis | yes | yes, Logos-specific |
| Inspection IR | function IR | `LogosIrLaw` summary |
| SemCode production | yes | no; deterministic rejection |
| Verifier admission | yes, for produced SemCode | not applicable; no artifact |
| VM execution | yes after admission | no |
| Canonical evidence | check/compile/verify/run examples | dump-ast/dump-ir and rejection example |

## Profiles, Maturity, and Versions

- Rust-like contract: `semantic.foundation.source/1.2`;
  **Qualified limited release**, still not Published Stable.
- Logos contract: `semantic.logos.declarative/0.1`; **Experimental**.
- Parser admission envelope: `semantic.foundation` / `1.0`; this shared policy
  envelope does not equalize maturity or execution authority.
- SemCode version remains capability-derived and belongs only to the executable
  artifact path.

Changing Logos grammar, semantic meaning, or inspection projection
incompatibly requires a new Logos contract version. It does not change the
Rust-like source contract or SemCode version by implication.

## CLI Contract

| Command family | Rust-like | Logos |
|---|---|---|
| `dump-ast` | inspect | inspect `LogosProgram` |
| `dump-ir`, `hash-ir` | function IR | `--profile logos` inspection projection |
| `check` | executable-source analysis | not an admitted Logos workflow |
| `compile`, `dump-bytecode`, `hash-smc` | SemCode-producing | rejected before artifact emission |
| `verify`, `run`, `run-smc` | verifier-first artifact/source execution | no Logos artifact or execution path |

The CLI may accept `logos` as a parser value internally so inspection commands
can select it. Bytecode-producing usage text advertises only `auto|rust`.
Explicit or auto-detected Logos input still reaches the canonical deterministic
boundary rejection rather than a hidden fallback.

## Files, Packages, and Source Mapping

- A current source file is either Rust-like or Logos for a tool invocation.
- Mixed Rust-like/Logos source is unsupported and rejected as one file; the CLI
  does not split it or silently choose one authority.
- No cross-profile imports, implicit bindings, or Logos package ecosystem are
  admitted in Stable Foundation.
- SSF-05/06 own Rust-like project/package completion and may consume this
  decision but cannot invent a Logos execution route.
- `LogosLaw` and `LogosWhen` carry frontend source marks. The current
  `LogosIrLaw` inspection summary does not retain a general source map and is
  not a generated executable binding artifact.
- A future binder must be explicit, versioned, validated, provenance-bearing,
  and subordinate to verifier admission.

## Demonstrated Gaps and Bounded Resolution

The compiler already rejects Logos before SemCode emission, so no compiler,
verifier, VM, or rule-engine expansion is needed. SSF-02 resolves only:

1. the missing authoritative Model A/B decision;
2. the missing separate Logos contract/version and maturity statement;
3. bytecode-command help that previously advertised `logos` as a successful
   profile value;
4. missing exact regression evidence for SemCode rejection and mixed profiles.

## SSF-03 Entry Conditions

SSF-03 may start only after:

1. specs, CLI help, status matrix, and canonical example agree on Model B;
2. positive Logos parse/projection and negative SemCode/mixed-profile tests pass;
3. exact-head repository CI passes and the SSF-02 PR is merged;
4. issue #1573 records the merge SHA and closes;
5. a separate governance PR activates only SSF-03.

## External Explanation

Semantic executes Rust-like source through SemCode, the verifier, and the VM;
Logos is currently an experimental declarative profile that can be parsed,
checked on its own terms, and projected for inspection, but it is not executed.
