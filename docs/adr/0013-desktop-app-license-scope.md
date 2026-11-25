# ADR 0013: Release scope for the desktop app

Status: accepted, 2026-11-21.

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

- **Frontend bundle.** `generate-package-notices.ps1 -FrontendPackage veylora-app` runs
  `scripts/frontend-bundle.mjs`, which builds the frontend with source maps and lists the npm packages
  actually compiled into it (10 today, including transitive ones such as `clsx` and `esm-env`) plus
  every emitted file. Each package gets its license files verbatim and its integrity from
  `app/pnpm-lock.yaml`. `svelte-toolbelt` declares no license in `package.json`; it is accepted only
  because its LICENSE file is plainly the MIT text, and the notice says so. Dev-only tooling is not
  in the bundle and is not part of the scope.
- **Fonts.** Each emitted `.woff2` must be byte-identical to a file in an installed
  `@fontsource-variable/*` package (no modification, no renaming); the notice lists every file's
  SHA-256 and carries the package's OFL-1.1 text and copyright. **OFL-1.1 is treated as accepted by the
  owner** because the owner chose Geist; this is recorded in `$AcceptedCopyleft` and can be reversed
  there. The Geist copyright lines declare no Reserved Font Name.
- **Icons.** `crates/veylora-app/icons-src/veylora-mark.svg` is simple original artwork (four offset
  strokes on a rounded square, no third-party shapes) committed by the owner; the icons are generated
  from it. They fall under the project notice. Trademark and domain clearance for the name and mark
  were never done and are not a license matter.
- **Binding.** A scope may list `files`; the checker binds each to its inventoried hash. The app's
  scope binds `app/package.json` and `app/pnpm-lock.yaml`, so a changed frontend lock fails the
  check until the inventory is regenerated.

## Not done
- ICC profiles: none are bundled.
- WebView2 prerequisite handling and a clean-machine run are open. `package-release.ps1` packages
  the desktop app as well as the CLI and launch-checks it on the development machine.
- The global gate (`check-licenses.ps1` with no `-Package`) still fails closed.
