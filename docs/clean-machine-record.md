# Clean-machine run record

Gate F needs one filled-in copy of this record per Windows build. A run on a development machine
does not count. Copy this file to `docs/clean-machine-records/<date>-<windows-build>.md`, fill it in,
and update `docs/compatibility-matrix.md` from it.

## How to run

You need two release zips (the version under test and the previous one), their SHA-256 values
(published separately from the zips), and a machine with no development tools.

1. **Machine.** A spare PC, a fresh VM, or Windows Sandbox. Windows Sandbox needs Windows 11
   Pro/Enterprise/Education with the feature enabled (admin rights, reboot). Windows 11 Home has
   neither Sandbox nor Hyper-V, so use a spare PC or another VM host there.
2. **PowerShell 7.2+.** The scripts require `pwsh` 7.2 or later. Windows ships only Windows
   PowerShell 5.1, so install PowerShell 7 on the clean machine first (the MSI from the official
   PowerShell release page). Record the version below. Nothing else may be installed: no Rust, Node,
   pnpm, Visual Studio or the VC++ redistributable.
3. **Copy in** the repository's `scripts/` and `installer/` folders (not the source tree), both
   zips and the hash values.
4. **Run** from the folder holding `scripts/` and `installer/`:

   ```powershell
   pwsh -NoProfile -File installer\windows\clean-machine-test.ps1 `
     -Zip C:\drop\veylora-rip-0.0.1.zip -ExpectedSha256 <hash of 0.0.1> `
     -UpdateZip C:\drop\veylora-rip-0.0.2.zip -UpdateSha256 <hash of 0.0.2> |
     Tee-Object C:\drop\clean-machine-output.txt
   ```

   `-Zip` is installed first and `-UpdateZip` is the update, so give the older release as `-Zip`
   and the version under test as `-UpdateZip`. Do not edit the script to make it pass.
5. **Capture** `$PSVersionTable`, `winver`, `Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows
   NT\CurrentVersion'` (DisplayVersion, CurrentBuild, UBR) and the full output file.
6. **Launch by hand** once: start `veylora-app.exe` from the unpacked release and confirm the
   window shows the Veylora RIP page. Screenshot it.

## Record

| Field | Value |
| --- | --- |
| Date | |
| Operator | |
| Machine kind (physical / VM / Sandbox) | |
| Windows edition and build (CurrentBuild.UBR) | |
| Architecture | |
| PowerShell version | |
| WebView2 runtime version (preinstalled or not) | |
| Release under test and zip SHA-256 | |
| Update zip and SHA-256 | |
| Anything installed beyond PowerShell 7 | |

### Result

| Step | Pass / fail | Note |
| --- | --- | --- |
| Zip verifies against the published hash | | |
| `veylora-cli --version` and `veylora-app.exe --version` | | |
| Diagnostic target render | | |
| Install, update, refused downgrade, rollback, recovery | | |
| WebView2 runtime check | | |
| App launch check shows embedded pages | | |
| Manual launch shows the Veylora RIP window | | |

Attach: `clean-machine-output.txt`, the screenshot, the version output.

### Failures and deviations

List every failure, every step run by hand, and every deviation from the instructions above. A run
with an unexplained deviation does not satisfy Gate F.

### Matrix update

Only after every step passes: add the Windows build to `docs/compatibility-matrix.md` as observed
on a clean machine, and change the VC++ runtime and WebView2 rows from "not yet observed" to the
observed result. Do not widen the claim to other Windows builds.
