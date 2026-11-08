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
      "sha256": "9DA16634096D40187943AE9ED16B570ACB2DE4B83E35D7EA363CF66B1136AABF"
    },
    {
      "path": "Cargo.lock",
      "sha256": "8EE6B4438FE278720095576965D627490AFE947F439838AE338AD30DBDBB87A6"
    },
    {
      "path": "Cargo.toml",
      "sha256": "1E777BDD372B612A18F21120CE168252EAF3B273F94F65A6AD946B191255DE4B"
    },
    {
      "path": "crates/veylora-app/build.rs",
      "sha256": "487059EAF8A947B80F20A9AACAC038A5047B2AD69D2401B827376C67D6FE847F"
    },
    {
      "path": "crates/veylora-app/Cargo.toml",
      "sha256": "1870B8DD4C4D12D70012A54D475F524AD5C4AC06ABBCA1EDA979D21BAD197113"
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
      "sha256": "D0D5EA79A4938585B8990694B8344E73FCF0E7A557C05DBEFADDE9A9BD5F112A"
    },
    {
      "path": "crates/veylora-device/Cargo.toml",
      "sha256": "ABB5F5C761AB480737B0D32C5790ACBA9B287122AA8DCF47117E138B123FBFEE"
    },
    {
      "path": "crates/veylora-diagnostics/Cargo.toml",
      "sha256": "B8513ADB6FCD68BBCF3A548620A5AD60BC34BCAA3081EF9AF97BC7BD6F50129D"
    },
    {
      "path": "crates/veylora-domain/Cargo.toml",
      "sha256": "A309EBC0CD29AD8A02FD5EE98E10AAC7F9BABEBA83480AE572AB451D716B28E4"
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
      "source": "https://docs.rs/crate/adler2/2.0.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/adler2/2.0.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:aho-corasick@1.1.5",
      "license": "Unlicense OR MIT",
      "source": "https://docs.rs/crate/aho-corasick/1.1.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/aho-corasick/1.1.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:alloc-no-stdlib@3.0.0",
      "license": "BSD-3-Clause",
      "source": "https://docs.rs/crate/alloc-no-stdlib/3.0.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/alloc-no-stdlib/3.0.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:alloc-stdlib@0.3.0",
      "license": "BSD-3-Clause",
      "source": "https://docs.rs/crate/alloc-stdlib/0.3.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/alloc-stdlib/0.3.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/anyhow/1.0.104/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/anyhow/1.0.104/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/base64/0.23.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/base64/0.23.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:bit-set@0.8.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/bit-set/0.8.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bit-set/0.8.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:bit-vec@0.8.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/bit-vec/0.8.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bit-vec/0.8.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/bitflags/2.13.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bitflags/2.13.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:block-buffer@0.10.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/block-buffer/0.10.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/block-buffer/0.10.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/brotli-decompressor/6.3.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/brotli-decompressor/6.3.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:brotli@9.1.1",
      "license": "BSD-3-Clause AND MIT",
      "source": "https://docs.rs/crate/brotli/9.1.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/brotli/9.1.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/byteorder/1.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/byteorder/1.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:bytes@1.12.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/bytes/1.12.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/bytes/1.12.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/camino/1.2.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/camino/1.2.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cargo_metadata@0.19.2",
      "license": "MIT",
      "source": "https://docs.rs/crate/cargo_metadata/0.19.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cargo_metadata/0.19.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/cargo-platform/0.1.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cargo-platform/0.1.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/cfb/0.14.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cfb/0.14.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/cfg-if/1.0.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cfg-if/1.0.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/cookie/0.18.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cookie/0.18.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/cpufeatures/0.2.17/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cpufeatures/0.2.17/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:crc32fast@1.5.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/crc32fast/1.5.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/crc32fast/1.5.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:crossbeam-channel@0.5.17",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/crossbeam-channel/0.5.17/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/crossbeam-channel/0.5.17/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:crossbeam-utils@0.8.23",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/crossbeam-utils/0.8.23/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/crossbeam-utils/0.8.23/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:crypto-common@0.1.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/crypto-common/0.1.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/crypto-common/0.1.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cssparser-macros@0.7.1",
      "license": "MPL-2.0",
      "source": "https://docs.rs/crate/cssparser-macros/0.7.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cssparser-macros/0.7.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:cssparser@0.37.0",
      "license": "MPL-2.0",
      "source": "https://docs.rs/crate/cssparser/0.37.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/cssparser/0.37.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:ctor@1.0.13",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/ctor/1.0.13/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ctor/1.0.13/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:darling_core@0.24.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/darling_core/0.24.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/darling_core/0.24.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:darling_macro@0.24.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/darling_macro/0.24.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/darling_macro/0.24.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:darling@0.24.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/darling/0.24.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/darling/0.24.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/deranged/0.5.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/deranged/0.5.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:derive_more-impl@2.1.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/derive_more-impl/2.1.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/derive_more-impl/2.1.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:derive_more@2.1.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/derive_more/2.1.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/derive_more/2.1.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:digest@0.10.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/digest/0.10.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/digest/0.10.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dirs-sys@0.5.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/dirs-sys/0.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dirs-sys/0.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dirs@7.0.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/dirs/7.0.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dirs/7.0.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/displaydoc/0.2.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/displaydoc/0.2.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/dom_query/0.28.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dom_query/0.28.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dpi@0.1.2",
      "license": "Apache-2.0 AND MIT",
      "source": "https://docs.rs/crate/dpi/0.1.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dpi/0.1.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dtoa-short@0.3.5",
      "license": "MPL-2.0",
      "source": "https://docs.rs/crate/dtoa-short/0.3.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dtoa-short/0.3.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dtoa@1.0.11",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/dtoa/1.0.11/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dtoa/1.0.11/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dunce@1.0.5",
      "license": "CC0-1.0 OR MIT-0 OR Apache-2.0",
      "source": "https://docs.rs/crate/dunce/1.0.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dunce/1.0.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:dyn-clone@1.0.20",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/dyn-clone/1.0.20/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/dyn-clone/1.0.20/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/equivalent/1.0.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/equivalent/1.0.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:erased-serde@0.4.11",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/erased-serde/0.4.11/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/erased-serde/0.4.11/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:fastrand@2.5.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/fastrand/2.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/fastrand/2.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:fdeflate@0.3.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/fdeflate/0.3.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/fdeflate/0.3.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/flate2/1.1.11/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/flate2/1.1.11/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:fnv@1.0.7",
      "license": "Apache-2.0 / MIT",
      "source": "https://docs.rs/crate/fnv/1.0.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/fnv/1.0.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:foldhash@0.2.0",
      "license": "Zlib",
      "source": "https://docs.rs/crate/foldhash/0.2.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/foldhash/0.2.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/form_urlencoded/1.2.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/form_urlencoded/1.2.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/generic-array/0.14.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/generic-array/0.14.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:getrandom@0.3.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/getrandom/0.3.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/getrandom/0.3.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:getrandom@0.4.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/getrandom/0.4.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/getrandom/0.4.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/glob/0.3.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/glob/0.3.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/hashbrown/0.12.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/hashbrown/0.12.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:hashbrown@0.17.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/hashbrown/0.17.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/hashbrown/0.17.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/heck/0.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/heck/0.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/html5ever/0.39.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/html5ever/0.39.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/http/1.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/http/1.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/ico/0.6.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ico/0.6.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:icu_collections@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/icu_collections/2.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/icu_collections/2.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:icu_locale_core@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/icu_locale_core/2.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/icu_locale_core/2.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:icu_normalizer_data@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/icu_normalizer_data/2.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/icu_normalizer_data/2.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:icu_normalizer@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/icu_normalizer/2.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/icu_normalizer/2.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:icu_properties_data@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/icu_properties_data/2.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/icu_properties_data/2.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:icu_properties@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/icu_properties/2.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/icu_properties/2.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:icu_provider@2.4.0",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/icu_provider/2.4.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/icu_provider/2.4.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:ident_case@1.0.1",
      "license": "MIT/Apache-2.0",
      "source": "https://docs.rs/crate/ident_case/1.0.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/ident_case/1.0.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:idna_adapter@1.2.2",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/idna_adapter/1.2.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/idna_adapter/1.2.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:idna@1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/idna/1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/idna/1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:indexmap@1.9.3",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/indexmap/1.9.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/indexmap/1.9.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:indexmap@2.14.2",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/indexmap/2.14.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/indexmap/2.14.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:infer@0.22.0",
      "license": "MIT",
      "source": "https://docs.rs/crate/infer/0.22.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/infer/0.22.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/itoa/1.0.18/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/itoa/1.0.18/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/json-patch/4.2.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/json-patch/4.2.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:jsonptr@0.7.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/jsonptr/0.7.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/jsonptr/0.7.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:keyboard-types@0.8.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/keyboard-types/0.8.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/keyboard-types/0.8.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/libc/0.2.191/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/libc/0.2.191/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "id": "cargo:litemap@0.8.3",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/litemap/0.8.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/litemap/0.8.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:lock_api@0.4.14",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/lock_api/0.4.14/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/lock_api/0.4.14/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:log@0.4.35",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/log/0.4.35/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/log/0.4.35/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:markup5ever@0.39.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/markup5ever/0.39.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/markup5ever/0.39.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:memchr@2.8.3",
      "license": "Unlicense OR MIT",
      "source": "https://docs.rs/crate/memchr/2.8.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/memchr/2.8.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/mime/0.3.18/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/mime/0.3.18/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:miniz_oxide@0.8.9",
      "license": "MIT OR Zlib OR Apache-2.0",
      "source": "https://docs.rs/crate/miniz_oxide/0.8.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/miniz_oxide/0.8.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:miniz_oxide@0.9.1",
      "license": "MIT OR Zlib OR Apache-2.0",
      "source": "https://docs.rs/crate/miniz_oxide/0.9.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/miniz_oxide/0.9.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/muda/0.20.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/muda/0.20.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/new_debug_unreachable/1.0.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/new_debug_unreachable/1.0.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/num-conv/0.2.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/num-conv/0.2.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/once_cell/1.21.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/once_cell/1.21.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:option-ext@0.2.0",
      "license": "MPL-2.0",
      "source": "https://docs.rs/crate/option-ext/0.2.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/option-ext/0.2.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/parking_lot_core/0.9.12/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/parking_lot_core/0.9.12/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:parking_lot@0.12.5",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/parking_lot/0.12.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/parking_lot/0.12.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:percent-encoding@2.3.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/percent-encoding/2.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/percent-encoding/2.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/phf_generator/0.13.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/phf_generator/0.13.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:phf_macros@0.13.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/phf_macros/0.13.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/phf_macros/0.13.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:phf_shared@0.13.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/phf_shared/0.13.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/phf_shared/0.13.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:phf@0.13.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/phf/0.13.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/phf/0.13.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:pin-project-lite@0.2.17",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/pin-project-lite/0.2.17/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/pin-project-lite/0.2.17/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/plist/1.10.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/plist/1.10.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:png@0.18.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/png/0.18.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/png/0.18.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/potential_utf/0.1.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/potential_utf/0.1.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:powerfmt@0.2.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/powerfmt/0.2.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/powerfmt/0.2.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:precomputed-hash@0.1.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/precomputed-hash/0.1.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/precomputed-hash/0.1.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/proc-macro2/1.0.107/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/proc-macro2/1.0.107/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:quick-xml@0.42.0",
      "license": "MIT",
      "source": "https://docs.rs/crate/quick-xml/0.42.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/quick-xml/0.42.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:quote@1.0.47",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/quote/1.0.47/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/quote/1.0.47/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/raw-window-handle/0.6.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/raw-window-handle/0.6.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/regex-automata/0.4.18/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/regex-automata/0.4.18/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:regex-syntax@0.8.11",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/regex-syntax/0.8.11/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/regex-syntax/0.8.11/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:regex@1.13.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/regex/1.13.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/regex/1.13.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/rustc-hash/2.1.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/rustc-hash/2.1.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
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
      "source": "https://docs.rs/crate/same-file/1.0.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/same-file/1.0.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:schemars_derive@0.8.22",
      "license": "MIT",
      "source": "https://docs.rs/crate/schemars_derive/0.8.22/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/schemars_derive/0.8.22/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:schemars@0.8.22",
      "license": "MIT",
      "source": "https://docs.rs/crate/schemars/0.8.22/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/schemars/0.8.22/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/scopeguard/1.2.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/scopeguard/1.2.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:selectors@0.38.0",
      "license": "MPL-2.0",
      "source": "https://docs.rs/crate/selectors/0.38.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/selectors/0.38.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:semver@1.0.28",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/semver/1.0.28/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/semver/1.0.28/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_core@1.0.229",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde_core/1.0.229/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_core/1.0.229/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_derive_internals@0.29.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde_derive_internals/0.29.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_derive_internals/0.29.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_derive@1.0.229",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde_derive/1.0.229/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_derive/1.0.229/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_json@1.0.154",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde_json/1.0.154/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_json/1.0.154/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_repr@0.1.21",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde_repr/0.1.21/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_repr/0.1.21/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/serde_spanned/1.1.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_spanned/1.1.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_with_macros@3.24.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde_with_macros/3.24.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_with_macros/3.24.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde_with@3.24.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde_with/3.24.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde_with/3.24.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde-untagged@0.1.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde-untagged/0.1.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde-untagged/0.1.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serde@1.0.229",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serde/1.0.229/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serde/1.0.229/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serialize-to-javascript-impl@0.1.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serialize-to-javascript-impl/0.1.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serialize-to-javascript-impl/0.1.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:serialize-to-javascript@0.1.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/serialize-to-javascript/0.1.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/serialize-to-javascript/0.1.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:servo_arc@0.4.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/servo_arc/0.4.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/servo_arc/0.4.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:sha2@0.10.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/sha2/0.10.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/sha2/0.10.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/simd-adler32/0.3.10/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/simd-adler32/0.3.10/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:siphasher@1.0.4",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/siphasher/1.0.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/siphasher/1.0.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/smallvec/1.16.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/smallvec/1.16.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/softbuffer/0.4.8/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/softbuffer/0.4.8/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/stable_deref_trait/1.2.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/stable_deref_trait/1.2.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/string_cache/0.9.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/string_cache/0.9.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:strsim@0.11.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/strsim/0.11.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/strsim/0.11.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/syn/2.0.119/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/syn/2.0.119/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:syn@3.0.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/syn/3.0.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/syn/3.0.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/synstructure/0.14.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/synstructure/0.14.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/tao/0.37.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tao/0.37.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/tauri-codegen/2.7.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-codegen/2.7.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri-macros@2.7.4",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/tauri-macros/2.7.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-macros/2.7.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri-runtime-wry@2.12.3",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/tauri-runtime-wry/2.12.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-runtime-wry/2.12.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri-runtime@2.12.2",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/tauri-runtime/2.12.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-runtime/2.12.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tauri-utils@2.10.2",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/tauri-utils/2.10.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri-utils/2.10.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/tauri/2.12.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tauri/2.12.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tendril@0.5.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/tendril/0.5.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tendril/0.5.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:thiserror-impl@1.0.69",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/thiserror-impl/1.0.69/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/thiserror-impl/1.0.69/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:thiserror-impl@2.0.21",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/thiserror-impl/2.0.21/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/thiserror-impl/2.0.21/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:thiserror@1.0.69",
      "license": "UNKNOWN",
      "source": "https://docs.rs/crate/thiserror/1.0.69/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/thiserror/1.0.69/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:thiserror@2.0.21",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/thiserror/2.0.21/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/thiserror/2.0.21/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:time-core@0.1.9",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/time-core/0.1.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/time-core/0.1.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:time-macros@0.2.32",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/time-macros/0.2.32/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/time-macros/0.2.32/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:time@0.3.55",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/time/0.3.55/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/time/0.3.55/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tinystr@0.8.5",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/tinystr/0.8.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tinystr/0.8.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/tokio/1.54.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tokio/1.54.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/toml_datetime/1.1.3+spec-1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml_datetime/1.1.3+spec-1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/toml_parser/1.1.5+spec-1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml_parser/1.1.5+spec-1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:toml_writer@1.2.0+spec-1.1.0",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/toml_writer/1.2.0+spec-1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml_writer/1.2.0+spec-1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/toml/1.2.1+spec-1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/toml/1.2.1+spec-1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/tracing-core/0.1.36/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tracing-core/0.1.36/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:tracing@0.1.44",
      "license": "MIT",
      "source": "https://docs.rs/crate/tracing/0.1.44/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/tracing/0.1.44/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/typeid/1.0.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/typeid/1.0.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:typenum@1.20.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/typenum/1.20.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/typenum/1.20.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:unicode-ident@1.0.27",
      "license": "(MIT OR Apache-2.0) AND Unicode-3.0",
      "source": "https://docs.rs/crate/unicode-ident/1.0.27/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/unicode-ident/1.0.27/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:unicode-segmentation@1.13.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/unicode-segmentation/1.13.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/unicode-segmentation/1.13.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:url@2.5.8",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/url/2.5.8/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/url/2.5.8/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:urlpattern@0.6.0",
      "license": "MIT",
      "source": "https://docs.rs/crate/urlpattern/0.6.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/urlpattern/0.6.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:utf_types@1.0.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/utf_types/1.0.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/utf_types/1.0.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:utf8_iter@1.1.0",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/utf8_iter/1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/utf8_iter/1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:uuid@1.29.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/uuid/1.29.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/uuid/1.29.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "license": "UNLICENSED",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "UNRESOLVED: project owner distribution authorization",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:veylora-cli@0.0.0",
      "license": "UNLICENSED",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "UNRESOLVED: project owner distribution authorization",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:veylora-device@0.0.0",
      "license": "UNLICENSED",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "UNRESOLVED: project owner distribution authorization",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:veylora-diagnostics@0.0.0",
      "license": "UNLICENSED",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "UNRESOLVED: project owner distribution authorization",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:veylora-domain@0.0.0",
      "license": "UNLICENSED",
      "source": "Cargo.toml [workspace.package] and member Cargo.toml",
      "licenseSource": "UNRESOLVED: project owner distribution authorization",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/walkdir/2.5.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/walkdir/2.5.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/web_atoms/0.2.6/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/web_atoms/0.2.6/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/web-time/1.1.0/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/web-time/1.1.0/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/webview2-com-macros/0.8.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/webview2-com-macros/0.8.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:webview2-com-sys@0.39.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/webview2-com-sys/0.39.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/webview2-com-sys/0.39.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:webview2-com@0.39.1",
      "license": "MIT",
      "source": "https://docs.rs/crate/webview2-com/0.39.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/webview2-com/0.39.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/winapi-util/0.1.11/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/winapi-util/0.1.11/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/window-vibrancy/0.8.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/window-vibrancy/0.8.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/windows-collections/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-collections/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-core@0.62.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-core/0.62.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-core/0.62.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-future@0.3.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-future/0.3.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-future/0.3.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-implement@0.60.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-implement/0.60.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-implement/0.60.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-interface@0.59.3",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-interface/0.59.3/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-interface/0.59.3/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-link@0.2.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-link/0.2.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-link/0.2.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-numerics@0.3.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-numerics/0.3.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-numerics/0.3.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-result@0.4.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-result/0.4.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-result/0.4.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-strings@0.5.1",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-strings/0.5.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-strings/0.5.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/windows-sys/0.61.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-sys/0.61.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/windows-threading/0.2.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-threading/0.2.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows-version@0.1.7",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows-version/0.1.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows-version/0.1.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:windows@0.62.2",
      "license": "MIT OR Apache-2.0",
      "source": "https://docs.rs/crate/windows/0.62.2/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/windows/0.62.2/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/winnow/1.0.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/winnow/1.0.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/writeable/0.6.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/writeable/0.6.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:wry@0.57.1",
      "license": "Apache-2.0 OR MIT",
      "source": "https://docs.rs/crate/wry/0.57.1/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/wry/0.57.1/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/yoke-derive/0.8.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/yoke-derive/0.8.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:yoke@0.8.4",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/yoke/0.8.4/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/yoke/0.8.4/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:zerofrom-derive@0.1.8",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/zerofrom-derive/0.1.8/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/zerofrom-derive/0.1.8/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:zerofrom@0.1.8",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/zerofrom/0.1.8/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/zerofrom/0.1.8/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:zerotrie@0.2.5",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/zerotrie/0.2.5/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/zerotrie/0.2.5/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:zerovec-derive@0.11.7",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/zerovec-derive/0.11.7/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/zerovec-derive/0.11.7/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "cargo:zerovec@0.11.9",
      "license": "Unicode-3.0",
      "source": "https://docs.rs/crate/zerovec/0.11.9/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/zerovec/0.11.9/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "source": "https://docs.rs/crate/zmij/1.0.23/source/Cargo.toml",
      "licenseSource": "https://docs.rs/crate/zmij/1.0.23/source/ (inspect LICENSE*, COPYING*, NOTICE*)",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@fontsource-variable/geist-mono@5.3.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@fontsource-variable/geist-mono/5.3.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@fontsource-variable/geist@5.3.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@fontsource-variable/geist/5.3.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
    },
    {
      "id": "npm:@lucide/svelte@1.70.0",
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@lucide/svelte/1.70.0",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/@tauri-apps/api/2.12.4",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/bits-ui/2.20.6",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
      "license": "UNKNOWN",
      "source": "https://registry.npmjs.org/svelte/5.57.3",
      "licenseSource": "UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies",
      "status": "unresolved",
      "notice": "",
      "noticeSha256": ""
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
    }
  ],
  "blockers": [
    "Review every exact package license, copyright, NOTICE and redistribution obligation. Manifest declarations are not clearance.",
    "Frontend lock contains pnpm only, not the declared application graph. Resolve and review the complete frontend lock before release.",
    "Review native libraries, external tools, profiles, fonts and fixture rights described above. Verify final installer and About notices.",
    "Project workspace license is UNLICENSED. Obtain explicit project distribution authorization."
  ]
}
```
