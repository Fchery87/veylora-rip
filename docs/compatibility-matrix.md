# Compatibility matrix

Only what has been observed is marked as tested.

## Operating system

| Windows | Status |
| --- | --- |
| Windows 11 x64 (build 10.0.26200, Home) | Developed and tested here. Intended supported target. |
| Windows 11 other builds, Windows 10, ARM64, Server | Not tested. Not supported until a clean-machine run is recorded. |

## Runtime prerequisites

| Component | Needed by | Status |
| --- | --- | --- |
| Visual C++ runtime | `veylora-cli` (default Rust MSVC build) | Unconfirmed. Record the result of `clean-machine-test.ps1` on a machine without it, or link the CRT statically. |
| WebView2 runtime | Desktop app only | Not packaged yet. Distribution choice (installed runtime, bootstrapper or fixed runtime) is open; see `docs/dependency-inventory.md`. |
| Rust, Node, pnpm, cargo | Nothing at runtime | Excluded by the packaging checks; the installation test runs with them off `PATH`. |

## Device setup prerequisites

- The product targets one converted Epson ET-8550. **No native support is claimed.** Physical
  submission is refused and disabled until encoder evidence and the physical gate exist.
- A job needs a device setup, a print mode and a recipe. Compatibility is checked at preflight and
  incompatible combinations are refused.
- A setup is unverified until an operator supplies evidence for that machine. Verification does not
  travel in job archives and is not part of a release.
- Installing a release installs no printer driver and changes no printer setting.

## Data compatibility

- A release declares the newest workspace data schema it reads (`max_data_schema`). An update is
  refused if the workspace is newer. Older data is migrated by the persistence stores, which refuse
  data from a newer schema themselves.
- Job archives carry a layout version (`docs/job-format.md`); an unknown version is refused by name.

## Rollback

One previous release is kept. Rolling back does not downgrade workspace data. If a newer release
migrated the data, the older release will refuse it (schema out of range) rather than misread it, so
a rollback after a migrating update needs a workspace backup.
