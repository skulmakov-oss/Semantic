# Semantic Agent Context-Economy & Checkpoint Protocol

Status: canonical agent context management protocol
Authority: subordinate to [`AGENTS.md`](../../AGENTS.md), [`CONSTRAINTS.md`](../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../.harness/current.task.yaml)

---

## 1. Core Philosophy & Purpose

The goal of the Semantic context-economy protocol is to maintain the coding agent's LLM context window as a **hot working set** rather than an unbounded, noisy accumulation of entire multi-turn execution history:
$$\text{LLM Working Context} = \text{Hot Working Set (Immediate Focus + Authoritative References)}$$

### The Cardinal Rule
$$\text{\textbf{Compress Representation}} \neq \text{\textbf{Lose Knowledge}}$$

Completed task history is structured into a versioned, machine-readable, deterministic Semantic context checkpoint ([`.harness/context-checkpoint.schema.json`](../../.harness/context-checkpoint.schema.json)) while preserving exact identifiers, owner decisions, unresolved findings, blockers, verification results, and provenance.

---

## 2. Critical Capability Boundary

Repository governance explicitly distinguishes implementable agent working-memory management from proprietary host internals:

- **Agent-Managed Context Economy (Implementable)**:
  - What agents actively retain in their prompt / hot working set;
  - What completed milestones are structured into checkpoints;
  - What raw tool outputs are evictable after fact extraction;
  - How state is validated, serialized, and rehydrated across task phases.
- **Host/Model Internal Context Window (Host-Controlled)**:
  - Repository governance does **not** claim physical control over private, internal token compaction of underlying LLM platforms (Codex, ChatGPT, Gemini, Claude).
  - No manufactured token estimates or fake prompt tricks may be presented as repository architecture.

---

## 3. Four Canonical Context Classes

All information encountered during task execution is classified into one of four tiers:

```text
┌──────────────────────────────────────────────────────────────────┐
│                             PINNED                               │
│  Lossless authoritative anchors, owner decisions, active blockers│
├──────────────────────────────────────────────────────────────────┤
│                             ACTIVE                               │
│  Current hot working set: immediate diffs, active failure traces │
├──────────────────────────────────────────────────────────────────┤
│                           COMPRESSED                             │
│  Completed milestones, typed facts, structured checkpoints       │
├──────────────────────────────────────────────────────────────────┤
│                           EVICTABLE                              │
│  Raw command outputs, full query logs (after fact extraction)    │
└──────────────────────────────────────────────────────────────────┘
```

### A. PINNED (Lossless Invariants)
Critical state that must never be discarded or loosely summarized:
- Active Harness task envelope and allowed/forbidden paths;
- Explicit repository-owner directives and governance decisions;
- Unresolved review findings and active blocker descriptions;
- Exact commit SHAs, PR numbers, issue IDs, and thread IDs;
- Security, verifier, Quad Logic, and public contract wording where exact semantics matter.

*Implementation note: PINNED files (e.g. `AGENTS.md`, `CONSTRAINTS.md`) are recorded in checkpoints via path and git blob SHA (`git hash-object`), not copied in full. The agent re-reads them upon rehydration.*

### B. ACTIVE (Hot Working Set)
Information required for immediate, ongoing execution:
- Specific source files currently undergoing modification;
- Unresolved compiler diagnostics, test failure outputs, and active stack traces;
- Active technical hypotheses under test;
- Nearest planned next actions.

### C. COMPRESSED (Structured Checkpoint State)
Completed task history transformed into machine-readable JSON:
- Proven facts with durable provenance locators;
- Completed verification commands and observed exit codes;
- Resolved review threads with accepted architectural rationales;
- Superseded checkpoint references.

### D. EVICTABLE (Redundant Raw Material)
Raw tool logs, voluminous search transcripts, and full test outputs that can be safely excluded from active agent prompts **only after** all durable facts, categories, and provenance locators have been extracted and validated.

---

## 4. Checkpoint Authority Hierarchy

A checkpoint is **task memory, never repository authority**:
$$\text{Platform/Safety} > \text{AGENTS.md} > \text{CONSTRAINTS.md} > \text{Owner Transition} > \text{Harness} > \text{Normative Specs} > \text{\textbf{Checkpoint}} > \text{Agent Plan}$$

### Hard Invariants:
1. A checkpoint **cannot** authorize file paths, dependency changes, CI changes, tool fallbacks, or PR merges.
2. Even if a checkpoint records that an action was previously permitted, current repository authority (`.harness/current.task.yaml` and `CONSTRAINTS.md`) must be re-read upon restore.
3. If a checkpoint contradicts live repository authority, **live repository authority wins**.

---

## 5. Typed Knowledge Model

To prevent semantic degradation, checkpoints must store knowledge in distinct typed categories:

