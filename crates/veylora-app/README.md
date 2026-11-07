# veylora-app

Tauri 2 desktop shell for Veylora RIP. It is a thin adapter: it exposes typed
commands that return read models, and the Svelte frontend in `../../app` renders
them. Authoritative behavior stays in Rust application services (ADR 0001).

Status: this crate has not been compiled. No Rust toolchain was available while
the shell was written, so treat it as unvalidated until `cargo check` passes.

## Before the first build

1. `tauri::generate_context!()` requires the bundle icons referenced by Tauri's
   defaults, and `icons/icon.ico` does not exist yet, so the build will fail
   until it does. Generate the icon set from a square source image (1024 px PNG
   recommended) from this directory:

   ```powershell
   cd app
   npx pnpm@12.18.0 exec tauri icon path\to\source.png --output ..\crates\veylora-app\icons
   ```

   This needs the Tauri CLI (`@tauri-apps/cli`), which `app/package.json` does
   not install yet. Add it as a pinned devDependency first, or use
   `cargo install tauri-cli --locked --version "^2"` and run
   `cargo tauri icon` from `crates/veylora-app`. The generated icons should be
   committed. No icon has been fabricated here.
2. `tauri.conf.json` runs `pnpm --dir ../../app dev` and `pnpm --dir ../../app build`.
   Those assume a global `pnpm` on PATH. The repository pins pnpm through
   `npx pnpm@12.18.0`, so either install that pnpm version globally or change
   the two commands to `npx pnpm@12.18.0 --dir ../../app ...`. The paths are
   relative to `crates/veylora-app` and have not been validated.

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
