# Issue 1967 Reconciliation

Baseline: revalidated after rebase against `origin/main` at `c3ac7a63b27852161f948149878c7c14260dcad4`.
Scope: 13 remediation packages and 82 findings from GitHub issue 1967.
Historical audit CSV/JSON remain immutable; this is derived current-state evidence.

| Finding | REM | Final classification | Current evidence / action |
|---|---|---|---|
| FND-044 | REM-007 | FIXED_LATER_CONFIRMED | OWN0 global-root decode and ownership E2E regressions |
| FND-045 | REM-007 | FIXED_LATER_CONFIRMED | write-event cursor synchronization tests |
| FND-048 | REM-007 | FIXED_LATER_CONFIRMED | out-of-range OWN0 roots reject deterministically |
| FND-125 | REM-008 | FIXED_BY_THIS_PR | quoted verifier text routes to controlled observation |
| FND-131 | REM-008 | FIXED_LATER_CONFIRMED | unit VM APIs use discard observation runtime |
| FND-132 | REM-008 | FIXED_LATER_CONFIRMED | VM remains publicly constructible |
| FND-324 | REM-008 | FIXED_BY_THIS_PR | profile harness uses capability-aware Pure host |
| FND-325 | REM-008 | FIXED_BY_THIS_PR | profile workloads retain explicit LoadQ assertions |
| FND-330 | REM-008 | FIXED_BY_THIS_PR | classified transition fixture preserves overlapping counters |
| FND-334 | REM-008 | FIXED_BY_THIS_PR | missing expected semantic locals now fail |
| FND-337 | REM-008 | FIXED_BY_THIS_PR | helper-mix fixture asserts exact branch counters |
| FND-042 | REM-009 | FIXED_BY_THIS_PR | alloc-only tests import the vec macro explicitly |
| FND-170 | REM-009 | FIXED_LATER_CONFIRMED | runtime-core public API snapshot is synchronized |
| FND-193 | REM-009 | FIXED_BY_THIS_PR | verifier spec declares unique function names |
| FND-019 | REM-010 | FIXED_LATER_CONFIRMED | manifest admission rejects escaped imports |
| FND-058 | REM-010 | FIXED_LATER_CONFIRMED | bundle loader enforces importer package boundary |
| FND-062 | REM-010 | FIXED_LATER_CONFIRMED | mixed bare and selected imports preserve aliases |
| FND-177 | REM-010 | FIXED_LATER_CONFIRMED | manifest entry normalization handles backslashes |
| FND-178 | REM-010 | FIXED_LATER_CONFIRMED | run project entry is canonically root-contained |
| FND-181 | REM-010 | FIXED_LATER_CONFIRMED | CLI project-root contract and help are synchronized |
| FND-182 | REM-010 | FIXED_LATER_CONFIRMED | dump-bytecode rejects symlink entry escapes |
| FND-183 | REM-010 | FIXED_LATER_CONFIRMED | hash-ir rejects symlink entry escapes |
| FND-189 | REM-010 | FIXED_LATER_CONFIRMED | absolute-path rejection uses portable cases |
| FND-370 | REM-010 | FIXED_LATER_CONFIRMED | shared project resolver canonicalizes containment |
| FND-371 | REM-010 | FIXED_LATER_CONFIRMED | test discovery sorts losslessly or rejects invalid names |
| FND-157 | REM-011 | FIXED_LATER_CONFIRMED | 7hell is wired through shared CLI dispatch |
| FND-159 | REM-011 | FIXED_LATER_CONFIRMED | type-check failures map to Type Hell |
| FND-160 | REM-011 | FIXED_BY_THIS_PR | parser Law-name syntax uses E0217 while semantic duplicate Law retains E0221; structured origin remains authoritative |
| FND-161 | REM-011 | FIXED_LATER_CONFIRMED | VM blocker graph contains no self-cycle |
| FND-162 | REM-011 | FIXED_LATER_CONFIRMED | verifier failure report matches blocked VM behavior |
| FND-163 | REM-011 | FIXED_BY_THIS_PR | successful 7hell executes the verified entry once and qualifies the observations returned by that execution |
| FND-167 | REM-011 | FIXED_LATER_CONFIRMED | report quality validates blocked_by targets |
| FND-069 | REM-012 | FIXED_BY_THIS_PR | call argument register addition is checked |
| FND-070 | REM-012 | FIXED_BY_THIS_PR | scalar-only backend hot paths no longer reprobe CPU caps |
| FND-078 | REM-012 | FIXED_BY_THIS_PR | Q-table proof checks the last actual written key |
| FND-320 | REM-012 | FIXED_LATER_CONFIRMED | MaskIndexIter is re-exported without alloc gating |
| FND-322 | REM-012 | FIXED_LATER_CONFIRMED | QuadroBank participates in deterministic batch sweep |
| FND-122 | REM-013 | FIXED_BY_THIS_PR | unknown host channels deny in both capability paths |
| FND-123 | REM-013 | FIXED_BY_THIS_PR | audit event is non-exhaustive outside prom-audit |
| FND-051 | REM-018 | FIXED_LATER_CONFIRMED | record cleanup regression uses overlapping writes |
| FND-052 | REM-018 | FIXED_LATER_CONFIRMED | record cleanup stability targets overlapping paths |
| FND-063 | REM-018 | FIXED_LATER_CONFIRMED | execution integrity normalizes generated selected-import symbols |
| FND-068 | REM-018 | FIXED_BY_THIS_PR | prom-state and prom-rules API targets restored |
| FND-135 | REM-018 | FIXED_BY_THIS_PR | Option acceptance executes emitted artifact |
| FND-136 | REM-018 | FIXED_BY_THIS_PR | Result Err fixture asserts bound payload |
| FND-139 | REM-018 | FIXED_BY_THIS_PR | Sequence acceptance executes emitted artifact |
| FND-140 | REM-018 | FIXED_BY_THIS_PR | Sequence fixture asserts order-sensitive traversal |
| FND-141 | REM-018 | FIXED_BY_THIS_PR | Map acceptance executes emitted artifact |
| FND-195 | REM-018 | FIXED_BY_THIS_PR | token admission is outside runtime timing |
| FND-310 | REM-018 | FIXED_BY_THIS_PR | legacy guards fail on I/O and scan non-UTF8 lossily |
| FND-319 | REM-018 | FIXED_BY_THIS_PR | all public path commands reject leading unknown flags |
| FND-344 | REM-018 | FIXED_LATER_CONFIRMED | probe snapshot rejects malformed hash shape |
| FND-150 | REM-019 | FIXED_BY_THIS_PR | golden trace hashes full IR debug content |
| FND-151 | REM-019 | FIXED_BY_THIS_PR | output hash derives from collected VM observations |
| FND-152 | REM-019 | FIXED_BY_THIS_PR | replay equality compares source IR and SemCode hashes |
| FND-179 | REM-020 | FIXED_BY_THIS_PR | failed compile proves absent and protected output states |
| FND-180 | REM-020 | FIXED_LATER_CONFIRMED | project failure tests retain no-output assertions |
| FND-185 | REM-020 | FIXED_LATER_CONFIRMED | project fixture discovery does not skip hidden entries |
| FND-188 | REM-020 | FIXED_LATER_CONFIRMED | compile dot covers relative -o artifact path |
| FND-790 | REM-021 | FIXED_LATER_CONFIRMED | audit stdout omits session and caller metadata |
| FND-791 | REM-021 | FIXED_LATER_CONFIRMED | audit confidentiality regression covers persisted provenance |
| FND-792 | REM-021 | FIXED_LATER_CONFIRMED | session summary omits session ID |
| FND-793 | REM-021 | FIXED_LATER_CONFIRMED | mixed-caller errors do not echo caller identity |
| FND-050 | REM-024 | FIXED_BY_THIS_PR | OWN0 contract requires CAP_OWNERSHIP_PATHS |
| FND-053 | REM-024 | FIXED_BY_THIS_PR | architecture pipeline restores verifier stage |
| FND-054 | REM-024 | FIXED_BY_THIS_PR | runtime roadmap distinguishes canonical and raw VM APIs |
| FND-134 | REM-024 | FIXED_BY_THIS_PR | PCC audit separates representation validation and execution |
| FND-197 | REM-024 | FIXED_BY_THIS_PR | verifier matrix lists every host effect family |
| FND-326 | REM-024 | FIXED_BY_THIS_PR | scalar audit records true opcode rankings |
| FND-327 | REM-024 | FIXED_BY_THIS_PR | VM-M6 validation record matches commands |
| FND-328 | REM-024 | FIXED_BY_THIS_PR | VM-M7 command ledger excludes unrun fmt |
| FND-331 | REM-024 | FIXED_BY_THIS_PR | helper audit distinguishes two chains from four calls |
| FND-333 | REM-024 | FIXED_BY_THIS_PR | result-surface audit separates raw from admitted execution |
| FND-335 | REM-024 | FIXED_BY_THIS_PR | admission diagram models VerifiedSemCode before entry token |
| FND-340 | REM-024 | FIXED_BY_THIS_PR | PCC closeout no longer contradicts expression-valued match |
| FND-341 | REM-024 | FIXED_BY_THIS_PR | PCC bridge audit is rebased on merged local stack |
| FND-111 | REM-026 | FIXED_BY_THIS_PR | overall 7hell result has explicit precedence |
| FND-117 | REM-026 | FIXED_LATER_CONFIRMED | PCC3 mapping points to present CTF impact document |
| FND-156 | REM-026 | FIXED_BY_THIS_PR | PCC4 mapping removes nonexistent fixture anchor |
| FND-158 | REM-026 | FIXED_BY_THIS_PR | stage matrices use canonical report tokens |
| FND-164 | REM-026 | FIXED_BY_THIS_PR | waypoint failure semantics include Lowering Hell |
| FND-323 | REM-026 | FIXED_LATER_CONFIRMED | P4 evidence-repair section records diagnostic closure |

## Totals

- `FIXED_BY_THIS_PR`: 44
- `FIXED_LATER_CONFIRMED`: 38
- `ARCHITECTURALLY_SUPERSEDED_WITH_EVIDENCE`: 0
- `FALSE_POSITIVE_CONFIRMED`: 0
- unresolved or unverified: 0

## Qualification Contract

The PR must pass touched-crate tests, focused integration suites, public API contracts, VM profiling, 7hell CI, formatting, clippy, repository hygiene, admission preflight, Harness scope enforcement, and `git diff --check`. Review and merge remain separate owner-controlled phases.
