import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { unsafeStatic, html as staticHtml } from 'lit/static-html.js';
import { viewRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from './registry-controller';

type Route = { viewId: string; param?: string };

@customElement('mus-content')
export class MusContent extends LitElement {
  static styles = css`
    :host { display: flex; }
  `;

  @state() private route: Route | undefined = parseHash();

  constructor() {
    super();
    new RegistrySubscription(this, viewRegistry);
    window.addEventListener('hashchange', () => {
      this.route = parseHash();
    });
  }

  render() {
    const route = this.route ?? { viewId: viewRegistry.list()[0]?.id ?? '' };
    if (!route.viewId) return html`<p>No views registered.</p>`;

    if (route.param === undefined) {
      const view = viewRegistry.get(route.viewId);
      if (!view) return html`<p>Unknown view: ${route.viewId}</p>`;
      const tag = unsafeStatic(view.element);
      return staticHtml`<${tag}></${tag}>`;
    }

    const detailId = `${route.viewId}-detail`;
    const view = viewRegistry.get(detailId);
    if (!view) return html`<p>Unknown view: ${route.viewId}/${route.param}</p>`;
    const tag = unsafeStatic(view.element);
    return staticHtml`<${tag} slug=${route.param}></${tag}>`;
  }
}

function parseHash(): Route | undefined {
  const h = window.location.hash.replace(/^#/, '').trim();
  if (!h.length) return undefined;
  const slash = h.indexOf('/');
  if (slash === -1) return { viewId: h };
  const param = h.slice(slash + 1);
  // Empty param (`#thing/`) → treat as no param.
  if (!param.length) return { viewId: h.slice(0, slash) };
  return { viewId: h.slice(0, slash), param };
}

declare global { interface HTMLElementTagNameMap { 'mus-content': MusContent } }
