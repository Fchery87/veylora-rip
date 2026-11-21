# Release process

1. **Gate.** `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, and `scripts/tests/verify-release.tests.ps1` pass.
2. **License check.** `scripts/check-licenses.ps1` must pass. It fails closed and is not yet
   passing (task 9.3), so `package-release.ps1` cannot produce a release today.
3. **Package.** `scripts/package-release.ps1 -Version X.Y.Z -MaxDataSchema N` builds, stages,
   writes `release-manifest.json`, verifies, zips and writes `veylora-rip-X.Y.Z.zip.sha256`. It
   never overwrites an existing release. `MaxDataSchema` is `current_schema_version()` of the
   persistence crate; raise it only with a migration and tests.
4. **Verify independently.** `scripts/verify-release.ps1 -Path <zip> -ExpectedSha256 <hash>
   -RunSmokeTest` on a different machine from the one that built it.
5. **Installation test.** `installer/windows/clean-machine-test.ps1` on a clean Windows machine,
   with the update zip of the previous version. Record the machine, Windows build and output. Gate F
   requires this record; a run on a development machine does not count.
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