| Category Enum | Description | Integrity Rule |
|---|---|---|
| `PROVEN_FACT` | Verified repository truth | Must include verifiable provenance (file, line, SHA, test output) |
| `OWNER_DECISION` | Explicit directive from repo owner | Must cite exact source issue comment or instruction |
| `REVIEWER_CLAIM` | Finding or suggestion from review | Must include `thread_id`, claim description, and status (`ACTIVE`, `ADDRESSED`, `ACCEPTED`, `REJECTED`) |
| `HYPOTHESIS` | Technical proposition under investigation | Must declare required confirming evidence |
| `UNRESOLVED_QUESTION` | Open question pending clarification | Must indicate whether it constitutes a blocker |
| `BLOCKER` | Condition preventing task completion | Must describe the minimum decision required to resolve |
| `VERIFICATION_RESULT` | Result of executed verification gate | Must include exact command, target HEAD SHA, exit code, and status |
| `NEXT_ACTION` | Planned immediate step | Ordered sequence of actionable tasks |

### Anti-Pattern: No Category Blurring
Converting a `REVIEWER_CLAIM` into a `PROVEN_FACT` without independent verification is a defect.
Converting an `HYPOTHESIS` into an `OWNER_DECISION` is a defect.

---

## 6. Dangerous Compression Anti-Patterns

Compressing constraints into approximate or weaker prose alters repository semantics and is strictly prohibited:

```text
❌ SOURCE: "No autonomous fallback. Owner authorization is required."
   INVALID COMPRESSION: "Fallbacks are generally discouraged."

❌ SOURCE: "Raw diagnostic VM APIs may exist only within their documented non-canonical perimeter."
   INVALID COMPRESSION: "All raw VM execution is forbidden."

❌ SOURCE: "Normative spec vs fixture conflict -> STOP and report contract drift."
   INVALID COMPRESSION: "Prefer whichever evidence is newer."

❌ SOURCE: "Landed on main != stable != released."
   INVALID COMPRESSION: "Feature is effectively stable because CI passed."
```

---

## 7. Checkpoint Schema & Validation

Checkpoints adhere strictly to [`.harness/context-checkpoint.schema.json`](../../.harness/context-checkpoint.schema.json).

### Deterministic Validation (`scripts/context_checkpoint_check.ps1`)
The validator script validates checkpoint structure and detects repository staleness:

```powershell
# Validate checkpoint syntax and schema compliance
pwsh -File scripts/context_checkpoint_check.ps1 -Checkpoint <path-to-checkpoint.json>

# Validate against live repository HEAD, active Harness task, and authority blob SHAs
pwsh -File scripts/context_checkpoint_check.ps1 -Checkpoint <path-to-checkpoint.json> -AgainstCurrentRepo

# Run validator self-test suite (positive and negative qualification)
pwsh -File scripts/context_checkpoint_check.ps1 -SelfTest
```

### Staleness & Drift Detection:
- **HEAD Staleness**: If `repository.head_sha` differs from live `git rev-parse HEAD`, repository-dependent facts must be revalidated.
- **Authority Drift**: If blob SHAs for `AGENTS.md`, `CONSTRAINTS.md`, or `.harness/current.task.yaml` differ from live `git hash-object`, the checkpoint is flagged as **STALE** and governing rules are reloaded.

---

## 8. Checkpoint Lifecycle & Budget Triggers

```text
┌──────────────┐      ┌──────────────┐      ┌──────────────┐      ┌────────────────────────┐
│    CREATE    │ ───> │   VALIDATE   │ ───> │     USE      │ ───> │ REFRESH / REVALIDATE   │
│ (Triggered)  │      │ (Schema/Repo)│      │ (Hot Focus)  │      │ (Supersede Old State)  │
└──────────────┘      └──────────────┘      └──────────────┘      └────────────────────────┘
```

### Operating Budget Modes:
1. **Telemetry Mode** (when the execution host provides reliable token metrics):
   - **Soft Threshold ($\approx 0.65$)**: Trigger checkpointing of completed milestones and evict redundant raw logs.
   - **Hard Threshold ($\approx 0.80$)**: Immediate checkpoint consolidation before taking further modifying actions.
2. **Event-Trigger Mode** (default when host token metrics are unavailable):
   - Lifecycle phase completion (e.g. `AUTHORIZE` $\rightarrow$ `IMPLEMENT`);
   - Review cycle completion (all findings addressed and verified);
   - Major architectural decisions or history reconstructions;
   - Post-large tool output processing (e.g., full test suites);
   - Prior to subagent delegation or fresh-agent session handoff.

*Note: Checkpoints are NOT created after every trivial command.*

---

## 9. Toolstack Memory Integration

- **Codebase Memory MCP**: External source graph memory. Checkpoints store compact symbol names and query keys; full symbol sources are re-queried via MCP as needed.
- **mcp-local-rag**: External document and history memory. Checkpoints store document identifiers and confirmed decisions; full text passages are not duplicated.
- **Git / CI / Harness**: Verification memory. Checkpoints store exact commands, target SHAs, and exit codes.
- **No Chain-of-Thought Storage**: Checkpoints record operational state and evidence, never internal model thoughts or hidden scratchpads.

---

## 10. Storage & Hygiene Policy

- Ephemeral per-task checkpoints are generated in `.harness/context/` (ignored by `.gitignore`).
- Checkpoints serve as runtime memory and must **not** be committed to repository history unless explicitly designated as permanent qualification evidence.
