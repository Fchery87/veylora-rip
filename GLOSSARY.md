# Veylora RIP Glossary

## Logical channel

A semantic ink or output plane used by the renderer, such as C, M, Y, K, or W. A logical channel is not necessarily one physical printer slot.

## Physical slot

A physical ink position or output channel in the converted printer configuration.

## Protocol channel

The channel identifier used by the printer protocol or encoded device stream. It may differ from both the logical channel and the physical slot label.

## DeviceSetup

A versioned description of one physical printer configuration, including printer identity, firmware observation, conversion map, physical slots, ink, film, feed path, and verification evidence.

## PrintMode

An immutable tested bundle describing raster geometry, protocol encoding, media behavior, channel order, deposition behavior, and compatible quality settings for a DeviceSetup.

## Recipe

A named collection of calibration, profile, white policy, screening, ink limits, and finishing conditions used to produce a reproducible output.

## White policy

The declared method for generating, combining, shaping, limiting, and allocating the white plane.

## Job snapshot

An immutable record of all output-affecting inputs and references required to reproduce a job.

## Submission attempt

One append-only attempt to send a retained printer artifact to a device or transport.

## Ambiguous submission

A submission whose physical result cannot be determined reliably after timeout, disconnect, crash, or partial transport. It must enter review and must not be retried automatically.

## Logical plane

A continuous-tone or calibrated semantic output plane before allocation to physical channels and device screening.

## Physical-slot raster

The raster after logical channels have been mapped and allocated to physical printer slots.

## Screened device raster

The final raster or drop-code representation produced for the printer device after calibration, limiting, and screening.

## File sink

A deterministic output target that retains the exact printer-ready artifact without submitting it to a physical device.

## Native backend

The Veylora-owned device encoding and transport path that does not require a commercial RIP at runtime.
