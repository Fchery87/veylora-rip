# Dependency inventory

Task 9.3, audited 2026-11-08. **Release status: BLOCKED.** This is an engineering
inventory, not a redistribution grant or legal opinion. No commercial RIP is an
allowed runtime dependency. Digital Factory, AcroRIP and PrintFab are reference
products only, never bundled components.

## Scope and authoritative evidence

The machine-readable block below inventories every package identity currently in
`Cargo.lock`, the `packages` section of `app/pnpm-lock.yaml`, all direct declarations
in `app/package.json`, and the package manager pin. All targets and build/test
packages are included conservatively, not just the eventual Windows runtime.
A lock entry is evidence of a resolved identity, **not proof that it ships**.
Snapshot: 408 Rust identities (403 external plus five workspace packages),
15 locked npm/tool identities and 15 additional frontend declarations, totaling
438 component records. There are 68 hashed evidence files. Of the component
license declarations, 194 remain `UNKNOWN`, five are project `UNLICENSED`, and
239 were extracted from exact-version cached Rust manifests. None is cleared.

- Cargo: exact registry crate archive identified by `Cargo.lock` checksum is the
  authoritative distribution. Its `Cargo.toml`, `LICENSE*`, `COPYING*` and `NOTICE*`
  establish declarations and obligations. Version-specific docs.rs source links
  identify review locations, not proof that their contents have been inspected.
  License expressions were read where an exact-version local Cargo registry
  manifest was available. These are declarations only, not approvals. Missing
  cache entries remain `UNKNOWN`. Local cache integrity was not independently
  verified against every archive checksum, so every component remains unresolved.
- npm: exact registry version metadata and integrity-pinned tarball, its
  `package.json`, licenses, notices and bundled subdependencies are authoritative.
  The registry URLs below are review targets, not fetched or approved evidence.
  There is no installed `app/node_modules` in this worktree. No registry license
  is inferred from a package name or another version.
- Frontend **lock completeness defect**: the current pnpm v9-format lock contains
  only pnpm 12.18.0 and its 14 platform executable packages. It contains none of
  the 15 direct application/tool dependencies in `app/package.json`. Their exact
  manifest pins are inventoried as declarations, not falsely labeled locked.
  Resolve the lock in the owning frontend task, then regenerate/review the full
  transitive inventory. This task does not modify manifests or locks.
- Local Veylora packages inherit `UNLICENSED` from workspace `Cargo.toml`.
  `publish = false` is not a redistribution permission. Project owner authorization
  and the intended distribution terms remain a release blocker.

## Native libraries and platform dependencies

These are candidate platform/build surfaces visible in the locked graph. The
actual release link and installer contents must be audited before clearance.
All obligations below are **unresolved**, even where Rust wrapper metadata is known.

| Component/surface | Evidence and authoritative license source | Scope and blocker |
| --- | --- | --- |
| Tauri, wry, tao and WebView2 loader/runtime | Locked crate source licenses, `webview2-com`/`webview2-com-sys` package contents, Microsoft WebView2 distribution terms at https://developer.microsoft.com/microsoft-edge/webview2/ | Windows shell. Wrapper licenses do not grant runtime redistribution. Decide installed runtime vs bootstrapper vs fixed runtime, review exact SDK loader and runtime terms. |
| Windows API bindings and OS DLLs | `windows*` crates in lock, crate licenses, Microsoft Windows SDK terms | Bindings and system components are different artifacts. Do not copy system DLLs or SDK redistributables without terms. |
| GTK, GLib, GObject, GIO, GDK, Pango, Cairo, ATK, WebKitGTK, JavaScriptCore, libsoup | `*-sys` and corresponding wrapper crates in Cargo.lock, native project release LICENSE/COPYING files | Non-Windows transitive graph. Native licenses are not established by wrapper metadata. No Linux binary redistribution is cleared. |
| Apple frameworks/Objective-C bindings | Locked `objc*`, `core-*` crates and Apple SDK terms | Non-Windows graph. No Apple framework redistribution is cleared. |
| Compression, image decoding and crypto transitive code | Exact locked packages including PNG/flate and crypto crates, their source and bundled notices | Check vendored native or copied code, features, build scripts and final linkage. Do not assume all code is covered by the top-level crate license. |
| LittleCMS, SQLite, Ghostscript, PDF renderers and other planned engines | Future implementation manifests and exact upstream source/license required | Not dependencies declared by the current workspace manifests. No permission or runtime requirement is invented. Adding one invalidates this inventory. |

## External tools

| Tool | Evidence/source | Distribution policy and status |
| --- | --- | --- |
| Rust, Cargo, rustfmt, Clippy | CI uses stable toolchain, https://github.com/rust-lang/rust/tree/master/LICENSES and toolchain distribution notices | Build only, unpinned stable changes. Not included in runtime clearance. Review exact toolchain if redistributed. |
| Node.js 22, npm/npx | `.github/workflows/app.yml`, https://github.com/nodejs/node/blob/main/LICENSE | Build only. Review exact Node release and bundled dependency notices if shipped. |
| pnpm 12.18.0 and `@pnpm/exe.*` | Lock integrity records and exact registry package tarballs | Build only but includes native executable payloads. Unreviewed. |
| Vite+ 1.5.0, TypeScript, Svelte tooling, Tauri CLI | `app/package.json`, exact npm tarballs and nested binary payload notices | Build/test tools. Declaration alone does not prove that every tool file is excluded from an installer. Unreviewed. |
| Git, PowerShell 7.2+ | This check and repository workflow, https://github.com/git/git/blob/master/COPYING and https://github.com/PowerShell/PowerShell/blob/master/LICENSE.txt | Developer/release-check prerequisites only, not app runtime dependencies. Review exact releases if redistributed. |
| MSVC/Windows SDK, installer toolchain | Selected Windows builder and future installer scripts, Microsoft and installer vendor terms | Exact versions, redist contents and permissions not yet established. Tauri bundling is currently disabled. |
| GitHub Actions checkout/setup-node/rust-toolchain | `.github/workflows/*.yml`, each action's exact resolved source LICENSE | CI service/build workflow only. Major branch/tag pins are not immutable license evidence. Not bundled or cleared. |

## Profiles, fonts and fixtures

See [profile-and-font-licensing.md](profile-and-font-licensing.md). Evidence files
below include all tracked/non-ignored profile/font/image/library extensions,
`fixtures/**`, `*/tests/golden/**`, application font imports, manifests, locks,
build scripts, Tauri configuration and CI workflows. These hashes detect additions,
removals and content changes in that scope. Icons and synthetic fixtures have
unresolved provenance and distribution authorization, not assumed public-domain status.

## Release scopes

The repository-wide gate below still fails closed. Separately, a **release scope** gates one
packaged program: `scripts/check-licenses.ps1 -Package veylora-cli` checks the exact set Cargo
links for `x86_64-pc-windows-msvc` (normal dependencies only; dev and build-only crates are not
shipped) plus the Rust standard library of the toolchain that builds it.

- `scripts/generate-package-notices.ps1` is the single writer. It marks a crate `reviewed` only
  if its cached archive matches the `Cargo.lock` checksum, its declared SPDX expression can be
  satisfied with permissive licenses only, and the crate source carries license text, which is
  copied verbatim into `THIRD_PARTY_NOTICES.md` (qcms ships no LICENSE file, so the identical
  grant in its source headers is reproduced). Anything else stays unresolved and is listed as a
  scope blocker. This is an automated policy decision under the owner's authorization for this
  scope, not a lawyer's review of each license.
- The review is bound to the hashes of `Cargo.lock` and the member manifests; drift fails.
- The program is built with a statically linked C runtime, so no Visual C++ runtime is needed
  or redistributed. The standard library's own third-party notice file is shipped as
  `licenses/rust-std-COPYRIGHT.html` and checked against the reviewed hash.
- **Project terms:** proprietary, all rights reserved (owner decision 2026-11-20). Binaries may
  be distributed; no license to source is granted. `Cargo.toml` keeps `UNLICENSED`, which is why
  the repository-wide gate still reports the workspace license blocker.
- **Not covered, still blocked:** the Tauri desktop app, the frontend packages and lock defect,
  icons, ICC profiles, fonts and fixtures. Passing the CLI scope clears nothing else.

## Executable release gate

From repository root, using PowerShell 7.2+ and Git:

```powershell
pwsh -NoProfile -File scripts/check-licenses.ps1
pwsh -NoProfile -File scripts/tests/check-licenses.tests.ps1
pwsh -NoProfile -File scripts/tests/check-licenses.tests.ps1 -AssertWorkspaceBlocked
```

Normal check is read-only/offline. Exit 0 means metadata, reviewed status, recorded
notice text hashes and evidence coverage passed. Exit 1 means blocked or malformed.
The current workspace **must return 1**. Each component needs a nonempty license,
manifest source, license source, `reviewed` status, notice key and SHA-256 of the
notice block's trimmed LF-normalized UTF-8 text. Notice delimiters are
`<!-- notice:KEY -->` and `<!-- /notice:KEY -->`. Full required copyright/license
and NOTICE content belongs between them, not merely an SPDX label or a link.
The checker verifies exact recorded text, not the legal sufficiency of that text.
An accountable human must review and approve obligations before recording a hash.

`UNKNOWN`, `UNLICENSED`, `NOASSERTION`, unresolved statuses, missing notices,
commercial RIP names, missing metadata, changed evidence, new dependencies/assets,
unsupported lock formats, malformed inventory and explicit blockers fail closed.
Do not clear blockers just to make the command green. `reviewed` requires documented
redistribution scope and fulfillment of obligations, not a guessed permissive license.

For an explicitly unreviewed refresh after dependency changes:

```powershell
pwsh -NoProfile -File scripts/check-licenses.ps1 -RefreshUnreviewedInventory
```

This replaces the machine block, resets **all** package review statuses and notice
references, and retains conservative blockers. It is a capture command, not a
release check, and must never replace the normal check in CI. Review the diff.
It reads only locally available Rust manifests and does not download or alter locks.

### Boundaries and release follow-up

- This is not a full TOML/YAML/SPDX solver or a binary SBOM scanner. It recognizes
  current Cargo lock package headers and pnpm v9 package keys. Hashes catch broader
  manifest/lock edits. Repeated identities from distinct Cargo sources require
  source-aware schema extension and review before release.
- Frontend declarations must be exact pins matching a pnpm package key. Semver
  ranges, aliases and workspace protocols fail closed until a resolver-aware
  review/schema is implemented. This checker does not infer their resolved version.
- Git-ignored build output, downloaded installers, operator-local profiles and
  assets hidden under unrecognized extensions are not automatically discovered.
  Native libraries, external tools and fixture provenance require human review.
- Review final executable/installer payloads and transitive binary notices, ship
  required notices accessibly, and verify the About/licenses view and installed
  files. Repository success alone cannot prove installed notice delivery.
- Coordinating packaging/CI must invoke the normal check and honor its exit code.
  This change deliberately does not edit another worker's packaging or CI files.
- Byte hashes of evidence are intentional. Different checkout newline policies
  can require a reviewed refresh. Do not silently bless content changes.
- 2026-11-08: `feat/icc-color-boundary` adds the `qcms` 0.3.0 crate (crates.io
  metadata `MIT`, pure Rust, no C build dependency) as the ICC engine for
  `veylora-domain::color`. The machine-inventory block below has **not** been
  regenerated for this change and the release gate therefore remains blocked
  until `scripts/check-licenses.ps1 -RefreshUnreviewedInventory` is run and a
  reviewer walks the new component through the UNRESOLVED -> reviewed states.
  Nothing in this change is cleared for distribution; the `qcms` entry is
  tracked for honesty, not clearance. See ADR 0002 for the engine choice.

