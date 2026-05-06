# POST-UI Admission Checklist

Status: Draft
Track: POST-UI
Purpose: implementation gate for future UI PRs

## 1. Why this checklist exists

UI is a side-effect boundary.

It must not bypass:

- verifier admission;
- capability checks;
- runtime lifecycle checks;
- host ABI discipline.

## 2. PR gates

### Gate UI-A - Contract gate

Required before code changes:

- `ui_ownership_map.md` exists.
- `ui_contract_map.md` exists.
- `ui_abi_capability_admission.md` exists.
- first-slice operations are listed.
- non-goals are explicit.

### Gate UI-B - Capability gate

Before executable UI operation support:

- every UI operation maps to one `UiCapabilityKind`;
- missing capability fails closed;
- denial carries operation context;
- default/gate manifests do not silently include UI capabilities.

### Gate UI-C - Verifier gate

Before VM execution support:

- SemCode can expose required UI admission metadata;
- verifier rejects unknown UI operation ids;
- verifier rejects missing UI capabilities;
- verifier keeps POST-UI outside stable profile.

### Gate UI-D - Runtime lifecycle gate

Before platform backend support:

- window lifecycle state is explicit;
- frame lifecycle state is explicit;
- event polling state is explicit;
- draw/frame submission cannot bypass lifecycle.

### Gate UI-E - Audit/diagnostics gate

Before user-facing release:

- denials are diagnosable;
- lifecycle violations are diagnosable;
- host faults are separated from capability denials;
- replay/event stream assumptions are documented.

## 3. Implementation order

```text
B1 ownership docs
  ↓
B2 ABI/capability admission docs
  ↓
B3 prom-ui contract refinement
  ↓
B4 prom-cap UI admission tests
  ↓
B5 verifier UI admission plan
  ↓
B6 runtime lifecycle state plan
  ↓
implementation only after gates are clear
```

## 4. Stop rules

Stop implementation if:

- UI operation bypasses `prom-cap`;
- UI runtime requires parser/typechecker knowledge;
- VM stores native platform UI handles directly;
- Workbench becomes owner of UI application contract;
- stable-v1 docs imply UI is already shipped.
