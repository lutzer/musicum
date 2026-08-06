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

## Routing

The shell uses hash-based routing. All routing goes through
`src/shell/router.ts` — components never read or write `window.location.hash`
directly.

URL grammar:

```
#<viewId>            → list view for <viewId>
#<viewId>/<param>    → detail view; the router looks up "<viewId>-detail"
                       and passes <param> as its `slug` attribute
#<viewId>/           → trailing slash normalizes to the list view
(empty)              → falls back to router.defaultViewId (#welcome)
```

The `-detail` suffix is a naming convention: register `foo` (list) and
`foo-detail` (detail, usually with `sidebar: false`) and you get master/detail
routing for free. The nav highlights the parent entry on detail routes because
`router.activeViewId()` returns only the leading segment.

Router API (`src/shell/router.ts`):

- `router.current()` / `router.parse(hash)` — read current or arbitrary hash
- `router.activeViewId()` — leading segment only, for nav highlighting
- `router.hashFor(viewId, param?)` — build a hash string (use in `href=`)
- `router.navigate(viewId, param?)` — imperative navigation
- `router.resolve(route, viewRegistry)` — resolve to a view element or missing
- `router.subscribe(handler)` — listen for route changes; returns unsubscribe
- `router.ensureDefault()` — navigate to the default view if hash is empty

To add a new routable view, register it in `main.ts` (or via a plugin's
`PluginContext.registerView`) and link to it with
`href=${router.hashFor('myview')}` from the nav or a detail link with
`router.hashFor('myview', slug)`.
