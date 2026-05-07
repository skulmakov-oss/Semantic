# AGENTS.md

Use the `semantic` skill for work on Semantic Language, SemCode, verifier admission, VM execution, Quad Logic, runtime ownership, quotas, PROMETHEUS boundary crates, docs/spec, roadmap status, tests, and PR planning.

Repository discipline:

- one logical change per PR;
- tests where behavior changes;
- docs/spec sync where public contract changes;
- no silent release claim widening;
- landed on main does not automatically mean stable or promised.

Do not bypass verifier-first admission.
Do not introduce nondeterminism into Semantic core.
Do not add direct external effects outside PROMETHEUS capability boundaries.
