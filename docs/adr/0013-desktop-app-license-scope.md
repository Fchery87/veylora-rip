# ADR 0013: Release scope for the desktop app (Rust side)

Status: accepted, 2026-11-21. The frontend bundle, fonts and icons are not covered yet.

## Context
ADR 0010 cleared `veylora-cli` only. The owner chose to package the desktop app as a zip with the
existing side-by-side install scripts (installer technology and signing: see `installer/README.md`;
no certificate yet). Scoped to `veylora-app`, the generator found 224 crates and 11 blockers.

## Decision
- `generate-package-notices.ps1 -Package veylora-cli,veylora-app` writes one scope per program.
  Regenerating with a single package drops the other scope, so always pass both.
- **MPL-2.0 accepted by the owner (2026-11-21)** for unmodified third-party crates (`cssparser`,
  `cssparser-macros`, `selectors`, `dtoa-short`, `option-ext`, all pulled in by Tauri's HTML stack).
  The notice carries the MPL text and a copyleft note; the corresponding source is the published
  crate whose checksum `Cargo.lock` pins. If any of these crates is ever patched or vendored, the
  patched files must be published under MPL-2.0 and this decision revisited. Other copyleft licenses
  (GPL, LGPL) are still blockers.
- Crates that ship no license text get it from `docs/license-evidence/clarifications.json`: upstream
  text, SHA-256 checked before use. `selectors` ships MPL-2.0 headers but no LICENSE file, so it uses
  the MPL text from `cssparser`. `webview2-com-sys` statically links Microsoft's WebView2 loader
  (its DLL reports 1.0.3800.47); the notice includes that SDK's BSD-3-style license, taken from the
  NuGet package of the same version.
- Fixed a generator bug: a `[package]` table that was the last table in a manifest was read as having
  no license (`heck`, `strsim`, `option-ext`).

## Not done (the app still cannot be packaged)
- The bundled frontend: Svelte runtime, bits-ui and its dependencies, `@lucide/svelte`,
  `@tauri-apps/api` are compiled into the app and need notices.
- Fonts (Geist, OFL-1.1), app icons and any ICC profiles: `docs/profile-and-font-licensing.md`.
- `package-release.ps1` packages only the CLI; the app executable, WebView2 prerequisite handling
  and a clean-machine run are open.
- The global gate (`check-licenses.ps1` with no `-Package`) still fails closed.
