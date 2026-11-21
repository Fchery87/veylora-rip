# Installer and update safety

Status: the release, verification and update-safety machinery exists and is tested. **No installer
is chosen or built yet, and nothing here has been run on a clean machine.**

## What exists

| Piece | Where | What it does |
| --- | --- | --- |
| Release manifest and verifier | `crates/veylora-release` | Lists every file with size and SHA-256; refuses extra or missing files, unsafe paths and development tooling |
| Side-by-side install store | `crates/veylora-release` (`InstallStore`) | `releases/<version>/` plus an atomically replaced `install-state.json`; keeps current and previous only |
| Update guard | `Workspace::update_blockers` | Refuses an update or rollback while a submission or render is in flight, or when job state cannot be read |
| CLI | `veylora-cli release …` | `manifest`, `verify`, `status`, `install`, `rollback`, `recover`, `blockers` |
| Packaging | `scripts/package-release.ps1` | License check, release build, stage, manifest, verify, zip, `.sha256` |
| Independent verification | `scripts/verify-release.ps1` | Re-checks a directory or zip in PowerShell without trusting the shipped program |
| Installation test | `installer/windows/clean-machine-test.ps1` | Install, update, refuse, roll back and recover with development tools removed from `PATH` |

## Behaviour that is guaranteed (and tested)

- An update never starts while a submission is in flight, a render is in progress, or job state is
  unreadable. Ambiguous and review jobs do not block; their state is durable.
- Nothing changes before the source release verifies, the version is strictly newer (numeric
  comparison) and the release can read the workspace's data schema.
- The switch is journaled. A crash before the commit is recognised on the next start and the update
  is abandoned; the old release is still current.
- A release that fails its post-install smoke test is removed and the old release restored.
- A damaged current release falls back to the previous one only if that verifies; otherwise
  recovery reports the damage and changes nothing.
- Rollback goes back one release, once, after verifying it, and is guarded like an update.
- Workspace data is outside the install root and is never touched by install or rollback.

## What is not done

- **Installer technology.** Options are the Tauri bundler's NSIS or MSI targets, or a signed zip
  with these scripts. The choice affects signing and WebView2 handling and is a product decision.
  `tauri.conf.json` has `bundle.active = false`.
- **The desktop app is not packaged.** Only `veylora-cli` is.
- **Signatures.** No signing certificate exists. `verify-release.ps1` reports unsigned executables
  and fails with `-RequireSignature`. Until releases are signed, integrity rests on a SHA-256 that
  the publisher distributes separately from the zip.
- **Clean-machine record.** The installation test has run only on the development machine.
- **Runtime prerequisites** (the Visual C++ runtime for the CLI, the WebView2 runtime for the app)
  have not been confirmed on a clean machine. See `docs/compatibility-matrix.md`.
- **Automatic updates.** There is no updater, no update feed and no network access. Updating is an
  explicit operator action with a release the operator already holds.
