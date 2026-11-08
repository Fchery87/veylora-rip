# Native feasibility evidence required

Status: needs-info
Date: 2026-11-08
Blocks: native encoder Task 5.1, physical submission Task 5.3, Gates A/D/E, production acceptance.

The source audit found no captured protocol artifact or measured printer configuration. Existing UI values (Home bench rev 3, PM-exp-01, W1/W2) are fixtures, not hardware facts.

## Evidence to collect before implementing supported protocol fields

1. Actual local printer identity, firmware observation, connection, conversion supplier and operating system.
2. Physical tank/slot labels, actual ink assignments, ink product, film and feed path. Never infer tank fills from a vendor channel sequence.
3. Labeled channel test with per-patch observed physical slot and photographs or measurements.
4. Legally obtained reference output for operator-owned bounded artwork, recording permission/source, software version, exact settings and SHA-256 hashes. Start with blank, one mark, shifted mark and one changed channel at a time.
5. Observations of framing, channel IDs, packing, compression, page origin, band order, termination, feed and eject. Unknown fields remain unknown.
6. Repeated native target observations across restart and fresh film, including intended deposition order and pressed-transfer orientation.
7. Supplier instructions and SDS for ink, film, powder and finishing. Physical operations stay under operator control.

## Acceptance boundary

A hash proves file identity, not that a printer executed it. A successful spool response proves transport acceptance, not a usable physical print. Software must not promote an inferred map or fixture mode on either basis. No automatic retry is permitted after a partial or uncertain submission.

The investigation CLI may retain diagnostic artifacts and observations without submitting. It must refuse unsupported native encoding and submission explicitly rather than labeling JSON, SVG, PNG or synthetic framed bytes as ET-8550 printer-ready output.
