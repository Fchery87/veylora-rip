# Profile and Font Licensing

Task 9.3 deliverable. This file records licensing for fonts and color profiles that
ship inside Veylora RIP. It is an engineering record, not legal advice. Re-check
each entry before the first distribution build.

Status: fonts inventoried. Color/ICC profiles and fixture assets are not yet
inventoried (Task 9.3 step 1).

## Fonts

`docs/design.md` section 5.4 requires Geist and Geist Mono to be bundled with the
app and never fetched at runtime.

| Font | Package | Version | License | Use |
| --- | --- | --- | --- | --- |
| Geist | `@fontsource-variable/geist` | 5.3.0 | SIL OFL 1.1 | UI text |
| Geist Mono | `@fontsource-variable/geist-mono` | 5.3.0 | SIL OFL 1.1 | IDs, hashes, timestamps, loupe values |

- Copyright: "Copyright 2024 The Geist Project Authors" (https://github.com/vercel/geist-font).
- Source of record: the `LICENSE` file inside each package. Full text is reproduced in
  `THIRD_PARTY_NOTICES.md`.
- Both are imported in `app/src/main.ts`; Vite emits the `woff2` subsets into
  `app/dist/assets` and they are embedded into the Tauri binary.

### OFL 1.1 conditions that apply to Veylora

| OFL condition | How Veylora complies |
| --- | --- |
| Fonts may be bundled and redistributed with software, including sold software. | Allowed. Veylora bundles them inside the app. |
| The font files must not be sold by themselves. | Veylora never offers the font files as a separate product or download. |
| Each copy must carry the copyright notice and the license text. | `THIRD_PARTY_NOTICES.md` carries both. The release check (Task 9.3 step 3) must fail if it is missing from the installer and from the About/licenses view. |
| Fonts and derivatives must stay under OFL. | No change to the font license. Veylora's own code license does not extend to the fonts. |
| Reserved Font Names must not be used by Modified Versions. | The shipped copyright notice declares no Reserved Font Name. |
| Names of copyright holders must not be used to promote the product. | Marketing and UI copy must not imply endorsement by the Geist authors or Vercel. |

### Fontsource repackaging

The `woff2` files come from Fontsource, which converts and subsets the upstream
fonts. Under OFL a format change counts as a Modified Version, which is permitted
because no Reserved Font Name is declared. Do not rename the font families in
`font-family` declarations (`Geist Variable`, `Geist Mono Variable`) to anything
that implies they are the upstream release.

### Open items

- Add the font licenses to the installer's license screen and an in-app licenses
  view before distribution (Task 9.2/9.3).
- Re-verify the license text after any Fontsource version bump; the package
  `LICENSE` is the file of record.

## Color profiles

Not yet inventoried. Record each bundled ICC profile's source, license, and
redistribution permission here before it ships. Do not bundle a vendor profile
without written redistribution terms.
