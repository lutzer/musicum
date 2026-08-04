# CLAUDE.md — apps/desktop

## Overview
Tauri 2 desktop shell for musicum. TypeScript + Lit web components in a
Vite-served renderer; Rust backend in `src-tauri/` bridges to `musicum-core`.
The shell hosts built-in views and dynamically loaded plugin bundles.

## Layout
```
src/
  main.ts              # entry: registers built-in views, boots plugin loader
  base/                # generic UI primitives (mus-button, mus-card, mus-icon, mus-list-view)
  shell/               # app chrome (mus-app, mus-nav, mus-content, mus-slot)
  views/               # built-in views (welcome, files, clips)
  core-api/            # typed wrapper around Tauri `invoke` — the ONLY place IPC lives
  plugin-api/          # public SDK re-exports + view/slot registries (imported by plugins)
  plugin-loader/       # loads plugin manifests + dynamic import via asset protocol
  vendor/              # entry bundled as the runtime `plugin-api` module for plugins
src-tauri/src/
  lib.rs               # Tauri setup, DB init, invoke handler wiring
  commands/            # #[tauri::command] handlers: app_info, plugins, library
  plugin_fs.rs         # plugin discovery on disk
plugin-template/       # scaffold copied when creating a new plugin
```

## Dev commands
- `npm run dev` — Vite renderer only (no Tauri, no runtime plugin loading)
- `npm run tauri:dev` — full desktop app in dev mode
- `npm run build` — `tsc --noEmit` + Vite build (typecheck gate)
- `npm run tauri:build` — production desktop bundle (required for runtime plugins)
- `npm test` — Vitest (jsdom) unit tests
- `cargo clippy --all` (from repo root) — lint Rust after any `src-tauri` change

## Conventions
- **Element prefix `mus-`**. Every `@customElement` needs a matching
  `declare global { interface HTMLElementTagNameMap { ... } }` augmentation.
- **All Tauri IPC goes through `core-api/`.** Views and plugins never call
  `invoke` directly — extend `CoreApi` in `plugin-api/types.ts` and implement in
  `core-api/index.ts`.
- **Adding a built-in view**: create in `src/views/`, import in `main.ts`, and
  register via `viewRegistry.register('__built_in__', {...})`.
- **Plugins register through `PluginContext`** (`ctx.registerView`,
  `ctx.registerSlot`) — never touch registries directly. See `plugin-loader/loader.ts`.
- **Hash routing**: `mus-content` reads `window.location.hash` for the active
  view id; `mus-nav` sets it. No router library.
- **Styling**: Lit `css` tagged templates + shared CSS variables (`--mus-bg`,
  `--mus-border`, `--sidebar-bg`, …) defined in `src/styles.css`. If introducing new colors or numeric values, add them to the shared css instead of definining them in the components.
- **Tests colocated** as `*.test.ts` next to source; `vitest.config.ts` uses
  jsdom.

## Gotchas
- **Runtime plugin loading needs a production build.** Dev server can't serve
  arbitrary on-disk plugin bundles via the Tauri asset protocol.
- **Registry IDs are global.** Duplicate view ids or slot entry ids are
  rejected with a console error — the first registration wins.
- **`apiVersion` mismatch skips the plugin.** Bump `SUPPORTED_API_VERSION` in
  `plugin-loader/loader.ts` only when the plugin ABI actually changes.
- **Shared core lib.** `src-tauri` depends on `musicum-core`; the CLI in
  `apps/cli` uses the same crate. Don't add desktop-only logic to core — see
  the repo root `CLAUDE.md`.

## Supplemental docs
- `plugin-template/` — reference for the plugin ABI and build output