<!-- machine-inventory -->
```json license-inventory
{
  "schema": 1,
  "files": [
    {
      "path": ".github/workflows/app.yml",
      "sha256": "A5831B2E6330D0EBA895CB1C8D2893BF5FB3B24491802A6A5CB0797A627B1071"
    },
    {
      "path": ".github/workflows/ci.yml",
      "sha256": "0063EFAA65889924757A15DA4F8CA68BC9126E727406997214BCA684B74D5DF9"
    },
    {
      "path": "app/package.json",
      "sha256": "741BABC768E165B848F2FEBFF4100762185BE04D4A4F8EC2E35F2ADAABD33AE5"
    },
    {
      "path": "app/pnpm-lock.yaml",
      "sha256": "375D8B7E30CAAD2A8C6E0F0DCAE9A2C2F5074C130EB236E606291D5A42354BDF"
    },
    {
      "path": "app/src/main.ts",
      "sha256": "E331D8AA02A863D3925E4427969A46727C3B1B0C6B0E99CE435D2FDCCABB6B6F"
    },
    {
      "path": "Cargo.lock",
      "sha256": "4EC8AC549B79039298E7088616AE3D6E39FEA4DBE5C1B90C2489362EC0F25E61"
    },
    {
      "path": "Cargo.toml",
      "sha256": "E8058342D40B566EA1F7AFAB6928A8476E0342815F0FBE972A5AC2EBBCB877A3"
    },
    {
      "path": "crates/veylora-app/build.rs",
      "sha256": "487059EAF8A947B80F20A9AACAC038A5047B2AD69D2401B827376C67D6FE847F"
    },
    {
      "path": "crates/veylora-app/Cargo.toml",
      "sha256": "AD2D1BC7EF9A90EB827C38E9C8FBAE3406BBD3D079F595BA90831C15150167A3"
    },
    {
      "path": "crates/veylora-app/icons-src/veylora-mark.svg",
      "sha256": "2AC56E1E1F46BCB27766D617A7237F8D9AE37B7C8653254169416EBAB50E2DC5"
    },
    {
      "path": "crates/veylora-app/icons/128x128.png",
      "sha256": "2A1596D76BAA0F2684E061157ED7BDFED8EBC2245C9947B29FC67118D341FBB6"
    },
    {
      "path": "crates/veylora-app/icons/128x128@2x.png",
      "sha256": "DB48021F62B51BE98F55F355460F31F37BC7585AF69E40F71E402DA718296A72"
    },
    {
      "path": "crates/veylora-app/icons/32x32.png",
      "sha256": "B474E906271D6C2ACEEA70EEF7EE81935DB0B501E7EDA4DE36D7E9623D678CAC"
    },
    {
      "path": "crates/veylora-app/icons/64x64.png",
      "sha256": "E0B6F34FBCE1BC88E4E15989610E64E7F2FC982E9ABC289DBA1E784E3676B187"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-hdpi/ic_launcher_foreground.png",
      "sha256": "07BEB588F284A7F4451F3E6F4FB1172C84F6EB3B6CF4CCBD950C0DD3FE95E22A"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-hdpi/ic_launcher_round.png",
      "sha256": "5F57AA9C4B3DFFED2B07F22615284C2B23D0955607CE4B01D8AF7F7A53EFB077"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-hdpi/ic_launcher.png",
      "sha256": "1FD4E8B4640F56EA5D06C9B534EEC950C3039C24556929EC52C0B832517F4F47"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-mdpi/ic_launcher_foreground.png",
      "sha256": "0DB2161479BA6A093F34606BE018659EC49E3B99B990626C141324B891E8B4DF"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-mdpi/ic_launcher_round.png",
      "sha256": "7512ED5A85E923CDD30DBAB39E799455D57DC2498D8AACE4777CA0906A3F0F6F"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-mdpi/ic_launcher.png",
      "sha256": "D64AC3E231FB809113308B1C394A173FF174A9393ED86DC6F9A41788A19689BE"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xhdpi/ic_launcher_foreground.png",
      "sha256": "4D625DF110C48B122FBD9982D7BC017F8007909166FBE1E0A8747CAAE06E11FE"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xhdpi/ic_launcher_round.png",
      "sha256": "688097511756637D95A9AF31A89414366150196C6DAC0102588139429C6DC14D"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xhdpi/ic_launcher.png",
      "sha256": "B5E12F272AA164D5CFE6B28088A32CF428949AD111EC50F022842E5A2F3616A9"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xxhdpi/ic_launcher_foreground.png",
      "sha256": "C92E5DDE548FA5C90690C94CBA2C5ECE9833F0A56DA71AF5CE01B2EA44888CCF"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xxhdpi/ic_launcher_round.png",
      "sha256": "13635F735C08FA7A73499615291AC4D42A2F0B09D7A9EA88A7C0B4D745E7905A"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xxhdpi/ic_launcher.png",
      "sha256": "E0E65541FC2894576EA830E222D2B5EB033AE57705497A79281F5C1EE1AF68E7"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xxxhdpi/ic_launcher_foreground.png",
      "sha256": "E488DA7B501F0997A619159B64A9669B72D25B59CD8914F5767FA4B8FB78D69D"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xxxhdpi/ic_launcher_round.png",
      "sha256": "BDA3EF91BC857397BABDA914E6CC60C14EE2BBF24661B62633252B14E600F44F"
    },
    {
      "path": "crates/veylora-app/icons/android/mipmap-xxxhdpi/ic_launcher.png",
      "sha256": "D0AD9C228229F2DA93F5DC411C37666633195455B3F01C7FDCFB6931DC3508C2"
    },
    {
      "path": "crates/veylora-app/icons/icon.icns",
      "sha256": "2DD3DFE801FDD88897D71D42B1BEBCBE9C3AB333019BEE028FCEAFA80EF37C5A"
    },
    {
      "path": "crates/veylora-app/icons/icon.ico",
      "sha256": "B0A8E06A0B9675D8FF7E2F6ADDCC778871089FD3F0B19F4CF0609D71430A0B0E"
    },
    {
      "path": "crates/veylora-app/icons/icon.png",
      "sha256": "7A5B2590013886BE881FD913B1D2ECE90A4F03356BD68B69CC217E636C897EB9"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-20x20@1x.png",
      "sha256": "22F9ABEC7C9D2C6C005F1F4362158FDE72D9C93E40C5B0685E36B8269B11A33F"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-20x20@2x-1.png",
      "sha256": "95C0F3C5F43F4D4220359B89D68104ADB53EDF581F452946E7AF0763F947B43C"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-20x20@2x.png",
      "sha256": "95C0F3C5F43F4D4220359B89D68104ADB53EDF581F452946E7AF0763F947B43C"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-20x20@3x.png",
      "sha256": "A34B5E0F1F044604625F61A5996A350ED86755E5E419F0898B6E6B62382394C9"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-29x29@1x.png",
      "sha256": "D6E9C34344C8FEF31163CD697F23AE7E0EEF6592F4874F1AA913FBF9C3B052D7"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-29x29@2x-1.png",
      "sha256": "AFBDC402620C6759BF9A7D5934BB644C52EDF9B0113A84B03446B787D5D982A0"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-29x29@2x.png",
      "sha256": "AFBDC402620C6759BF9A7D5934BB644C52EDF9B0113A84B03446B787D5D982A0"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-29x29@3x.png",
      "sha256": "99F626B8477508EC7ED7A84AE1DAC795ECCF3C87678DB6FE3FD12AABD6200671"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-40x40@1x.png",
      "sha256": "95C0F3C5F43F4D4220359B89D68104ADB53EDF581F452946E7AF0763F947B43C"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-40x40@2x-1.png",
      "sha256": "5B65DCB494816CEABCF58E61608ED58836B83ACA67EA60EAD9BDB74D414916BC"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-40x40@2x.png",
      "sha256": "5B65DCB494816CEABCF58E61608ED58836B83ACA67EA60EAD9BDB74D414916BC"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-40x40@3x.png",
      "sha256": "C76DE28473F8F3ECD51CCEA36E9B879644C51B1C06C48B59E1D17CD65C886C32"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-512@2x.png",
      "sha256": "0038E84841F78FB00440B74381452BD4B9B03A6AAEFD383B5CED1778A0295659"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-60x60@2x.png",
      "sha256": "C76DE28473F8F3ECD51CCEA36E9B879644C51B1C06C48B59E1D17CD65C886C32"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-60x60@3x.png",
      "sha256": "566410E4A58A09E937C1B4FD7BC381FC5AE57FC5FFB428AD62192B5E5A0C4A67"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-76x76@1x.png",
      "sha256": "198744CD7E31EF374E70B00E968D96015D5B3543851EF01C4A153DC2D27950D6"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-76x76@2x.png",
      "sha256": "EEED2FAC3EF18E5479ED1F33C1A94638908F831533E77EC987E0B170ACCC5F11"
    },
    {
      "path": "crates/veylora-app/icons/ios/AppIcon-83.5x83.5@2x.png",
      "sha256": "51A41A492470A78BB20775420B68BE15AF96344E26D386AE8F5E20602F459876"
    },
    {
      "path": "crates/veylora-app/icons/Square107x107Logo.png",
      "sha256": "EFAC73112D3002D19FF76295BA929604BFB55D23EF8A8B355A612279416404D3"
    },
    {
      "path": "crates/veylora-app/icons/Square142x142Logo.png",
      "sha256": "ECCCB791752CC3A7847E37A221FE61FEF6B3A7A97B7C8F924DAA4AFFA4CEC004"
    },
    {
      "path": "crates/veylora-app/icons/Square150x150Logo.png",
      "sha256": "3CB40B36C805810502FC4B08144CE7A307D34C13764AB557763E9C9B0D145E86"
    },
    {
      "path": "crates/veylora-app/icons/Square284x284Logo.png",
      "sha256": "7E31FD0C0870065179E150375843D04E822257264E5DAAAAF6448DE935567393"
    },
    {
      "path": "crates/veylora-app/icons/Square30x30Logo.png",
      "sha256": "0B4898E82F146181D5F996D73B0B9A7B408CF9EBEC37577E3C9DF95D4F0F0769"
    },
    {
      "path": "crates/veylora-app/icons/Square310x310Logo.png",
      "sha256": "79C3AF6CB9E410F8D19D1BE735664894DA13AFA225B6E124D709347B29E43DED"
    },
    {
      "path": "crates/veylora-app/icons/Square44x44Logo.png",
      "sha256": "E998BD1E123F85A04ED3EB51A1B1EFA6F5B8D09EEAFD322EEA5A787C5D498427"
    },
    {
      "path": "crates/veylora-app/icons/Square71x71Logo.png",
      "sha256": "4C6690E79F7A7B1E4B8D893FE0B65DD20C9D573C33CFFB637EE7A8A7027D5E00"
    },
    {
      "path": "crates/veylora-app/icons/Square89x89Logo.png",
      "sha256": "F9BF526EA5D47FF72596C1DC613ED2E1ABA86A6C7D3F93FB6489582BFD1BBDB0"
    },
    {
      "path": "crates/veylora-app/icons/StoreLogo.png",
      "sha256": "4137707A9B9D780BA218D9627527FD209675581E18B7217CE09AA6AF60BA22D9"
    },
    {
      "path": "crates/veylora-app/tauri.conf.json",
      "sha256": "8AFBC411CE6C389DD5E2F6123B6A3CD0E5B2937F2AAF7FD7DFE86CC9E48074A1"
    },
    {
      "path": "crates/veylora-app/tests/golden/presenters.json",
      "sha256": "FDCCC8DE5E5149496F1A70B6E7B3A32A6C0BBB7F295F9893F6CEFDBAA07C3A02"
    },
    {
      "path": "crates/veylora-cli/Cargo.toml",
      "sha256": "3817BFA322CFAA6419ADF95A17600290AB9B5A566D84875A4C61B318AEE046E5"
    },
    {
      "path": "crates/veylora-color/Cargo.toml",
      "sha256": "9D5DBD96CF7853D03ACDE9A8343B7B4A810E6956C81151C8002BBB9090AEA9FE"
    },
    {
      "path": "crates/veylora-compose/Cargo.toml",
      "sha256": "9CCDBFCA73946807B75A0EFE5F5816D1D0B70B6383344D06853E92DE98F3404B"
    },
    {
      "path": "crates/veylora-device/Cargo.toml",
      "sha256": "787D95A7E6B9165B274C9D23652527FEF2C0B911FEC9D09B6967228ADF69555B"
    },
    {
      "path": "crates/veylora-diagnostics/Cargo.toml",
      "sha256": "B8513ADB6FCD68BBCF3A548620A5AD60BC34BCAA3081EF9AF97BC7BD6F50129D"
    },
    {
      "path": "crates/veylora-domain/Cargo.toml",
      "sha256": "582F1DF4C58081D5D89957F149FDACDD7CBF96863FD8561D3B3676A98C63A56A"
    },
    {
      "path": "crates/veylora-image/Cargo.toml",
      "sha256": "D96DACBC241E55FDEA709AA9E83AD59CEAC8ECE613E0527480E47C4ED579EBAA"
    },
    {
      "path": "crates/veylora-layout/Cargo.toml",
      "sha256": "8E50A071C5723CF07F417C7067FF1EBC4E2BC73DE95D68A791D1C051C0B6E317"
    },
    {
      "path": "crates/veylora-persistence/Cargo.toml",
      "sha256": "FCC7DF46DC9F4C46850D46EDBFDB307724EE7E85225590B17174FF2714E31EAC"
    },
    {
      "path": "crates/veylora-release/Cargo.toml",
      "sha256": "405A8B8653C0BE7BB63DED32CF8A8F187F7778BCE7F6FFFA9FE372BE41A376CC"
    },
    {
      "path": "crates/veylora-render/Cargo.toml",
      "sha256": "553B6CC2052A507D51BD2D33FB8218BB67EBD37F90CC112C1957BCC2C8011E03"
    },
    {
      "path": "crates/veylora-screen/Cargo.toml",
      "sha256": "98E716A37E9CA0CC373902848D16CA457FB1AAA00EC6D129D1F7FFEAF7B720A1"
    },
    {
      "path": "crates/veylora-separation/Cargo.toml",
      "sha256": "E47096A85C95FAE1AC3F4C09294A93D9E66585A9CBCD4B154E52219A3F3B6573"
    },
    {
      "path": "crates/veylora-transport/Cargo.toml",
      "sha256": "B95FF222999BB650DAD77AFA3AABC8B8B43CD31DE4A25E32F1ABF473A84D1A90"
    },
    {
      "path": "crates/veylora-workflow/Cargo.toml",
      "sha256": "794CA932A9879B83263FAA6FC1510D29A51D69D18CBBE158EB05672E1F37851B"
    },
    {
      "path": "fixtures/images/alpha-ramp.png",
      "sha256": "684FFAD6FB1558221B206FA6E9F6757BA4113B5C359FAA3EFCBA1CA29435E70F"
    },
    {
      "path": "fixtures/images/conflicting-color-tags.png",
      "sha256": "1719A6D7B0F6ECE933ED01C265FD7D40932F510190AB78FE5278E2D9369E18B4"
    },
    {
      "path": "fixtures/images/generate.cjs",
      "sha256": "4AEC953318B15029A734C9DCC6D7B422A8D75C398CD287E0BE06A33C7A2598C2"
    },
    {
      "path": "fixtures/images/hidden-rgb.png",
      "sha256": "E0050DF646F740024F7EABEFBBE064310AD3CE57EC21477382F00EAA188ABEBD"
    },
    {
      "path": "fixtures/images/non-square-ppi.png",
      "sha256": "E9E546475BF3D602D91C722F39D014FD3F57495DBB1270E7D9CF26D2898EF115"
    },
    {
      "path": "fixtures/images/README.md",
      "sha256": "1D419C092E5D813C06B3988EFF8AA47341A42377D5DD50C1212C94597AF14598"
    },
    {
      "path": "fixtures/images/srgb.png",
      "sha256": "41172845A57A3105444614B2EF806E6591BC815F2D7CB6C1ABC23D2BBE25B019"
    },
    {
      "path": "fixtures/images/unsupported-apng.png",
      "sha256": "9269DC6A9C1878BB2DC5B4ED9A6191DA30CA3FAF9C6603BF30C4369DD7FA4E10"
    },
    {
      "path": "fixtures/images/unsupported-hdr.png",
      "sha256": "E0EB39AE704E263B855E3E7D2D619ED835EA9706544598A2A16D93E565871B98"
    },
    {
      "path": "fixtures/physical/README.md",
      "sha256": "C6D32A6836720F2990E156423B5E1D78A54E95A5EA9F6A6D2E2E5482A1E8C42D"
    },
    {
      "path": "fixtures/targets/channel-identity.json",
      "sha256": "18AF94DBC0B362E5E7E16D771795A6A0647FE9C8C6CA9D8967E0FE691F6E6AAC"
    }
  ],
  "components": [
    {
      "id": "cargo:adler2@2.0.1",
      "license": "0BSD OR MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/adler2/2.0.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-adler2-2.0.1",
      "noticeSha256": "44569F4A8B8498999B7CA38D14AAA3992AEF591E265D2F9F485226C2B84B9CA7"
    },
    {
      "id": "cargo:aho-corasick@1.1.5",
      "license": "Unlicense OR MIT",
      "source": "https://crates.io/api/v1/crates/aho-corasick/1.1.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-aho-corasick-1.1.5",
      "noticeSha256": "14D74D59C803F8840D2AF06FF53ADC488C481ECDE2675A87E39BF3F6DAF95E2F"
    },
    {
      "id": "cargo:alloc-no-stdlib@3.0.0",
      "license": "BSD-3-Clause",
      "source": "https://crates.io/api/v1/crates/alloc-no-stdlib/3.0.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-alloc-no-stdlib-3.0.0",
      "noticeSha256": "5DD0172A8A8991505B7A7E52F4E63EF388030589959E47EFF71AF6FD8AC4A941"
    },
    {
      "id": "cargo:alloc-stdlib@0.3.0",
      "license": "BSD-3-Clause",
      "source": "https://crates.io/api/v1/crates/alloc-stdlib/0.3.0/download (checksum from Cargo.lock)",
      "licenseSource": "license text from source-file headers of the checksum-verified crate (no LICENSE file shipped) plus upstream license text kept in docs/license-evidence (SHA-256 checked); permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-alloc-stdlib-0.3.0",
      "noticeSha256": "5B49FFD6ECBE4773F0C4E1D0790E6B9E8B2D4A1B42DD4E4F6682C5827D6FE674"
    },
    {
      "id": "cargo:android_system_properties@0.1.6",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/android_system_properties/0.1.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/android_system_properties/0.1.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:anyhow@1.0.104",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/anyhow/1.0.104/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-anyhow-1.0.104",
      "noticeSha256": "9D6F85C531344D61E1A902AF9CBF5CC84CB82983ACB6122CBF080AD4B017F459"
    },
    {
      "id": "cargo:atk-sys@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/atk-sys/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/atk-sys/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:atk@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/atk/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/atk/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:atomic-waker@1.1.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/atomic-waker/1.1.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/atomic-waker/1.1.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:autocfg@1.5.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/autocfg/1.5.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/autocfg/1.5.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:base64@0.21.7",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/base64/0.21.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/base64/0.21.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:base64@0.23.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/base64/0.23.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-base64-0.23.1",
      "noticeSha256": "706A40561C9EC71AA733F6AA0046148FAC2D32998B8D672DE4FBDC979AB78E9A"
    },
    {
      "id": "cargo:bit-set@0.8.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/bit-set/0.8.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-bit-set-0.8.0",
      "noticeSha256": "083450C52B2896E23339DABEAB577197D57D9A4FAAE14D591D78E82E93C556EF"
    },
    {
      "id": "cargo:bit-vec@0.8.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/bit-vec/0.8.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-bit-vec-0.8.0",
      "noticeSha256": "01A58C87ACA91791537DB8A7D6B180BD1DD5E2FCB3D323C8B077C3B5681C51D0"
    },
    {
      "id": "cargo:bitflags@1.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/bitflags/1.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bitflags/1.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:bitflags@2.13.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/bitflags/2.13.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-bitflags-2.13.2",
      "noticeSha256": "D8D2CDCE50E2975D15E683768DB306F1B3DDE62128BE82840880D74640325C3A"
    },
    {
      "id": "cargo:block-buffer@0.10.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/block-buffer/0.10.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-block-buffer-0.10.4",
      "noticeSha256": "69469D2130322C72BFAF1690D563C3700FEF49EFEBEB6F8B3715B89A8B039248"
    },
    {
      "id": "cargo:block2@0.6.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/block2/0.6.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/block2/0.6.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:brotli-decompressor@6.3.0",
      "license": "BSD-3-Clause/MIT",
      "source": "https://crates.io/api/v1/crates/brotli-decompressor/6.3.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-brotli-decompressor-6.3.0",
      "noticeSha256": "9CC3DAF7729E30497C5F3C96AA8AAD30993F8A1DEC2093E6914F94485387BF26"
    },
    {
      "id": "cargo:brotli@9.1.1",
      "license": "BSD-3-Clause AND MIT",
      "source": "https://crates.io/api/v1/crates/brotli/9.1.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-brotli-9.1.1",
      "noticeSha256": "10DBFEB28F1E7519B5D06536222915B140D9D7678E65A9B9442954A0E13E8FCF"
    },
    {
      "id": "cargo:bs58@0.5.1",
      "license": "MIT/Apache-2.0",
      "source": "https://docs.rs/crate/bs58/0.5.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bs58/0.5.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:bumpalo@3.20.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/bumpalo/3.20.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bumpalo/3.20.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:bytemuck@1.25.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/bytemuck/1.25.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bytemuck/1.25.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:byteorder@1.5.0",
      "license": "Unlicense OR MIT",
      "source": "https://crates.io/api/v1/crates/byteorder/1.5.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-byteorder-1.5.0",
      "noticeSha256": "64A80C83AE915AD2C7260008C33E31C1BD3E0B74A0B5209C15560C0EA939EBA3"
    },
    {
      "id": "cargo:bytes@1.12.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/bytes/1.12.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-bytes-1.12.1",
      "noticeSha256": "D45D4DDDC0B309138F69A5D9EDB556255781879CF70AB8A495F5CD7DE27C973E"
    },
    {
      "id": "cargo:cairo-rs@0.18.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/cairo-rs/0.18.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cairo-rs/0.18.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cairo-sys-rs@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/cairo-sys-rs/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cairo-sys-rs/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:camino@1.2.6",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/camino/1.2.6/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-camino-1.2.6",
      "noticeSha256": "8B1B3AE5F63C6D140E1A3541859ECE245749F970DE9736AD3F30A1AA4C25DB5E"
    },
    {
      "id": "cargo:cargo_metadata@0.19.2",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/cargo_metadata/0.19.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-cargo_metadata-0.19.2",
      "noticeSha256": "29D967261FE82ED9560BDC88D3B9BFDD4EC0D2E4220A4B5B1E9555210D46DD14"
    },
    {
      "id": "cargo:cargo_toml@1.0.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/cargo_toml/1.0.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cargo_toml/1.0.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cargo-platform@0.1.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/cargo-platform/0.1.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-cargo-platform-0.1.9",
      "noticeSha256": "5A75B17C749C5524F8DFDE665826DFAA224A5B332483CC9F5BBBBC147A143479"
    },
    {
      "id": "cargo:cc@1.11.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/cc/1.11.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cc/1.11.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cesu8@1.1.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/cesu8/1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cesu8/1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cfb@0.14.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/cfb/0.14.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-cfb-0.14.0",
      "noticeSha256": "525A06CDB8BB38E08B77A74A065E8D1A57324B6D2D3F91010EDAC41798E4D175"
    },
    {
      "id": "cargo:cfg-expr@0.15.8",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/cfg-expr/0.15.8/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cfg-expr/0.15.8/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cfg-if@1.0.5",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/cfg-if/1.0.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-cfg-if-1.0.5",
      "noticeSha256": "790A8D926C117DAAF540A4B1CDA1A68E4E46FC7B63889DFC0EFB9D96B53B998D"
    },
    {
      "id": "cargo:chrono@0.4.45",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/chrono/0.4.45/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/chrono/0.4.45/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:combine@4.6.9",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/combine/4.6.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/combine/4.6.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cookie@0.18.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/cookie/0.18.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-cookie-0.18.2",
      "noticeSha256": "602602EA2866E0404BCDDFCA0CEBB14296CAD83B9F7F4D023D588468D7A43F2E"
    },
    {
      "id": "cargo:core-foundation-sys@0.8.7",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/core-foundation-sys/0.8.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/core-foundation-sys/0.8.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:core-foundation@0.10.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/core-foundation/0.10.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/core-foundation/0.10.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:core-graphics-types@0.2.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/core-graphics-types/0.2.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/core-graphics-types/0.2.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:core-graphics@0.25.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/core-graphics/0.25.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/core-graphics/0.25.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cpufeatures@0.2.17",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/cpufeatures/0.2.17/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-cpufeatures-0.2.17",
      "noticeSha256": "DEA2855385F7F90D60F49CB426E4E0E1FF7A8C936A5C816C9B0C240C39C5782D"
    },
    {
      "id": "cargo:crc32fast@1.5.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/crc32fast/1.5.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-crc32fast-1.5.4",
      "noticeSha256": "8A8673BA0F187534BAD4A661883D8A2F3994ED37A242F0D331A2A044667ED492"
    },
    {
      "id": "cargo:crossbeam-channel@0.5.17",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/crossbeam-channel/0.5.17/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-crossbeam-channel-0.5.17",
      "noticeSha256": "1182AB02926835CF230B52D887B18B7AE4E315273FED0364F51362955AB5CAB5"
    },
    {
      "id": "cargo:crossbeam-utils@0.8.23",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/crossbeam-utils/0.8.23/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-crossbeam-utils-0.8.23",
      "noticeSha256": "711983AC000699A58980A9329E135A2715024755472E4351A66B4EC87123EEC3"
    },
    {
      "id": "cargo:crypto-common@0.1.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/crypto-common/0.1.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-crypto-common-0.1.7",
      "noticeSha256": "E8F28ECA512D4F1B844F5BF494E195533F0838AE269A42530B508D175AA9CFD8"
    },
    {
      "id": "cargo:cssparser-macros@0.7.1",
      "license": "MPL-2.0",
      "source": "https://crates.io/api/v1/crates/cssparser-macros/0.7.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; project owner decision 2026-11-21: MPL-2.0 accepted for unmodified third-party crates; notice reproduced, published crate source is the corresponding source",
      "status": "reviewed",
      "notice": "cargo-cssparser-macros-0.7.1",
      "noticeSha256": "8E2C8B498D5DE077588ABEDBA45DD74EF6DC29DE632C17FDDEB10083FF04E5CA"
    },
    {
      "id": "cargo:cssparser@0.37.0",
      "license": "MPL-2.0",
      "source": "https://crates.io/api/v1/crates/cssparser/0.37.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; project owner decision 2026-11-21: MPL-2.0 accepted for unmodified third-party crates; notice reproduced, published crate source is the corresponding source",
      "status": "reviewed",
      "notice": "cargo-cssparser-0.37.0",
      "noticeSha256": "92A81B3E2FF7BAB53CF0B16BCE497179ACAC15A27E2B256FBBD67F66626E9432"
    },
    {
      "id": "cargo:ctor@1.0.13",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/ctor/1.0.13/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-ctor-1.0.13",
      "noticeSha256": "126E3DBE6B6E739C12D25AB32C74A1C4B34B5048A2A5D29F904DC0B1C71EA6F9"
    },
    {
      "id": "cargo:darling_core@0.24.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/darling_core/0.24.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-darling_core-0.24.1",
      "noticeSha256": "B53A596FB9C6927F23DF9F55CB9ECD5634A9DB969A1A5749FDB76E62CB814A69"
    },
    {
      "id": "cargo:darling_macro@0.24.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/darling_macro/0.24.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-darling_macro-0.24.1",
      "noticeSha256": "00C449083ECDC2368286320D2C88C7E93CE265BB75293A5F1ACB484E22B6B8D9"
    },
    {
      "id": "cargo:darling@0.24.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/darling/0.24.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-darling-0.24.1",
      "noticeSha256": "EA54174A1111A7C039ABE87F24813E3A4897592A74A987BEDAD4311747B33D91"
    },
    {
      "id": "cargo:dbus@0.9.12",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/dbus/0.9.12/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dbus/0.9.12/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:deranged@0.5.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/deranged/0.5.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-deranged-0.5.9",
      "noticeSha256": "DAB8C8F1B07EF71530DB584670EF9E34B0798F9F362779B6C6405EF21278710C"
    },
    {
      "id": "cargo:derive_more-impl@2.1.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/derive_more-impl/2.1.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-derive_more-impl-2.1.1",
      "noticeSha256": "4D3E0BE9725CDB33FB6C5E7DDA6FA4E088510F8B7C2724E7C1888E1AB987871C"
    },
    {
      "id": "cargo:derive_more@2.1.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/derive_more/2.1.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-derive_more-2.1.1",
      "noticeSha256": "4A785B8AEE310BFD14F9A378225B924203D17074D4D01E4C514D254B413FB866"
    },
    {
      "id": "cargo:digest@0.10.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/digest/0.10.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-digest-0.10.7",
      "noticeSha256": "1591C6A828009B8F3BB22E6F48FACDB4121D4DAB96ED2CB7A748FE7B5CCEAD6A"
    },
    {
      "id": "cargo:dirs-sys@0.5.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/dirs-sys/0.5.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-dirs-sys-0.5.0",
      "noticeSha256": "37D18EF285D267B2E9ACCA40AB0C5E1FBD3FB0FE8BA1E99CE61024136D1F07FB"
    },
    {
      "id": "cargo:dirs@7.0.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/dirs/7.0.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-dirs-7.0.0",
      "noticeSha256": "E6447D94597399808FCB7D8CB09FD97B753025C06A4217B4FFD9D583D90409AF"
    },
    {
      "id": "cargo:dispatch2@0.3.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/dispatch2/0.3.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dispatch2/0.3.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:displaydoc@0.2.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/displaydoc/0.2.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-displaydoc-0.2.7",
      "noticeSha256": "B3E5042493036E686FBE66C5037117E95E54273B9097921C7CB297CCE83216D6"
    },
    {
      "id": "cargo:dlopen2_derive@0.4.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/dlopen2_derive/0.4.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dlopen2_derive/0.4.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dlopen2@0.8.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/dlopen2/0.8.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dlopen2/0.8.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dom_query@0.28.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/dom_query/0.28.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-dom_query-0.28.0",
      "noticeSha256": "67E42D8208A8F7406E6E0147A252661DE630F1321C72C55E6E879C576916F048"
    },
    {
      "id": "cargo:dpi@0.1.2",
      "license": "Apache-2.0 AND MIT",
      "source": "https://crates.io/api/v1/crates/dpi/0.1.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-dpi-0.1.2",
      "noticeSha256": "5392F8FD94274FCEFCF5BCC1E6FD20B455301E977A19527B3B37ADF3BEB1523B"
    },
    {
      "id": "cargo:dtoa-short@0.3.5",
      "license": "MPL-2.0",
      "source": "https://crates.io/api/v1/crates/dtoa-short/0.3.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; project owner decision 2026-11-21: MPL-2.0 accepted for unmodified third-party crates; notice reproduced, published crate source is the corresponding source",
      "status": "reviewed",
      "notice": "cargo-dtoa-short-0.3.5",
      "noticeSha256": "5D58C403156DDD1B3E6AB223D859D663459BA60CC56CF598A34A5C5DBEE5E769"
    },
    {
      "id": "cargo:dtoa@1.0.11",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/dtoa/1.0.11/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-dtoa-1.0.11",
      "noticeSha256": "A2F07B4A78C46E4A3187EAED1A6D46278669F48F1BAF41CDF9E35E58458C2254"
    },
    {
      "id": "cargo:dunce@1.0.5",
      "license": "CC0-1.0 OR MIT-0 OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/dunce/1.0.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-dunce-1.0.5",
      "noticeSha256": "7AD4DEBCC68A08CDBE1DB68593C1721B72E192F2E12016E15F982FE2099FF0B6"
    },
    {
      "id": "cargo:dyn-clone@1.0.20",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/dyn-clone/1.0.20/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-dyn-clone-1.0.20",
      "noticeSha256": "7EA8F6E52AFCF90E7F7D6B967114744EECEE2A8FE37A29A345E81CCC3EEFACDA"
    },
    {
      "id": "cargo:embed_plist@1.2.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/embed_plist/1.2.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/embed_plist/1.2.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:embed-resource@3.0.12",
      "license": "MIT",
      "source": "https://docs.rs/crate/embed-resource/3.0.12/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/embed-resource/3.0.12/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:equivalent@1.0.3",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/equivalent/1.0.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-equivalent-1.0.3",
      "noticeSha256": "3789E8E412DB720530FF8E079B473216C6ABD2314C13616E68AF9406032252EF"
    },
    {
      "id": "cargo:erased-serde@0.4.11",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/erased-serde/0.4.11/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-erased-serde-0.4.11",
      "noticeSha256": "CF9F0828B471770B68DA2B9B095B3CC677ACF635CF6B63A5B0626EE1E1B85230"
    },
    {
      "id": "cargo:errno@0.3.15",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/errno/0.3.15/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/errno/0.3.15/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:fastrand@2.5.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/fastrand/2.5.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-fastrand-2.5.0",
      "noticeSha256": "7C839A5B17DFF8D22DDD1C0037DB1FDC9473EE0AC0969845B5C4457C2E3A5EF1"
    },
    {
      "id": "cargo:fdeflate@0.3.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/fdeflate/0.3.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-fdeflate-0.3.7",
      "noticeSha256": "4CDCBAB4D5688D7955AA24E3356A3E5C0511F4066CC272CA266FE83BFF157558"
    },
    {
      "id": "cargo:field-offset@0.3.6",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/field-offset/0.3.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/field-offset/0.3.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:find-msvc-tools@0.1.15",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/find-msvc-tools/0.1.15/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/find-msvc-tools/0.1.15/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:flate2@1.1.11",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/flate2/1.1.11/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-flate2-1.1.11",
      "noticeSha256": "835A4C2B53B6B21AB78DFD934E0731691F9468CE5F2788CADC20B0625EBFEED3"
    },
    {
      "id": "cargo:fnv@1.0.7",
      "license": "Apache-2.0 / MIT",
      "source": "https://crates.io/api/v1/crates/fnv/1.0.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-fnv-1.0.7",
      "noticeSha256": "FA6FD50C9A21FEAF6C694F4E6AF3FF0732CB079F205EC41E65CB0728137634AD"
    },
    {
      "id": "cargo:foldhash@0.2.0",
      "license": "Zlib",
      "source": "https://crates.io/api/v1/crates/foldhash/0.2.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-foldhash-0.2.0",
      "noticeSha256": "5009D86846FBD3572C91BCC5C381F37C8EFE4CF905D8A33103FCBEEBDE65A8D8"
    },
    {
      "id": "cargo:foreign-types-macros@0.2.4",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/foreign-types-macros/0.2.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/foreign-types-macros/0.2.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:foreign-types-shared@0.3.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/foreign-types-shared/0.3.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/foreign-types-shared/0.3.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:foreign-types@0.5.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/foreign-types/0.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/foreign-types/0.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:form_urlencoded@1.2.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/form_urlencoded/1.2.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-form_urlencoded-1.2.2",
      "noticeSha256": "E12C3B36C21AF5F67FFBB4CA44786A5AE07ABDD22A7AFFC8D085D2987E96F86F"
    },
    {
      "id": "cargo:futures-channel@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-channel/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-channel/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:futures-core@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-core/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-core/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:futures-executor@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-executor/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-executor/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:futures-io@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-io/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-io/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:futures-macro@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-macro/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-macro/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:futures-sink@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-sink/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-sink/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:futures-task@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-task/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-task/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:futures-util@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/futures-util/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/futures-util/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gdk-pixbuf-sys@0.18.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gdk-pixbuf-sys/0.18.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gdk-pixbuf-sys/0.18.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gdk-pixbuf@0.18.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gdk-pixbuf/0.18.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gdk-pixbuf/0.18.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gdk-sys@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gdk-sys/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gdk-sys/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gdk@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gdk/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gdk/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gdkwayland-sys@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gdkwayland-sys/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gdkwayland-sys/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gdkx11-sys@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gdkx11-sys/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gdkx11-sys/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gdkx11@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gdkx11/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gdkx11/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:generic-array@0.14.7",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/generic-array/0.14.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-generic-array-0.14.7",
      "noticeSha256": "8F81C1CFDA0D812F8A12C5A5DA16DEB62CA2881A76E1980348456C464F690000"
    },
    {
      "id": "cargo:getrandom@0.3.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/getrandom/0.3.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-getrandom-0.3.4",
      "noticeSha256": "C51E14ACF08172439512796C0C1A08C6E2CC60EFE9EDF47967A0380B1DFEE904"
    },
    {
      "id": "cargo:getrandom@0.4.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/getrandom/0.4.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-getrandom-0.4.3",
      "noticeSha256": "05977A84ECBB1743D70C45F34D00B8D0324C586BA89C0A908811C7FDF44C128E"
    },
    {
      "id": "cargo:gio-sys@0.18.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gio-sys/0.18.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gio-sys/0.18.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gio@0.18.4",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gio/0.18.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gio/0.18.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:glib-macros@0.18.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/glib-macros/0.18.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/glib-macros/0.18.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:glib-sys@0.18.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/glib-sys/0.18.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/glib-sys/0.18.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:glib@0.18.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/glib/0.18.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/glib/0.18.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:glob@0.3.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/glob/0.3.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-glob-0.3.4",
      "noticeSha256": "FAB5C0FC6EEF7499DC5BCA5E58458FB9B6A37FAB3C7EFD7F53CD7525AA5426D9"
    },
    {
      "id": "cargo:gobject-sys@0.18.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gobject-sys/0.18.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gobject-sys/0.18.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gtk-sys@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gtk-sys/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gtk-sys/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gtk@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gtk/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gtk/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:gtk3-macros@0.18.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/gtk3-macros/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/gtk3-macros/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:hashbrown@0.12.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/hashbrown/0.12.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-hashbrown-0.12.3",
      "noticeSha256": "59AD479C091DC895CF92E1B04CEC6ED3720056B43C976A0FC639090B00DDCF20"
    },
    {
      "id": "cargo:hashbrown@0.17.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/hashbrown/0.17.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-hashbrown-0.17.1",
      "noticeSha256": "94682CB97F725854CBDB0C778F384B8E7F92EA4DEE139EAE26C3594171848E16"
    },
    {
      "id": "cargo:heck@0.4.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/heck/0.4.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/heck/0.4.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:heck@0.5.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/heck/0.5.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-heck-0.5.0",
      "noticeSha256": "79A8DE1157E0A3C91D464016630A9B64D07BB76EB1CE0B8BAD5544783A28372C"
    },
    {
      "id": "cargo:hex@0.4.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/hex/0.4.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/hex/0.4.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:html5ever@0.39.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/html5ever/0.39.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-html5ever-0.39.0",
      "noticeSha256": "FC42F1DE0F29D1CAED93FBB8F2C92F53079D14C292AAE7FD508E4F516E051013"
    },
    {
      "id": "cargo:http-body-util@0.1.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/http-body-util/0.1.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/http-body-util/0.1.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:http-body@1.1.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/http-body/1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/http-body/1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:http@1.5.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/http/1.5.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-http-1.5.0",
      "noticeSha256": "A007F63D4AE5DC5A751AB034C700E3907C1F8CAE00C4AE235EA5056839765C24"
    },
    {
      "id": "cargo:httparse@1.10.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/httparse/1.10.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/httparse/1.10.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:hyper-util@0.1.21",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/hyper-util/0.1.21/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/hyper-util/0.1.21/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:hyper@1.12.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/hyper/1.12.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/hyper/1.12.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:iana-time-zone-haiku@0.1.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/iana-time-zone-haiku/0.1.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/iana-time-zone-haiku/0.1.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:iana-time-zone@0.1.65",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/iana-time-zone/0.1.65/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/iana-time-zone/0.1.65/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:ico@0.6.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/ico/0.6.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-ico-0.6.0",
      "noticeSha256": "4951349C5F230D23B7EE49AB6A25503722C8E9EE32380B985E23F270734413C6"
    },
    {
      "id": "cargo:icu_collections@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/icu_collections/2.4.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-icu_collections-2.4.0",
      "noticeSha256": "AFCE3502793DE628C4407C410E692BEB4AE4D7733880DFC677F5BF114F205D1B"
    },
    {
      "id": "cargo:icu_locale_core@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/icu_locale_core/2.4.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-icu_locale_core-2.4.0",
      "noticeSha256": "17A5647E38BC0CB6C37053665837E35A3F87C186ACE8588C6757F83A15435E40"
    },
    {
      "id": "cargo:icu_normalizer_data@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/icu_normalizer_data/2.4.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-icu_normalizer_data-2.4.0",
      "noticeSha256": "CB01EC6037E1246A4B1EDDC419661E3D4218288B5B8B073F7116A5F5127087DA"
    },
    {
      "id": "cargo:icu_normalizer@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/icu_normalizer/2.4.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-icu_normalizer-2.4.0",
      "noticeSha256": "C9AF10F65B6C48565006BF21AF5D6EB989353F6138D41A2846496CC4D4313B26"
    },
    {
      "id": "cargo:icu_properties_data@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/icu_properties_data/2.4.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-icu_properties_data-2.4.0",
      "noticeSha256": "3BCCFB74DE171D8981AEEAE059B713965C5334DA6C035D751CE8D0B735133063"
    },
    {
      "id": "cargo:icu_properties@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/icu_properties/2.4.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-icu_properties-2.4.0",
      "noticeSha256": "79EDD0341E833FD77AC47A553C879465F4AE829E1623EF4CC33CCBB03332B034"
    },
    {
      "id": "cargo:icu_provider@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/icu_provider/2.4.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-icu_provider-2.4.0",
      "noticeSha256": "0F54067BA166500B6CE9CECD96256CA9A2723678F3AE42A2EB00A2F9DBDEEE8B"
    },
    {
      "id": "cargo:ident_case@1.0.1",
      "license": "MIT/Apache-2.0",
      "source": "https://crates.io/api/v1/crates/ident_case/1.0.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-ident_case-1.0.1",
      "noticeSha256": "436B0ACBBCB9DE9BB6B4739345A96E8CB2A5DD0AC9DC652C1C4478A6EF672B91"
    },
    {
      "id": "cargo:idna_adapter@1.2.2",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/idna_adapter/1.2.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-idna_adapter-1.2.2",
      "noticeSha256": "3AA4BBD94637ADB20D6F6FEE0CCA95A73BC95DF8A64CEC092EF0689B3CCB57B8"
    },
    {
      "id": "cargo:idna@1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/idna/1.1.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-idna-1.1.0",
      "noticeSha256": "D1B51576D9BB09703E0A7F45A5EF316B8EB40A83D4CD4A3550661C8CEFAB06AE"
    },
    {
      "id": "cargo:indexmap@1.9.3",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/indexmap/1.9.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-indexmap-1.9.3",
      "noticeSha256": "030506FF80F3EC24086EEC6481B66BEAE4001B2FF506D404FD3AB2F876EDE9D2"
    },
    {
      "id": "cargo:indexmap@2.14.2",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/indexmap/2.14.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-indexmap-2.14.2",
      "noticeSha256": "3D16B5D1282FBDE4053C72B44D87503AD62AE5F13D0CFFA980B617A78227FD6E"
    },
    {
      "id": "cargo:infer@0.22.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/infer/0.22.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-infer-0.22.0",
      "noticeSha256": "BD784C5BBE3C8F3E25C113B14E6AF1A25E7F9DA2D5E33FE5696A20C1F09751CE"
    },
    {
      "id": "cargo:ipnet@2.12.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/ipnet/2.12.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ipnet/2.12.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:itoa@1.0.18",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/itoa/1.0.18/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-itoa-1.0.18",
      "noticeSha256": "3686BC0E6106DD427A6FC0D8FEA4096152EDD98BF105A74A3DD9BADBDEF94A0B"
    },
    {
      "id": "cargo:javascriptcore-rs-sys@1.1.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/javascriptcore-rs-sys/1.1.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/javascriptcore-rs-sys/1.1.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:javascriptcore-rs@1.1.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/javascriptcore-rs/1.1.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/javascriptcore-rs/1.1.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jiff-core@0.1.3",
      "license": "Unlicense OR MIT",
      "source": "https://docs.rs/crate/jiff-core/0.1.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jiff-core/0.1.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jiff-static@0.2.40",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/jiff-static/0.2.40/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jiff-static/0.2.40/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jiff-tzdb-platform@0.1.3",
      "license": "Unlicense OR MIT",
      "source": "https://docs.rs/crate/jiff-tzdb-platform/0.1.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jiff-tzdb-platform/0.1.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jiff-tzdb@0.1.9",
      "license": "Unlicense OR MIT",
      "source": "https://docs.rs/crate/jiff-tzdb/0.1.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jiff-tzdb/0.1.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jiff@0.2.40",
      "license": "Unlicense OR MIT",
      "source": "https://docs.rs/crate/jiff/0.2.40/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jiff/0.2.40/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jni-sys-macros@0.4.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/jni-sys-macros/0.4.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jni-sys-macros/0.4.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jni-sys@0.3.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/jni-sys/0.3.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jni-sys/0.3.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jni-sys@0.4.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/jni-sys/0.4.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jni-sys/0.4.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jni@0.21.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/jni/0.21.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jni/0.21.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:js-sys@0.3.107",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/js-sys/0.3.107/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/js-sys/0.3.107/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:json-patch@4.2.0",
      "license": "MIT/Apache-2.0",
      "source": "https://crates.io/api/v1/crates/json-patch/4.2.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-json-patch-4.2.0",
      "noticeSha256": "03D8FAF1B4133D837CE391B5897E3B7A7146F751B299F4B3E82FCA25376B69F0"
    },
    {
      "id": "cargo:jsonptr@0.7.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/jsonptr/0.7.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-jsonptr-0.7.1",
      "noticeSha256": "14E4037D3EABBC5C311C20E09B728BE0951CFD42FD4C82268F206D6C45B0BDB4"
    },
    {
      "id": "cargo:keyboard-types@0.8.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/keyboard-types/0.8.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-keyboard-types-0.8.3",
      "noticeSha256": "62F3A5FA535DE4550B7B32C998A34EC145441014B3663396E74DEDF3303952A8"
    },
    {
      "id": "cargo:libappindicator-sys@0.9.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/libappindicator-sys/0.9.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/libappindicator-sys/0.9.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:libappindicator@0.9.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/libappindicator/0.9.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/libappindicator/0.9.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:libc@0.2.191",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/libc/0.2.191/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-libc-0.2.191",
      "noticeSha256": "7441F10EFC01B56F0DDEF6B21B408892B716A5D5FF2A38F3B75399B8694E087A"
    },
    {
      "id": "cargo:libdbus-sys@0.2.7",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/libdbus-sys/0.2.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/libdbus-sys/0.2.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:libloading@0.7.4",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/libloading/0.7.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/libloading/0.7.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:libredox@0.1.25",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/libredox/0.1.25/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/libredox/0.1.25/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:linux-raw-sys@0.12.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/linux-raw-sys/0.12.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/linux-raw-sys/0.12.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:litemap@0.8.3",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/litemap/0.8.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-litemap-0.8.3",
      "noticeSha256": "9A1F28F19164B92076949638A11066C9C9802BFE38FCE8596064434C39E1055A"
    },
    {
      "id": "cargo:lock_api@0.4.14",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/lock_api/0.4.14/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-lock_api-0.4.14",
      "noticeSha256": "648BC27248FBB6E11ADC94E9EEA8253CE19F65025B759A7528393F7149EAA3F4"
    },
    {
      "id": "cargo:log@0.4.35",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/log/0.4.35/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-log-0.4.35",
      "noticeSha256": "373BE93291D6D96D8CAFCFC37DFFEFD0F25E412C7B0855E8B4588ECC196C85CF"
    },
    {
      "id": "cargo:markup5ever@0.39.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/markup5ever/0.39.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-markup5ever-0.39.0",
      "noticeSha256": "E47A799DB93B54E2F4AEC3873BF4C9C11EC8F5D5E91C35F29EAA429EFDD3BE63"
    },
    {
      "id": "cargo:memchr@2.8.3",
      "license": "Unlicense OR MIT",
      "source": "https://crates.io/api/v1/crates/memchr/2.8.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-memchr-2.8.3",
      "noticeSha256": "5FA23CF0559BB4D458C17E04A4D919B9C7D06115518EA394E37B3650A8177832"
    },
    {
      "id": "cargo:memoffset@0.9.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/memoffset/0.9.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/memoffset/0.9.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:mime@0.3.18",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/mime/0.3.18/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-mime-0.3.18",
      "noticeSha256": "C7DB856E4680D268B99516A02A65534A39514B21920CA6D243AE3D3714017787"
    },
    {
      "id": "cargo:miniz_oxide@0.8.9",
      "license": "MIT OR Zlib OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/miniz_oxide/0.8.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-miniz_oxide-0.8.9",
      "noticeSha256": "BDEA9118ABBA2EED16C8DB8C32AD83BCEBDB2046BD11072AFF1EFF2662CA7686"
    },
    {
      "id": "cargo:miniz_oxide@0.9.1",
      "license": "MIT OR Zlib OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/miniz_oxide/0.9.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-miniz_oxide-0.9.1",
      "noticeSha256": "CD50A3776215B87999932DF0BC5D0842C793EA47B59329EED4F2AF3C0C7BDFE6"
    },
    {
      "id": "cargo:mio@1.2.4",
      "license": "MIT",
      "source": "https://docs.rs/crate/mio/1.2.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/mio/1.2.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:muda@0.20.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/muda/0.20.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-muda-0.20.0",
      "noticeSha256": "2DF62209DEB6B0A1EBD3911719E25BCEAB025FDE44C3D59176C15B9A61250ED1"
    },
    {
      "id": "cargo:ndk-context@0.1.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/ndk-context/0.1.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ndk-context/0.1.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:ndk-sys@0.6.0+11769913",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/ndk-sys/0.6.0+11769913/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ndk-sys/0.6.0+11769913/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:ndk@0.9.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/ndk/0.9.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ndk/0.9.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:new_debug_unreachable@1.0.6",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/new_debug_unreachable/1.0.6/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-new_debug_unreachable-1.0.6",
      "noticeSha256": "8751D14717DED7A1EAAAEA36B8A3228AB91B27EB1F7C6E22CC2DCB52C61A045E"
    },
    {
      "id": "cargo:num_enum_derive@0.7.6",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/num_enum_derive/0.7.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/num_enum_derive/0.7.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:num_enum@0.7.6",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/num_enum/0.7.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/num_enum/0.7.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:num-conv@0.2.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/num-conv/0.2.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-num-conv-0.2.2",
      "noticeSha256": "6639D2638FECD3A42CDA76B4ECC3FFBFE5660C50DB24A4A98A8C605D3D93757B"
    },
    {
      "id": "cargo:num-traits@0.2.19",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/num-traits/0.2.19/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/num-traits/0.2.19/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-app-kit@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-app-kit/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-app-kit/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-cloud-kit@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-cloud-kit/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-cloud-kit/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-core-data@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-core-data/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-core-data/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-core-foundation@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-core-foundation/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-core-foundation/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-core-graphics@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-core-graphics/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-core-graphics/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-core-image@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-core-image/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-core-image/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-core-location@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-core-location/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-core-location/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-core-text@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-core-text/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-core-text/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-encode@4.1.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-encode/4.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-encode/4.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-exception-helper@0.1.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-exception-helper/0.1.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-exception-helper/0.1.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-foundation@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-foundation/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-foundation/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-io-surface@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-io-surface/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-io-surface/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-quartz-core@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-quartz-core/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-quartz-core/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-ui-kit@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-ui-kit/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-ui-kit/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-user-notifications@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-user-notifications/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-user-notifications/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2-web-kit@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2-web-kit/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2-web-kit/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:objc2@0.6.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/objc2/0.6.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/objc2/0.6.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:once_cell@1.21.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/once_cell/1.21.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-once_cell-1.21.4",
      "noticeSha256": "000FAED41D17AE9EE604E065F6419D0C05CB416E53271E7C45F78480DFEB64ED"
    },
    {
      "id": "cargo:option-ext@0.2.0",
      "license": "MPL-2.0",
      "source": "https://crates.io/api/v1/crates/option-ext/0.2.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; project owner decision 2026-11-21: MPL-2.0 accepted for unmodified third-party crates; notice reproduced, published crate source is the corresponding source",
      "status": "reviewed",
      "notice": "cargo-option-ext-0.2.0",
      "noticeSha256": "5CCFA1EAFE3B2185CEA42A4CF2031C7FD56F9C0B9BBA5DCFA460CA0E7F9D2AED"
    },
    {
      "id": "cargo:pango-sys@0.18.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/pango-sys/0.18.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/pango-sys/0.18.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:pango@0.18.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/pango/0.18.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/pango/0.18.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:parking_lot_core@0.9.12",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/parking_lot_core/0.9.12/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-parking_lot_core-0.9.12",
      "noticeSha256": "30FFB3B849FC72F53B6980BF881FEE4B1033D194CB4943DDD0E2E8BF1C3A4CAE"
    },
    {
      "id": "cargo:parking_lot@0.12.5",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/parking_lot/0.12.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-parking_lot-0.12.5",
      "noticeSha256": "D2D92691F56C1BFC9D96495F0A7EBDB9C6B075D3C36DEC6764B955BAD58460B4"
    },
    {
      "id": "cargo:percent-encoding@2.3.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/percent-encoding/2.3.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-percent-encoding-2.3.2",
      "noticeSha256": "FD99C3D906D40960ED4883BEBE1C30A2D4E3125B5A786C35B70E1EC6ED3A9ED8"
    },
    {
      "id": "cargo:phf_codegen@0.13.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/phf_codegen/0.13.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/phf_codegen/0.13.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:phf_generator@0.13.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/phf_generator/0.13.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-phf_generator-0.13.1",
      "noticeSha256": "A45DC869876A481A35020EDF4F70281F2C2B2CEBDBC592480367F0D135CD91DE"
    },
    {
      "id": "cargo:phf_macros@0.13.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/phf_macros/0.13.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-phf_macros-0.13.1",
      "noticeSha256": "5E76FCC439193495DBC9B00904B3885F0BAD296EB6A39D1D8AE646E66D6ADDBF"
    },
    {
      "id": "cargo:phf_shared@0.13.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/phf_shared/0.13.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-phf_shared-0.13.1",
      "noticeSha256": "D67AF9315B231389A04669EEB63A71853D1D93D5779486AD7DB2F4F0A97191AD"
    },
    {
      "id": "cargo:phf@0.13.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/phf/0.13.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-phf-0.13.1",
      "noticeSha256": "326CBF4ABB6B9EFE5BECA207B1A39112C037EEA5694368AEECD5CC02FAE51CFB"
    },
    {
      "id": "cargo:pin-project-lite@0.2.17",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/pin-project-lite/0.2.17/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-pin-project-lite-0.2.17",
      "noticeSha256": "2AF78FAEBB1AA6898E168EE609047B1138BA820E50FAFBF852D5D7917EEDA3CA"
    },
    {
      "id": "cargo:pkg-config@0.3.34",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/pkg-config/0.3.34/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/pkg-config/0.3.34/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:plist@1.10.3",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/plist/1.10.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-plist-1.10.3",
      "noticeSha256": "B760458654D2BBBECC7472297ED88583F1B6FDE341AE5860F04FB8076D056497"
    },
    {
      "id": "cargo:png@0.18.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/png/0.18.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-png-0.18.1",
      "noticeSha256": "33D05821A63B1E88ACBB0183E539C9A5188ED427E6A31810566CC2587C4B849A"
    },
    {
      "id": "cargo:portable-atomic-util@0.2.8",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/portable-atomic-util/0.2.8/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/portable-atomic-util/0.2.8/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:portable-atomic@1.15.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/portable-atomic/1.15.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/portable-atomic/1.15.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:potential_utf@0.1.6",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/potential_utf/0.1.6/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-potential_utf-0.1.6",
      "noticeSha256": "A10D38168CDA8E7F887773348B0766172092381E42FD9E0979192FBA9C5FAB93"
    },
    {
      "id": "cargo:powerfmt@0.2.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/powerfmt/0.2.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-powerfmt-0.2.1",
      "noticeSha256": "E41E6FC5E0E37FDD0DD2BBE0B214DE5D8001414DC0AB9926562D46B6626432FF"
    },
    {
      "id": "cargo:precomputed-hash@0.1.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/precomputed-hash/0.1.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-precomputed-hash-0.1.1",
      "noticeSha256": "87830D39117B98951FBC5AF1580348F57B99BF86FDA20EDE42256B4A9F2FF8EA"
    },
    {
      "id": "cargo:proc-macro-crate@1.3.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/proc-macro-crate/1.3.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/proc-macro-crate/1.3.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:proc-macro-crate@2.0.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/proc-macro-crate/2.0.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/proc-macro-crate/2.0.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:proc-macro-crate@3.5.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/proc-macro-crate/3.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/proc-macro-crate/3.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:proc-macro-error-attr@1.0.4",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/proc-macro-error-attr/1.0.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/proc-macro-error-attr/1.0.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:proc-macro-error@1.0.4",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/proc-macro-error/1.0.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/proc-macro-error/1.0.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:proc-macro2@1.0.107",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/proc-macro2/1.0.107/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-proc-macro2-1.0.107",
      "noticeSha256": "DCBFBDA7221CE578D93F4D3416332A480B9B7A0375124B6CAF0C3B2AB8D3C8C3"
    },
    {
      "id": "cargo:qcms@0.3.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/qcms/0.3.0/download (checksum from Cargo.lock)",
      "licenseSource": "license text from source-file headers of the checksum-verified crate (no LICENSE file shipped); permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-qcms-0.3.0",
      "noticeSha256": "497835806E951F6327B2ACEC5F96C935E6ECF1DD227D1CE40E08A99BB8BDF1B6"
    },
    {
      "id": "cargo:quick-xml@0.42.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/quick-xml/0.42.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-quick-xml-0.42.0",
      "noticeSha256": "82172FD1E45646C64A3F3948E7ADFE2AAE82C19634F89C69384A11AECD955343"
    },
    {
      "id": "cargo:quote@1.0.47",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/quote/1.0.47/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-quote-1.0.47",
      "noticeSha256": "836CD81CC11BEF99FACC44598FD22BE3FDF74E8150DF3C231FB29101CFA89CCA"
    },
    {
      "id": "cargo:r-efi@5.3.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/r-efi/5.3.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/r-efi/5.3.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:r-efi@6.0.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/r-efi/6.0.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/r-efi/6.0.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:raw-window-handle@0.6.2",
      "license": "MIT OR Apache-2.0 OR Zlib",
      "source": "https://crates.io/api/v1/crates/raw-window-handle/0.6.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-raw-window-handle-0.6.2",
      "noticeSha256": "F64008F47260031573C2EF64DB735461E60551E1B49C9B1ADB797441111B2B49"
    },
    {
      "id": "cargo:redox_syscall@0.5.18",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/redox_syscall/0.5.18/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/redox_syscall/0.5.18/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:redox_users@0.5.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/redox_users/0.5.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/redox_users/0.5.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:ref-cast-impl@1.0.27",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/ref-cast-impl/1.0.27/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ref-cast-impl/1.0.27/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:ref-cast@1.0.27",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/ref-cast/1.0.27/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ref-cast/1.0.27/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:regex-automata@0.4.18",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/regex-automata/0.4.18/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-regex-automata-0.4.18",
      "noticeSha256": "4B00337D67F6EA220BA41CB4837DD0482533C4FE16762939AF3D5428071785EF"
    },
    {
      "id": "cargo:regex-syntax@0.8.11",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/regex-syntax/0.8.11/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-regex-syntax-0.8.11",
      "noticeSha256": "AC83D5C169AC20D92E502BC9F656067C159228EAB43F2860BA106D2575D7FA06"
    },
    {
      "id": "cargo:regex@1.13.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/regex/1.13.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-regex-1.13.1",
      "noticeSha256": "55D1D3CBF228FC186FF850B3EA5989621C0259D9879032BA6C8855ABBFA0E9E1"
    },
    {
      "id": "cargo:reqwest@0.13.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/reqwest/0.13.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/reqwest/0.13.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:rfd@0.17.2",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/rfd/0.17.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-rfd-0.17.2",
      "noticeSha256": "0F81F8DFF29DAE6ECCBF3EFB763725582CE48E408C8B2E7698DC51590E6EDA2A"
    },
    {
      "id": "cargo:rustc_version@0.4.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/rustc_version/0.4.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/rustc_version/0.4.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:rustc-hash@2.1.3",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/rustc-hash/2.1.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-rustc-hash-2.1.3",
      "noticeSha256": "14F78D9089DEAC66115E6C934BA8B35D4FAFB67E64E44E3CCA017B15AC9A4546"
    },
    {
      "id": "cargo:rustix@1.1.6",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/rustix/1.1.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/rustix/1.1.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:rustversion@1.0.23",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/rustversion/1.0.23/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/rustversion/1.0.23/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:same-file@1.0.6",
      "license": "Unlicense/MIT",
      "source": "https://crates.io/api/v1/crates/same-file/1.0.6/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-same-file-1.0.6",
      "noticeSha256": "A9497EA619A77C92E10DDA4B3BC7D026ED720F2C092565C2C4D9331D16CA663D"
    },
    {
      "id": "cargo:schemars_derive@0.8.22",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/schemars_derive/0.8.22/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-schemars_derive-0.8.22",
      "noticeSha256": "B7BD31DEEA2DE7FEC931B5309F4C70CF326CC59E91EAC69D873CD442533B7B3D"
    },
    {
      "id": "cargo:schemars@0.8.22",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/schemars/0.8.22/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-schemars-0.8.22",
      "noticeSha256": "DB8D1F6D88C4B71020FB7FA3D2D88A39BDBDB8B8F0CAABDFC92C0555273ACF55"
    },
    {
      "id": "cargo:schemars@0.9.0",
      "license": "MIT",
      "source": "https://docs.rs/crate/schemars/0.9.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/schemars/0.9.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:schemars@1.2.3",
      "license": "MIT",
      "source": "https://docs.rs/crate/schemars/1.2.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/schemars/1.2.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:scopeguard@1.2.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/scopeguard/1.2.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-scopeguard-1.2.0",
      "noticeSha256": "95277185D6CD1AAB27A726A8D36AC372DA4266297659DEA7BF22FF5C38212B3F"
    },
    {
      "id": "cargo:selectors@0.38.0",
      "license": "MPL-2.0",
      "source": "https://crates.io/api/v1/crates/selectors/0.38.0/download (checksum from Cargo.lock)",
      "licenseSource": "license text from source-file headers of the checksum-verified crate (no LICENSE file shipped) plus upstream license text kept in docs/license-evidence (SHA-256 checked); project owner decision 2026-11-21: MPL-2.0 accepted for unmodified third-party crates; notice reproduced, published crate source is the corresponding source",
      "status": "reviewed",
      "notice": "cargo-selectors-0.38.0",
      "noticeSha256": "0CEF4CDDA45FCA017DB6FB610FD70DDE1E9CFA10C20C3AB9E12D27B57C99E2F4"
    },
    {
      "id": "cargo:semver@1.0.28",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/semver/1.0.28/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-semver-1.0.28",
      "noticeSha256": "896FF2A5B8CEDAFF27F71E06F6E9AB65CA53AFBEFCC364657C91829DE2E0E7A5"
    },
    {
      "id": "cargo:serde_core@1.0.229",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_core/1.0.229/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_core-1.0.229",
      "noticeSha256": "09BFAE9244517B3FFA124BFE2473E48F0B38488AF339A1CF59594FA34C48BD17"
    },
    {
      "id": "cargo:serde_derive_internals@0.29.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_derive_internals/0.29.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_derive_internals-0.29.1",
      "noticeSha256": "BD160561EB023CF87010370C588824385CF5FC1B554E61EC8F064591DE8E393A"
    },
    {
      "id": "cargo:serde_derive@1.0.229",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_derive/1.0.229/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_derive-1.0.229",
      "noticeSha256": "56C6ED2ED9F22EAFBDF8ED066BE61DF0A12763B0A8B1313AFCC2BC493A4E273B"
    },
    {
      "id": "cargo:serde_json@1.0.154",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_json/1.0.154/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_json-1.0.154",
      "noticeSha256": "B15842CE2BAB94C45FEF049204391530A699183BA2433ADE969086EA58215989"
    },
    {
      "id": "cargo:serde_repr@0.1.21",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_repr/0.1.21/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_repr-0.1.21",
      "noticeSha256": "522626477628CF1510255155B9D07EE83E6B578C0000C355C204F3ED45442A13"
    },
    {
      "id": "cargo:serde_spanned@0.6.9",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/serde_spanned/0.6.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_spanned/0.6.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_spanned@1.1.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_spanned/1.1.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_spanned-1.1.3",
      "noticeSha256": "E7BDD5FC5CEDDBEFA1EE03518F324678503CAD1DCAA15B28BFE62698D1ABB987"
    },
    {
      "id": "cargo:serde_with_macros@3.24.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_with_macros/3.24.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_with_macros-3.24.0",
      "noticeSha256": "49E988D4720A46E8F55482F91411228F2B69739FEED7D27115247A59867CCC0D"
    },
    {
      "id": "cargo:serde_with@3.24.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde_with/3.24.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde_with-3.24.0",
      "noticeSha256": "A54D6B56010BA2E9AA3ED59A992BEF2FF66063D90F309A72B3766427E0D37196"
    },
    {
      "id": "cargo:serde-untagged@0.1.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde-untagged/0.1.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde-untagged-0.1.9",
      "noticeSha256": "115DBB6F9D5F66A52F4208291702F7FB782DA3C16F27393EA7E7943A5ADCD511"
    },
    {
      "id": "cargo:serde@1.0.229",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serde/1.0.229/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serde-1.0.229",
      "noticeSha256": "8CE03DC1F1AC0D97EE77E85AE6A244ADB46944C050206843D2FE14EF8CDF6877"
    },
    {
      "id": "cargo:serialize-to-javascript-impl@0.1.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serialize-to-javascript-impl/0.1.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serialize-to-javascript-impl-0.1.2",
      "noticeSha256": "CA7AF6E6615F2B61B931F6338ECF10E456B8F086484B37775CDB67B8D01725F5"
    },
    {
      "id": "cargo:serialize-to-javascript@0.1.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/serialize-to-javascript/0.1.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-serialize-to-javascript-0.1.2",
      "noticeSha256": "B21ED4656C530861F5C809FBAB703A9F17CC3FA34A74857262F6F8B219E1BAE3"
    },
    {
      "id": "cargo:servo_arc@0.4.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/servo_arc/0.4.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-servo_arc-0.4.3",
      "noticeSha256": "E6FB1D237FE7BF9EDC6388ECB15388E16353B81A94B7E97263F413EBA6A4B548"
    },
    {
      "id": "cargo:sha2@0.10.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/sha2/0.10.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-sha2-0.10.9",
      "noticeSha256": "607A1C8660F4CC189002CBEF5C22D892BF45F5EA92914AB3FA62AA25CDC85FDC"
    },
    {
      "id": "cargo:shlex@2.0.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/shlex/2.0.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/shlex/2.0.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:simd-adler32@0.3.10",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/simd-adler32/0.3.10/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-simd-adler32-0.3.10",
      "noticeSha256": "DD6C56F85DF68B434EADA07E0FF037916719C1DAE66ABF1B996158BE0BAA7155"
    },
    {
      "id": "cargo:siphasher@1.0.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/siphasher/1.0.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-siphasher-1.0.4",
      "noticeSha256": "0BDF1668E36F125A61EEC6A77B18F6D1B8DE00B512B43C59550D6B358999CF0A"
    },
    {
      "id": "cargo:slab@0.4.12",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/slab/0.4.12/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/slab/0.4.12/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:smallvec@1.16.5",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/smallvec/1.16.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-smallvec-1.16.5",
      "noticeSha256": "3844C5FBB21565F175947351CA60C7CEF503708AA1DD8C4C4438A7FA1A4CC273"
    },
    {
      "id": "cargo:socket2@0.6.5",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/socket2/0.6.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/socket2/0.6.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:softbuffer@0.4.8",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/softbuffer/0.4.8/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-softbuffer-0.4.8",
      "noticeSha256": "075349DC2E97C09293C79BB3C652BBA6D43F8F1E87245E71707D6F8060DB6395"
    },
    {
      "id": "cargo:soup3-sys@0.5.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/soup3-sys/0.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/soup3-sys/0.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:soup3@0.5.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/soup3/0.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/soup3/0.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:stable_deref_trait@1.2.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/stable_deref_trait/1.2.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-stable_deref_trait-1.2.1",
      "noticeSha256": "651DE7AD262D3C10F78F04434707E2652285EA8B5E91B609AFD552A4655907BB"
    },
    {
      "id": "cargo:static_vcruntime@3.0.0",
      "license": "MIT OR Apache-2.0 OR Zlib",
      "source": "https://docs.rs/crate/static_vcruntime/3.0.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/static_vcruntime/3.0.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:string_cache_codegen@0.6.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/string_cache_codegen/0.6.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/string_cache_codegen/0.6.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:string_cache@0.9.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/string_cache/0.9.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-string_cache-0.9.0",
      "noticeSha256": "47774C1C82BB15FE9CEE971D077212B6C9B9B9CE4CA4F960E562C862048D9DFE"
    },
    {
      "id": "cargo:strsim@0.11.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/strsim/0.11.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-strsim-0.11.1",
      "noticeSha256": "B2771B3FB63CED37404A5F280B2A90A3F50F4BC81C18EF5EEBB4B3338B999A28"
    },
    {
      "id": "cargo:swift-rs@1.0.9",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/swift-rs/1.0.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/swift-rs/1.0.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:syn@1.0.109",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/syn/1.0.109/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/syn/1.0.109/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:syn@2.0.119",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/syn/2.0.119/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-syn-2.0.119",
      "noticeSha256": "7BA38148CFEAA4C4CA2253E0C87CB121294E2C91FB7FB0645C6E202954733A0F"
    },
    {
      "id": "cargo:syn@3.0.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/syn/3.0.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-syn-3.0.9",
      "noticeSha256": "918FB51402C362BB4AD1DF273C28564A70EEB5CB4102BDF7099A1665025E6089"
    },
    {
      "id": "cargo:sync_wrapper@1.0.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/sync_wrapper/1.0.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/sync_wrapper/1.0.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:synstructure@0.14.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/synstructure/0.14.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-synstructure-0.14.0",
      "noticeSha256": "C9AC69AB33574B4EB8F3C10A7E05B9D118AD9254D087611D742BD12D380BA3FB"
    },
    {
      "id": "cargo:system-deps@6.2.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/system-deps/6.2.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/system-deps/6.2.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tao-macros@0.1.4",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/tao-macros/0.1.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tao-macros/0.1.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tao@0.37.2",
      "license": "Apache-2.0",
      "source": "https://crates.io/api/v1/crates/tao/0.37.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tao-0.37.2",
      "noticeSha256": "2995941F84B517EEAD958168B96A942724435119DFF3DD45D3AEE9D6F35BDEB9"
    },
    {
      "id": "cargo:target-lexicon@0.12.16",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/target-lexicon/0.12.16/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/target-lexicon/0.12.16/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri-build@2.7.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/tauri-build/2.7.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-build/2.7.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri-codegen@2.7.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri-codegen/2.7.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-codegen-2.7.4",
      "noticeSha256": "F7538EBC0B785C20E8AD77C741D7C014B6B6ACCFB9E6436437E9E070F3668730"
    },
    {
      "id": "cargo:tauri-macros@2.7.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri-macros/2.7.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-macros-2.7.4",
      "noticeSha256": "E6C98FE4286C0EB2DD0A614A3A7194AC1F0A9CED37FAA1EFC2C0608FC6945FED"
    },
    {
      "id": "cargo:tauri-plugin-dialog@2.8.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri-plugin-dialog/2.8.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-plugin-dialog-2.8.4",
      "noticeSha256": "CA1701C94ECE30A7F480ACB30692F974FDCC747384B05A9A398695C2663C96FD"
    },
    {
      "id": "cargo:tauri-plugin-fs@2.7.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri-plugin-fs/2.7.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-plugin-fs-2.7.1",
      "noticeSha256": "0F28DA4FF9C4CAFFCE758BEDB6AF858D6864C0527B2819B36611575AE4894FCB"
    },
    {
      "id": "cargo:tauri-plugin@2.7.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/tauri-plugin/2.7.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-plugin/2.7.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri-runtime-wry@2.12.3",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri-runtime-wry/2.12.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-runtime-wry-2.12.3",
      "noticeSha256": "541464B44D1784CCED4E5FA8CE2323CDC44DE9E4A36127FCB829A7D7A932809A"
    },
    {
      "id": "cargo:tauri-runtime@2.12.2",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri-runtime/2.12.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-runtime-2.12.2",
      "noticeSha256": "CC921EEB948B8DF4DED248E5CBF4BBE7DF99134089F1211CE602A303A8870B4B"
    },
    {
      "id": "cargo:tauri-utils@2.10.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri-utils/2.10.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-utils-2.10.4",
      "noticeSha256": "265526CE3405A0A99271D9617CA65CBEF7F28F207ED467AD926C04A74A37A8B9"
    },
    {
      "id": "cargo:tauri-winres@0.3.7",
      "license": "MIT",
      "source": "https://docs.rs/crate/tauri-winres/0.3.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-winres/0.3.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri@2.12.5",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/tauri/2.12.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tauri-2.12.5",
      "noticeSha256": "C771786981275958A372694F017329363E903607D5FCD822FDB0E608604486D8"
    },
    {
      "id": "cargo:tempfile@3.27.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/tempfile/3.27.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tempfile/3.27.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tendril@0.5.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/tendril/0.5.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tendril-0.5.1",
      "noticeSha256": "F67E0925B956F92384E8CA24AD3562129F1096028BCDC2182E0903EE6CFE9682"
    },
    {
      "id": "cargo:thiserror-impl@1.0.69",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/thiserror-impl/1.0.69/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/thiserror-impl/1.0.69/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:thiserror-impl@2.0.21",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/thiserror-impl/2.0.21/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-thiserror-impl-2.0.21",
      "noticeSha256": "471BFB8AA6EB7DB5C95D53278366451E03626C9D04BABD22E2942D984ECAC76D"
    },
    {
      "id": "cargo:thiserror@1.0.69",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/thiserror/1.0.69/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/thiserror/1.0.69/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:thiserror@2.0.21",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/thiserror/2.0.21/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-thiserror-2.0.21",
      "noticeSha256": "E43FBBB94DBE99C3F6C5FAEE1159C89F2E89C8C8A4174B62927D2EAFAB75E961"
    },
    {
      "id": "cargo:time-core@0.1.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/time-core/0.1.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-time-core-0.1.9",
      "noticeSha256": "14F45079EC0EE15A5E85FC872EA9D7F3E94FBA8DF77B6CB6A573E742142AE7E0"
    },
    {
      "id": "cargo:time-macros@0.2.32",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/time-macros/0.2.32/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-time-macros-0.2.32",
      "noticeSha256": "DB2E33FCB2C98B0F25A6607AA8298A5B5F43FE6B9350D770460BEF762C6D3CE4"
    },
    {
      "id": "cargo:time@0.3.55",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/time/0.3.55/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-time-0.3.55",
      "noticeSha256": "4723E4919FD9D9858AB38AAAAAF2CFAFED71949ADEBC41629988A113AB57ADAF"
    },
    {
      "id": "cargo:tinystr@0.8.5",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/tinystr/0.8.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tinystr-0.8.5",
      "noticeSha256": "2BF0B51801BD68894F74EFD1FE8CDA73178A4F9D65832052D03229C5E9E9A876"
    },
    {
      "id": "cargo:tinyvec@1.13.3",
      "license": "Zlib OR Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/tinyvec/1.13.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tinyvec/1.13.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tokio-util@0.7.20",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/tokio-util/0.7.20/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tokio-util/0.7.20/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tokio@1.54.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/tokio/1.54.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tokio-1.54.1",
      "noticeSha256": "D989681824981AF0B204E2F5FBA8EFDCEFCA78E82F9C9BD87C4CB4B34A8561A8"
    },
    {
      "id": "cargo:toml_datetime@0.6.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/toml_datetime/0.6.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml_datetime/0.6.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:toml_datetime@1.1.3+spec-1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/toml_datetime/1.1.3+spec-1.1.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-toml_datetime-1.1.3+spec-1.1.0",
      "noticeSha256": "E08D4D4301B9C4B9161013C30775ACF5F1609373893340F9E0713D218E564A05"
    },
    {
      "id": "cargo:toml_edit@0.19.15",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/toml_edit/0.19.15/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml_edit/0.19.15/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:toml_edit@0.20.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/toml_edit/0.20.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml_edit/0.20.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:toml_edit@0.25.19+spec-1.1.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/toml_edit/0.25.19+spec-1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml_edit/0.25.19+spec-1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:toml_parser@1.1.5+spec-1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/toml_parser/1.1.5+spec-1.1.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-toml_parser-1.1.5+spec-1.1.0",
      "noticeSha256": "3338238DBE294504AE9413DEBCE1932B3F5B0138173F704C59716255B90E0F42"
    },
    {
      "id": "cargo:toml_writer@1.2.0+spec-1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/toml_writer/1.2.0+spec-1.1.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-toml_writer-1.2.0+spec-1.1.0",
      "noticeSha256": "A4FB1442A3C3379D9D6A53BCE4817C301ABB7697A8EA29361FB0A471D68E2F12"
    },
    {
      "id": "cargo:toml@0.8.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/toml/0.8.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml/0.8.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:toml@1.2.1+spec-1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/toml/1.2.1+spec-1.1.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-toml-1.2.1+spec-1.1.0",
      "noticeSha256": "CD54E5B62DAE801014688F58D167D4B463FF173308D44FD84A56F98D2B666091"
    },
    {
      "id": "cargo:tower-http@0.6.11",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/tower-http/0.6.11/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tower-http/0.6.11/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tower-layer@0.3.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/tower-layer/0.3.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tower-layer/0.3.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tower-service@0.3.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/tower-service/0.3.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tower-service/0.3.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tower@0.5.3",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/tower/0.5.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tower/0.5.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tracing-core@0.1.36",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/tracing-core/0.1.36/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tracing-core-0.1.36",
      "noticeSha256": "19314B4C5A40DA5F0ECABA914DBB0FDA9A95869A38B99ED369D60905377B9B05"
    },
    {
      "id": "cargo:tracing@0.1.44",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/tracing/0.1.44/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-tracing-0.1.44",
      "noticeSha256": "D017B2E5DC608A6321F21D8C57C9EC556DB42D9CD4A28C5E5A7EEBDAAC61FE6F"
    },
    {
      "id": "cargo:tray-icon@0.25.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/tray-icon/0.25.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tray-icon/0.25.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:try-lock@0.2.5",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/try-lock/0.2.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/try-lock/0.2.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:typeid@1.0.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/typeid/1.0.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-typeid-1.0.3",
      "noticeSha256": "4E431F617285F1F067681369E02818C365B0DFB5645DB1135D9AED22416A89C4"
    },
    {
      "id": "cargo:typenum@1.20.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/typenum/1.20.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-typenum-1.20.1",
      "noticeSha256": "A6CF4DED8707E6C7BF77598E3D939C1D78D2E61D34F35FB65ED0837E5747135C"
    },
    {
      "id": "cargo:unicode-ident@1.0.27",
      "license": "(MIT OR Apache-2.0) AND Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/unicode-ident/1.0.27/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-unicode-ident-1.0.27",
      "noticeSha256": "A32D9B7626589DDAD5AECE21FC14A58C886E7765F4A0CD2E81EA344D6D6B5E2D"
    },
    {
      "id": "cargo:unicode-segmentation@1.13.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/unicode-segmentation/1.13.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-unicode-segmentation-1.13.3",
      "noticeSha256": "E0BED8202661F557BF17389274D8AB48CE53CD3F3BB7764BD4EA421BE5CABD85"
    },
    {
      "id": "cargo:url@2.5.8",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/url/2.5.8/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-url-2.5.8",
      "noticeSha256": "298023B5CB607F4A1A4CDF3489090BDB244AB58869D07681B018E9287A619282"
    },
    {
      "id": "cargo:urlpattern@0.6.0",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/urlpattern/0.6.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-urlpattern-0.6.0",
      "noticeSha256": "DF90181F8B87F2D7918A1825BB54F17B8D52B6267B49F02EC9DA6E4350BB393C"
    },
    {
      "id": "cargo:utf_types@1.0.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/utf_types/1.0.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-utf_types-1.0.0",
      "noticeSha256": "C4FB6D8D57A41AEEF154C49420511A7370CD1FBE603D41C3FC4B7CE366CB8C9C"
    },
    {
      "id": "cargo:utf8_iter@1.1.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/utf8_iter/1.1.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-utf8_iter-1.1.0",
      "noticeSha256": "19692EC772226646BB2F5D969C3CB62F89C45269EAEC70F766F4CE92894B17D9"
    },
    {
      "id": "cargo:uuid@1.29.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/uuid/1.29.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-uuid-1.29.1",
      "noticeSha256": "B419410EE85825C26A8C30DCE636BFB76C931946A2ED42B0A7691E6233448F01"
    },
    {
      "id": "cargo:version_check@0.9.5",
      "license": "MIT/Apache-2.0",
      "source": "https://docs.rs/crate/version_check/0.9.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/version_check/0.9.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:version-compare@0.2.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/version-compare/0.2.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/version-compare/0.2.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:veylora-app@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-cli@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-color@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-compose@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-device@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-diagnostics@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-domain@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-image@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-layout@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-persistence@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-release@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-render@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-screen@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-separation@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-transport@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:veylora-workflow@0.0.0",
      "license": "LicenseRef-Veylora-Proprietary",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised",
      "status": "reviewed",
      "notice": "veylora-rip",
      "noticeSha256": "12FD9F1834F91635830E3C011950D844CDB16DF04EDF47058AA7114ECBC79EB6"
    },
    {
      "id": "cargo:vswhom-sys@0.1.3",
      "license": "MIT",
      "source": "https://docs.rs/crate/vswhom-sys/0.1.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/vswhom-sys/0.1.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:vswhom@0.1.0",
      "license": "MIT",
      "source": "https://docs.rs/crate/vswhom/0.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/vswhom/0.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:walkdir@2.5.0",
      "license": "Unlicense/MIT",
      "source": "https://crates.io/api/v1/crates/walkdir/2.5.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-walkdir-2.5.0",
      "noticeSha256": "3B1AE99BB9C45F5283F5B3639139671BD54426D5D5B25C4950FEA09051CA13F2"
    },
    {
      "id": "cargo:want@0.3.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/want/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/want/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasi@0.11.1+wasi-snapshot-preview1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasi/0.11.1+wasi-snapshot-preview1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasi/0.11.1+wasi-snapshot-preview1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasip2@1.0.4+wasi-0.2.12",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasip2/1.0.4+wasi-0.2.12/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasip2/1.0.4+wasi-0.2.12/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasm-bindgen-futures@0.4.80",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasm-bindgen-futures/0.4.80/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasm-bindgen-futures/0.4.80/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasm-bindgen-macro-support@0.2.130",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasm-bindgen-macro-support/0.2.130/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasm-bindgen-macro-support/0.2.130/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasm-bindgen-macro@0.2.130",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasm-bindgen-macro/0.2.130/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasm-bindgen-macro/0.2.130/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasm-bindgen-shared@0.2.130",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasm-bindgen-shared/0.2.130/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasm-bindgen-shared/0.2.130/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasm-bindgen@0.2.130",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasm-bindgen/0.2.130/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasm-bindgen/0.2.130/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wasm-streams@0.5.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wasm-streams/0.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wasm-streams/0.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:web_atoms@0.2.6",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/web_atoms/0.2.6/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-web_atoms-0.2.6",
      "noticeSha256": "4D7E711A2B3C193B93C562A18FA6B77C24CD83F20F4EEBCF1C061E97CCFC045D"
    },
    {
      "id": "cargo:web-sys@0.3.107",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/web-sys/0.3.107/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/web-sys/0.3.107/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:web-time@1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/web-time/1.1.0/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-web-time-1.1.0",
      "noticeSha256": "74F5E431C505E733414FCA1D924DE92CEDCBF85B87A9C7D46DEED08C35D7C129"
    },
    {
      "id": "cargo:webkit2gtk-sys@2.0.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/webkit2gtk-sys/2.0.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/webkit2gtk-sys/2.0.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:webkit2gtk@2.0.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/webkit2gtk/2.0.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/webkit2gtk/2.0.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:webview2-com-macros@0.8.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/webview2-com-macros/0.8.1/download (checksum from Cargo.lock)",
      "licenseSource": "license text from source-file headers of the checksum-verified crate (no LICENSE file shipped) plus upstream license text kept in docs/license-evidence (SHA-256 checked); permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-webview2-com-macros-0.8.1",
      "noticeSha256": "FA24E99F320EAB5BE7827F77B94D6EDCAC9AE062F9503E8E506E45AF241942C2"
    },
    {
      "id": "cargo:webview2-com-sys@0.39.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/webview2-com-sys/0.39.1/download (checksum from Cargo.lock)",
      "licenseSource": "license text from source-file headers of the checksum-verified crate (no LICENSE file shipped) plus upstream license text kept in docs/license-evidence (SHA-256 checked); permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-webview2-com-sys-0.39.1",
      "noticeSha256": "EC049A7B814F802FEFE0D553E7D55DCFC3E0A9600616DFCE86477F19BEC4C974"
    },
    {
      "id": "cargo:webview2-com@0.39.1",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/webview2-com/0.39.1/download (checksum from Cargo.lock)",
      "licenseSource": "license text from source-file headers of the checksum-verified crate (no LICENSE file shipped) plus upstream license text kept in docs/license-evidence (SHA-256 checked); permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-webview2-com-0.39.1",
      "noticeSha256": "CD6E85C0C1DA1508C64ADDC35F99DCF861700400BFF0C427A82A903651D3D216"
    },
    {
      "id": "cargo:winapi-i686-pc-windows-gnu@0.4.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/winapi-i686-pc-windows-gnu/0.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/winapi-i686-pc-windows-gnu/0.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:winapi-util@0.1.11",
      "license": "Unlicense OR MIT",
      "source": "https://crates.io/api/v1/crates/winapi-util/0.1.11/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-winapi-util-0.1.11",
      "noticeSha256": "EC98EC4575241B41A5A37FF0DBACDF33351E441B27946669C13057F43E1DD2F9"
    },
    {
      "id": "cargo:winapi-x86_64-pc-windows-gnu@0.4.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/winapi-x86_64-pc-windows-gnu/0.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/winapi-x86_64-pc-windows-gnu/0.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:winapi@0.3.9",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/winapi/0.3.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/winapi/0.3.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:window-vibrancy@0.8.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/window-vibrancy/0.8.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-window-vibrancy-0.8.1",
      "noticeSha256": "BEC39F3D6E5CFA3AC12CC9E1D664ABB38755AF96F9F80658CCB16CCC2AC0600D"
    },
    {
      "id": "cargo:windows_aarch64_gnullvm@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows_aarch64_gnullvm/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows_aarch64_gnullvm/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows_aarch64_msvc@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows_aarch64_msvc/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows_aarch64_msvc/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows_i686_gnu@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows_i686_gnu/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows_i686_gnu/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows_i686_msvc@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows_i686_msvc/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows_i686_msvc/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows_x86_64_gnu@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows_x86_64_gnu/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows_x86_64_gnu/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows_x86_64_gnullvm@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows_x86_64_gnullvm/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows_x86_64_gnullvm/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows_x86_64_msvc@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows_x86_64_msvc/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows_x86_64_msvc/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-collections@0.3.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-collections/0.3.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-collections-0.3.2",
      "noticeSha256": "2D96303504852780A006C3EC095C6ACD7064C4B9355CCA8E78161EBCC8BE71CE"
    },
    {
      "id": "cargo:windows-core@0.62.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-core/0.62.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-core-0.62.2",
      "noticeSha256": "A50746A7E82AA49612548B8D82BB76E74AACE8390CF8926272EB1EA997A0DD11"
    },
    {
      "id": "cargo:windows-future@0.3.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-future/0.3.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-future-0.3.2",
      "noticeSha256": "8F04F371DABB61E5D7AB8C39DFC1039CDF8A828DF7F78FFAF56FD7E1007BED98"
    },
    {
      "id": "cargo:windows-implement@0.60.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-implement/0.60.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-implement-0.60.2",
      "noticeSha256": "85D7E75ECD06BD16C7801F8163AA71356AC70905CF78F7802C244BB7CC0716DF"
    },
    {
      "id": "cargo:windows-interface@0.59.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-interface/0.59.3/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-interface-0.59.3",
      "noticeSha256": "8FD9BFD11181DBD286F20D89F0485B6F3ED59A5937F84A13FDAEE716C12C7C2A"
    },
    {
      "id": "cargo:windows-link@0.2.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-link/0.2.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-link-0.2.1",
      "noticeSha256": "E8063392F3B3DA9F9A01270E731F785444C85CE57ACFF9DCFA7833D2924B1FF1"
    },
    {
      "id": "cargo:windows-numerics@0.3.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-numerics/0.3.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-numerics-0.3.1",
      "noticeSha256": "910A192D7167581D00748208C65F7DCEDE56FD0475B6624CD8252FDCC95B44C8"
    },
    {
      "id": "cargo:windows-result@0.4.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-result/0.4.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-result-0.4.1",
      "noticeSha256": "1269128A1B714D126CFE7C3B6DA221AEF3B88999AFB40EF52B3DB421D5BD969C"
    },
    {
      "id": "cargo:windows-strings@0.5.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-strings/0.5.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-strings-0.5.1",
      "noticeSha256": "9A70C926ABB19133F87199075A565253BF1DAA968EA2F9A91B8F27B1A3EBEAFE"
    },
    {
      "id": "cargo:windows-sys@0.45.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows-sys/0.45.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-sys/0.45.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-sys@0.61.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-sys/0.61.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-sys-0.61.2",
      "noticeSha256": "6A7327781A64257665F579A0386306E50A3BFCAA06606FFD88E7B0F511A167F2"
    },
    {
      "id": "cargo:windows-targets@0.42.2",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/windows-targets/0.42.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-targets/0.42.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-threading@0.2.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-threading/0.2.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-threading-0.2.1",
      "noticeSha256": "326086FB1118980C959BDD0031553AA27D47FAA548592242A272285E08D081DB"
    },
    {
      "id": "cargo:windows-version@0.1.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows-version/0.1.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-version-0.1.7",
      "noticeSha256": "47FBB71D64BC286D4CBCCCB87FDCD5DD4AC238A1A05A83FB07E866DBBFD0D66B"
    },
    {
      "id": "cargo:windows@0.62.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://crates.io/api/v1/crates/windows/0.62.2/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-windows-0.62.2",
      "noticeSha256": "34F45C666E1F9895C52D9ACAA3F8EEF9BCBFD6636B01991253E6CE82E4DB99A0"
    },
    {
      "id": "cargo:winnow@0.5.40",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/winnow/0.5.40/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/winnow/0.5.40/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:winnow@1.0.5",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/winnow/1.0.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-winnow-1.0.5",
      "noticeSha256": "D9D8F5403C4C1641CA68856DA16FD4434B5B7AA4BD20B02DFF488F2A40F77B03"
    },
    {
      "id": "cargo:winreg@0.56.0",
      "license": "MIT",
      "source": "https://docs.rs/crate/winreg/0.56.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/winreg/0.56.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wit-bindgen@0.57.1",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/wit-bindgen/0.57.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wit-bindgen/0.57.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:writeable@0.6.4",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/writeable/0.6.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-writeable-0.6.4",
      "noticeSha256": "862D275FA636297F01293DF0848B1390980059E6F58C2C5BB99995C19EDA2FBB"
    },
    {
      "id": "cargo:wry@0.57.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://crates.io/api/v1/crates/wry/0.57.1/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-wry-0.57.1",
      "noticeSha256": "1C653F83E63BE1A88A0233043A4FBD4CCAAE10681549DA6531059DD6B5DD7F63"
    },
    {
      "id": "cargo:x11-dl@2.21.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/x11-dl/2.21.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/x11-dl/2.21.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:x11@2.21.0",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/x11/2.21.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/x11/2.21.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:yoke-derive@0.8.4",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/yoke-derive/0.8.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-yoke-derive-0.8.4",
      "noticeSha256": "57052AC9AC98FB8E0C2CBD8958834853B260F7114A9DB224F2B8EA34046D2657"
    },
    {
      "id": "cargo:yoke@0.8.4",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/yoke/0.8.4/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-yoke-0.8.4",
      "noticeSha256": "08D20B29723B69E5F14C164DE8CE8961B37608CB18C45851FF8D4BD68A7929C3"
    },
    {
      "id": "cargo:zerofrom-derive@0.1.8",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/zerofrom-derive/0.1.8/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-zerofrom-derive-0.1.8",
      "noticeSha256": "58CCB5426767129B54330E0BFAE1DD03ED22E63CA10DDD8C4DCCE138E91F915A"
    },
    {
      "id": "cargo:zerofrom@0.1.8",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/zerofrom/0.1.8/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-zerofrom-0.1.8",
      "noticeSha256": "42FF6C5952B1B3E7ED95D618D567B42D087132AFC6145BE94EE29782D8E7E1CD"
    },
    {
      "id": "cargo:zerotrie@0.2.5",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/zerotrie/0.2.5/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-zerotrie-0.2.5",
      "noticeSha256": "62FA28B8E2046AA661E7FCC3CD3F14150F938F7DF41B81FA0C14FB4710914FE8"
    },
    {
      "id": "cargo:zerovec-derive@0.11.7",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/zerovec-derive/0.11.7/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-zerovec-derive-0.11.7",
      "noticeSha256": "2E4E0F0C450C587798C53A64A9D2884A84869E6541F52B23F68206F6687F1D4D"
    },
    {
      "id": "cargo:zerovec@0.11.9",
      "license": "Unicode-3.0",
      "source": "https://crates.io/api/v1/crates/zerovec/0.11.9/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-zerovec-0.11.9",
      "noticeSha256": "1095EF43E5B4919423723E4A3EE727B8D8E75E18482F3AB6313422E8EE2F1FD2"
    },
    {
      "id": "cargo:zlib-rs@0.6.8",
      "license": "Zlib",
      "source": "https://docs.rs/crate/zlib-rs/0.6.8/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/zlib-rs/0.6.8/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:zmij@1.0.23",
      "license": "MIT",
      "source": "https://crates.io/api/v1/crates/zmij/1.0.23/download (checksum from Cargo.lock)",
      "licenseSource": "verbatim license files from the checksum-verified crate source; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "cargo-zmij-1.0.23",
      "noticeSha256": "727A95D404711F004DC855AE7E34AD458FCAD5C9E443EE1FCDCA4418F7E540D5"
    },
    {
      "id": "npm:@fontsource-variable/geist-mono@5.3.0",
      "license": "OFL-1.1",
      "source": "https://registry.npmjs.org/@fontsource-variable/geist-mono/5.3.0",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; accepted 2026-11-21 for the unmodified Geist fonts the app bundles (the owner chose Geist; OFL-1.1 permits bundling with the copyright and license retained; fonts are never sold on their own and are not renamed or modified)",
      "status": "reviewed",
      "notice": "npm-fontsource-variable-geist-mono-5.3.0",
      "noticeSha256": "A03C5E1157BE029E2F3E7994393D646AE455FD12304023E98A3AA5FB91317ED1"
    },
    {
      "id": "npm:@fontsource-variable/geist@5.3.0",
      "license": "OFL-1.1",
      "source": "https://registry.npmjs.org/@fontsource-variable/geist/5.3.0",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; accepted 2026-11-21 for the unmodified Geist fonts the app bundles (the owner chose Geist; OFL-1.1 permits bundling with the copyright and license retained; fonts are never sold on their own and are not renamed or modified)",
      "status": "reviewed",
      "notice": "npm-fontsource-variable-geist-5.3.0",
      "noticeSha256": "FB4F8258E09E1D8BC157099FC305B4634CF4D64134DEC7F105BF2EBAA94C0B91"
    },
    {
      "id": "npm:@lucide/svelte@1.70.0",
      "license": "ISC",
      "source": "https://registry.npmjs.org/@lucide/svelte/1.70.0",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-lucide-svelte-1.70.0",
      "noticeSha256": "6ED3DB4AE3E44444B921071113E69BB5903778A278EAB42C5DE7FCB5C5C15D34"
    },
    {
      "id": "npm:@pnpm/exe.android-arm64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.android-arm64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.android-x64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.android-x64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.darwin-arm64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.darwin-arm64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.darwin-x64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.darwin-x64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.freebsd-x64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.freebsd-x64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.linux-arm64-musl@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.linux-arm64-musl/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.linux-arm64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.linux-arm64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.linux-ppc64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.linux-ppc64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.linux-riscv64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.linux-riscv64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.linux-s390x@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.linux-s390x/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.linux-x64-musl@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.linux-x64-musl/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.linux-x64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.linux-x64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.win32-arm64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.win32-arm64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@pnpm/exe.win32-x64@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@pnpm/exe.win32-x64/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@sveltejs/vite-plugin-svelte@7.3.1",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@sveltejs/vite-plugin-svelte/7.3.1",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@tauri-apps/api@2.12.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://registry.npmjs.org/@tauri-apps/api/2.12.4",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-tauri-apps-api-2.12.4",
      "noticeSha256": "71DC9B77E72EB302D7641F939E6D64DFD074C1F56D11BE9E2068F92A6C7814FA"
    },
    {
      "id": "npm:@tauri-apps/cli@2.12.4",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@tauri-apps/cli/2.12.4",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@testing-library/jest-dom@7.0.1",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@testing-library/jest-dom/7.0.1",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@testing-library/svelte@5.4.2",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@testing-library/svelte/5.4.2",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@testing-library/user-event@14.6.8",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@testing-library/user-event/14.6.8",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:bits-ui@2.20.6",
      "license": "MIT",
      "source": "https://registry.npmjs.org/bits-ui/2.20.6",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-bits-ui-2.20.6",
      "noticeSha256": "62A325DCF5884D9EE7175152A9A88D8C863BFBC3866B4696DBFAAF7B85B59F5B"
    },
    {
      "id": "npm:jsdom@31.0.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/jsdom/31.0.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:pnpm@12.18.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/pnpm/12.18.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:svelte-check@4.7.8",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/svelte-check/4.7.8",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:svelte@5.57.3",
      "license": "MIT",
      "source": "https://registry.npmjs.org/svelte/5.57.3",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-svelte-5.57.3",
      "noticeSha256": "03B362B9703F6B3EEC781E5CA324BE67D591D2DA74D88ADBA68B0730DA2782BF"
    },
    {
      "id": "npm:typescript@6.0.3",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/typescript/6.0.3",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:vite-plus@1.5.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/vite-plus/1.5.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:clsx@2.1.1",
      "license": "MIT",
      "source": "https://registry.npmjs.org/clsx/2.1.1",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-clsx-2.1.1",
      "noticeSha256": "E778B72EC7FF6478F54D44DE0DBB75D145128F0214C7A434BDBECF077D1F6BD6"
    },
    {
      "id": "npm:esm-env@1.2.2",
      "license": "MIT",
      "source": "https://registry.npmjs.org/esm-env/1.2.2",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-esm-env-1.2.2",
      "noticeSha256": "D923AF17272662F4CE46D48D8E9B1907AA53CABBAF5966391B9E3595EED9D15F"
    },
    {
      "id": "npm:inline-style-parser@0.2.7",
      "license": "MIT",
      "source": "https://registry.npmjs.org/inline-style-parser/0.2.7",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-inline-style-parser-0.2.7",
      "noticeSha256": "ED97E898A798FF71D3C3D97EFF6D2DB2D71FD055B1159DE95590E9B905798ABC"
    },
    {
      "id": "npm:runed@0.35.1",
      "license": "MIT",
      "source": "https://registry.npmjs.org/runed/0.35.1",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-runed-0.35.1",
      "noticeSha256": "893BE84DE363BE8583123C4B333BCB917AA03DB32498521A814EA854CBDD3D98"
    },
    {
      "id": "npm:style-to-object@1.0.14",
      "license": "MIT",
      "source": "https://registry.npmjs.org/style-to-object/1.0.14",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-style-to-object-1.0.14",
      "noticeSha256": "D1B45A6C61A505A0EFDAAADA7E0D8AF7324E4C12FBC3CDB06C678F887ABB2243"
    },
    {
      "id": "npm:svelte-toolbelt@0.10.7",
      "license": "MIT",
      "source": "https://registry.npmjs.org/svelte-toolbelt/0.10.7",
      "licenseSource": "verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; permissive terms; automated under owner-approved policy",
      "status": "reviewed",
      "notice": "npm-svelte-toolbelt-0.10.7",
      "noticeSha256": "04390DB8880FDA802543A1DFD259FD6AC4CDC2EF13D0F869BFD985F164175659"
    },
    {
      "id": "toolchain:rust-std@1.99.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://github.com/rust-lang/rust/tree/1.99.1/library",
      "licenseSource": "rustc 1.99.1 distribution: share/doc/rust/licenses and COPYRIGHT-library.html",
      "status": "reviewed",
      "notice": "toolchain-rust-std-1.99.1",
      "noticeSha256": "8B96674E7F9E4A3519C20D47745501321AC4BC52837B99A471AB62ADCB30CBED"
    }
  ],
  "blockers": [
    "Review every exact package license, copyright, NOTICE and redistribution obligation. Manifest declarations are not clearance.",
    "Frontend lock contains pnpm only, not the declared application graph. Resolve and review the complete frontend lock before release.",
    "Review native libraries, external tools, profiles, fonts and fixture rights described above. Verify final installer and About notices.",
    "Project workspace license is UNLICENSED. Obtain explicit project distribution authorization."
  ],
  "scopes": {
    "veylora-cli": {
      "target": "x86_64-pc-windows-msvc",
      "extra": [
        "toolchain:rust-std@1.99.1"
      ],
      "files": [],
      "rustStdCopyrightSha256": "5647BE074C8EDF7339FD863055923A8FC80BC5610A8D4661EC3B767B9D392C27",
      "blockers": []
    },
    "veylora-app": {
      "target": "x86_64-pc-windows-msvc",
      "extra": [
        "toolchain:rust-std@1.99.1",
        "npm:@lucide/svelte@1.70.0",
        "npm:@tauri-apps/api@2.12.4",
        "npm:bits-ui@2.20.6",
        "npm:clsx@2.1.1",
        "npm:esm-env@1.2.2",
        "npm:inline-style-parser@0.2.7",
        "npm:runed@0.35.1",
        "npm:style-to-object@1.0.14",
        "npm:svelte@5.57.3",
        "npm:svelte-toolbelt@0.10.7",
        "npm:@fontsource-variable/geist-mono@5.3.0",
        "npm:@fontsource-variable/geist@5.3.0"
      ],
      "files": [
        "app/package.json",
        "app/pnpm-lock.yaml"
      ],
      "rustStdCopyrightSha256": "5647BE074C8EDF7339FD863055923A8FC80BC5610A8D4661EC3B767B9D392C27",
      "blockers": []
    }
  }
}
```
