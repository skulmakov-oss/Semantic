# Semantic Visual Identity v1.2

Status: Approved

Identity version: v1.2

Approval applies to the owner-supplied visual direction and original reference.
The flat source constructions and open-source wordmark substitute in this PR
remain subject to ordinary human review; they are not an exact traced master.

Visual identity versioning is independent from Semantic language/runtime release versioning.

## 1. Purpose and decision

Adopt the v1.2 master/utility identity system as Semantic's canonical visual
identity. It expresses the native four-state model and deterministic,
verifier-first architecture. Official assets are versioned; state colors and
quadrant order remain stable; future brand changes require explicit review.

This is a documentation and identity-asset decision, not a language, runtime,
UI-token, qualification, or release decision. The normative [Quad Logic Frame
v1](../spec/quad_logic_frame_v1.md), [constraints](../../CONSTRAINTS.md), and
technical project description continue to define Semantic's semantics.

## 2. Master Mark

The master lockup is a smoked-black acrylic computational cube, a front 2x2
N/F/T/S state matrix, a heavy lowercase `semantic` wordmark, and the descriptor
`DETERMINISTIC VERIFIED EXECUTION`. The body communicates structural and
execution discipline; the surface communicates native quad states.

Material: matte/frosted state faces, restrained physical depth, subtle bevels,
controlled reflections. No neon halo, excessive glow, decorative glassmorphism,
or unrelated effects. Preserve the supplied material reference as rendered
artwork, not a pseudo-vector approximation.

Canonical artwork: [original reference board](../../assets/brand/semantic/v1.2/previews/semantic_visual_identity_v1_2.png),
1448 x 1086 PNG. SHA-256:
`3f2b97ca2f91a90d5beb4f86489454e80d2750a7b2680683b2b079acc6b6c73d`.
The master remains inside this original board. No standalone `semantic_master_mark.svg`
or misleading standalone master PNG is provided. Obtain the approved original
standalone artwork before replacing the README hero or packaging master lockups.

## 3. Utility Mark

[Canonical monochrome SVG](../../assets/brand/semantic/v1.2/utility/semantic_utility_mark.svg)
and [reverse SVG](../../assets/brand/semantic/v1.2/utility/semantic_utility_mark_white.svg)
use a stable cube with left side, top, and front matrix. Flat black/white solid
fills and keylines replace material rendering. No gradients, raster images,
opacity effects, highlights, reflections, or textures are required.

The SVG is a reproducible flat construction aligned to the reference, not an
exact vectorization of its rendered pixels. Identity geometry is stable;
rendering/material is context-dependent. Front-plane coordinates are a
100-unit grid with four 44-unit tiles, a 6-unit central gutter, and fixed
projection `matrix(0.52 -0.17 0 0.60 60 54)` in a 128-unit canvas.

## 4. Wordmark and descriptor

Use lowercase `semantic`. Intended character: heavy neo-grotesque/Swiss
grotesk, near-black, visually balanced with the cube. The reference contains
the approved rendered lockup. Separate [black](../../assets/brand/semantic/v1.2/wordmark/semantic_wordmark_black.svg)
and [white](../../assets/brand/semantic/v1.2/wordmark/semantic_wordmark_white.svg)
SVGs use an outlined Open Sans ExtraBold 1.10 open-source substitute. They
have no embedded font or runtime dependency. See [provenance and license](../../assets/brand/semantic/v1.2/wordmark/NOTICE.md).
This substitute is documented rather than represented as the reference's
unknown exact typeface. Official lockup typography changes require review.

The descriptor is `DETERMINISTIC VERIFIED EXECUTION`. It supplements the full
canonical project description; it does not replace it or assert new release status.
No independent descriptor font asset is introduced.

## 5. Four-state mapping and palette

The only canonical numeric palette source is [palette.json](../../assets/brand/semantic/v1.2/palette.json).
Suggested v1.2 reference values are adopted unchanged. Asset fills reproduce
those values; gradients in the raster reference do not redefine them.

