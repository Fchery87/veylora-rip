# Synthetic PNG fixture provenance

Created for Veylora RIP bounded import tests on 2026-11-08. All sample values and tags are synthetic,
authored in `generate.cjs`. No external artwork, device capture, ICC profile, physical measurement,
or commercial RIP output is included. These fixtures establish software behavior only.

Regenerate with `node fixtures/images/generate.cjs` using Node's built-in zlib, fs and path modules.
PNG chunk framing and CRC are generated locally. Compression bytes can differ across zlib versions,
but decoded samples and declared tags are fixed by the generator. The checked-in binaries are canonical
for these tests. All images are 8-bit, noninterlaced RGBA, one row tall.

| File | Width | Purpose |
| --- | ---: | --- |
| hidden-rgb.png | 2 | Saturated hidden RGB `(255,0,127,0)` and opaque white `(255,255,255,255)` |
| alpha-ramp.png | 4 | RGB `(70,80,90)` with alpha 0, 64, 128, 255 |
| srgb.png | 2 | Explicit sRGB rendering intent 0 |
| conflicting-color-tags.png | 2 | sRGB with contradictory gamma 1.0 |
| unsupported-apng.png | 2 | Animation control marker, reject rather than select a frame |
| unsupported-hdr.png | 2 | cICP BT.2020/PQ HDR marker |
| non-square-ppi.png | 2 | pHYs 3780 by 7560 pixels per meter |

The test module additionally synthesizes zlib bombs, corrupt/truncated streams, chunk payloads and
minimal ICC header/tag-table envelopes. Those ICC envelopes are intentionally not usable color
transform profiles and are not presented as measured or calibrated profiles.
