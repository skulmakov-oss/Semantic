# Option/Result Ownership Parity Audit

## Executive Verdict

**Full Parity is Achievable via Direct Reuse.**
An audit of the current Semantic language codebase reveals that Option and Result are already deeply integrated into the ADT matching infrastructure. The parser natively parses Option::Some(x) as Expr::AdtCtor and Option::Some(ref value) as MatchPattern::Adt. The frontend canonicalizes them into MatchFamilySpec, generating PatternPathElem::Variant and PatternPathElem::VariantField exactly as it does for custom ADTs. The lowering layer (legacy_lowering.rs) already delegates Option/Result extraction to IrInstr::AdtTag and IrInstr::AdtGet, and emits OwnershipPathEventKind using the identical dt_payload method.

Therefore, Option and Result automatically inherit the AdtPayload ownership path capabilities recently built for PCC-ADT Payload Ownership Slice. The primary work for OR-1 through OR-5 will be writing missing positive/negative E2E test coverage specifically asserting Option/Result forms, rather than inventing new ownership rules or adding new opcodes.

---

## Current Support Matrix

| Subsystem | Option / Result Implementation | ADT Machinery Reuse |
| :--- | :--- | :--- |
| **Parser** | Parses Option::Some(x) / Result::Ok(x) | Natively parses as Expr::AdtCtor and MatchPattern::Adt. |
| **Typecheck** | infer_std_form_ctor_type intercepts Option/Result | Yes, unifies via MatchFamilySpec, extracting PatternPathElem::VariantField(idx). |
| **Conflict Tracking** | Same overlap checks | Yes, checks prefix overlaps and sibling variances exactly as ADT. |
| **Lowering (IR)** | Handled in legacy_lowering.rs (dt_name == "Option") | Yes, lowers to IrInstr::AdtTag and IrInstr::AdtGet. |
| **Lowering (Ownership)** | ownership_events.push | Yes, uses path.adt_payload(adt_pat.variant_name, idx) identical to ADT. |
| **SemCode Format** | Represented transparently | Uses AccessPathComponent::AdtPayload component format without distinction. |
| **Verifier** | Blind to standard library origins | Accepts standard AdtPayload validation rules safely. |
| **VM** | Represented via Value::Adt | Uses the native sm-vm ownership tracking mapped to AdtCarrier<Value>. |

---

## Reuse Opportunities from ADT Payload Ownership

1. **Vocabulary (AdtPayload component)**: Option and Result payloads map 1:1 onto AdtPayload { variant, index }. No new path component (e.g. OptionPayload) is necessary.
2. **Overlap Semantics**: Since the VM tracks overlap via the AdtPayload index and variant namespace, conflicts on Some versus None or Ok versus Err natively reject same-path borrowing and allow separate-path borrowing. 
3. **Verifier Negative Security**: Verifier rules preventing out-of-bounds variants and malformed payloads automatically protect Option and Result representations since they appear as standard ADT sequences at the SemCode boundary.

---

## Gaps

While the internal representations map perfectly to ADTs, the test coverage and qualification limits are currently missing:
1. **Tests**: There are no Option-specific or Result-specific sm-front overlap tests asserting that ef value within Option::Some(ref value) creates a borrowing conflict.
2. **E2E Golden Coverage**: We need dedicated sm-verify / golden execution files specifically pushing Option/Result payloads through the entire runtime boundary to prevent regressions.
3. **7hell Local Gate**: 7hell currently guards standard ADTs but should explicitly run an Option/Result integration smoke test to guard standard library forms.

---

## Proposed OR-1..OR-5 PR Plan

Since the core pipeline already maps Option/Result to AdtPayload, the subsequent steps will focus on *qualification* and *test-driven hardening*:

- **OR-1 / OR-2 — Typecheck & Lowering Validation**: Add dedicated frontend tests to crates/sm-front/tests/ asserting that Option and Result payload paths trigger ownership conflicts properly (positive/negative typecheck tests).
- **OR-3 — E2E Positive Golden**: Create a golden program (e.g., 	est_e2e_option_result_ownership.sm) that demonstrates source -> SemCode -> VM success for Option and Result.
- **OR-4 — Negative VM Coverage**: Add test cases to sm-vm to verify that Option overlapping borrows natively reject correctly in the VM logic.
- **OR-5 — 7hell Update**: Include the Option/Result golden file inside the 7hell mini-runner to securely gate this parity.

---

## Explicit Non-Goals

1. Adding any new VM representation for Option/Result (e.g., Value::Option). We will stick to Value::Adt.
2. Changing the SemCode format or adding new OwnershipPathEventKind variants.
3. Adding generic variance checks or polymorphic standard library traits beyond standard Option/Result payload binding.
4. Altering the MatchPattern::Adt parser representation.
