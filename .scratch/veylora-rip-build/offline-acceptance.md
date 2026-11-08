# Offline acceptance checklist

Use original feature requirements, not only execution-plan headings. No lane is complete until reviewed implementation and tests establish these observable behaviors.

## Immutable inputs

- Snapshot retains typed layout and recipe data or content-addressed records with validated resolution.
- Setup, mode, profile, mask, calibration and software versions are pinned by hash.
- IDs cannot resolve to different bytes without a blocking mismatch.
- Creation metadata, duplicate asset IDs, missing mask references, invalid schemas and nonfinite numeric fields are tested.
- Serializable records cannot bypass constructor validation.

## Import and color

- Actual PNG pixels decode with declared encoding, alpha association and decoder version.
- Original bytes, hidden RGB and metadata survive import.
- Bounds precede allocation and decompression, including compressed ICC metadata.
- Grayscale masks have an explicit supported path separate from RGB artwork.
- cICP/iCCP/sRGB/cHRM/gAMA precedence and conflicts are explicit; unsupported HDR/APNG never silently flatten.
- Actual licensed ICC engine converts source to linear working representation and supported CMYK output profile, with intent, BPC and neutral/black policy recorded.
- No placeholder RGB-to-CMYK arithmetic or rejection-only adapter counts as color implementation.

## Separation and rendering

- White source, tonal/highlight policy, response curve, blend, physical choke/spread, coverage and wet-load limits are retained.
- Supplied mask cannot silently alter process alpha; deliberate extension masks are explicit.
- Opaque white survives crop and remains eligible with zero CMYK.
- Knockout/perforation final clear mask applies to every plane after morphology and remains clear after screening.
- Absolute page coordinates preserve screen phase across bands.
- Allocation does not double white; slot calibration remains separate.
- Disk-backed render uses bounded buffers and halos, cancellation and disk-space checks.
- Retained stage artifacts supply inspector samples and tiles, not recomputed substitutes.

## Durability and workflow

- State transitions and append-only attempt events survive restart.
- Failure before write differs from partial write and timeout after final write.
- Ambiguous states cannot automatically retry or duplicate attempts.
- Diagnostic export is not mislabeled printer-ready, and real rendered data is not mislabeled fixture data.
- Preflight distinguishes offline rendering eligibility from native submission eligibility.
- CLI and desktop execute the same application services.
- Reprint uses frozen inputs and explicitly creates a new attempt when appropriate.

## Calibration and distribution

- Chart IDs, patch counts, measurement provenance, malformed/missing patches and holdout separation are checked.
- Synthetic measurements never promote physical recipe status.
- Portable bundles reject path traversal, missing assets, checksum mismatches and incompatible schemas.
- Reports redact paths and exclude artwork unless opted in, with no automatic upload.
- Packaging and license checks run on actual artifacts. Unperformed clean-machine and physical acceptance stay pending.
