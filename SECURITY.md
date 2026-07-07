# Security Policy

Semantic is a deterministic, verifier-first execution platform under active development.

This file is the public security-reporting entrypoint for the repository. It is intentionally short. Detailed Semantic security architecture is tracked separately in the repository security documents and roadmap issues.

## Current support status

Semantic does not currently claim a published stable security target.

The repository status model distinguishes between:

```text
Published Stable
Qualified Limited Release
Landed On main, Not Yet Promised
Out Of Scope
```

Security support follows that vocabulary. Behavior that has landed on `main` is not automatically a stable security guarantee.

## Reporting a security issue

Use GitHub repository security advisories when available.

If repository security advisories are not available to you, open a minimal public issue that asks for a security contact path, without including exploit details, private source, tokens, keys, credentials, or sensitive reproduction data.

When reporting, include only what is necessary:

```text
- affected component or command
- short impact summary
- reproduction outline without secrets
- expected boundary that appears to fail
- affected commit/tag/version if known
```

Do not publish detailed exploit instructions, private project data, or sensitive artifacts before the maintainer has a safe channel to review the report.

## Security-sensitive areas

The following areas are considered security-sensitive for Semantic:

```text
- verifier-first admission bypass
- SemCode artifact validation/admission mismatch
- deterministic VM execution or replay boundary break
- capability boundary bypass
- unexpected broad host effects
- unexpected file, process, network, or device access
- project-root or manifest path boundary escape
- unsafe handling of untrusted `.sm` projects
- unsafe handling of untrusted `.smc` artifacts
- unsafe handling of UI snapshots or event scripts
- unsafe handling of external command output
- Workbench/Studio source or trace exposure later
- ALM trace, skill, or adapter exposure later
- package, extension, release, or update provenance issues later
```

## Non-claims

This repository does not currently claim:

```text
- production-grade sandboxing
- broad host I/O support
- general file/network/process capability support
- stable SemCode binary ABI
- package registry trust
- extension/plugin marketplace safety
- Workbench/Studio processing of arbitrary hostile projects
- ALM skill/adaptor safety
- hidden telemetry or cloud-sync behavior
```

The absence of a claim is intentional. Future Workbench, Studio, ALM, package, extension, and distribution layers must be documented and reviewed before they are treated as supported security surfaces.

## Semantic security model

The intended Semantic security posture is:

```text
Untrusted project by default.
Verifier-first admission always.
Capability-gated effects only.
UI displays, never admits.
ALM suggests, never mutates directly.
Traces and skills stay local unless explicitly reviewed/exported.
Packages/extensions wait for trust policy.
```

The first explicit Semantic threat-model track is:

```text
#1371 security: Semantic threat model and untrusted project policy
```

Planned detailed documents include:

```text
docs/security/threat_model_v0.md
docs/security/untrusted_project_policy_v0.md
docs/security/workbench_studio_command_safety_v0.md
docs/security/alm_skill_security_v0.md
docs/security/ui_snapshot_event_security_v0.md
```

Until those documents exist, this `SECURITY.md` is only the public reporting and boundary statement. It does not widen release, compatibility, runtime, package, Workbench, Studio, or ALM claims.

## Disclosure handling

The maintainer will review security reports in the context of the repository's current maturity and published status vocabulary.

Accepted reports should result in one or more of:

```text
- clarification of a security boundary
- documentation update
- deterministic diagnostic improvement
- verifier/runtime/tooling fix
- release note or advisory when appropriate
```

No public stability or security guarantee is created by this file alone.
