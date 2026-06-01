# CTF-WP8 - Map Open-Edge Policy

Status: policy draft
Owner: language maturity / execution contract
Parent lane: `docs/roadmap/language_maturity/core_trust_freeze/index.md`
Scope: Map missing-key / iteration / quota evidence boundary after PCC and current CTF sync wave
Non-goal: implementation, collections widening, release readiness, CI gate, or CTF closure

## Purpose

CTF-WP8 defines the open-edge policy for `Map<K,V>`.

The admitted Map baseline already has selected replay evidence for insert/lookup and persistent update.

This policy records the remaining open edges so Map does not drift from bounded baseline into accidental broad collections freeze.

This document does not add Map runtime behavior.

It does not close Map policy edges.

It does not claim release readiness.

## Current Baseline

The current Map evidence is bounded to the admitted baseline only.

Covered today:

- insert / lookup replay evidence;
- persistent update replay evidence;
- selected admitted Map baseline behavior.

Not covered today:

- missing-key behavior;
- iteration policy;
- memory / quota policy.

## Open-Edge Policy

| Map edge | Current status | Policy note | Required later evidence |
| --- | --- | --- | --- |
| missing-key behavior | open | stays outside the admitted baseline until separately evidenced | deterministic failure / diagnostic evidence |
| iteration policy | open | stays outside the admitted baseline until separately evidenced | deterministic replay or trace evidence |
| memory / quota policy | open | remains a later collections-trust concern | quota / trap / diagnostic evidence |

Policy rules:

1. Map may remain evidence-backed for selected admitted baseline surfaces.
2. Missing-key, iteration, and quota behavior must not be quietly promoted into freeze-candidate or frozen status.
3. Any future Map widening must name which edge is being admitted.
4. Project-root, registry, and remote-dependency behavior remain separate trust surfaces.
5. Golden traces for Map edges must not be claimed until the edge is explicitly opened and evidenced.

## Boundary Statement

Map remains bounded-open overall.

The current baseline is sufficient for the selected replay-backed surfaces, but it is not a broad collections freeze.

This policy keeps the open edges visible instead of letting them disappear into the admitted baseline.

## Required Future Evidence Shape

Before a Map edge can move beyond `audit-needed`, future evidence should specify:

- the exact edge;
- the exact input class;
- whether the behavior is a value result, diagnostic, or trap;
- whether repeated runs remain deterministic;
- whether the behavior stays within the current capability / effect boundary;
- whether the evidence is replay, trace, or ordinary test evidence.

## CTF Statement

CTF touched: none

Reason:
This is a docs-only policy for Map open edges. It does not change runtime value semantics, VM trap semantics, verifier behavior, capability/audit behavior, trace policy, project-root behavior, release gates, or CTF closure behavior.

## Status Impact

- Map open-edge policy is explicitly recorded;
- missing-key / iteration / quota remain open;
- selected admitted baseline evidence remains unchanged;
- no command behavior change;
- no execution behavior change;
- no verifier behavior change;
- no VM behavior change;
- no project-root behavior;
- no CI gate;
- no release readiness claim;
- no CTF closure.
