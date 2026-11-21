# Release process

1. **Gate.** `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, and `scripts/tests/verify-release.tests.ps1` pass.
2. **License check.** `scripts/check-licenses.ps1 -Package veylora-cli` and `-Package veylora-app`
   must both pass (`package-release.ps1` runs them). They cover exactly what each packaged program
   links for Windows, plus the app's bundled frontend and fonts (ADR 0013). The repository-wide check
   (no `-Package`) still fails closed (dev tooling, native tools); it does not gate packaging.
   After any dependency, frontend or toolchain change run `scripts/generate-package-notices.ps1
   -Package veylora-cli,veylora-app -FrontendPackage veylora-app` (always both packages) and commit
   the regenerated inventory and notices.
3. **Package.** `scripts/package-release.ps1 -Version X.Y.Z -MaxDataSchema N` (needs pnpm and the
   Rust toolchain on the build machine) builds the frontend, both programs with a static C runtime
   and `tauri/custom-protocol`, starts the app once to confirm it shows its embedded pages, stages,
   writes `release-manifest.json`, verifies, zips and writes `veylora-rip-X.Y.Z.zip.sha256`. It
   never overwrites an existing release. `MaxDataSchema` is `current_schema_version()` of the
   persistence crate; raise it only with a migration and tests.
4. **Verify independently.** `scripts/verify-release.ps1 -Path <zip> -ExpectedSha256 <hash>
   -RunSmokeTest` on a different machine from the one that built it. The smoke test runs
   `veylora-cli --version` and `veylora-app.exe --version` (no window, no WebView2 needed).
5. **Installation test.** `installer/windows/clean-machine-test.ps1` on a clean Windows machine,
   with the update zip of the previous version. Record the machine, Windows build and output. Gate F
   requires this record; a run on a development machine does not count. It also starts the desktop
   app, requires it to serve its embedded pages, and requires the WebView2 runtime.
6. **Publish.** The zip and its SHA-256 are published through separate channels. Until releases are
   signed, the hash is the only integrity evidence.
7. **Update.** The operator runs `veylora-cli release install --install-root <root> --from <dir>
   --workspace <dir>`. After a failed or interrupted update: `veylora-cli release recover`. To go
   back: `veylora-cli release rollback`.

Versions are `MAJOR.MINOR.PATCH`, numbers only, and a release must be strictly newer than the one
installed. There are no pre-release tags, so ordering is never ambiguous.

## What must be excluded from a release

`node_modules`, `.git`, `target`, `Cargo.toml`, `Cargo.lock`, `package.json`, lock files,
`tsconfig.json`, `vite.config.*`, `rust-toolchain*`, and `.rs`, `.ts`, `.pdb`, `.rlib`, `.rmeta`
and `.ilk` files. `veylora-release` and `verify-release.ps1` both refuse them.
