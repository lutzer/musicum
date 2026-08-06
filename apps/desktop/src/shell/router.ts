/**
 * Hash-based router for the desktop shell.
 *
 * URL grammar
 * -----------
 *   #<viewId>            → list view for <viewId>
 *   #<viewId>/<param>    → detail view; the router looks up "<viewId>-detail"
 *                          and passes <param> as its `slug` attribute
 *   #<viewId>/           → trailing slash is normalized to the list view
 *   (empty)              → falls back to `router.defaultViewId`
 *
 * View resolution
 * ---------------
 * A route with no `param` renders the registered view whose id equals
 * `viewId`. A route with a `param` renders the view whose id is
 * `${viewId}-detail`. Detail views are typically registered with
 * `sidebar: false` so they don't appear in the nav — the nav still highlights
 * the parent entry because `router.activeViewId()` returns only the leading
 * segment.
 *
 * All shell components subscribe through `router.subscribe(...)` and should
 * never touch `window.location.hash` directly. Detail links are built with
 * `router.hashFor(id, param)`; imperative navigation goes through
 * `router.navigate(id, param)`.
 */

import type { ViewRegistry } from '../plugin-api/registry';

export type Route = { viewId: string; param?: string };

export type Resolved =
  | { kind: 'view';    element: string; param?: string }
  | { kind: 'missing'; viewId: string; param?: string };

export class Router {
  readonly defaultViewId = 'welcome';

  parse(hash: string = window.location.hash): Route | undefined {
    const h = hash.replace(/^#/, '').trim();
    if (!h.length) return undefined;
    const slash = h.indexOf('/');
    if (slash === -1) return { viewId: h };
    const param = h.slice(slash + 1);
    if (!param.length) return { viewId: h.slice(0, slash) };
    return { viewId: h.slice(0, slash), param };
  }

  current(): Route | undefined {
    return this.parse();
  }

  activeViewId(): string | undefined {
    return this.current()?.viewId;
  }

  hashFor(viewId: string, param?: string): string {
    return param === undefined || param === ''
      ? `#${viewId}`
      : `#${viewId}/${param}`;
  }

  navigate(viewId: string, param?: string): void {
    window.location.hash = this.hashFor(viewId, param);
  }

  ensureDefault(): void {
    if (!window.location.hash) this.navigate(this.defaultViewId);
  }

  resolve(route: Route, registry: ViewRegistry): Resolved {
    if (route.param === undefined) {
      const view = registry.get(route.viewId);
      return view
        ? { kind: 'view', element: view.element }
        : { kind: 'missing', viewId: route.viewId };
    }
    const view = registry.get(`${route.viewId}-detail`);
    return view
      ? { kind: 'view', element: view.element, param: route.param }
      : { kind: 'missing', viewId: route.viewId, param: route.param };
  }

  subscribe(handler: (route: Route | undefined) => void): () => void {
    const listener = () => handler(this.current());
    window.addEventListener('hashchange', listener);
    return () => window.removeEventListener('hashchange', listener);
  }
}

export const router = new Router();
