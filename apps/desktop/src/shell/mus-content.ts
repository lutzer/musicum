import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { unsafeStatic, html as staticHtml } from 'lit/static-html.js';
import { viewRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from './registry-controller';
import { router, type Route } from './router';

@customElement('mus-content')
export class MusContent extends LitElement {
  static styles = css`
    :host { display: flex; }
  `;

  @state() private route: Route | undefined = router.current();

  constructor() {
    super();
    new RegistrySubscription(this, viewRegistry);
    router.subscribe(route => { this.route = route; });
  }

  render() {
    const route = this.route ?? { viewId: viewRegistry.list()[0]?.id ?? '' };
    if (!route.viewId) return html`<p>No views registered.</p>`;

    const resolved = router.resolve(route, viewRegistry);
    if (resolved.kind === 'missing') {
      const path = resolved.param === undefined
        ? resolved.viewId
        : `${resolved.viewId}/${resolved.param}`;
      return html`<p>Unknown view: ${path}</p>`;
    }

    const tag = unsafeStatic(resolved.element);
    return resolved.param === undefined
      ? staticHtml`<${tag}></${tag}>`
      : staticHtml`<${tag} slug=${resolved.param}></${tag}>`;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-content': MusContent } }
