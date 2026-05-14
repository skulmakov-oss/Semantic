# Semantic Hello Implementation Readiness

Status: readiness checklist draft for `#477`

See also:

- [`semantic_hello_world_shape.md`](semantic_hello_world_shape.md)
- [`semantic_lexicon_density_closeout.md`](semantic_lexicon_density_closeout.md)

## 1. Purpose

This document prepares `#477` implementation readiness after `#478` Surface
Audit and `#479` Lexicon/Density closeout.

This is not implementation.
This is not grammar finalization.
This is not README/example alignment.
It is a readiness gate before any implementation PR.

## 2. Non-Goals

This document does not:

- change grammar
- change parser or typechecker behavior
- implement runtime / effect behavior
- change capability / effect admission
- implement `observe`
- implement `print`
- implement `entry` / `state` / `require` / `complete`
- implement Hello World
- implement a formatter
- rewrite README content
- rewrite examples
- rewrite fixtures
- rewrite tests
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Updated Post-#478 / #479 Constraints

- `print` is rejected-as-canonical.
- `observe` is preferred directional vocabulary, not executable yet.
- `entry`, `state`, `require`, `complete` are directional, not executable yet.
- `fn main`, `return`, and `assert` remain bridge-only where the current
  frontend requires them.
- `stdout` and generic `I/O` are not canonical source vocabulary.
- controlled observation must not become general I/O.
- Hello World remains required as proof of life.
- recommended shape is non-executable and not grammar-final.

## 4. Recommended Future Canonical Direction

```semantic
entry HelloWorld {
    state boot: quad = T;
    require boot == T;
    observe "Hello, World!";
    complete T;
}
```

Label: recommended canonical direction for later implementation planning.
Not executable.
Not grammar-final.

## 5. Legacy Fallback Status

```semantic
fn main() {
    print("Hello, World!");
    return;
}
```

Label: rejected as canonical.
May only be considered as bridge fallback or migration fixture if explicitly
scoped later.

## 6. Readiness Checklist Before Implementation

- grammar decision for `entry`
- grammar decision for `state`
- grammar decision for `require`
- grammar decision for `observe`
- grammar decision for `complete`
- type rules for observation payload
- text-only first wave or not
- capability / effect policy for observation sink
- audit policy for observation event
- determinism policy for observation order
- SemCode representation
- verifier admission rule
- VM / runtime observation handling
- CLI observation sink behavior
- diagnostics for invalid observation
- positive fixtures
- negative fixtures
- golden SemCode / run-smc stability tests
- CTF impact check
- docs / README alignment plan after implementation
- bridge migration plan if legacy syntax remains temporarily

## 7. Proposed Implementation PR Sequence

Proposed sequence only:

- `M-HELLO-1 — docs(grammar): decide Hello entry/observe grammar slice`
- `M-HELLO-2 — tests(hello): add pending/admission fixtures for Hello shape`
- `M-HELLO-3 — sema/ir: admit controlled observation surface`
- `M-HELLO-4 — semcode/vm: lower and execute controlled observation`
- `M-HELLO-5 — verify/runtime: enforce observation capability/audit policy`
- `M-HELLO-6 — tests(hello): add golden check/compile/verify/run/run-smc
  coverage`
- `M-HELLO-7 — docs(examples): align public Hello World docs`
- `M-HELLO-8 — close #477`

## 8. Blocking Conditions

Do not begin implementation until:

- maintainer explicitly accepts the shape
- grammar slice is scoped
- verifier / runtime impact is scoped
- observation / capability policy is scoped
- CTF impact lane is prepared

## 9. Relationship to `#477`

This PR does not close `#477`.
This PR prepares `#477` for implementation planning.
`#477` remains open after this readiness PR.

## 10. Acceptance Checklist

- readiness checklist added
- post-#478 / #479 constraints recorded
- recommended shape recorded as non-executable
- legacy shape rejected as canonical
- implementation PR sequence proposed
- blocking conditions recorded
- `#477` remains open
- no code/test/fixture changes
- no grammar changes
- no Hello World implementation
- no `print` / `observe` implementation
- no README/examples rewrite
- no Linguist readiness
- no UI / Workbench / I70
