<p align="center">
  <img width="902" height="817" alt="Screenshot 2026-10-09 015304" src="https://github.com/user-attachments/assets/454026e4-6667-4daa-9c74-b2a244252dec" />
</p>
# Semantic Language

**A deterministic, verifier-first programming language and execution platform with native four-state logic.**

Semantic exists to make reasoning, semantic state transitions, and policy decisions explicit and reproducible. Its fundamental `quad` domain preserves all four states `N/F/T/S`; null and conflict remain visible instead of silently becoming Boolean decisions. Programs compile to versioned SemCode, cross a verifier admission gate, and execute under deterministic rules and explicit resource limits.

The current compiler, verifier, and VM are implemented in Rust. Rust is the host and bootstrap implementation; Semantic's source semantics and artifact contracts are defined by this repository's [specifications](docs/spec/index.md). The next engineering program is self-hosting: closing the language foundations needed for a compiler written in Semantic, with compiler source and bootstrap qualification in [Semantic-Language](https://github.com/skulmakov-oss/Semantic-Language).

[Getting started](docs/getting_started.md) · [Specifications](docs/spec/index.md) · [Architecture](ARCHITECTURE.md) · [Self-hosting program](https://github.com/skulmakov-oss/Semantic/issues/1910)

## Four-state semantics

`quad` is a native semantic domain with the encoding defined by [Quad Logic Frame v1](docs/spec/quad_logic_frame_v1.md):

| State | Encoding | Meaning |
| --- | --- | --- |
| `N` | `00` | Null |
| `F` | `01` | Strict False |
| `T` | `10` | Strict True |
| `S` | `11` | Conflict / Super |

`bool` is the binary condition type used by `if`; `quad` carries four-state semantic values. A quad value selects control flow through an explicit comparison such as `signal == T` or through `match`. There is no implicit quad-to-bool conversion: `N` is not `false`, and `S` is not silently resolved into a Boolean value. See [types](docs/spec/types.md) and [source semantics](docs/spec/source_semantics.md).

## Verifier-first execution

```text
Semantic source (.sm)
  -> frontend and semantic analysis
  -> deterministic IR and lowering
  -> SemCode (.smc)
  -> verifier admission
  -> deterministic VM execution
  -> capability-controlled host boundary when effects are requested
```

Compilation constructs an artifact; verification admits it. The verifier checks the artifact's supported format, structure, instruction and capability contracts before canonical trusted execution. The VM executes admitted SemCode with explicit quotas and deterministic traps. Identical inputs, configuration, capability context, and execution budget produce deterministic outcomes. Explicitly documented raw/diagnostic APIs remain outside this trusted route.

The deterministic core does not perform direct filesystem, network, process, or OS effects. The CLI handles host I/O for source and artifacts. PROMETHEUS owns the host ABI, capabilities, gate policy, runtime sessions, rules, and audit/replay contracts. External runtime effects require explicit authority; missing capability means no effect. The [controlled application boundary](docs/spec/controlled_application_boundary_v0.md) defines the bounded text I/O and command-line contour, including root confinement and audit records.

## Try it

Repository development requires Git, the pinned Rust toolchain, and a C linker. Linux and macOS also need OpenBLAS for the Hub dependency; see [platform prerequisites](docs/getting_started.md#prerequisites).

```bash
git clone https://github.com/skulmakov-oss/Semantic.git
cd Semantic
cargo build --bin smc

cargo run --bin smc -- check examples/qualification/ssf11/f01_minimal/main.sm
cargo run --bin smc -- compile examples/qualification/ssf11/f01_minimal/main.sm -o minimal.smc
cargo run --bin smc -- verify minimal.smc
cargo run --bin smc -- run-smc minimal.smc
```

`run-smc` verifies the artifact before executing it. For the combined source workflow:

```bash
cargo run --bin smc -- run examples/canonical/rule_state_decision/src/main.sm
```

Explore [canonical examples](examples/canonical/), the [example index](docs/examples_index.md), and the [CLI contract](docs/spec/cli.md) for project-root workflows, diagnostics, disassembly, and artifact inspection.

## Architecture

| Layer | Owner | Responsibility |
| --- | --- | --- |
| Source | `sm-front`, `sm-sema` | Parsing, AST, semantic analysis, type checking, diagnostics |
| IR | `sm-ir` | Deterministic IR, lowering, producer policy |
| Artifact | `sm-format`, `sm-emit` | SemCode format/decode and producer-facing emission facade |
| Admission | `sm-verify` | Structural, instruction, and capability admission |
| Execution | `sm-runtime-core`, `sm-vm` | Runtime vocabulary, quotas, deterministic verified execution |
| CLI | `smc-cli` | Public commands and authorized host I/O |
| Host boundary | `prom-*` | ABI, capability policy, gates, sessions, state, rules, audit |
| Bootstrap compiler | [Semantic-Language](https://github.com/skulmakov-oss/Semantic-Language) | Future Semantic compiler source and bootstrap qualification |

Language semantics, SemCode, verifier, VM, and PROMETHEUS authority remain in this repository. Semantic-Language consumes those contracts; it is not a second specification or a verifier/VM rewrite. The current executable source profile is called Rust-like in the specifications; Logos is a separate experimental declarative inspection profile, without that execution authority.

See [SemCode](docs/spec/semcode.md), [verifier](docs/spec/verifier.md), [VM](docs/spec/vm.md), and the [module ownership map](docs/architecture/module_ownership_map.md).

The legacy perimeter (`crates/ton618-core`, `src/bin/ton618_core.rs`, `ton618_legacy/`) remains compatibility-only; canonical development belongs to the owners above. See the [legacy map](docs/legacy-map.md).

## Current status

Status snapshot: **2026-10-09**. Implementation, qualification, and publication are separate states; see the [public status model](docs/roadmap/public_status_model.md).

### Implemented and qualified

- **Published Stable Foundation:** [v1.2.0](https://github.com/skulmakov-oss/Semantic/releases/tag/v1.2.0), from `89641da8237f4fcefb50cf1958a50e4d4003aea7`, qualified and promoted **with explicit limits**, for Windows x64 only. Assets are unsigned. Its exact contour and exclusions are in the [final verdict](reports/semantic_stable_foundation_final_verdict.md) and [release posture](docs/roadmap/v1_readiness.md).
- **Executable foundation:** functions and control flow; quad, bool, numeric values, records, tuples and nominal ADTs; `Option`/`Result`; bounded sequences/maps; local project/package workflows; and the source-to-SemCode-to-verifier-to-VM path. The [Foundation source contract](docs/spec/foundation_source_profile_v1.md) bounds the language surface. The [Gate 1 verdict](reports/g1_release_scope_statement.md) separately records its narrower practical-programming qualification contour.
- **Post-release compiler foundations on main:** compiler-grade UTF-8 text inspection and checked plain-`u32` arithmetic/ordering are implemented and qualified; the tiny Semantic-written lexer probe has landed. These SHF additions are outside the v1.2.0 promise.

Current `main` is development beyond the published release. Landing or passing tests does not promote a feature to published stable.

### In progress

The [Self-Hosting Foundation program (#1910)](https://github.com/skulmakov-oss/Semantic/issues/1910) is underway at the language-foundation stage. Its completed checkpoints and next implementation task are listed below. **The compiler written in Semantic has not started; Semantic is not self-hosted, and no C1/C2 fixed point has been qualified.**

### Planned

Remaining compiler foundations include Bytes implementation, endian helpers and binary I/O, bit operations, compiler collections, executable generics, arenas/typed IDs, modules, diagnostics, and a Semantic-owned SemCode encoder. Lexer, parser, semantic analysis, lowering, emission, and bootstrap proof follow those prerequisites. See the [bootstrap roadmap](https://github.com/skulmakov-oss/Semantic-Language/blob/main/docs/ROADMAP.md).

Native UI, Workbench, and Semantic Studio are [retired from the active roadmap](docs/roadmap/ui_workbench_studio_retirement.md). Their historical implementation does not imply current qualification or planned promotion.

## Self-hosting and bootstrap

The two repositories have complementary roles:

- **Semantic:** Rust reference compiler C0, upstream language/runtime foundations, SemCode authority, verifier, VM, and capability boundary.
- **[Semantic-Language](https://github.com/skulmakov-oss/Semantic-Language):** compiler source `S` in Semantic and the construction, comparison, and qualification of bootstrap generations.

The [frozen bootstrap contract](https://github.com/skulmakov-oss/Semantic-Language/blob/main/docs/BOOTSTRAP_CONTRACT.md) defines:

```text
C0 = qualified Rust-hosted reference compiler at an exact pinned SHA
S  = source set of the compiler written in Semantic

C0(S) -> C1.smc
C1(S) -> C2.smc

C1.smc == C2.smc   (frozen comparison rule: byte-equality-v1)
```

Both C1 and C2 require verifier admission. Admitted C1 runs on the existing VM to produce C2; C2 is admitted and compared. The Bootstrap Seal additionally requires recorded provenance and independent reproduction. Fixed-point equality never bypasses verification, and Rust may continue to own verifier, VM, and host mechanics after the first self-hosting proof.

The [C0 reference manifest](https://github.com/skulmakov-oss/Semantic-Language/blob/main/reference/semantic-reference.toml) still pins `89641da8237f4fcefb50cf1958a50e4d4003aea7`; new upstream SHF work does not silently repin C0 or inherit its qualification. The Stable Foundation report's release-candidate name **C1** is unrelated to the future bootstrap artifact **C1.smc**.

| Checkpoint | Verified state | Evidence |
| --- | --- | --- |
| SHF-0 bootstrap protocol | Qualified / complete in Semantic-Language; compiler implementation not started | [Contract](https://github.com/skulmakov-oss/Semantic-Language/blob/main/bootstrap/contract.toml), [#11](https://github.com/skulmakov-oss/Semantic-Language/issues/11) |
| SHF-1A / SHF-1B text contract and primitives | Complete; implementation and qualification landed on main | [Text contract](docs/spec/compiler_text_v0.md), [PR #2005](https://github.com/skulmakov-oss/Semantic/pull/2005) |
| SHF-3A arithmetic / ordering | Complete; implementation and qualification landed on main; SHF-3B bit operations deferred | [u32 contract](docs/spec/compiler_u32_v0.md), [PR #2009](https://github.com/skulmakov-oss/Semantic/pull/2009) |
| SHF-1C lexer probe / SHF-1 completion | Qualified and landed; a byte-scanning probe, not the SHF-10 compiler lexer | [Text qualification](docs/spec/compiler_text_v0.md#94-lexer-probe), [PR #2011](https://github.com/skulmakov-oss/Semantic/pull/2011), [#2010](https://github.com/skulmakov-oss/Semantic/issues/2010) |
| SHF-2A1 Bytes contract | Complete and normative on main; contract only | [Bytes contract](docs/spec/compiler_bytes_v0.md), [PR #2013](https://github.com/skulmakov-oss/Semantic/pull/2013), [closeout #2014](https://github.com/skulmakov-oss/Semantic/pull/2014) |
| SHF-2A2 Bytes implementation | Open next implementation task; not implemented on this main snapshot | [#2015](https://github.com/skulmakov-oss/Semantic/issues/2015) |
| SHF-2B / SHF-2C, SHF-3B, SHF-9 / SHF-10 | Deferred / not started | [Bytes non-goals](docs/spec/compiler_bytes_v0.md#9-non-goals-and-deferred-capabilities), [#1910](https://github.com/skulmakov-oss/Semantic/issues/1910) |
| SHF-11 through SHF-17 | Planned; no compiler implementation, bootstrap generations, or fixed-point proof yet | [Bootstrap roadmap](https://github.com/skulmakov-oss/Semantic-Language/blob/main/docs/ROADMAP.md) |

The text and u32 documents retain some pre-merge probe wording; the merged PR and closed checkpoint establish its landed state. Semantic-Language's overview also retains an older SHF-1 status. Upstream completion does not promote its frozen bootstrap subset: **BSF-101 remains CANDIDATE** in the bootstrap contract.

## Specifications and development

| Read about | Source |
| --- | --- |
| Language surface | [Foundation source profile](docs/spec/foundation_source_profile_v1.md), [syntax](docs/spec/syntax.md), [types](docs/spec/types.md), [source semantics](docs/spec/source_semantics.md) |
| Quad | [Quad Logic Frame](docs/spec/quad_logic_frame_v1.md), [quad algebra](docs/core/quad_algebra.md) |
| Artifacts and trusted execution | [SemCode](docs/spec/semcode.md), [verifier](docs/spec/verifier.md), [VM](docs/spec/vm.md) |
| Compiler foundations | [Text](docs/spec/compiler_text_v0.md), [Bytes](docs/spec/compiler_bytes_v0.md), [u32](docs/spec/compiler_u32_v0.md) |
| Host effects | [Capabilities](docs/spec/capabilities.md), [controlled application boundary](docs/spec/controlled_application_boundary_v0.md) |
| Threats and provenance | [Threat model](docs/security/threat_model_v0.md), [artifact provenance policy](docs/security/artifact_provenance_and_signing_policy_v0.md) |
| Complete contracts | [Specification index](docs/spec/index.md) |

Contributions should preserve ownership boundaries, deterministic behavior, verifier-first execution, and the distinction between implemented, qualified, and published behavior. Read [AGENTS.md](AGENTS.md), [CONSTRAINTS.md](CONSTRAINTS.md), and the [verification guide](docs/agents/VERIFICATION.md) before changing the repository. Keep each PR focused and add contract evidence for behavior changes.

## License

[Apache License 2.0](LICENSE). Copyright 2026 Said Kulmakov. See [NOTICE](NOTICE) for attribution and third-party scope.
