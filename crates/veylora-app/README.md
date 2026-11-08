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

`icons/` is generated from `icons-src/placeholder-icon.png`, a generated
placeholder with no brand meaning. Replace the source with real artwork (1024 px
square PNG) and regenerate from `app/`:

```powershell
npx pnpm@12.18.0 exec tauri icon ..\crates\veylora-app\icons-src\<source>.png --output ..\crates\veylora-app\icons
```

## Running

Run from `crates/veylora-app` so the Tauri hooks resolve paths correctly:

```powershell
..\..\app\node_modules\.bin\tauri dev      # starts the Vite dev server, then the window
..\..\app\node_modules\.bin\tauri build    # release bundle
```

`tauri.conf.json` runs the frontend through `npx pnpm@12.18.0` in `../../app`
(the global `pnpm` on this machine is broken, so it is not used). The Tauri CLI
is pinned in `app/package.json`.

## Frontend commands (from `app/`)

```powershell
npx pnpm@12.18.0 install --frozen-lockfile
npx pnpm@12.18.0 exec vp check
npx pnpm@12.18.0 exec vp test run
npx pnpm@12.18.0 exec vp build
```

## Connected commands

Only `get_capabilities` is wired to the frontend (`app/src/lib/services/tauri.ts`).
The frontend falls back to mocked fixture data outside the Tauri webview and for
every service that has no Rust command yet. The file sink is mocked.
