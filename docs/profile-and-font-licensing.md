# Profile, font and fixture licensing

Task 9.3 inventory, 2026-11-08. **Distribution review is unresolved.** Rights to
use an asset locally do not imply rights to bundle it, export it in a job archive,
embed it in generated output, publish screenshots, or redistribute derivatives.

**2026-11-20 scope note:** the packaged `veylora-cli` bundles no fonts, ICC profiles, icons or fixtures, so none of the unresolved items below affect the CLI release scope (ADR 0010). They remain unresolved and block the desktop app and any profile-bearing release.

## Fonts

| Asset | Current evidence | Permission and blocker |
| --- | --- | --- |
| Geist Variable | `@fontsource-variable/geist` 5.3.0 in `app/package.json`, CSS imported by `app/src/main.ts` | Existing notice records SIL OFL 1.1, copyright 2024 The Geist Project Authors. Preserve notice text, but reverify against the exact package before approval. |
| Geist Mono Variable | `@fontsource-variable/geist-mono` 5.3.0 in the same manifest/imports | Same reported OFL 1.1. Exact emitted subsets, copyright and reserved-name declarations require verification. |
| System fallback fonts | `app/src/app.css`: system-ui, Segoe UI, Cascadia Mono, Consolas and platform generics | Referenced by CSS, not font binaries in this repository. Do not extract/bundle OS fonts based on their availability on a development machine. |

Authoritative sources are each exact published package's `package.json`, `LICENSE`
and font metadata, including any Fontsource conversion/subsetting provenance.
Upstream project: https://github.com/vercel/geist-font. Existing full OFL texts
remain in `THIRD_PARTY_NOTICES.md`; they are inherited evidence, not independently
verified here. The current lock does not include these font packages, and no
installed package exists in this worktree. Prior blanket claims of completed
font compliance are therefore withdrawn.

OFL permits bundling subject to its conditions, including retention of copyright
and license, no standalone sale of the font, restrictions on reserved names for
modified versions, and no implied endorsement. Determine reserved names from the
actual font/version, not from a guess. Fontsource subsets/conversions and the
actual release-emitted WOFF2 set need review. Do not claim that an About view or
installer includes notices until those artifacts have been tested.

Before marking a font reviewed, record package integrity, exact font file hashes,
source license text and hash, copyright/Reserved Font Names, transformations,
redistribution obligations and where the installed user can read the notices.

## ICC profiles

No `.icc` or `.icm` files are tracked/non-ignored in this worktree at inventory
time. There is no bundled printer profile permission to approve. Planned ICC
support and the existence of a standard color space do not confer redistribution
rights to a particular profile file.

For every future profile record: source/publisher, exact filename/version and hash,
license text or written grant, intended inclusion in installer/job/export/support
bundle, derivative restrictions, attribution, and review decision. User-supplied
profiles stay operator-controlled unless the applicable permission explicitly
allows transfer. Do not bundle commercial RIP/vendor profiles without permission.
An unidentified or proprietary profile blocks distribution rather than being
silently replaced with an approximate profile.

## Fixture and graphic assets

| Asset/source | Classification | Rights and release decision |
| --- | --- | --- |
| `fixtures/targets/channel-identity.json` | Repository synthetic target definition | No separate license grant is recorded. Verify authorship/project distribution authority. Not a physical result. |
| `fixtures/physical/README.md` | Evidence handling conventions only | No physical samples or measurements are bundled. Future photographs, measurements and reference artifacts require source/rights review and privacy consent where applicable. |
| `crates/veylora-app/tests/golden/presenters.json` | Synthetic presenter golden data | Repository fixture, not assumed public domain. Project authorization unresolved. |
| `crates/veylora-app/icons-src/veylora-mark.svg` and `crates/veylora-app/icons/**` | App artwork and generated multi-platform icon variants | Source and outputs are inventoried/hashes retained. Confirm original authorship, generation inputs and no third-party mark/assets before redistribution. No grant is invented. |
| `crates/veylora-cli/src/probe/svg.rs`, `crates/veylora-diagnostics/src/targets.rs`, test-generated data | Code-generated probe/target and test artifacts | No externally supplied image fixture is inferred. Rights follow reviewed code/input provenance, currently blocked by project authorization. Embedded SVG text/font substitution must not cause extraction of proprietary font files. |
| `scripts/tests/check-licenses.tests.ps1` | Synthetic license-check fixtures generated in a temporary Git repository | Example MIT label and example.invalid source links test validation only. Synthetic notice explicitly grants no permission. Not evidence about any real package. |
| Frontend Lucide icons | `@lucide/svelte` 1.70.0 manifest declaration | Design docs report ISC, but verify exact package and icon-specific/derived notices. Do not treat the design document as the authoritative license. |

For future fixture assets, record original author, acquisition/source, hash,
license/permission, derivation chain, privacy restrictions, and whether repository,
installer, screenshots, exported jobs and support bundles may include them.
Commercial reference output may be legitimate bench evidence without being
redistributable. Keep such evidence outside public repository/release artifacts.

## Enforcement boundary

`scripts/check-licenses.ps1` verifies notices and metadata and detects changes in
its documented evidence scope. New recognized assets or changed hashes fail the
normal check. The inventory's explicit asset/native/tool review blocker must stay
until a reviewer records rights and obligations for the final payload. Hashes are
identity evidence, not licensing permission. Packaging and About-view delivery
checks belong to the coordinating release task and remain required.