| Position | State | Normative name / encoding | README explanation | Color |
|---|---|---|---|---|
| Top left | N | Null / 00 | unknown / no sufficient evidence | Graphite |
| Top right | F | Strict False / 01 | evidence for false | Terracotta red |
| Bottom left | T | Strict True / 10 | evidence for true | Emerald green |
| Bottom right | S | Conflict / Super / 11 | evidence for both / conflict | Cobalt blue |

`Null` is the repository's canonical quad-state name, not a pointer, void
value, or an instruction to reinterpret the state. The brief's explanatory
terms match README; the normative names above must remain visible. S never
means merely the project name. Color alone must not carry semantic information
in technical UI: retain labels, ordering, and the textual legend.

## 6. App icon

The flat [app SVG](../../assets/brand/semantic/v1.2/icons/semantic_app_icon.svg)
uses the same cube projection and semantic fills. White N/S and black F/T
lettering preserve contrast without changing the palette. This is the utility
app rendition; the reference board's material app concept is retained there.
Packaging and future application integration require a separate task.

## 7. Semantic source-file icon

The official [.sm source SVG](../../assets/brand/semantic/v1.2/icons/semantic_file_icon_sm.svg)
and 256 px PNG place the same four-state cube in a document silhouette. A
neutral keyline supports light/dark project trees. Tiny `.sm` lettering is
omitted; the surrounding file name/extension supplies the type label.
External icon themes and IDE integrations are outside this change.

## 8. Small-size behavior and minimum sizes

| Size | Rendition |
|---|---|
| 128 px and above | Material master only when approved standalone artwork is available; flat utility is also valid |
| 64 px | Flat colored cube with N/F/T/S |
| 48, 32, 24, 16 px | Dedicated flat micro SVG without letters; preserve four colored quadrants |

The cube icon minimum is 16 px. For letter-bearing monochrome use, minimum
height is 64 px; use a letterless silhouette if constrained further and provide
a textual label. File icons may be displayed at 16 px or above with a file-name
label. Wordmark minimum width is 120 px. These are usage minima for the flat
sources; the original board is presentation artwork, not a micro icon.

## 9. Monochrome behavior

Use the black or reversed utility SVG for single-color reproduction. N/T remain
outlined dark tiles and F/S reversed light tiles, with labels identifying all
states. White areas represent substrate/negative space; do not treat them as
additional state colors. No reduced grayscale palette replaces the state legend.

## 10. Light and dark backgrounds

Use black wordmark/utility on light backgrounds, white wordmark/reversed utility
on dark backgrounds. App/file icons carry neutral keylines. Inspect contrast
on the final surface. The material reference board has its original light
background; no transparency or dark-background master is implied.

## 11. Spacing and clear space

Let `u` be one eighth of the flat icon canvas height (16 SVG units). Reserve
at least `u` outside the canvas on every side; never clip cube vertices. For
wordmarks reserve one quarter of the letter height on each side. Keep the
reference lockup's original composition and descriptor spacing; do not extract
and rearrange it without a separately reviewed lockup source.

## 12. Forbidden transformations

Do not recolor states, reorder quadrants, mirror, arbitrarily rotate, stretch,
add utility gradients/shadows, add master glow, replace state meanings, use
unapproved typography in official lockups, or combine the mark with unrelated
PROMETHEUS/Andromeda marks without a separate identity specification.

## 13. Asset/version policy

Canonical assets live under `assets/brand/semantic/v1.2/`. Preserve original
references and provenance. Flat SVGs are editable sources; PNGs are generated
exports. Future versions use a new versioned directory and explicit review;
never silently overwrite the meaning or reference of v1.2. Existing historical
`assets/brand/semantic-logo.png` is preserved and is not the v1.2 Master Mark.

## 14. Validation and delivery boundary

[Usage guidance](asset_usage.md) records offline commands and validation.
The exported set includes 16/24/32/48/64/128/256/512/1024 px app PNGs, utility
PNGs, and the source-file PNG. No proprietary fonts, third-party theme changes,
compiler/runtime changes, release claims, or unapproved material reconstruction
are part of this package.
