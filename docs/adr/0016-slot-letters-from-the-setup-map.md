# ADR 0016: Slot letters come from the setup's channel map

Status: accepted, 2026-11-21

## Decision
- Retained physical and screened planes are files named by protocol channel (`physical-p13.f32`,
  `screened-p13.u16`). The job's stored setup binds each logical channel to a physical slot and a
  protocol channel. `Workspace::job_device` now returns that map (`channels`, `channel_map_version`)
  with the tank and head labels of each slot.
- Inspect letters are assigned from that map, by `artifacts::slot_letters`: C, M, Y and K when the map
  has exactly one binding of that channel; white slots W1 and W2 by ascending slot number, when there
  are at most two. A map that is ambiguous (two cyan slots, three white slots) gets no letter for the
  ambiguous bindings, and choosing that letter shows nothing and says so.
- On the Slots and Screened stages a selected letter shows the one file its slot owns, with the
  protocol channel, slot, tank and head, and the plane statistics. With no letter chosen the files are
  listed by name as before.
- The view's note says which channel map version the letters come from and whether the setup is
  physically verified. An unverified setup's letters are "as recorded, not as observed on the
  printer": they say what the map claims, not what the ET-8550 does. The logical stage does not use
  the map and does not mention it.

## Why
The letters were withheld until they could be derived rather than assumed. They now are derived, from
the record the job was created with, so a reader can trace every letter to a binding. Nothing here
claims the map is right; that is what physical verification of the setup is for.
