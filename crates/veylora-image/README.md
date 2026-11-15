# Bounded PNG import

Public entry point: `decode_png(bytes: &[u8], limits: DecodeLimits) -> Result<SourceAsset, ImageError>`.
The crate depends only on the existing domain `AssetId` and `ContentHash` contracts, not job types.
`SourceAsset` retains exact original bytes/hash, a fresh asset ID, source metadata, optional profile bytes/hash,
alpha description, and a `DecodedImage`. Repeated imports have identical samples and hashes but fresh IDs.
Samples remain in source encoding. Alpha is straight coverage, never gamma transformed or premultiplied.
A downstream renderer MUST mask hidden RGB by coverage before generating ink or interpolating edges.

## Supported subset and policy

- Noninterlaced, 8-bit RGB, RGBA, grayscale, and grayscale-alpha PNG.
- No resampling, compositing, color conversion, orientation inference, or printer interaction.
- Color precedence: supported sRGB `cICP` tuple `[1,13,0,1]`, then `iCCP`, then `sRGB`, then `gAMA`/`cHRM`.
- Simultaneous ICC and sRGB tags fail. sRGB/cICP plus contradictory legacy gamma/chromaticities fail.
  Overridden ICC or legacy metadata produces a warning. Original bytes retain every tag.
  ICC versus cICP color equivalence is not evaluated by this importer.
- Untagged sources are explicitly `AssumedSrgb` with a warning under `decode_png`. `decode_png_with_policy`
  takes an `UntaggedPolicy`: `AssumeSrgb`, or `Reject` (`image.untagged_color_rejected`, before any raster
  allocation). The policy never overrides an explicit tag and is recorded in `ImageMetadata::untagged_policy`.
  Any later operator color assignment belongs to the color-policy layer and must be recorded there.
- `pHYs` X and Y densities remain independent. `physical_size_mm()` proposes size only when units are
  meters and both values are nonzero. Unknown units or zero density never fabricate physical dimensions.
- ICC validation covers zlib integrity, name, declared length, signature, v2/v4 header, source color model,
  profile class, PCS, intent, and tag table ranges/alignment. It does NOT validate tag payload semantics,
  required transform tags, or transform usability. The color engine must validate before use.
- Stable `ImageError::code()` strings distinguish resource, container, profile, color, and unsupported failures.

## Bounds and validation order

1. Check input byte cap. Scan container without allocation or decompression, validating signature, chunk
   lengths/CRC/types/order, dimensions, pixels, decoded sample bytes, and cumulative metadata bytes.
2. Reject unsupported compressed text, animation, HDR, and format features before decoder invocation.
3. ICC inflation first uses fixed 8 KiB scratch with a strict output cap, then allocates exactly the proven
   size and inflates again. One overflow sentinel byte is permitted in scratch, never retained.
4. Validate IDAT zlib checksum, exact filtered length and stream end using fixed scratch before raster
   allocation. This rejects extra output, truncated streams, concatenated streams and trailing compressed bytes.
5. Decode with PNG's separate scratch budget and exact-size fallible output allocation. Copy original bytes
   only after successful decoding. Decoder ICC/text processing is disabled to prevent duplicate inflation.

Limits are independent caps, not a single process RSS cap: retained source + decoded samples + profile +
decoder scratch coexist, along with fixed decompressor state and small metadata. Callers own the input
slice and must bound file reading before constructing it. Decode is synchronous and currently has no
cancellation callback. Extra validation inflation trades CPU for explicit resource and integrity checks.

## Explicit rejections

16-bit and sub-8-bit samples, indexed color, Adam7, tRNS, sBIT, EXIF, compressed/international text,
unknown chunks, APNG, PQ/HLG or mastering/content-light HDR tags, and non-sRGB cICP encodings.
These are rejected, not silently approximated. Plain text, background and timestamp chunks are retained
in original bytes but do not change samples. This is deliberately not a universal PNG validator or ICC engine.

## Verification

`cargo fmt --all -- --check`

`cargo clippy -p veylora-image --all-targets -- -D warnings`

`cargo test -p veylora-image`

Tests exercise synthetic fixtures, every truncation of a tiny PNG, CRC-valid malformed metadata,
IDAT/profile bombs, truncated/checksum-invalid zlib, split IDAT, decoder budgets, and source sample identity.
