# Semantic identity rationale

The v1.2 identity expresses Semantic's architecture: a deterministic,
verifier-first programming language and execution platform with native
four-state logic. It is not a generic decorative software cube.

## Architectural basis

The [README](../../README.md) and [trusted execution constraints](../../CONSTRAINTS.md)
establish the pipeline:

```text
Semantic source (.sm)
  -> frontend + semantic analysis
  -> deterministic construction / IR and lowering
  -> emission
  -> SemCode (.smc)
  -> verifier admission
  -> deterministic execution
```

Verifier admission remains distinct from execution. The black computational
body represents structure and controlled execution; the front state matrix
makes the native four-state domain visible:

```text
N F
T S

N = unknown / no sufficient evidence
F = evidence for false
T = evidence for true
S = evidence for both / conflict
```

The normative [Quad Logic Frame v1](../spec/quad_logic_frame_v1.md) names these
Null, Strict False, Strict True, and Conflict / Super respectively, with the
encodings 00, 01, 10, 11. These canonical names supplement the explanatory
legend in the supplied reference. The visual S tile never redefines S as the
project name. Conflict survives instead of being erased or reduced to bool.

## Design principle

Structural clarity and modular geometry make the four-state surface readable.
A strong lowercase wordmark gives the computational core a stable typographic
counterweight. Physical computation and restrained acrylic materiality belong
to large artwork; single-color and small tooling surfaces use flat geometry.
This separation preserves identity when reflections and texture cannot survive.

The state palette is semantic. Graphite, terracotta, emerald, and cobalt stay
attached to N/F/T/S in that order. The flat package preserves the supplied
palette values and avoids a collection of unrelated variants. This brand
specification does not introduce UI runtime tokens or change their authority.

## Historical/design inspiration

Inspired by the clarity, modular discipline, and systems-oriented thinking
associated with Swiss modernism and late-20th-century computing identity
design. Industrial systems design and strong typographic hierarchy inform
the method, rather than supplying source artwork.

NeXT-era identity work associated with Paul Rand is a historical reference
for clarity and system-level thinking. Historical design reference, not a
source asset or derivative mark.

Semantic does not reproduce or derive from the NeXT logo. The reference is methodological rather than graphical.

No affiliation, endorsement, ownership, or authorship by Paul Rand or NeXT
is claimed. The Semantic reference was supplied by the repository owner;
its original board is preserved, while flat constructions and the documented
open-source wordmark substitute are separately reviewable sources.
