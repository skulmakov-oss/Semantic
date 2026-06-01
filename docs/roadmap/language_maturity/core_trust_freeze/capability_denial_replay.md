# CTF-E4 - Capability Denial Replay

Status: draft replay plan
Owner: language maturity / execution contract
Parent lane: `docs/roadmap/language_maturity/core_trust_freeze/index.md`
Scope: denied-effect replay evidence after PCC and current CTF sync wave
Non-goal: implementation, capability widening, release readiness, CI gate, or CTF closure

## Purpose

CTF-E4 defines the replay evidence shape for denied capability and denied effect behavior.

The aim is to keep capability denial deterministic, reportable, and distinguishable from verifier rejection and VM traps as the platform surface widens.

This document does not add replay artifacts.

It does not widen host capability behavior.

It does not claim release readiness.

## Replay Boundary

Capability denial replay should cover the surfaces that can be mistaken for host effect widening:

| Surface | Replay question | Expected boundary |
| --- | --- | --- |
| `print(text)` | Does the same denied input produce the same denial classification? | capability / effect boundary only |
| `to_text` / formatting | Does text formatting remain admitted without host widening? | admitted type formatting only |
| `debug_render` | Does internal rendering stay internal-only? | no host-visible output |
| file IO | Does denied file access stay denied deterministically? | capability denial, not trap class drift |
| network IO | Does denied network access stay denied deterministically? | capability denial, not trap class drift |
| host gate read/write | Do host gate reads and writes remain denied where appropriate? | capability boundary only |
| pulse emit | Do pulse emissions remain effect-gated? | denied-effect classification |
| UI event/frame effects | Do UI-adjacent effects remain outside the core trust lane? | no host widening |
| audit emission | Does internal audit emission stay separate from host output? | audit surface only |

Replay evidence should confirm:

- the same denied input yields the same denial classification;
- denial reasons remain in the capability/effect boundary, not VM trap naming;
- no host mutation occurs;
- no hidden stdout, file, network, or telemetry channel appears;
- admitted input remains admitted across unrelated PCC or 7hell changes;
- capability denial remains distinct from verifier rejection and VM traps.

## Replay Shape

CTF-E4 is a plan for replay evidence, not a trap taxonomy change.

Future evidence should be able to show:

- input surface;
- denied capability or effect;
- replayable denial classification;
- stable denial reason text;
- no host-effect widening;
- no trap reclassification.

## Current Status

Capability denial replay is now explicitly planned as the next trust-evidence boundary.

Denied-effect behavior remains a determinism and reportability goal.

No command behavior changes are introduced by this document.

No execution behavior changes are introduced by this document.

## CTF Statement

CTF touched: none

Reason:
This is a docs-only replay plan for capability denial evidence. It does not change runtime value semantics, VM trap semantics, verifier behavior, capability/audit behavior, trace policy, project-root behavior, release gates, or CTF closure behavior.

## Status Impact

- capability denial replay is explicitly planned;
- denied-effect behavior remains deterministic and reportable as a goal;
- no command behavior change;
- no execution behavior change;
- no verifier behavior change;
- no VM behavior change;
- no project-root behavior;
- no CI gate;
- no release readiness claim;
- no CTF closure.
