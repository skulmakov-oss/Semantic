# UI-DNA2-9B P2 Input Resource Preflight Correction

Status: PASS

## Baseline

- Main SHA: `0eede9391f6f5d1aaf446e94326b74797f1973d7`
- Branch: `ui-dna2/shell-player-input-preflight-correction`
- Origin PR: `#1524`
- Origin thread: `PRRT_kwDOROOm386SAqRT`
- Origin comment: `3608926265`
- Post-merge finding severity: P2
- Finding appeared after merge: YES
- Technical validity: ACCEPTED

## Finding

The merged order performed stable-target identity and replay-cursor checks
before validating input-side resource bounds. An oversized transition could
therefore cause per-target or replay traversal before deterministic rejection.

## Corrected order

1. bounded session check;
2. bounded lifecycle check;
3. bounded outer-envelope and stimulus-class check;
4. input-side resource preflight;
5. stable-target validation;
6. replay-cursor validation;
7. candidate calculation without commit;
8. candidate-state/output validation;
9. complete state commit or previous-state preservation;
10. bounded output publication and diagnostic emission cap.

## Oversized-input behavior

- disposition: `Rejected`;
- per-target traversal: NONE;
- replay traversal: NONE;
- previous `ShellLocalState`: PRESERVED;
- diagnostics: subject to the immutable stage-10 emission cap.

## Preserved boundaries

- Shell Player implementation: NOT AUTHORIZED
- `ProjectionPatch` application: NOT AUTHORIZED
- Gate D: CLOSED
- production promotion: NOT AUTHORIZED
- closeout: NOT COMPLETE
- next authorized implementation slice: NONE

## Validation

- harness: PASS
- claim-boundary guard: PASS
- POST-UI fixture guard: PASS
- fast 7hell: PASS
- Rust 1.97.1 formatting: PASS
- diff check: PASS
