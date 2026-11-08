# veylora-app

Tauri 2 desktop shell for Veylora RIP. It is a thin adapter: it exposes typed
commands that return read models, and the Svelte frontend in `../../app` renders
them. Authoritative behavior stays in Rust application services (ADR 0001).

Status: `cargo check`, `cargo clippy -D warnings`, `cargo test` and `cargo fmt --check`
pass on Windows, and `tauri dev` opens the window and renders the Jobs screen.

## Toolchain (installed on this machine)

- Rust stable via `rustup` (`winget install Rustlang.Rustup`), `x86_64-pc-windows-msvc`.
- Visual Studio 2022 Build Tools with the C++ workload (MSVC linker).
- WebView2 Runtime (`winget install Microsoft.EdgeWebView2Runtime`).

Add `%USERPROFILE%\.cargo\bin` to PATH if a shell does not see `cargo`.

## Icons

`icons/` is generated from `icons-src/veylora-mark.svg`: a V built from offset
C, M, Y and W layers, using the ink values in `docs/design.md` section 5.2. It is
an original mark for this project and has not been trademark or domain cleared
(see `docs/veylora-rip-brand.md`). To change it, edit the SVG and regenerate from
`app/`:

```powershell
pnpm exec tauri icon ..\crates\veylora-app\icons-src\veylora-mark.svg --output ..\crates\veylora-app\icons
```

## Running

Run from `crates/veylora-app` so the Tauri hooks resolve paths correctly:

```powershell
..\..\app\node_modules\.bin\tauri dev      # starts the Vite dev server, then the window
..\..\app\node_modules\.bin\tauri build    # release bundle
```

`tauri.conf.json` runs the frontend through `pnpm` in `../../app`. `pnpm` must be
on PATH; it honors the `packageManager` pin (12.18.0) in `app/package.json`. The
Tauri CLI is pinned there too. CI still uses `npx pnpm@12.18.0` because runners
don't have a Corepack setup.

## Frontend commands (from `app/`)

```powershell
pnpm install --frozen-lockfile
pnpm exec vp check
pnpm exec vp test run
pnpm exec vp build
```

## Connected commands

Wired to the frontend through `app/src/lib/services/tauri.ts`: `get_capabilities`,
`list_jobs`, `get_job`, the view-model commands (`get_preflight_view`,
`get_job_actions`, `get_inspect_view`, `toggle_inspect_plane`, `get_review_view`)
and the job commands (`acknowledge_finding`, `render_and_hold`,
`write_to_file_sink`, `release_to_printer`, `resolve_review`, `set_white_*`).
Presentation logic lives in `src/screens/` and is tested in
`tests/preflight_presentation.rs`. Job data is a labeled fixture `JobStore` held
in memory; the print mode is fixed to PM-exp-01 on the Rust side.

The frontend falls back to mocked fixture data outside the Tauri webview and for
device setup, recipes, targets, artwork candidates, and diagnostics, which have no
Rust command yet. The file sink is mocked. Physical release is always refused.
