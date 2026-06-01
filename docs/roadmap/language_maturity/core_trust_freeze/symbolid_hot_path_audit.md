# CTF-WP7 - SymbolId Hot-Path Audit

Status: audit plan
Owner: language maturity / execution contract
Parent lane: `docs/roadmap/language_maturity/core_trust_freeze/index.md`
Scope: SymbolId and string hot-path audit after PCC and current CTF sync wave
Non-goal: implementation, runtime widening, release readiness, CI gate, or CTF closure

## Purpose

CTF-WP7 defines the audit boundary for names, symbols, and hot-path lookup.

The goal is to make sure practical language growth does not quietly reintroduce string-based execution paths where compact IDs or verified table indexes are required.

This document does not add runtime behavior.

It does not change any SymbolId semantics.

It does not claim release readiness.

## Audit Surface

The audit should review the current public and internal name surfaces that can drift into runtime hot paths:

| Surface | Audit question | Expected boundary |
| --- | --- | --- |
| source identifiers | Are source names resolved before runtime? | construction / diagnostics only |
| type names | Are type names canonicalized before execution? | pre-runtime identity only |
| SemCode function names | Are function names carried as verified references or table indexes? | verified table / compact ID |
| runtime symbols | Is the runtime symbol table deterministic? | compact runtime symbol path |
| debug names | Are debug names append-only and non-semantic? | debug-only map |
| record fields | Are field identifiers stable and non-string-based in execution? | field ID / descriptor |
| ADT variants | Are variants resolved through stable descriptors? | variant ID / descriptor |
| Option / Result variants | Are variants canonicalized before runtime use? | canonical variant ID |
| collection keys | Is key policy explicit and bounded? | explicit key policy |
| manifest / package names | Do package and module names stay off the VM hot path? | project/tooling surface only |
| import aliases | Are aliases resolved before execution? | pre-runtime resolution only |

## Audit Questions

The audit should answer:

- does any runtime dispatch depend on user-facing strings where a compact ID already exists;
- are debug names clearly separated from semantic identity;
- are symbol-like lookups resolved before VM execution;
- does the current registry still need additional rows;
- does any PCC or CTF work widen the hot path into string-based dispatch;
- does project-root naming stay a tooling concern rather than a VM concern.

## Guardrails

SymbolId hot-path review must not:

- introduce new string-based execution behavior;
- move semantic identity into debug output;
- widen host capability or effect boundaries;
- add project-root implementation;
- add CTF closure claims;
- add release readiness claims.

## Current Status

The current registry is still marked `audit-needed` for the most sensitive name surfaces.

CTF-WP7 is the next safe place to document the audit boundary before any implementation follows.

No code path changes are introduced by this document.

No test or fixture changes are introduced by this document.

## CTF Statement

CTF touched: none

Reason:
This is a docs-only audit plan for SymbolId / hot-path review. It does not change runtime value semantics, VM trap semantics, verifier behavior, capability/audit behavior, trace policy, project-root behavior, release gates, or CTF closure behavior.

## Status Impact

- SymbolId / string hot-path audit is explicitly planned;
- name and symbol surfaces remain a trust boundary review item;
- no command behavior change;
- no execution behavior change;
- no verifier behavior change;
- no VM behavior change;
- no project-root behavior;
- no CI gate;
- no release readiness claim;
- no CTF closure.
