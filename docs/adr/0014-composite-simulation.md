# ADR 0014: Composite simulation of retained logical planes

Status: accepted, 2026-11-21

## Decision
- `Workspace::composite_overview` (workflow) draws the five logical planes (C, M, Y, K, W) of the
  newest run as one RGB picture. Each plane is first checked against the manifest; one damaged or
  missing plane refuses the whole picture. Large pages are reduced by box average per plane, then
  composed, and the reduction factor is reported.
- The arithmetic is `plane::composite_color`: a fixed mid-gray backdrop (0.5); white ink lifts it to
  `0.5 + 0.5 * W`; each of C, M, Y then multiplies the matching channel by `1 - ink`, and K multiplies
  all three by `1 - K`. There is no ICC profile, ink model, dot gain or substrate.
- The picture is labeled a simulation of where ink goes, not color managed and not what the print will
  look like. The Inspect chip says `Composite simulation of retained planes` and `retained artifact`.
- It covers the Logical stage only. Physical slot and screened planes need the mode's channel map
  (slot letters), which is not mapped; asking for a composite there is refused with a message.
- In the view model the picture is the pseudo-file `composite` in `raster_files`, drawn through the
  ordinary plane-image call. It has no loupe, because it is not one plane.
- A damaged run is not composited. Choosing another stage in the UI leaves the composite.

## Why a fixed backdrop
DTF is printed on film and transferred to a garment the job does not know. A neutral backdrop makes
white ink visible without implying a garment color. Real RGB-to-CMYK separation and a color-managed
preview are separate work that waits on the licensed profile and color engine (see ADR 0010).
