# apps/desktop

Tauri 2 + Lit desktop shell for musicum.

## Dev
```
npm install
npm run tauri:dev
```

## Production build
```
npm run tauri:build
```

Runtime plugin loading (via the Tauri asset protocol) only works in production
builds; the dev server serves built-in views and any statically imported
bundles only. See `docs/plans/specs/2026-07-06-tauri-lit-gui-scaffold-design.md`.
