# Tesseract Semantic Unit (`t¤`) Contract

Status: draft v0

This document defines the initial contract for the Tesseract Semantic Unit, written as `t¤`. The unit is a meaning-first representation intended for future Semantic/SRIS integration where internal reasoning operates over semantic units rather than textual tokens.

## Purpose

`t¤` is the smallest stable semantic unit admitted into meaning-oriented reasoning, memory indexing, and future SRIS-compatible flows.

It is not a tokenizer output. It is a structured meaning carrier.

A token is a surface fragment. A `t¤` is an interpreted semantic object.

## Non-goals

This document does not replace the current parser, SemCode admission path, VM execution contract, or source syntax contract.

This document does not require current runtime code to stop using textual source, lexical analysis, or parser profiles.

This document introduces a future-facing semantic object contract that can be adopted incrementally.

## Core fields

A `t¤` unit must have these conceptual fields:

- `id` - stable semantic unit identifier.
- `concept` - primary meaning carried by the unit.
- `role` - local reasoning role, such as `subject`, `predicate`, `goal`, `condition`, `evidence`, `memory`, or `meta`.
- `context` - semantic context in which the unit is valid.
- `source` - origin of the unit, such as `user`, `perception`, `memory`, `rule`, `sris`, or `system`.
- `confidence` - confidence score for the interpretation.
- `relations` - typed links to other semantic units.
- `metadata` - additional verifier-visible or host-visible metadata.

## Reference shape

```text
t¤ {
  id: SemanticUnitId,
  concept: Meaning,
  role: SemanticRole?,
  context: SemanticContext?,
  source: SemanticSource?,
  confidence: Confidence,
  relations: [SemanticRelation],
  metadata: Map<String, Value>
}
```

## Relation model

Relations must be explicit. A relation connects one `t¤` to another `t¤` by type and weight.

```text
SemanticRelation {
  target: SemanticUnitId,
  type: RelationType,
  weight: f32
}
```

Initial relation types:

- `causes`
- `supports`
- `contradicts`
- `refines`
- `generalizes`
- `depends_on`
- `belongs_to_goal`
- `retrieved_from_memory`
- `generated_by_reasoning`

## Token replacement boundary

`t¤` is not a smaller token. It is a higher-level semantic primitive.

The transition boundary is:

```text
surface input -> perception/semantic extraction -> t¤ stream -> reasoning graph -> action/memory/output
```

Only the surface input layer may use text segmentation as an implementation detail. Core reasoning must be allowed to operate without token streams.

## SRIS integration

For SRIS, `t¤` is the preferred unit for:

- reasoning-chain construction;
- hypothesis evidence;
- reflective intelligence unit state;
- semantic memory indexing;
- SMFS-QE inode payloads;
- goal and intent routing.

## SMFS-QE integration

A semantic inode may store one or more `t¤` units as its meaning payload.

SMFS-QE should be able to use `t¤` fields for:

- semantic-aware quantization;
- meaning-level compression;
- relation graph indexing;
- encrypted internal meaning payloads;
- semantic replay and audit.

## Multi-width execution notes

`t¤` does not require a fixed hardware width.

Future engines may map semantic fingerprints, relation masks, confidence fields, or packed reasoning states onto x64, x128, x256, or x512 execution lanes.

Initial interpretation:

- x64 - base semantic identity and fast routing.
- x128 - stable semantic fingerprint and local relation checks.
- x256 - multi-relation reasoning batches.
- x512 - high-context hypothesis comparison or SMFS-QE batch operations.

This mapping is an optimization layer, not the semantic definition itself.

## Admission rule

A `t¤` unit may be admitted into reasoning only if:

1. it has a stable `id`;
2. it has a non-empty `concept`;
3. its `confidence` is within the accepted range;
4. its relation targets are either known, pending, or explicitly external;
5. its source is recorded.

## Compatibility rule

Current Semantic source files and SemCode remain valid.

`t¤` is an additive semantic contract for future meaning-first execution and SRIS integration.
