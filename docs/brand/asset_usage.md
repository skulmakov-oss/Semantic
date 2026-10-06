# Asset usage and reproduction

Use [Visual Identity v1.2](visual_identity_v1_2.md) for geometry, semantics,
clear space, minimum sizes, and the version policy. Canonical assets:
[`assets/brand/semantic/v1.2/`](../../assets/brand/semantic/v1.2/).

| Asset | Intended use | Supplied form |
|---|---|---|
| Master Mark | Documentation title pages, README/site hero, presentations, release artwork after separate authorization | Original approved reference board only; material lockup is not supplied separately |
| Utility Mark | Monochrome, CLI docs, print, stamps, constrained reproduction | Black/reverse SVG and 512 px PNG |
| Wordmark | Headers and reviewed textual lockups | Black/white outlined SVG; documented open-source substitute |
| App Icon | Applications and future separately authorized packaging | Colored SVG, letterless micro SVG, nine PNG sizes |
| .sm Icon | Semantic source files and project trees | Document/cube SVG and 256 px PNG |
| Preview | Review of approved direction | Original 1448 x 1086 board |
| Scale checks | Review on light/dark backgrounds | Actual-size micro icons plus enlarged pixel views |

## Choosing a rendition

At 16/24/32/48 px use the micro SVG or corresponding PNG: no tiny letters,
material effects, or typography. At 64 px and above use the lettered flat
app geometry. At larger sizes a true material mark may be used only when the
approved standalone source exists. The original board is not a standalone
icon, texture, master SVG, or exact-vector source.

Use the appropriate black/white wordmark or utility for background contrast.
The file-type source uses no `.sm` lettering and no external theme integration.
For accessibility, provide an image label such as "Semantic" or
"Semantic .sm source file"; where the states themselves matter, supply the full textual
legend. Do not use color alone to convey execution or diagnostic status.

The palette values remain unchanged. White lettering meets a 4.5:1 contrast
ratio on N/S; black meets 4.5:1 on F/T. The supplied reference's white lettering
is preserved in that artwork; the flat export uses contrast-aware lettering.
This is a lettering-only legibility adjustment, not state recoloring.

## DON'T rules

- No state recoloring or changes to N/F/T/S quadrant order.
- No mirroring, arbitrary rotation, stretching, or skew beyond the defined projection.
- No drop shadows, gradients, or material textures on utility marks.
- No glow added to the master artwork.
- No replacement of native state meanings.
- No unapproved typography substitutions in official lockups.
- No combining with PROMETHEUS or Andromeda marks without a separate specification.
- No claim that a utility reconstruction is the exact material master vector.
- No automatic website, IDE, favicon packaging, or application integration in this PR.

## Offline exports

The canonical flat sources are SVG; numeric state colors live in `palette.json`.
The script uses a deliberately limited polygon/rectangle/straight-path SVG
renderer and rejects unsupported elements. It does not synthesize material art.

Prerequisites: Python 3.11+ and an already available Pillow installation.
Validated locally with Python 3.13 and Pillow 11.2.1. No dependencies or network
fetches are added to the repository. PNG encoding is deterministic with these
versions; across Pillow versions validate rendered pixels, since compression
bytes may differ.

From repository root:

```text
python scripts/brand/export_assets.py
python scripts/brand/export_assets.py --check
```

The first command regenerates 12 PNG exports and the scale contact sheet.
The second is read-only: validate SVG XML/viewBoxes, IDs/references, absence
of scripts/external URLs/fonts/raster payloads, palette and order, reference
hash, export dimensions and pixels, and independent micro quadrant samples.
App exports: 16, 24, 32, 48, 64, 128, 256, 512, 1024 px. Utility exports: 512 px
black and reverse. Source-file export: 256 px. File-icon readability should
also be checked at 16/24/32 px on the target IDE surface before integration.

Optional wordmark regeneration needs the hash-matched licensed Open Sans
ExtraBold 1.10 source font and fontTools 4.57.0, already available during asset
creation. No font file is committed. Use the path to your licensed local copy:

```text
python scripts/brand/export_assets.py --wordmark-font PATH_TO_OPEN_SANS_EXTRABOLD
```

See [wordmark NOTICE](../../assets/brand/semantic/v1.2/wordmark/NOTICE.md) and
its adjacent Apache-2.0 license. Normal icon export requires neither fontTools
nor a source font. Do not substitute a different font under this command.

## Provenance and limitations

The original owner-supplied reference PNG is byte-for-byte preserved; its
SHA-256 is recorded in the specification and checked by the exporter.
`master/README.md` identifies the material-source boundary. It contains no
placeholder artwork. The preview board contains both approved material and
utility examples; do not relabel the historical ribbon logo as v1.2.

The new flat SVGs reproduce the conceptual geometry, not the exact pixel
edges of the board. The outlined wordmark is an explicitly documented
substitute, not an identified match to the board's original typeface.
The existing README hero remains until an approved standalone master lockup
can replace it in a separately reviewed change.
